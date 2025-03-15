use std::collections::{HashMap, HashSet};
use crate::ast::{Module, Stmt, Expr};
use crate::visitor::Visitor;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Variable,
    Function,
    Class,
    Parameter,
    Import,
    ImportFrom,
    Global,
    Nonlocal,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub line: usize,
    pub column: usize,
    pub is_defined: bool,
    pub is_referenced: bool,
    pub is_global: bool,
    pub is_nonlocal: bool,
}

impl Symbol {
    pub fn new(name: &str, symbol_type: SymbolType, line: usize, column: usize) -> Self {
        Symbol {
            name: name.to_string(),
            symbol_type,
            line,
            column,
            is_defined: false,
            is_referenced: false,
            is_global: false,
            is_nonlocal: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub symbols: HashMap<String, Symbol>,
    pub is_function: bool,
    pub is_class: bool,
    pub parent: Option<Box<Scope>>,
    pub children: Vec<Box<Scope>>,
}

impl Scope {
    pub fn new(name: &str, is_function: bool, is_class: bool) -> Self {
        Scope {
            name: name.to_string(),
            symbols: HashMap::new(),
            is_function,
            is_class,
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name.clone(), symbol);
    }

    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn get_symbol_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }

    pub fn add_child(&mut self, mut child: Box<Scope>) {
        child.parent = Some(Box::new(self.clone()));
        self.children.push(child);
    }
}

pub struct SymbolTableBuilder {
    current_scope: Box<Scope>,
    root_scope: Option<Box<Scope>>,
    used_names: HashSet<String>,
    undefined_names: HashSet<String>,
}

impl SymbolTableBuilder {
    pub fn new() -> Self {
        let root_scope = Box::new(Scope::new("module", false, false));
        
        SymbolTableBuilder {
            current_scope: root_scope.clone(),
            root_scope: Some(root_scope),
            used_names: HashSet::new(),
            undefined_names: HashSet::new(),
        }
    }

    pub fn enter_scope(&mut self, name: &str, is_function: bool, is_class: bool) {
        let new_scope = Box::new(Scope::new(name, is_function, is_class));
        
        // Save current scope as parent
        let mut old_scope = self.current_scope.clone();
        
        // Make new scope current
        self.current_scope = new_scope;
        
        // Add current scope as child of old scope
        old_scope.add_child(self.current_scope.clone());
        
        // Update the root if needed
        if self.root_scope.is_some() && self.root_scope.as_ref().unwrap().name == old_scope.name {
            self.root_scope = Some(old_scope);
        }
    }

    pub fn exit_scope(&mut self) {
        if let Some(parent) = &self.current_scope.parent {
            self.current_scope = parent.clone();
        }
        // If we're already at the root scope, don't panic - just silently do nothing
        // This prevents crashes when AST traversal tries to exit the root scope
    }

    pub fn define_symbol(&mut self, name: &str, symbol_type: SymbolType, line: usize, column: usize) {
        let mut symbol = Symbol::new(name, symbol_type, line, column);
        symbol.is_defined = true;
        
        // Check if symbol already exists in current scope
        if let Some(existing) = self.current_scope.get_symbol_mut(name) {
            // Update existing symbol
            existing.is_defined = true;
            existing.line = line;
            existing.column = column;
        } else {
            // Add new symbol
            self.current_scope.add_symbol(symbol);
        }
        
        // Add to used names
        self.used_names.insert(name.to_string());
    }

    fn mark_symbol_in_scope_tree_helper(&self, scope: &mut Box<Scope>, name: &str, target_scope_name: &str) -> bool {
        if scope.name == *target_scope_name {
            if let Some(symbol) = scope.get_symbol_mut(name) {
                symbol.is_referenced = true;
                return true;
            }
            return false;
        }
        
        // Create a vector to track which children were modified
        let mut modified_indices = Vec::new();
        
        // Check each child
        for (i, child) in scope.children.iter_mut().enumerate() {
            if self.mark_symbol_in_scope_tree_helper(child, name, target_scope_name) {
                modified_indices.push(i);
            }
        }
        
        !modified_indices.is_empty()
    }

    pub fn mark_symbol_referenced_in_parent(&mut self, name: &str, parent_scope_name: String) {
        // Extract and clone the root scope before attempting to modify it
        if let Some(root) = self.root_scope.clone() {
            // Create a mutable clone that we can modify
            let mut root_clone = root.clone();
            
            // Call helper method that doesn't borrow self
            let was_modified = self.mark_symbol_in_scope_tree_helper(&mut root_clone, name, &parent_scope_name);
            
            // Update the root scope if needed
            if was_modified {
                self.root_scope = Some(root_clone);
            }
        }
    }

