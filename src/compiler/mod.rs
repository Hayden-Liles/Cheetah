use crate::ast;
use crate::typechecker;
pub mod builtins;
pub mod closure;
pub mod context;
pub mod exception;
pub mod expr;
pub mod expr_non_recursive;
pub mod loop_transformers;
pub mod runtime;
pub mod scope;
pub mod stmt;
pub mod stmt_non_recursive;
pub mod tail_call_optimizer;
pub mod types;

use crate::compiler::context::CompilationContext;
use inkwell::passes::PassManager;
use inkwell::{context::Context, targets::TargetMachine};
use std::collections::HashMap;
use std::path::Path;
use stmt::StmtCompiler;
use types::Type;

// No need to import builtins modules directly as they're already available through the module system

/// Compiler for Cheetah language
pub struct Compiler<'ctx> {
    pub context: CompilationContext<'ctx>,
    pub optimize: bool,
}

impl<'ctx> Compiler<'ctx> {
    /// Create a new compiler with the given module name
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        Self {
            context: CompilationContext::new(context, module_name),
            optimize: true,
        }
    }

    pub fn emit_to_aot(&mut self, filename: &str) -> Result<(), String> {
        use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target};
        use std::path::Path;
        use std::process::Command;

        Target::initialize_all(&InitializationConfig::default());

        let triple = TargetMachine::get_default_triple();
        let target =
            Target::from_triple(&triple).map_err(|e| format!("No target for {}: {}", triple, e))?;

        let tm = target
            .create_target_machine(
                &triple,
                &TargetMachine::get_host_cpu_name().to_string(),
                &TargetMachine::get_host_cpu_features().to_string(),
                inkwell::OptimizationLevel::Aggressive,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or("Failed to create TargetMachine")?;

        let module = &mut self.context.module;
        module.set_triple(&triple);

        let obj_path = format!("{}.o", filename);
        tm.write_to_file(module, FileType::Object, Path::new(&obj_path))
            .map_err(|e| format!("Failed to write object file: {:?}", e))?;

        let runtime_lib_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(manifest) => format!("{}/target/release", manifest),
            Err(_) => {
                let mut exe = std::env::current_exe()
                    .map_err(|e| format!("Failed to locate current exe: {}", e))?;
                exe.pop();
                exe.pop();
                exe.push("lib");
                exe.push("cheetah");
                exe.to_string_lossy().into_owned()
            }
        };

        let llvm_config = std::env::var("LLVM_CONFIG").unwrap_or_else(|_| "llvm-config".into());
        let llvm_output = Command::new(&llvm_config)
            .arg("--libs")
            .arg("--system-libs")
            .output()
            .map_err(|e| format!("Failed to run {}: {}", llvm_config, e))?;
        if !llvm_output.status.success() {
            return Err(format!(
                "llvm-config failed: {}",
                String::from_utf8_lossy(&llvm_output.stderr)
            ));
        }
        let llvm_flags = String::from_utf8(llvm_output.stdout)
            .map_err(|e| format!("Invalid UTF-8 from llvm-config: {}", e))?;

        let mut cmd = Command::new("c++");
        cmd.arg(&obj_path)
            .arg("-L")
            .arg(&runtime_lib_dir)
            .arg("-lcheetah");

        for token in llvm_flags.split_whitespace() {
            cmd.arg(token);
        }

        cmd.arg("-lstdc++")
            .arg("-lz")
            .arg("-lzstd")
            .arg("-lffi")
            .arg("-ltinfo");

        cmd.arg("-o").arg(filename);

        let status = cmd
            .status()
            .map_err(|e| format!("Failed to spawn linker: {}", e))?;
        if !status.success() {
            return Err(format!("Linker exited with: {}", status));
        }

        println!("✅ AOT build → ./{}", filename);
        Ok(())
    }

    /// Compile an AST module to LLVM IR
    pub fn compile_module(&mut self, module: &ast::Module) -> Result<(), String> {
        if let Err(type_error) = typechecker::check_module(module) {
            return Err(format!("Type error: {}", type_error));
        }

        if self.optimize {
            let pass_manager = PassManager::create(());

            pass_manager.run_on(&self.context.module);
        }

        let void_type = Type::get_void_type(self.context.llvm_context);
        let fn_type = void_type.fn_type(&[], false);

        let function = self.context.module.add_function("main", fn_type, None);
        let basic_block = self
            .context
            .llvm_context
            .append_basic_block(function, "entry");

        self.context.builder.position_at_end(basic_block);

        let result = self.compile_module_body(module);

        if let Ok(_) = &result {
            let current_block = self.context.builder.get_insert_block().unwrap();
            if current_block.get_terminator().is_none() {
                self.context.builder.build_return(None).unwrap();
            }
        }

        result
    }

    /// Compile an AST module to LLVM IR without type checking
    /// This is useful for testing purposes when we want to bypass type checking
    pub fn compile_module_without_type_checking(
        &mut self,
        module: &ast::Module,
    ) -> Result<(), String> {
        let void_type = Type::get_void_type(self.context.llvm_context);
        let fn_type = void_type.fn_type(&[], false);

        let function = self.context.module.add_function("main", fn_type, None);
        let basic_block = self
            .context
            .llvm_context
            .append_basic_block(function, "entry");

        self.context.builder.position_at_end(basic_block);

        self.embed_runtime_functions();

        let mut function_defs = Vec::new();

        for stmt in &module.body {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef { name, params, .. } => {
                    self.declare_function(name, params)?;
                    function_defs.push(stmt);
                }
                _ => {}
            }
        }

        for stmt in &function_defs {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef {
                    name, params, body, ..
                } => {
                    self.compile_function_body(name, params, body)?;
                }
                _ => unreachable!("Only function definitions should be in function_defs"),
            }
        }

        for stmt in &module.body {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef { .. } => {}
                ast::Stmt::ClassDef {
                    name, bases, body, ..
                } => {
                    self.compile_class(name, bases, body)?;
                }
                _ => {
                    self.context.compile_stmt(stmt.as_ref())?;
                }
            }
        }

        let current_block = self.context.builder.get_insert_block().unwrap();
        if current_block.get_terminator().is_none() {
            self.context.builder.build_return(None).unwrap();
        }

        if let Err(err) = self.context.module.verify() {
            return Err(format!("Module verification failed: {}", err));
        }

        Ok(())
    }

    /// Compile the body of an AST module
    fn compile_module_body(&mut self, module: &ast::Module) -> Result<(), String> {
        self.embed_runtime_functions();

        let mut function_defs = Vec::new();

        for stmt in &module.body {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef { name, params, .. } => {
                    self.declare_function(name, params)?;
                    function_defs.push(stmt);
                }
                _ => {}
            }
        }

        for stmt in &function_defs {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef {
                    name, params, body, ..
                } => {
                    self.compile_function_body(name, params, body)?;
                }
                _ => unreachable!("Only function definitions should be in function_defs"),
            }
        }

        for stmt in &module.body {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef { .. } => {}
                ast::Stmt::ClassDef {
                    name, bases, body, ..
                } => {
                    self.compile_class(name, bases, body)?;
                }
                _ => {
                    self.context.compile_stmt(stmt.as_ref())?;
                }
            }
        }

        let current_block = self.context.builder.get_insert_block().unwrap();
        if current_block.get_terminator().is_none() {
            self.context.builder.build_return(None).unwrap();
        }

        if let Err(err) = self.context.module.verify() {
            return Err(format!("Module verification failed: {}", err));
        }

        Ok(())
    }

    fn embed_runtime_functions(&mut self) {
        self.create_conversion_functions();

        self.register_polymorphic_str();

        self.create_string_conversion_functions();

        runtime::register_runtime_functions(self.context.llvm_context, &mut self.context.module);

        self.context.register_len_function();
        self.context.register_print_function();
        self.context.register_min_max_functions();
    }

    fn create_conversion_functions(&mut self) {
        let context = self.context.llvm_context;
        let module = &mut self.context.module;

        if module.get_function("int_to_string").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[context.i64_type().into()], false);
            module.add_function("int_to_string", fn_type, None);
        }

        if module.get_function("float_to_string").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[context.f64_type().into()], false);
            module.add_function("float_to_string", fn_type, None);
        }

        if module.get_function("bool_to_string").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[context.i64_type().into()], false);
            module.add_function("bool_to_string", fn_type, None);
        }

        if module.get_function("range_1").is_none() {
            let fn_type = context
                .i64_type()
                .fn_type(&[context.i64_type().into()], false);
            let range_func = module.add_function("range_1", fn_type, None);
            self.context
                .functions
                .insert("range_1".to_string(), range_func);
        }

        if module.get_function("range_2").is_none() {
            let fn_type = context.i64_type().fn_type(
                &[context.i64_type().into(), context.i64_type().into()],
                false,
            );
            let range_func = module.add_function("range_2", fn_type, None);
            self.context
                .functions
                .insert("range_2".to_string(), range_func);
        }

        if module.get_function("range_3").is_none() {
            let fn_type = context.i64_type().fn_type(
                &[
                    context.i64_type().into(),
                    context.i64_type().into(),
                    context.i64_type().into(),
                ],
                false,
            );
            let range_func = module.add_function("range_3", fn_type, None);
            self.context
                .functions
                .insert("range_3".to_string(), range_func);
        }

        if let Some(range_func) = module.get_function("range_1") {
            self.context
                .functions
                .insert("range".to_string(), range_func);
        }
    }

    fn create_string_conversion_functions(&mut self) {
        let context = self.context.llvm_context;
        let module = &mut self.context.module;

        if module.get_function("string_to_int").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.i64_type().fn_type(&[str_ptr_type.into()], false);
            module.add_function("string_to_int", fn_type, None);
        }

        if module.get_function("string_to_float").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.f64_type().fn_type(&[str_ptr_type.into()], false);
            module.add_function("string_to_float", fn_type, None);
        }

        if module.get_function("string_to_bool").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.bool_type().fn_type(&[str_ptr_type.into()], false);
            module.add_function("string_to_bool", fn_type, None);
        }

        if module.get_function("free_string").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.void_type().fn_type(&[str_ptr_type.into()], false);
            module.add_function("free_string", fn_type, None);
        }

        if module.get_function("string_concat").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[str_ptr_type.into(), str_ptr_type.into()], false);
            module.add_function("string_concat", fn_type, None);
        }

        if module.get_function("string_equals").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context
                .bool_type()
                .fn_type(&[str_ptr_type.into(), str_ptr_type.into()], false);
            module.add_function("string_equals", fn_type, None);
        }

        if module.get_function("string_length").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.i64_type().fn_type(&[str_ptr_type.into()], false);
            module.add_function("string_length", fn_type, None);
        }

        if let Some(int_to_string) = module.get_function("int_to_string") {
            self.context
                .functions
                .insert("str".to_string(), int_to_string);
        }
    }

    fn register_polymorphic_str(&mut self) {
        let int_to_string = self
            .context
            .module
            .get_function("int_to_string")
            .expect("int_to_string function not found");

        let float_to_string = self
            .context
            .module
            .get_function("float_to_string")
            .expect("float_to_string function not found");

        let bool_to_string = self
            .context
            .module
            .get_function("bool_to_string")
            .expect("bool_to_string function not found");

        let mut str_variants = HashMap::new();
        str_variants.insert(Type::Int, int_to_string);
        str_variants.insert(Type::Float, float_to_string);
        str_variants.insert(Type::Bool, bool_to_string);

        self.context
            .polymorphic_functions
            .insert("str".to_string(), str_variants);

        self.context
            .functions
            .insert("str".to_string(), int_to_string);
        self.context
            .functions
            .insert("int_to_string".to_string(), int_to_string);
        self.context
            .functions
            .insert("float_to_string".to_string(), float_to_string);
        self.context
            .functions
            .insert("bool_to_string".to_string(), bool_to_string);
    }

    /// Declare a function (first pass)
    fn declare_function(&mut self, name: &str, params: &[ast::Parameter]) -> Result<(), String> {
        let context = self.context.llvm_context;

        let mut param_types = Vec::new();

        // For BoxedAny implementation, all parameters should be pointers
        for _ in params {
            param_types.push(context.ptr_type(inkwell::AddressSpace::default()).into());
        }

        // For BoxedAny implementation, all functions should return BoxedAny pointers
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        let function_type = ptr_type.fn_type(&param_types, false);

        let function = self.context.module.add_function(name, function_type, None);

        self.context.functions.insert(name.to_string(), function);

        Ok(())
    }

    /// Compile a function body (second pass)
    fn compile_function_body(
        &mut self,
        name: &str,
        params: &[ast::Parameter],
        body: &[Box<ast::Stmt>],
    ) -> Result<(), String> {
        let context = self.context.llvm_context;

        let function = match self.context.functions.get(name) {
            Some(&f) => f,
            None => return Err(format!("Function {} not found", name)),
        };

        let basic_block = context.append_basic_block(function, "entry");

        let current_block = self.context.builder.get_insert_block();

        self.context.builder.position_at_end(basic_block);

        self.context.push_scope(true, false, false);

        let mut local_vars = HashMap::new();

        for (i, param) in params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();

            let param_type = self.infer_parameter_type(name, &param.name);

            // For BoxedAny implementation, all parameters should be allocated as pointers
            let alloca = self
                .context
                .builder
                .build_alloca(
                    context.ptr_type(inkwell::AddressSpace::default()),
                    &param.name,
                )
                .unwrap();

            self.context
                .builder
                .build_store(alloca, param_value)
                .unwrap();

            local_vars.insert(param.name.clone(), alloca);

            self.context
                .add_variable_to_scope(param.name.clone(), alloca, param_type.clone());

            self.context
                .register_variable(param.name.clone(), param_type);
        }

        let old_function = self.context.current_function;
        let old_local_vars = std::mem::replace(&mut self.context.local_vars, local_vars);

        self.context.current_function = Some(function);

        for stmt in body {
            self.context.compile_stmt(stmt.as_ref())?;
        }

        if !self
            .context
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_some()
        {
            let zero = context.i64_type().const_int(0, false);
            self.context.builder.build_return(Some(&zero)).unwrap();
        }

        self.context.current_function = old_function;
        self.context.local_vars = old_local_vars;

        self.context.pop_scope();

        if let Some(block) = current_block {
            self.context.builder.position_at_end(block);
        }

        Ok(())
    }

    /// Compile a class definition
    fn compile_class(
        &mut self,
        name: &str,
        bases: &[Box<ast::Expr>],
        body: &[Box<ast::Stmt>],
    ) -> Result<(), String> {
        let _ = body;
        let _ = bases;
        let _ = name;

        Ok(())
    }

    /// Save the compiled module to a file
    pub fn write_to_file(&self, path: &Path) -> Result<(), String> {
        match self.context.module.print_to_file(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to write module to file: {}", e)),
        }
    }

    /// Get the LLVM IR representation as a string
    pub fn get_ir(&self) -> String {
        self.context.module.print_to_string().to_string()
    }

    pub fn get_module(&self) -> &inkwell::module::Module<'ctx> {
        &self.context.module
    }

    /// Infer the type of a function parameter based on function name and parameter name
    fn infer_parameter_type(&self, function_name: &str, param_name: &str) -> Type {
        // For BoxedAny implementation, most parameters should be Type::Any
        // This will make them use BoxedAny pointers in the LLVM IR

        match (function_name, param_name) {
            // List parameters should be Type::Any for BoxedAny implementation
            ("get_first", "lst") => Type::Any,
            ("append_to_list", "lst") => Type::Any,
            (_, "lst") => Type::Any,

            // Tuple parameters should be Type::Any for BoxedAny implementation
            ("unpack_tuple", "t") => Type::Any,

            ("process_nested_tuple", "t") => Type::Any,

            ("sum_tuple", "t") => Type::Any,

            ("process_tuples", "t1") => Type::Any,
            ("process_tuples", "t2") => Type::Any,

            ("unpack_simple", "t") => Type::Any,

            ("unpack_nested", "t") => Type::Any,

            ("unpack_multiple", "t1") => Type::Any,
            ("unpack_multiple", "t2") => Type::Any,

            ("outer", "t") => Type::Any,

            ("scope_test", "t") => Type::Any,

            ("fibonacci_pair", "n") => Type::Any,

            ("get_first_word", "text") => Type::Any,
            (_, "text") => Type::Any,
            (_, "str") => Type::Any,
            (_, "string") => Type::Any,

            ("get_value", "data") => Type::Any,
            ("create_person", _) => Type::Any,
            ("add_phone", "person") => Type::Any,
            ("process_dict", "data") => Type::Any,
            ("get_value_with_default", "data") => Type::Any,
            ("get_nested_value", "data") => Type::Any,
            ("get_name", "person") => Type::Any,
            ("identity", "d") => Type::Any,
            ("create_dict", "keys") => Type::Any,
            ("create_dict", "values") => Type::Any,
            (_, "keys") => Type::Any,
            (_, "values") => Type::Any,
            (_, "dict") => Type::Any,
            (_, "data") => Type::Any,
            (_, "person") => Type::Any,
            (_, "user") => Type::Any,
            (_, "map") => Type::Any,
            (_, "d") => Type::Any,

            _ if param_name.starts_with("tuple")
                || param_name == "t"
                || param_name.starts_with("t") && param_name.len() <= 3 =>
            {
                Type::Any
            }

            // Default to Type::Any for BoxedAny implementation
            _ => Type::Any,
        }
    }
}
