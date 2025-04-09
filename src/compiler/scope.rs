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
}

/// Manages a stack of scopes during compilation
#[derive(Debug)]
pub struct ScopeStack<'ctx> {
    /// Stack of scopes, with the innermost scope at the end
    scopes: Vec<Scope<'ctx>>,
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
}