    pub fn reference_symbol(&mut self, name: &str, line: usize, column: usize) {
        // First, check if the symbol exists in current scope without borrowing self mutably
        let found_in_current = self.current_scope.symbols.contains_key(name);
        
        if found_in_current {
            // Get the symbol from current scope and mark it as referenced
            if let Some(existing) = self.current_scope.get_symbol_mut(name) {
                existing.is_referenced = true;
                return;
            }
        }
        
        // Walk up the parent chain checking for the symbol
        let mut found = false;
        let mut parent_scope_name = None;
        
        {
            // Use a separate scope for this check to limit the borrow
            let mut scope = &self.current_scope;
            while let Some(parent) = &scope.parent {
                if parent.symbols.contains_key(name) {
                    found = true;
                    parent_scope_name = Some(parent.name.clone());
                    break;
                }
                scope = parent;
            }
        }
        
        if found {
            // If we found the symbol in a parent scope, mark it as referenced
            if let Some(scope_name) = parent_scope_name {
                self.mark_symbol_referenced_in_parent(name, scope_name);
            }
            return;
        }
        
        // If not found, add to undefined names
        self.undefined_names.insert(name.to_string());
        
        // Also add as a referenced but undefined symbol in current scope
        let mut symbol = Symbol::new(name, SymbolType::Variable, line, column);
        symbol.is_referenced = true;
        self.current_scope.add_symbol(symbol);
    }

    pub fn mark_as_global(&mut self, name: &str) {
        if let Some(existing) = self.current_scope.get_symbol_mut(name) {
            existing.is_global = true;
        } else {
            let mut symbol = Symbol::new(name, SymbolType::Global, 0, 0);
            symbol.is_global = true;
            self.current_scope.add_symbol(symbol);
        }
    }

    pub fn mark_as_nonlocal(&mut self, name: &str) {
        if let Some(existing) = self.current_scope.get_symbol_mut(name) {
            existing.is_nonlocal = true;
        } else {
            let mut symbol = Symbol::new(name, SymbolType::Nonlocal, 0, 0);
            symbol.is_nonlocal = true;
            self.current_scope.add_symbol(symbol);
        }
    }

    pub fn get_root_scope(&self) -> Option<&Box<Scope>> {
        self.root_scope.as_ref()
    }

    pub fn get_undefined_names(&self) -> &HashSet<String> {
        &self.undefined_names
    }

    pub fn print_symbol_table(&self) {
        if let Some(root) = &self.root_scope {
            self.print_scope(root, 0);
        }
    }

