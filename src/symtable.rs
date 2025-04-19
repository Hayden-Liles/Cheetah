use crate::ast::{Expr, Module, Stmt};
use crate::visitor::Visitor;
use std::collections::{HashMap, HashSet};

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

        let mut old_scope = self.current_scope.clone();

        self.current_scope = new_scope;

        old_scope.add_child(self.current_scope.clone());

        if self.root_scope.is_some() && self.root_scope.as_ref().unwrap().name == old_scope.name {
            self.root_scope = Some(old_scope);
        }
    }

    pub fn exit_scope(&mut self) {
        if let Some(parent) = &self.current_scope.parent {
            self.current_scope = parent.clone();
        }
    }

    pub fn define_symbol(
        &mut self,
        name: &str,
        symbol_type: SymbolType,
        line: usize,
        column: usize,
    ) {
        let mut symbol = Symbol::new(name, symbol_type, line, column);
        symbol.is_defined = true;

        if let Some(existing) = self.current_scope.get_symbol_mut(name) {
            existing.is_defined = true;
            existing.line = line;
            existing.column = column;
        } else {
            self.current_scope.add_symbol(symbol);
        }

        self.used_names.insert(name.to_string());
    }

    fn mark_symbol_in_scope_tree_helper(
        &self,
        scope: &mut Box<Scope>,
        name: &str,
        target_scope_name: &str,
    ) -> bool {
        if scope.name == *target_scope_name {
            if let Some(symbol) = scope.get_symbol_mut(name) {
                symbol.is_referenced = true;
                return true;
            }
            return false;
        }

        let mut modified_indices = Vec::new();

        for (i, child) in scope.children.iter_mut().enumerate() {
            if self.mark_symbol_in_scope_tree_helper(child, name, target_scope_name) {
                modified_indices.push(i);
            }
        }

        !modified_indices.is_empty()
    }

    pub fn mark_symbol_referenced_in_parent(&mut self, name: &str, parent_scope_name: String) {
        if let Some(root) = self.root_scope.clone() {
            let mut root_clone = root.clone();

            let was_modified =
                self.mark_symbol_in_scope_tree_helper(&mut root_clone, name, &parent_scope_name);

            if was_modified {
                self.root_scope = Some(root_clone);
            }
        }
    }

    pub fn reference_symbol(&mut self, name: &str, line: usize, column: usize) {
        let found_in_current = self.current_scope.symbols.contains_key(name);

        if found_in_current {
            if let Some(existing) = self.current_scope.get_symbol_mut(name) {
                existing.is_referenced = true;
                return;
            }
        }

        let mut found = false;
        let mut parent_scope_name = None;

        {
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
            if let Some(scope_name) = parent_scope_name {
                self.mark_symbol_referenced_in_parent(name, scope_name);
            }
            return;
        }

        self.undefined_names.insert(name.to_string());

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
            println!(
                "{}{}: {:?} (defined: {}, referenced: {}, global: {}, nonlocal: {})",
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
            Stmt::FunctionDef {
                name,
                params,
                body,
                decorator_list,
                returns,
                line,
                column,
                is_async: _is_async,
            } => {
                self.define_symbol(name, SymbolType::Function, *line, *column);

                for decorator in decorator_list {
                    self.visit_expr(decorator);
                }

                self.enter_scope(name, true, false);

                for param in params {
                    self.define_symbol(&param.name, SymbolType::Parameter, *line, *column);

                    if let Some(typ) = &param.typ {
                        self.visit_expr(typ);
                    }

                    if let Some(default) = &param.default {
                        self.visit_expr(default);
                    }
                }

                if let Some(ret) = returns {
                    self.visit_expr(ret);
                }

                for stmt in body {
                    self.visit_stmt(stmt);
                }

                self.exit_scope();
            }
            Stmt::ClassDef {
                name,
                bases,
                keywords,
                body,
                decorator_list,
                line,
                column,
            } => {
                self.define_symbol(name, SymbolType::Class, *line, *column);

                for decorator in decorator_list {
                    self.visit_expr(decorator);
                }

                for base in bases {
                    self.visit_expr(base);
                }

                for (_, value) in keywords {
                    self.visit_expr(value);
                }

                self.enter_scope(name, false, true);

                for stmt in body {
                    self.visit_stmt(stmt);
                }

                self.exit_scope();
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.visit_expr(value);
                }
            }
            Stmt::Delete { targets, .. } => {
                for target in targets {
                    self.visit_expr(target);
                }
            }
            Stmt::Assign { targets, value, .. } => {
                self.visit_expr(value);

                for target in targets {
                    self.visit_expr_as_target(target);
                }
            }
            Stmt::AugAssign { target, value, .. } => {
                self.visit_expr(value);
                self.visit_expr_as_target(target);
            }
            Stmt::AnnAssign {
                target,
                annotation,
                value,
                ..
            } => {
                self.visit_expr(annotation);
                if let Some(value) = value {
                    self.visit_expr(value);
                }
                self.visit_expr_as_target(target);
            }
            Stmt::For {
                target,
                iter,
                body,
                orelse,
                ..
            } => {
                self.visit_expr(iter);
                self.visit_expr_as_target(target);

                for stmt in body {
                    self.visit_stmt(stmt);
                }

                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
            }
            Stmt::While {
                test, body, orelse, ..
            } => {
                self.visit_expr(test);

                for stmt in body {
                    self.visit_stmt(stmt);
                }

                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
            }
            Stmt::If {
                test, body, orelse, ..
            } => {
                self.visit_expr(test);

                for stmt in body {
                    self.visit_stmt(stmt);
                }

                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
            }
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
            }
            Stmt::Raise { exc, cause, .. } => {
                if let Some(exc) = exc {
                    self.visit_expr(exc);
                }

                if let Some(cause) = cause {
                    self.visit_expr(cause);
                }
            }
            Stmt::Try {
                body,
                handlers,
                orelse,
                finalbody,
                ..
            } => {
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
            }
            Stmt::Assert { test, msg, .. } => {
                self.visit_expr(test);

                if let Some(msg) = msg {
                    self.visit_expr(msg);
                }
            }
            Stmt::Import { names, .. } => {
                for alias in names {
                    let import_name = if let Some(asname) = &alias.asname {
                        asname
                    } else {
                        &alias.name
                    };

                    self.define_symbol(import_name, SymbolType::Import, 0, 0);
                }
            }
            Stmt::ImportFrom { names, .. } => {
                for alias in names {
                    let import_name = if let Some(asname) = &alias.asname {
                        asname
                    } else {
                        &alias.name
                    };

                    self.define_symbol(import_name, SymbolType::ImportFrom, 0, 0);
                }
            }
            Stmt::Global { names, .. } => {
                for name in names {
                    self.mark_as_global(name);
                }
            }
            Stmt::Nonlocal { names, .. } => {
                for name in names {
                    self.mark_as_nonlocal(name);
                }
            }
            Stmt::Expr { value, .. } => {
                self.visit_expr(value);
            }
            Stmt::Pass { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
            Stmt::Match { subject, cases, .. } => {
                self.visit_expr(subject);

                for (pattern, guard, body) in cases {
                    self.visit_expr(pattern);

                    if let Some(guard_expr) = guard {
                        self.visit_expr(guard_expr);
                    }

                    for stmt in body {
                        self.visit_stmt(stmt);
                    }
                }
            }
        }
    }

    fn visit_expr(&mut self, expr: &'ast Expr) -> () {
        match expr {
            Expr::Name {
                id,
                ctx: _ctx,
                line,
                column,
            } => {
                self.reference_symbol(id, *line, *column);
            }
            Expr::BoolOp { values, .. } => {
                for value in values {
                    self.visit_expr(value);
                }
            }
            Expr::BinOp { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            Expr::UnaryOp { operand, .. } => {
                self.visit_expr(operand);
            }
            Expr::Lambda {
                args,
                body,
                line,
                column,
            } => {
                self.enter_scope("lambda", true, false);

                for param in args {
                    self.define_symbol(&param.name, SymbolType::Parameter, *line, *column);

                    if let Some(typ) = &param.typ {
                        self.visit_expr(typ);
                    }

                    if let Some(default) = &param.default {
                        self.visit_expr(default);
                    }
                }

                self.visit_expr(body);

                self.exit_scope();
            }
            Expr::IfExp {
                test, body, orelse, ..
            } => {
                self.visit_expr(test);
                self.visit_expr(body);
                self.visit_expr(orelse);
            }
            Expr::Dict { keys, values, .. } => {
                for key in keys {
                    if let Some(key) = key {
                        self.visit_expr(key);
                    }
                }

                for value in values {
                    self.visit_expr(value);
                }
            }
            Expr::Set { elts, .. } => {
                for elt in elts {
                    self.visit_expr(elt);
                }
            }
            Expr::ListComp {
                elt, generators, ..
            } => {
                self.enter_scope("listcomp", true, false);

                for comp in generators {
                    self.visit_comprehension(comp);
                }

                self.visit_expr(elt);

                self.exit_scope();
            }
            Expr::SetComp {
                elt, generators, ..
            } => {
                self.enter_scope("setcomp", true, false);

                for comp in generators {
                    self.visit_comprehension(comp);
                }

                self.visit_expr(elt);

                self.exit_scope();
            }
            Expr::DictComp {
                key,
                value,
                generators,
                ..
            } => {
                self.enter_scope("dictcomp", true, false);

                for comp in generators {
                    self.visit_comprehension(comp);
                }

                self.visit_expr(key);
                self.visit_expr(value);

                self.exit_scope();
            }
            Expr::GeneratorExp {
                elt, generators, ..
            } => {
                self.enter_scope("genexpr", true, false);

                for comp in generators {
                    self.visit_comprehension(comp);
                }

                self.visit_expr(elt);

                self.exit_scope();
            }
            Expr::Await { value, .. } => {
                self.visit_expr(value);
            }
            Expr::Yield { value, .. } => {
                if let Some(value) = value {
                    self.visit_expr(value);
                }
            }
            Expr::YieldFrom { value, .. } => {
                self.visit_expr(value);
            }
            Expr::Compare {
                left, comparators, ..
            } => {
                self.visit_expr(left);

                for comparator in comparators {
                    self.visit_expr(comparator);
                }
            }
            Expr::Call {
                func,
                args,
                keywords,
                ..
            } => {
                self.visit_expr(func);

                for arg in args {
                    self.visit_expr(arg);
                }

                for (_, value) in keywords {
                    self.visit_expr(value);
                }
            }
            Expr::Attribute { value, .. } => {
                self.visit_expr(value);
            }
            Expr::Subscript { value, slice, .. } => {
                self.visit_expr(value);
                self.visit_expr(slice);
            }
            Expr::Starred { value, .. } => {
                self.visit_expr(value);
            }
            Expr::List { elts, .. } => {
                for elt in elts {
                    self.visit_expr(elt);
                }
            }
            Expr::Tuple { elts, .. } => {
                for elt in elts {
                    self.visit_expr(elt);
                }
            }
            Expr::Num { .. }
            | Expr::Str { .. }
            | Expr::Bytes { .. }
            | Expr::NameConstant { .. }
            | Expr::Ellipsis { .. }
            | Expr::Constant { .. }
            | Expr::FormattedValue { .. }
            | Expr::JoinedStr { .. } => {}
            Expr::NamedExpr { target, value, .. } => {
                self.visit_expr(value);
                self.visit_expr_as_target(target);
            }
            Expr::Slice {
                lower, upper, step, ..
            } => {
                if let Some(lower_expr) = lower {
                    self.visit_expr(lower_expr);
                }
                if let Some(upper_expr) = upper {
                    self.visit_expr(upper_expr);
                }
                if let Some(step_expr) = step {
                    self.visit_expr(step_expr);
                }
            }
        }
    }

    fn visit_expr_as_target(&mut self, expr: &'ast Expr) -> () {
        match expr {
            Expr::Name {
                id, line, column, ..
            } => {
                self.define_symbol(id, SymbolType::Variable, *line, *column);
            }
            Expr::Tuple { elts, .. } | Expr::List { elts, .. } => {
                for elt in elts {
                    self.visit_expr_as_target(elt);
                }
            }
            Expr::Starred { value, .. } => {
                self.visit_expr_as_target(value);
            }
            Expr::Attribute { value, .. } => {
                self.visit_expr(value);
            }
            Expr::Subscript { value, slice, .. } => {
                self.visit_expr(value);
                self.visit_expr(slice);
            }
            _ => {
                self.visit_expr(expr);
            }
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

    fn visit_alias(&mut self, _alias: &'ast crate::ast::Alias) -> () {}

    fn visit_parameter(&mut self, _param: &'ast crate::ast::Parameter) -> () {}
}
