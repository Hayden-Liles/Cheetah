use std::collections::HashMap;
use inkwell::values::PointerValue;
use crate::compiler::types::Type;

/// Represents a scope in the compilation context
#[derive(Debug, Clone)]
pub struct Scope<'ctx> {
    /// Maps variable names to their storage locations
    pub variables: HashMap<String, PointerValue<'ctx>>,
    /// Maps variable names to their types
    pub types: HashMap<String, Type>,
    /// Variables declared as global in this scope
    pub global_vars: Vec<String>,
    /// Variables declared as nonlocal in this scope
    pub nonlocal_vars: Vec<String>,
    /// Whether this scope is a function scope
    pub is_function: bool,
    /// Whether this scope is a loop scope
    pub is_loop: bool,
    /// Whether this scope is a class scope
    pub is_class: bool,
    /// Maps nonlocal variable names to their captured pointers
    /// This is used for nested functions to access variables from outer scopes
    pub captured_vars: HashMap<String, PointerValue<'ctx>>,
    /// Variables that need to be heap-allocated because they're accessed by nested functions
    pub heap_vars: Vec<String>,
    /// Maps original variable names to their unique names in the current scope
    /// This is used for nonlocal variables to avoid LLVM's dominance validation issues
    pub nonlocal_mappings: HashMap<String, String>,
}

impl<'ctx> Scope<'ctx> {
    /// Create a new empty scope
    pub fn new(is_function: bool, is_loop: bool, is_class: bool) -> Self {
        Self {
            variables: HashMap::new(),
            types: HashMap::new(),
            global_vars: Vec::new(),
            nonlocal_vars: Vec::new(),
            is_function,
            is_loop,
            is_class,
            captured_vars: HashMap::new(),
            heap_vars: Vec::new(),
            nonlocal_mappings: HashMap::new(),
        }
    }

    /// Add a mapping from an original variable name to a unique name
    pub fn add_nonlocal_mapping(&mut self, original_name: String, unique_name: String) {
        self.nonlocal_mappings.insert(original_name, unique_name);
    }

    /// Get the unique name for a nonlocal variable
    pub fn get_nonlocal_mapping(&self, original_name: &str) -> Option<&String> {
        self.nonlocal_mappings.get(original_name)
    }

    /// Get a variable's storage location
    pub fn get_variable(&self, name: &str) -> Option<&PointerValue<'ctx>> {
        self.variables.get(name)
    }

    /// Get a variable's type
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    /// Add a variable to this scope
    pub fn add_variable(&mut self, name: String, ptr: PointerValue<'ctx>, ty: Type) {
        self.variables.insert(name.clone(), ptr);
        self.types.insert(name, ty);
    }

    /// Add a type to this scope
    pub fn add_type(&mut self, name: String, ty: Type) {
        self.types.insert(name, ty);
    }

    /// Check if a variable is declared as global in this scope
    pub fn is_global(&self, name: &str) -> bool {
        self.global_vars.contains(&name.to_string())
    }

    /// Check if a variable is declared as nonlocal in this scope
    pub fn is_nonlocal(&self, name: &str) -> bool {
        self.nonlocal_vars.contains(&name.to_string())
    }

    /// Add a captured variable to this scope
    pub fn add_captured_variable(&mut self, name: String, ptr: PointerValue<'ctx>) {
        self.captured_vars.insert(name, ptr);
    }

    /// Get a captured variable's storage location
    pub fn get_captured_variable(&self, name: &str) -> Option<&PointerValue<'ctx>> {
        self.captured_vars.get(name)
    }

    /// Check if a variable is captured in this scope
    pub fn is_captured(&self, name: &str) -> bool {
        self.captured_vars.contains_key(name)
    }

    /// Mark a variable as needing heap allocation
    pub fn mark_as_heap_var(&mut self, name: String) {
        if !self.heap_vars.contains(&name) {
            self.heap_vars.push(name);
        }
    }

    /// Check if a variable needs heap allocation
    pub fn is_heap_var(&self, name: &str) -> bool {
        self.heap_vars.contains(&name.to_string())
    }

    /// Declare a variable as global in this scope
    pub fn declare_global(&mut self, name: String) {
        if !self.global_vars.contains(&name) {
            self.global_vars.push(name);
        }
    }

    /// Declare a variable as nonlocal in this scope
    pub fn declare_nonlocal(&mut self, name: String) {
        if !self.nonlocal_vars.contains(&name) {
            self.nonlocal_vars.push(name);
        }
    }
}

/// Manages a stack of scopes during compilation
#[derive(Debug, Clone)]
pub struct ScopeStack<'ctx> {
    /// Stack of scopes, with the innermost scope at the end
    pub scopes: Vec<Scope<'ctx>>,
}

impl<'ctx> ScopeStack<'ctx> {
    /// Create a new empty scope stack
    pub fn new() -> Self {
        let mut stack = Self {
            scopes: Vec::new(),
        };

        // Add global scope
        stack.push_scope(false, false, false);

        stack
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self, is_function: bool, is_loop: bool, is_class: bool) {
        self.scopes.push(Scope::new(is_function, is_loop, is_class));
    }

