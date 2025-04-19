// In stmt.rs
use crate::ast::Stmt;
use crate::compiler::context::CompilationContext;
use crate::compiler::types::Type;

pub trait StmtCompiler<'ctx> {
    /// Compile a statement
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String>;

    /// Allocate a variable on the heap
    fn allocate_heap_variable(
        &mut self,
        name: &str,
        ty: &Type,
    ) -> inkwell::values::PointerValue<'ctx>;
}

impl<'ctx> StmtCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        use crate::compiler::stmt_non_recursive::StmtNonRecursive;
        self.compile_stmt_non_recursive(stmt)
    }

    fn allocate_heap_variable(
        &mut self,
        name: &str,
        ty: &Type,
    ) -> inkwell::values::PointerValue<'ctx> {
        self.allocate_heap_variable(name, ty)
    }
}
