use crate::ast;
pub mod types;
pub mod context;
pub mod expr;
pub mod stmt;
pub mod runtime;

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

        // Process all top-level statements
        for stmt in &module.body {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef { name, params, body, .. } => {
                    // Function definitions are handled separately
                    self.compile_function(name, params, body)?;
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

    /// Compile a function definition
    fn compile_function(&mut self, name: &str, params: &[ast::Parameter], body: &[Box<ast::Stmt>]) -> Result<(), String> {
        let _ = body;
        let _ = params;
        let _ = name;
        // Implementation will use type mapping to create function signature
        // This is where you'll use the new type mapping functionality

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