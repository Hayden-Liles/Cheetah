use crate::ast::Module;
use crate::compiler::types::TypeError;

mod checker;
mod environment;
mod inference;

pub use checker::TypeChecker;
pub use environment::TypeEnvironment;

/// Result type for type checking operations
pub type TypeResult<T> = Result<T, TypeError>;

/// Main entry point for type checking a module
pub fn check_module(module: &Module) -> TypeResult<()> {
    let mut checker = TypeChecker::new();
    checker.check_module(module)
}