    fn print_scope(&self, scope: &Box<Scope>, indent: usize) {
        println!("{}Scope: {}", "  ".repeat(indent), scope.name);
        
        for (name, symbol) in &scope.symbols {
            println!("{}{}: {:?} (defined: {}, referenced: {}, global: {}, nonlocal: {})",
                "  ".repeat(indent + 1),
                name,
                symbol.symbol_type,
                symbol.is_defined,
                symbol.is_referenced,
                symbol.is_global,
                symbol.is_nonlocal
            );
        }
        
        for child in &scope.children {
            self.print_scope(child, indent + 1);
        }
    }
}

impl<'ast> Visitor<'ast, ()> for SymbolTableBuilder {
    fn visit_module(&mut self, module: &'ast Module) -> () {
        for stmt in &module.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) -> () {
        match stmt {
            Stmt::FunctionDef { name, params, body, decorator_list, returns, line, column, is_async: _is_async } => {
                // Define function in current scope
                self.define_symbol(name, SymbolType::Function, *line, *column);
                
                // Visit decorators in current scope
                for decorator in decorator_list {
                    self.visit_expr(decorator);
                }
                
                // Enter new function scope
                self.enter_scope(name, true, false);
                
                // Visit parameters in function scope
                for param in params {
                    self.define_symbol(&param.name, SymbolType::Parameter, *line, *column);
                    
                    if let Some(typ) = &param.typ {
                        self.visit_expr(typ);
                    }
                    
                    if let Some(default) = &param.default {
                        self.visit_expr(default);
                    }
                }
                
                // Visit return annotation if present
                if let Some(ret) = returns {
                    self.visit_expr(ret);
                }
                
                // Visit function body
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                
                // Exit function scope
                self.exit_scope();
            },
            Stmt::ClassDef { name, bases, keywords, body, decorator_list, line, column } => {
                // Define class in current scope
                self.define_symbol(name, SymbolType::Class, *line, *column);
                
                // Visit decorators in current scope
                for decorator in decorator_list {
                    self.visit_expr(decorator);
                }
                
                // Visit bases in current scope
                for base in bases {
                    self.visit_expr(base);
                }
                
                // Visit keywords in current scope
                for (_, value) in keywords {
                    self.visit_expr(value);
                }
                
                // Enter new class scope
                self.enter_scope(name, false, true);
                
                // Visit class body
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                
                // Exit class scope
                self.exit_scope();
            },
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.visit_expr(value);
                }
            },
            Stmt::Delete { targets, .. } => {
                for target in targets {
                    self.visit_expr(target);
                }
            },
            Stmt::Assign { targets, value, .. } => {
                // Visit value first, as it might reference symbols
                self.visit_expr(value);
                
                // Visit targets as definitions
                for target in targets {
                    self.visit_expr_as_target(target);
                }
            },
            Stmt::AugAssign { target, value, .. } => {
                self.visit_expr(value);
                self.visit_expr_as_target(target);
            },
            Stmt::AnnAssign { target, annotation, value, .. } => {
                self.visit_expr(annotation);
                if let Some(value) = value {
                    self.visit_expr(value);
                }
                self.visit_expr_as_target(target);
            },
            Stmt::For { target, iter, body, orelse, .. } => {
                self.visit_expr(iter);
                self.visit_expr_as_target(target);
                
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                
                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
            },
            Stmt::While { test, body, orelse, .. } => {
                self.visit_expr(test);
                
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                
                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
            },
            Stmt::If { test, body, orelse, .. } => {
                self.visit_expr(test);
                
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                
                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
            },
            Stmt::With { items, body, .. } => {
                for (item, target) in items {
                    self.visit_expr(item);
                    if let Some(target) = target {
                        self.visit_expr_as_target(target);
                    }
                }
                
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            },
            Stmt::Raise { exc, cause, .. } => {
                if let Some(exc) = exc {
                    self.visit_expr(exc);
                }
                
                if let Some(cause) = cause {
                    self.visit_expr(cause);
                }
            },
            Stmt::Try { body, handlers, orelse, finalbody, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                
                for handler in handlers {
                    self.visit_except_handler(handler);
                }
                
                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
                
                for stmt in finalbody {
                    self.visit_stmt(stmt);
                }
            },
            Stmt::Assert { test, msg, .. } => {
                self.visit_expr(test);
                
                if let Some(msg) = msg {
                    self.visit_expr(msg);
                }
            },
            Stmt::Import { names, .. } => {
                for alias in names {
                    let import_name = if let Some(asname) = &alias.asname {
                        asname
                    } else {
                        &alias.name
                    };
                    
                    self.define_symbol(import_name, SymbolType::Import, 0, 0);
                }
            },
            Stmt::ImportFrom { names, .. } => {
                for alias in names {
                    let import_name = if let Some(asname) = &alias.asname {
                        asname
                    } else {
                        &alias.name
                    };
                    
                    self.define_symbol(import_name, SymbolType::ImportFrom, 0, 0);
                }
            },
            Stmt::Global { names, .. } => {
                for name in names {
                    self.mark_as_global(name);
                }
            },
            Stmt::Nonlocal { names, .. } => {
                for name in names {
                    self.mark_as_nonlocal(name);
                }
            },
            Stmt::Expr { value, .. } => {
                self.visit_expr(value);
            },
            Stmt::Pass { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {
                // These statements don't introduce symbols
            },
        }
    }

