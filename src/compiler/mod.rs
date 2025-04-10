use crate::ast;
use crate::typechecker;
pub mod types;
pub mod context;
pub mod expr;
pub mod stmt;
pub mod runtime;
pub mod scope;
pub mod closure;

use crate::compiler::context::CompilationContext;
use inkwell::context::Context;
use stmt::StmtCompiler;
use types::Type;
use std::path::Path;
use std::collections::HashMap;

/// Compiler for Cheetah language
pub struct Compiler<'ctx> {
    context: CompilationContext<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    /// Create a new compiler with the given module name
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        Self {
            context: CompilationContext::new(context, module_name),
        }
    }

    /// Compile an AST module to LLVM IR
    pub fn compile_module(&mut self, module: &ast::Module) -> Result<(), String> {
        // Type check the module first
        if let Err(type_error) = typechecker::check_module(module) {
            return Err(format!("Type error: {}", type_error));
        }

        // Get types for function signature
        let void_type = Type::get_void_type(self.context.llvm_context);
        let fn_type = void_type.fn_type(&[], false);

        // Create a main function for our module
        let function = self.context.module.add_function("main", fn_type, None);
        let basic_block = self.context.llvm_context.append_basic_block(function, "entry");

        // Position builder at the end of the entry block
        self.context.builder.position_at_end(basic_block);

        // Embed runtime support functions
        self.embed_runtime_functions();

        // First pass: register all function declarations
        let mut function_defs = Vec::new();

        for stmt in &module.body {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef { name, params, .. } => {
                    // Register the function declaration
                    self.declare_function(name, params)?;
                    function_defs.push(stmt);
                },
                _ => {}
            }
        }

        // Second pass: compile function bodies
        for stmt in &function_defs {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef { name, params, body, .. } => {
                    // Compile the function body
                    self.compile_function_body(name, params, body)?;
                },
                _ => unreachable!("Only function definitions should be in function_defs")
            }
        }

        // Third pass: compile all other statements
        for stmt in &module.body {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef { .. } => {
                    // Already handled in previous passes
                },
                ast::Stmt::ClassDef { name, bases, body, .. } => {
                    // Class definitions are handled separately
                    self.compile_class(name, bases, body)?;
                },
                _ => {
                    // All other statements are compiled in the main function
                    self.context.compile_stmt(stmt.as_ref())?;
                }
            }
        }

        // Check if the current block already has a terminator
        // (this could happen if the last statement was a return or an unconditional branch)
        let current_block = self.context.builder.get_insert_block().unwrap();
        if current_block.get_terminator().is_none() {
            // Add a return void instruction only if there's no terminator
            self.context.builder.build_return(None).unwrap();
        }

        // Verify the module
        if let Err(err) = self.context.module.verify() {
            return Err(format!("Module verification failed: {}", err));
        }

        Ok(())
    }

    fn embed_runtime_functions(&mut self) {

        // First, create all the conversion functions
        self.create_conversion_functions();

        // Then register the polymorphic str function
        self.register_polymorphic_str();

        // Finally, create the string conversion functions
        self.create_string_conversion_functions();
    }

    fn create_conversion_functions(&mut self) {
        let context = self.context.llvm_context;
        let module = &mut self.context.module;

        // int_to_string
        if module.get_function("int_to_string").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[context.i64_type().into()], false);
            module.add_function("int_to_string", fn_type, None);
        }

        // float_to_string
        if module.get_function("float_to_string").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[context.f64_type().into()], false);
            module.add_function("float_to_string", fn_type, None);
        }

        // bool_to_string
        if module.get_function("bool_to_string").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[context.i64_type().into()], false);
            module.add_function("bool_to_string", fn_type, None);
        }
    }

    fn create_string_conversion_functions(&mut self) {
        let context = self.context.llvm_context;
        let module = &mut self.context.module;

        // string_to_int
        if module.get_function("string_to_int").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.i64_type().fn_type(&[str_ptr_type.into()], false);
            module.add_function("string_to_int", fn_type, None);
        }

        // string_to_float
        if module.get_function("string_to_float").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.f64_type().fn_type(&[str_ptr_type.into()], false);
            module.add_function("string_to_float", fn_type, None);
        }

        // string_to_bool
        if module.get_function("string_to_bool").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.bool_type().fn_type(&[str_ptr_type.into()], false);
            module.add_function("string_to_bool", fn_type, None);
        }

        // free_string
        if module.get_function("free_string").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.void_type().fn_type(&[str_ptr_type.into()], false);
            module.add_function("free_string", fn_type, None);
        }

        // string_concat - for string concatenation
        if module.get_function("string_concat").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[str_ptr_type.into(), str_ptr_type.into()], false);
            module.add_function("string_concat", fn_type, None);
        }

        // string_equals - for string comparison
        if module.get_function("string_equals").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.bool_type().fn_type(&[str_ptr_type.into(), str_ptr_type.into()], false);
            module.add_function("string_equals", fn_type, None);
        }

        // string_length - for getting string length
        if module.get_function("string_length").is_none() {
            let str_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = context.i64_type().fn_type(&[str_ptr_type.into()], false);
            module.add_function("string_length", fn_type, None);
        }

        // Register str as an alias for int_to_string to support str() built-in
        if let Some(int_to_string) = module.get_function("int_to_string") {
            self.context.functions.insert("str".to_string(), int_to_string);
        }
    }

    fn register_polymorphic_str(&mut self) {
        // Get the functions for different type conversions
        let int_to_string = self.context.module.get_function("int_to_string")
            .expect("int_to_string function not found");

        let float_to_string = self.context.module.get_function("float_to_string")
            .expect("float_to_string function not found");

        let bool_to_string = self.context.module.get_function("bool_to_string")
            .expect("bool_to_string function not found");

        // Create a map of type to function for the str function
        let mut str_variants = HashMap::new();
        str_variants.insert(Type::Int, int_to_string);
        str_variants.insert(Type::Float, float_to_string);
        str_variants.insert(Type::Bool, bool_to_string);

        // Register the polymorphic function
        self.context.polymorphic_functions.insert("str".to_string(), str_variants);

        // Also register each variant in the regular function map for backward compatibility
        self.context.functions.insert("str".to_string(), int_to_string);
        self.context.functions.insert("int_to_string".to_string(), int_to_string);
        self.context.functions.insert("float_to_string".to_string(), float_to_string);
        self.context.functions.insert("bool_to_string".to_string(), bool_to_string);
    }

    /// Declare a function (first pass)
    fn declare_function(&mut self, name: &str, params: &[ast::Parameter]) -> Result<(), String> {
        // Get the LLVM context
        let context = self.context.llvm_context;

        // Create parameter types
        let mut param_types = Vec::new();

        // Process parameters
        for _ in params {
            // For now, all parameters are i64 (Int type)
            param_types.push(context.i64_type().into());
        }

        // Create function type (for now, all functions return i64)
        let return_type = context.i64_type();
        let function_type = return_type.fn_type(&param_types, false);

        // Create the function
        let function = self.context.module.add_function(name, function_type, None);

        // Register the function in our context
        self.context.functions.insert(name.to_string(), function);

        Ok(())
    }

    /// Compile a function body (second pass)
    fn compile_function_body(&mut self, name: &str, params: &[ast::Parameter], body: &[Box<ast::Stmt>]) -> Result<(), String> {
        // Get the LLVM context
        let context = self.context.llvm_context;

        // Get the function
        let function = match self.context.functions.get(name) {
            Some(&f) => f,
            None => return Err(format!("Function {} not found", name)),
        };

        // Create a basic block for the function
        let basic_block = context.append_basic_block(function, "entry");

        // Save the current position
        let current_block = self.context.builder.get_insert_block();

        // Position at the end of the new block
        self.context.builder.position_at_end(basic_block);

        // Create a new scope for the function
        self.context.push_scope(true, false, false); // Create a new scope for the function (is_function=true)

        // For backward compatibility
        let mut local_vars = HashMap::new();

        // Add parameters to the local variables
        for (i, param) in params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();

            // Create an alloca for this variable
            let alloca = self.context.builder.build_alloca(context.i64_type(), &param.name).unwrap();

            // Store the parameter value in the alloca
            self.context.builder.build_store(alloca, param_value).unwrap();

            // Remember the alloca for this variable
            local_vars.insert(param.name.clone(), alloca);

            // Add the parameter to the current scope
            self.context.add_variable_to_scope(param.name.clone(), alloca, Type::Int);

            // Register the parameter type in the type environment (for backward compatibility)
            self.context.register_variable(param.name.clone(), Type::Int);
        }

        // Note: Global variables will be accessed directly through the get_variable_ptr method

        // Save the current function and local variables
        let old_function = self.context.current_function;
        let old_local_vars = std::mem::replace(&mut self.context.local_vars, local_vars);

        // Set the current function
        self.context.current_function = Some(function);

        // Compile the function body
        for stmt in body {
            self.context.compile_stmt(stmt.as_ref())?;
        }

        // If the function doesn't end with a return statement, add one
        if !self.context.builder.get_insert_block().unwrap().get_terminator().is_some() {
            // Return 0 by default
            let zero = context.i64_type().const_int(0, false);
            self.context.builder.build_return(Some(&zero)).unwrap();
        }

        // Restore the previous function and local variables
        self.context.current_function = old_function;
        self.context.local_vars = old_local_vars;

        // Pop the function scope
        self.context.pop_scope();

        // Restore the previous position
        if let Some(block) = current_block {
            self.context.builder.position_at_end(block);
        }

        Ok(())
    }

    /// Compile a class definition
    fn compile_class(&mut self, name: &str, bases: &[Box<ast::Expr>], body: &[Box<ast::Stmt>]) -> Result<(), String> {
        let _ = body;
        let _ = bases;
        let _ = name;
        // Implementation will use the new class type creation

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
}