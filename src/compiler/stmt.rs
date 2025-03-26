// In stmt.rs
use crate::ast::Stmt;
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::{ExprCompiler, AssignmentCompiler};

pub trait StmtCompiler<'ctx> {
    /// Compile a statement
    fn compile_stmt(&self, stmt: &Stmt) -> Result<(), String>;
}

impl<'ctx> StmtCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_stmt(&self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            // Compile an expression statement
            Stmt::Expr { value, .. } => {
                // Just compile the expression for its side effects
                let _ = self.compile_expr(value)?;
                Ok(())
            },
            
            // Compile an assignment statement
            Stmt::Assign { targets, value, .. } => {
                // Compile the right-hand side expression
                let (val, val_type) = self.compile_expr(value)?;
                
                // For each target on the left-hand side, assign the value
                for target in targets {
                    self.compile_assignment(target, val, &val_type)?;
                }
                
                Ok(())
            },
            
            // Add handling for other statement types as needed
            _ => Err(format!("Unsupported statement type: {:?}", stmt)),
        }
    }
}