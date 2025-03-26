use std::fmt;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CodegenError {
    #[error("Undefined symbol: {0}")]
    UndefinedSymbol(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Failed to compile function: {0}")]
    FunctionError(String),
    
    #[error("Failed to compile expression: {0}")]
    ExpressionError(String),
    
    #[error("Cannot find module: {0}")]
    ModuleNotFound(String),
    
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    
    #[error("Internal compiler error: {0}")]
    InternalError(String),
}

impl CodegenError {
    pub fn undefined_symbol(name: &str) -> Self {
        CodegenError::UndefinedSymbol(name.to_string())
    }
    
    pub fn type_error(msg: &str) -> Self {
        CodegenError::TypeError(msg.to_string())
    }
    
    pub fn function_error(msg: &str) -> Self {
        CodegenError::FunctionError(msg.to_string())
    }
    
    pub fn expression_error(msg: &str) -> Self {
        CodegenError::ExpressionError(msg.to_string())
    }
    
    pub fn module_not_found(name: &str) -> Self {
        CodegenError::ModuleNotFound(name.to_string())
    }
    
    pub fn unsupported_feature(feature: &str) -> Self {
        CodegenError::UnsupportedFeature(feature.to_string())
    }
    
    pub fn internal_error(msg: &str) -> Self {
        CodegenError::InternalError(msg.to_string())
    }
}