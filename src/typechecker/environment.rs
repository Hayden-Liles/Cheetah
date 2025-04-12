use crate::compiler::types::Type;
use std::collections::HashMap;

/// Represents a scope in the type environment
#[derive(Debug, Clone)]
pub struct Scope {
    /// Maps variable names to their types
    variables: HashMap<String, Type>,
    /// Maps function names to their types
    functions: HashMap<String, Type>,
    /// Maps class names to their types
    classes: HashMap<String, Type>,
}

impl Scope {
    /// Create a new empty scope
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
        }
    }
}

/// Type environment for tracking variable types during type checking
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// Stack of scopes, with the innermost scope at the end
    scopes: Vec<Scope>,
    /// Current return type for function checking
    current_return_type: Option<Type>,
}

// Make Scope public so it can be accessed from outside
impl Scope {
    /// Get a reference to the variables in this scope
    pub fn get_variables(&self) -> &HashMap<String, Type> {
        &self.variables
    }
}

impl TypeEnvironment {
    /// Create a new empty type environment
    pub fn new() -> Self {
        let mut env = Self {
            scopes: Vec::new(),
            current_return_type: None,
        };

        // Add global scope
        env.push_scope();

        // Add built-in types and functions
        env.add_builtin_types();

        env
    }

    /// Add built-in types and functions to the environment
    fn add_builtin_types(&mut self) {
        // Add built-in functions
        self.add_function("print".to_string(), Type::function(
            vec![Type::Any],
            Type::None
        ));

        self.add_function("len".to_string(), Type::function(
            vec![Type::Any],
            Type::Int
        ));

        self.add_function("str".to_string(), Type::function(
            vec![Type::Any],
            Type::String
        ));

        self.add_function("int".to_string(), Type::function(
            vec![Type::Any],
            Type::Int
        ));

        self.add_function("float".to_string(), Type::function(
            vec![Type::Any],
            Type::Float
        ));

        self.add_function("bool".to_string(), Type::function(
            vec![Type::Any],
            Type::Bool
        ));

        // Add more built-in functions as needed
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pop the innermost scope from the stack
    pub fn pop_scope(&mut self) {
        if !self.scopes.is_empty() {
            self.scopes.pop();
        }
    }

    /// Set the current return type for function checking
    pub fn set_return_type(&mut self, return_type: Type) {
        self.current_return_type = Some(return_type);
    }

    /// Get the current return type for function checking
    pub fn get_return_type(&self) -> Option<&Type> {
        self.current_return_type.as_ref()
    }

    /// Clear the current return type
    pub fn clear_return_type(&mut self) {
        self.current_return_type = None;
    }

    /// Add a variable to the innermost scope
    pub fn add_variable(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            println!("Adding variable '{}' with type {:?}", name, ty);
            scope.variables.insert(name, ty);
        }
    }

    /// Add a function to the innermost scope
    pub fn add_function(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.functions.insert(name, ty);
        }
    }

    /// Update a function's type in the environment
    pub fn update_function(&mut self, name: String, ty: Type) {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter_mut().rev() {
            if scope.functions.contains_key(&name) {
                scope.functions.insert(name, ty);
                return;
            }
        }

        // If the function wasn't found, add it to the innermost scope
        self.add_function(name, ty);
    }

    /// Add a class to the innermost scope
    pub fn add_class(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.classes.insert(name, ty);
        }
    }

    /// Look up a variable's type in the environment
    pub fn lookup_variable(&self, name: &str) -> Option<&Type> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.variables.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Look up a function's type in the environment
    pub fn lookup_function(&self, name: &str) -> Option<&Type> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.functions.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Look up a class's type in the environment
    pub fn lookup_class(&self, name: &str) -> Option<&Type> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.classes.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Check if a name is defined in the environment (variable, function, or class)
    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup_variable(name).is_some() ||
        self.lookup_function(name).is_some() ||
        self.lookup_class(name).is_some()
    }

    /// Get a reference to the current (innermost) scope
    pub fn get_current_scope(&self) -> Option<&Scope> {
        self.scopes.last()
    }

    /// Set a variable's type in the environment
    pub fn set_variable_type(&mut self, name: &str, ty: Type) {
        println!("Setting variable type for '{}' to {:?}", name, ty);
        // Search from innermost to outermost scope
        for scope in self.scopes.iter_mut().rev() {
            if scope.variables.contains_key(name) {
                scope.variables.insert(name.to_string(), ty);
                return;
            }
        }

        // If the variable wasn't found, add it to the innermost scope
        self.add_variable(name.to_string(), ty);
    }

    // We don't need to track parent expressions in the environment
}
