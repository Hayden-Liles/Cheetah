use crate::ast;
pub mod types;
pub mod context;
pub mod expr;

use crate::compiler::context::CompilationContext;
use inkwell::context::Context;
use types::Type;
use std::path::Path;

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
    pub fn compile_module(&self, module: &ast::Module) -> Result<(), String> {
        // Get types for function signature
        let void_type = Type::get_void_type(self.context.llvm_context);
        let fn_type = void_type.fn_type(&[], false);
        
        // Create a main function for our module
        let function = self.context.module.add_function("main", fn_type, None);
        let basic_block = self.context.llvm_context.append_basic_block(function, "entry");
        
        // Position builder at end of the entry block
        self.context.builder.position_at_end(basic_block);
        
        // Add a return void instruction
        let _ = self.context.builder.build_return(None);
        
        // For each function in the AST, process it
        for stmt in &module.body {
            match stmt.as_ref() {
                ast::Stmt::FunctionDef { name, params, body, .. } => {
                    self.compile_function(name, params, body)?;
                }
                ast::Stmt::ClassDef { name, bases, body, .. } => {
                    self.compile_class(name, bases, body)?;
                }
                _ => {
                    // Handle other statement types
                    self.compile_stmt(stmt)?;
                }
            }
        }
        
        // Verify the module
        if let Err(err) = self.context.module.verify() {
            return Err(format!("Module verification failed: {}", err));
        }
        
        Ok(())
    }
    
    /// Compile a function definition
    fn compile_function(&self, name: &str, params: &[ast::Parameter], body: &[Box<ast::Stmt>]) -> Result<(), String> {
        let _ = body;
        let _ = params;
        let _ = name;
        // Implementation will use type mapping to create function signature
        // This is where you'll use the new type mapping functionality
        
        Ok(())
    }
    
    /// Compile a class defi_nameon
    fn compile_class(&self, name: &str, bases: &[Box<ast::Expr>], body: &[Box<ast::Stmt>]) -> Result<(), String> {
        let _ = body;
        let _ = bases;
        let _ = name;
        // Implementation will use the new class type creation
        
        Ok(())
    }
    
    /// Compile a statement_stmt
    fn compile_stmt(&self, stmt: &Box<ast::Stmt>) -> Result<(), String> {
        let _ = stmt;
        // Implementation will use type mapping for various statement types
        
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
}