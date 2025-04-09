use std::collections::HashMap;
use inkwell::values::PointerValue;
use crate::compiler::types::Type;

/// Represents a closure environment for a nested function
pub struct ClosureEnvironment<'ctx> {
    /// Name of the function this environment belongs to
    pub function_name: String,
    
    /// Map of captured variable names to their storage locations
    pub captured_vars: HashMap<String, PointerValue<'ctx>>,
    
    /// Map of captured variable names to their types
    pub captured_types: HashMap<String, Type>,
    
    /// LLVM struct type for this environment
    pub env_type: Option<inkwell::types::StructType<'ctx>>,
    
    /// Pointer to the environment structure (when allocated)
    pub env_ptr: Option<PointerValue<'ctx>>,
}

impl<'ctx> ClosureEnvironment<'ctx> {
    /// Create a new empty closure environment
    pub fn new(function_name: String) -> Self {
        Self {
            function_name,
            captured_vars: HashMap::new(),
            captured_types: HashMap::new(),
            env_type: None,
            env_ptr: None,
        }
    }
    
    /// Add a variable to the environment
    pub fn add_variable(&mut self, name: String, ptr: PointerValue<'ctx>, ty: Type) {
        self.captured_vars.insert(name.clone(), ptr);
        self.captured_types.insert(name, ty);
    }
    
    /// Get a captured variable's storage location
    pub fn get_variable(&self, name: &str) -> Option<&PointerValue<'ctx>> {
        self.captured_vars.get(name)
    }
    
    /// Get a captured variable's type
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.captured_types.get(name)
    }
    
    /// Check if a variable is captured in this environment
    pub fn has_variable(&self, name: &str) -> bool {
        self.captured_vars.contains_key(name)
    }
    
    /// Get the number of captured variables
    pub fn size(&self) -> usize {
        self.captured_vars.len()
    }
    
    /// Check if the environment is empty (no captured variables)
    pub fn is_empty(&self) -> bool {
        self.captured_vars.is_empty()
    }
}