    fn visit_expr(&mut self, expr: &'ast Expr) -> () {
        match expr {
            Expr::Name { id, ctx: _ctx, line, column } => {
                // Reference the name
                self.reference_symbol(id, *line, *column);
            },
            Expr::BoolOp { values, .. } => {
                for value in values {
                    self.visit_expr(value);
                }
            },
            Expr::BinOp { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            },
            Expr::UnaryOp { operand, .. } => {
                self.visit_expr(operand);
            },
            Expr::Lambda { args, body, line, column } => {
                // Enter a new anonymous function scope
                self.enter_scope("lambda", true, false);
                
                // Define parameters
                for param in args {
                    self.define_symbol(&param.name, SymbolType::Parameter, *line, *column);
                    
                    if let Some(typ) = &param.typ {
                        self.visit_expr(typ);
                    }
                    
                    if let Some(default) = &param.default {
                        self.visit_expr(default);
                    }
                }
                
                // Visit body
                self.visit_expr(body);
                
                // Exit lambda scope
                self.exit_scope();
            },
            Expr::IfExp { test, body, orelse, .. } => {
                self.visit_expr(test);
                self.visit_expr(body);
                self.visit_expr(orelse);
            },
            Expr::Dict { keys, values, .. } => {
                for key in keys {
                    if let Some(key) = key {
                        self.visit_expr(key);
                    }
                }
                
                for value in values {
                    self.visit_expr(value);
                }
            },
            Expr::Set { elts, .. } => {
                for elt in elts {
                    self.visit_expr(elt);
                }
            },
            Expr::ListComp { elt, generators, .. } => {
                // Handle list comprehension with its own scope
                self.enter_scope("listcomp", true, false);
                
                for comp in generators {
                    self.visit_comprehension(comp);
                }
                
                self.visit_expr(elt);
                
                self.exit_scope();
            },
            Expr::SetComp { elt, generators, .. } => {
                // Handle set comprehension with its own scope
                self.enter_scope("setcomp", true, false);
                
                for comp in generators {
                    self.visit_comprehension(comp);
                }
                
                self.visit_expr(elt);
                
                self.exit_scope();
            },
            Expr::DictComp { key, value, generators, .. } => {
                // Handle dict comprehension with its own scope
                self.enter_scope("dictcomp", true, false);
                
                for comp in generators {
                    self.visit_comprehension(comp);
                }
                
                self.visit_expr(key);
                self.visit_expr(value);
                
                self.exit_scope();
            },
            Expr::GeneratorExp { elt, generators, .. } => {
                // Handle generator expression with its own scope
                self.enter_scope("genexpr", true, false);
                
                for comp in generators {
                    self.visit_comprehension(comp);
                }
                
                self.visit_expr(elt);
                
                self.exit_scope();
            },
            Expr::Await { value, .. } => {
                self.visit_expr(value);
            },
            Expr::Yield { value, .. } => {
                if let Some(value) = value {
                    self.visit_expr(value);
                }
            },
            Expr::YieldFrom { value, .. } => {
                self.visit_expr(value);
            },
            Expr::Compare { left, comparators, .. } => {
                self.visit_expr(left);
                
                for comparator in comparators {
                    self.visit_expr(comparator);
                }
            },
            Expr::Call { func, args, keywords, .. } => {
                self.visit_expr(func);
                
                for arg in args {
                    self.visit_expr(arg);
                }
                
                for (_, value) in keywords {
                    self.visit_expr(value);
                }
            },
            Expr::Attribute { value, .. } => {
                self.visit_expr(value);
            },
            Expr::Subscript { value, slice, .. } => {
                self.visit_expr(value);
                self.visit_expr(slice);
            },
            Expr::Starred { value, .. } => {
                self.visit_expr(value);
            },
            Expr::List { elts, .. } => {
                for elt in elts {
                    self.visit_expr(elt);
                }
            },
            Expr::Tuple { elts, .. } => {
                for elt in elts {
                    self.visit_expr(elt);
                }
            },
            // Literals and constants don't introduce or reference symbols
            Expr::Num { .. } | Expr::Str { .. } | Expr::Bytes { .. } | 
            Expr::NameConstant { .. } | Expr::Ellipsis { .. } | 
            Expr::Constant { .. } | Expr::FormattedValue { .. } | 
            Expr::JoinedStr { .. } => {},
            Expr::NamedExpr { target, value, .. } => {
                // Visit the value first
                self.visit_expr(value);
                // Visit target as a definition
                self.visit_expr_as_target(target);
            },
        }
    }

    fn visit_expr_as_target(&mut self, expr: &'ast Expr) -> () {
        match expr {
            Expr::Name { id, line, column, .. } => {
                // Define the name as a variable
                self.define_symbol(id, SymbolType::Variable, *line, *column);
            },
            Expr::Tuple { elts, .. } | Expr::List { elts, .. } => {
                // For tuple/list assignments, define each element
                for elt in elts {
                    self.visit_expr_as_target(elt);
                }
            },
            Expr::Starred { value, .. } => {
                self.visit_expr_as_target(value);
            },
            Expr::Attribute { value, .. } => {
                // For attribute assignments, we don't define a symbol but 
                // we need to visit the value expression
                self.visit_expr(value);
            },
            Expr::Subscript { value, slice, .. } => {
                // For subscript assignments, we don't define a symbol but
                // we need to visit both the value and slice expressions
                self.visit_expr(value);
                self.visit_expr(slice);
            },
            _ => {
                // For any other expression type, fall back to regular visit
                self.visit_expr(expr);
            },
        }
    }

    fn visit_except_handler(&mut self, handler: &'ast crate::ast::ExceptHandler) -> () {
        if let Some(typ) = &handler.typ {
            self.visit_expr(typ);
        }
        
        if let Some(name) = &handler.name {
            self.define_symbol(name, SymbolType::Variable, handler.line, handler.column);
        }
        
        for stmt in &handler.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_comprehension(&mut self, comp: &'ast crate::ast::Comprehension) -> () {
        self.visit_expr(comp.iter.as_ref());
        self.visit_expr_as_target(comp.target.as_ref());
        
        for if_expr in &comp.ifs {
            self.visit_expr(if_expr);
        }
    }

    fn visit_alias(&mut self, _alias: &'ast crate::ast::Alias) -> () {
        // Already handled in Import and ImportFrom statements
    }

    fn visit_parameter(&mut self, _param: &'ast crate::ast::Parameter) -> () {
        // Already handled in FunctionDef and Lambda
    }
}