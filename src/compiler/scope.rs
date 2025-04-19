use crate::compiler::types::Type;
use inkwell::values::PointerValue;
use std::collections::HashMap;

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
        let mut stack = Self { scopes: Vec::new() };

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
        for scope in self.scopes.iter().rev() {
            if let Some(ptr) = scope.get_variable(name) {
                return Some(ptr);
            }
        }
        None
    }

    /// Get a variable's type
    pub fn get_type(&self, name: &str) -> Option<&Type> {
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
        let current_index = self.scopes.len() - 1;

        let mut found_ptr = None;
        let mut found_type = None;
        let mut found_scope_index = 0;

        for i in (0..current_index).rev() {
            if let Some(ptr) = self.scopes[i].get_variable(name) {
                found_ptr = Some(*ptr);
                found_type = self.scopes[i].get_type(name).cloned();
                found_scope_index = i;
                break;
            }
        }

        if let (Some(ptr), Some(var_type)) = (found_ptr, found_type) {
            if let Some(current_scope) = self.current_scope_mut() {
                current_scope.add_captured_variable(name.to_string(), ptr);

                current_scope.add_type(name.to_string(), var_type);

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
        if let Some(current_scope) = self.current_scope() {
            if current_scope.is_global(name) {
                if let Some(global_scope) = self.global_scope() {
                    return global_scope.get_variable(name);
                }
            }

            if current_scope.is_nonlocal(name) {
                if let Some(ptr) = current_scope.get_captured_variable(name) {
                    return Some(ptr);
                }

                if let Some(unique_name) = current_scope.get_nonlocal_mapping(name) {
                    if let Some(ptr) = current_scope.get_variable(unique_name) {
                        return Some(ptr);
                    }
                }

                let current_index = self.scopes.len() - 1;

                if current_index > 0 {
                    let parent_scope_index = current_index - 1;

                    if let Some(ptr) = self.scopes[parent_scope_index].get_variable(name) {
                        return Some(ptr);
                    }

                    if self.scopes[parent_scope_index].is_nonlocal(name) {
                        if let Some(parent_unique_name) =
                            self.scopes[parent_scope_index].get_nonlocal_mapping(name)
                        {
                            if let Some(ptr) =
                                self.scopes[parent_scope_index].get_variable(parent_unique_name)
                            {
                                return Some(ptr);
                            }
                        }

                        if parent_scope_index > 0 {
                            let grandparent_scope_index = parent_scope_index - 1;
                            if let Some(ptr) =
                                self.scopes[grandparent_scope_index].get_variable(name)
                            {
                                return Some(ptr);
                            }
                        }
                    }
                }

                for i in (0..current_index - 1).rev() {
                    if let Some(ptr) = self.scopes[i].get_variable(name) {
                        return Some(ptr);
                    }
                }

                return None;
            }
        }

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

    /// Get a variable's type from the scope stack, respecting nonlocal declarations
    pub fn get_type_respecting_declarations(&self, name: &str) -> Option<Type> {
        if let Some(current_scope) = self.current_scope() {
            if current_scope.is_global(name) {
                if let Some(global_scope) = self.global_scope() {
                    return global_scope.get_type(name).cloned();
                }
            }

            if current_scope.is_nonlocal(name) {
                if let Some(unique_name) = current_scope.get_nonlocal_mapping(name) {
                    if let Some(var_type) = current_scope.get_type(unique_name) {
                        return Some(var_type.clone());
                    }
                }

                let current_index = self.scopes.len() - 1;

                if current_index > 0 {
                    let parent_scope_index = current_index - 1;

                    if let Some(var_type) = self.scopes[parent_scope_index].get_type(name) {
                        return Some(var_type.clone());
                    }

                    if self.scopes[parent_scope_index].is_nonlocal(name) {
                        if let Some(parent_unique_name) =
                            self.scopes[parent_scope_index].get_nonlocal_mapping(name)
                        {
                            if let Some(var_type) =
                                self.scopes[parent_scope_index].get_type(parent_unique_name)
                            {
                                return Some(var_type.clone());
                            }
                        }

                        if parent_scope_index > 0 {
                            let grandparent_scope_index = parent_scope_index - 1;
                            if let Some(var_type) =
                                self.scopes[grandparent_scope_index].get_type(name)
                            {
                                return Some(var_type.clone());
                            }
                        }
                    }
                }

                for i in (0..current_index - 1).rev() {
                    if let Some(var_type) = self.scopes[i].get_type(name) {
                        return Some(var_type.clone());
                    }
                }

                return None;
            }
        }

        self.get_type(name).cloned()
    }
}
