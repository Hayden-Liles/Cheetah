use crate::ast::Module;
use crate::compiler::types::TypeError;

mod environment;
mod inference;
mod checker;

pub use environment::TypeEnvironment;
pub use checker::TypeChecker;

/// Result type for type checking operations
pub type TypeResult<T> = Result<T, TypeError>;

/// Main entry point for type checking a module
pub fn check_module(module: &Module) -> TypeResult<()> {
    let mut checker = TypeChecker::new();
    checker.check_module(module)
}
