use crate::ast;
pub mod types;
pub mod context;
pub mod expr;
pub mod stmt;

use crate::compiler::context::CompilationContext;
use inkwell::context::Context;
use stmt::StmtCompiler;
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
    pub fn compile_module(&mut self, module: &ast::Module) -> Result<(), String> {
        // Get types for function signature
        let void_type = Type::get_void_type(self.context.llvm_context);
        let fn_type = void_type.fn_type(&[], false);
        
        // Create a main function for our module
        let function = self.context.module.add_function("main", fn_type, None);
        let basic_block = self.context.llvm_context.append_basic_block(function, "entry");
        
        // Position builder at the end of the entry block
        self.context.builder.position_at_end(basic_block);
        
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
}