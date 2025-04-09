use std::collections::HashMap;
use inkwell::values::PointerValue;
use crate::compiler::types::Type;

/// Represents a scope in the compilation context
#[derive(Debug)]
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
        }
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

    /// Check if a variable is declared as global in this scope
    pub fn is_global(&self, name: &str) -> bool {
        self.global_vars.contains(&name.to_string())
    }

    /// Check if a variable is declared as nonlocal in this scope
    pub fn is_nonlocal(&self, name: &str) -> bool {
        self.nonlocal_vars.contains(&name.to_string())
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
#[derive(Debug)]
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
                // If it's declared as nonlocal, look it up in outer function scopes
                let mut found_function = false;

                for scope in self.scopes.iter().rev().skip(1) { // Skip current scope
                    if found_function {
                        // We're now in an outer function scope, look for the variable
                        if let Some(ptr) = scope.get_variable(name) {
                            return Some(ptr);
                        }
                    }

                    // Mark when we've found the current function scope
                    if scope.is_function {
                        found_function = true;
                    }
                }
            }
        }

        // If not declared as global or nonlocal, use normal variable lookup
        self.get_variable(name)
    }
}