    /// Pop the innermost scope from the stack
    pub fn pop_scope(&mut self) -> Option<Scope<'ctx>> {
        self.scopes.pop()
    }

    /// Get the innermost scope
    pub fn current_scope(&self) -> Option<&Scope<'ctx>> {
        self.scopes.last()
    }

    /// Get the innermost scope mutably
    pub fn current_scope_mut(&mut self) -> Option<&mut Scope<'ctx>> {
        self.scopes.last_mut()
    }

    /// Get a variable's storage location
    pub fn get_variable(&self, name: &str) -> Option<&PointerValue<'ctx>> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(ptr) = scope.get_variable(name) {
                return Some(ptr);
            }
        }
        None
    }

    /// Get a variable's type
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get_type(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Add a variable to the current scope
    pub fn add_variable(&mut self, name: String, ptr: PointerValue<'ctx>, ty: Type) {
        if let Some(scope) = self.current_scope_mut() {
            scope.add_variable(name, ptr, ty);
        }
    }

    /// Find the innermost function scope
    pub fn find_function_scope(&self) -> Option<&Scope<'ctx>> {
        for scope in self.scopes.iter().rev() {
            if scope.is_function {
                return Some(scope);
            }
        }
        None
    }

    /// Find the innermost loop scope
    pub fn find_loop_scope(&self) -> Option<&Scope<'ctx>> {
        for scope in self.scopes.iter().rev() {
            if scope.is_loop {
                return Some(scope);
            }
        }
        None
    }

    /// Find the innermost class scope
    pub fn find_class_scope(&self) -> Option<&Scope<'ctx>> {
        for scope in self.scopes.iter().rev() {
            if scope.is_class {
                return Some(scope);
            }
        }
        None
    }

    /// Get the global scope
    pub fn global_scope(&self) -> Option<&Scope<'ctx>> {
        self.scopes.first()
    }

    /// Get the global scope mutably
    pub fn global_scope_mut(&mut self) -> Option<&mut Scope<'ctx>> {
        self.scopes.first_mut()
    }

    /// Declare a variable as global in the current scope
    pub fn declare_global(&mut self, name: String) {
        if let Some(scope) = self.current_scope_mut() {
            scope.declare_global(name);
        }
    }

    /// Declare a variable as nonlocal in the current scope
    pub fn declare_nonlocal(&mut self, name: String) {
        if let Some(scope) = self.current_scope_mut() {
            scope.declare_nonlocal(name);
        }
    }

    /// Capture a variable from an outer scope for use in the current scope
    /// Returns true if the variable was found and captured, false otherwise
    pub fn capture_variable(&mut self, name: &str) -> bool {
        // Get the current scope index
        let current_index = self.scopes.len() - 1;

        // Look for the variable in outer scopes
        let mut found_ptr = None;
        let mut found_type = None;
        let mut found_scope_index = 0;

        // Skip the current scope and look in all outer scopes
        for i in (0..current_index).rev() {
            if let Some(ptr) = self.scopes[i].get_variable(name) {
                found_ptr = Some(*ptr);
                found_type = self.scopes[i].get_type(name).cloned();
                found_scope_index = i;
                break;
            }
        }

        // If we found the variable, capture it in the current scope
        if let (Some(ptr), Some(var_type)) = (found_ptr, found_type) {
            if let Some(current_scope) = self.current_scope_mut() {
                // Add the variable to the captured variables
                current_scope.add_captured_variable(name.to_string(), ptr);

                // Also add the type information
                current_scope.add_type(name.to_string(), var_type);

                // Mark the variable as needing heap allocation in its original scope
                self.scopes[found_scope_index].mark_as_heap_var(name.to_string());

                return true;
            }
        }

        false
    }

    /// Mark a variable as needing heap allocation in the current scope
    pub fn mark_as_heap_var(&mut self, name: String) {
        if let Some(scope) = self.current_scope_mut() {
            scope.mark_as_heap_var(name);
        }
    }

    /// Check if a variable needs heap allocation in the current scope
    pub fn is_heap_var(&self, name: &str) -> bool {
        if let Some(scope) = self.current_scope() {
            scope.is_heap_var(name)
        } else {
            false
        }
    }

    /// Get a variable's storage location, respecting global and nonlocal declarations
    pub fn get_variable_respecting_declarations(&self, name: &str) -> Option<&PointerValue<'ctx>> {
        // Check if the variable is declared as global in the current scope
        if let Some(current_scope) = self.current_scope() {
            if current_scope.is_global(name) {
                // If it's declared as global, look it up in the global scope
                if let Some(global_scope) = self.global_scope() {
                    return global_scope.get_variable(name);
                }
            }

            // Check if the variable is declared as nonlocal
            if current_scope.is_nonlocal(name) {
                // First check if it's a captured variable in the current scope
                if let Some(ptr) = current_scope.get_captured_variable(name) {
                    return Some(ptr);
                }

                // Check if there's a mapping for this nonlocal variable
                if let Some(unique_name) = current_scope.get_nonlocal_mapping(name) {
                    // Look up the unique name in the current scope
                    if let Some(ptr) = current_scope.get_variable(unique_name) {
                        return Some(ptr);
                    }
                }

                // If it's declared as nonlocal, look it up in outer scopes (not just function scopes)
                // Start from the current scope's index - 1 (the outer scope)
                let current_index = self.scopes.len() - 1;

                // Look in all outer scopes
                for i in (0..current_index).rev() {
                    if let Some(ptr) = self.scopes[i].get_variable(name) {
                        return Some(ptr);
                    }
                }

                // If we get here, the nonlocal variable wasn't found
                return None;
            }
        }

        // If not declared as global or nonlocal, use normal variable lookup
        self.get_variable(name)
    }

    /// Add a mapping from an original variable name to a unique name in the current scope
    pub fn add_nonlocal_mapping(&mut self, original_name: String, unique_name: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.add_nonlocal_mapping(original_name, unique_name);
        }
    }

    /// Get the unique name for a nonlocal variable in the current scope
    pub fn get_nonlocal_mapping(&self, original_name: &str) -> Option<&String> {
        if let Some(scope) = self.scopes.last() {
            scope.get_nonlocal_mapping(original_name)
        } else {
            None
        }
    }
}
