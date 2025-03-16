use crate::ast::{
    Module, Stmt, Expr, ExceptHandler, Parameter, Comprehension, Alias
};

pub trait Visitor<'ast, T> {
    fn visit_module(&mut self, module: &'ast Module) -> T;
    fn visit_stmt(&mut self, stmt: &'ast Stmt) -> T;
    fn visit_expr(&mut self, expr: &'ast Expr) -> T;
    
    // Add this method to handle targets in assignments
    fn visit_expr_as_target(&mut self, expr: &'ast Expr) -> T {
        // Default implementation just calls visit_expr
        self.visit_expr(expr)
    }
    
    // Optional methods for more specific node types
    fn visit_except_handler(&mut self, handler: &'ast ExceptHandler) -> T;
    fn visit_comprehension(&mut self, comp: &'ast Comprehension) -> T;
    fn visit_alias(&mut self, alias: &'ast Alias) -> T;
    fn visit_parameter(&mut self, param: &'ast Parameter) -> T;
}

// Example of a simple visitor that prints the AST structure
pub struct AstPrinter {
    indent: usize,
}

impl AstPrinter {
    pub fn new() -> Self {
        AstPrinter { indent: 0 }
    }
    
    fn indent(&self) -> String {
        "  ".repeat(self.indent)
    }
    
    fn with_indent<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.indent += 1;
        let result = f(self);
        self.indent -= 1;
        result
    }
}

impl<'ast> Visitor<'ast, String> for AstPrinter {
    fn visit_module(&mut self, module: &'ast Module) -> String {
        let mut result = String::from("Module:\n");
        
        for stmt in &module.body {
            result.push_str(&format!("{}\n", self.visit_stmt(stmt)));
        }
        
        result
    }
    
    fn visit_stmt(&mut self, stmt: &'ast Stmt) -> String {
        match stmt {
            Stmt::FunctionDef { name, params, body, decorator_list, returns, .. } => {
                let mut result = format!("{}FunctionDef: {}\n", self.indent(), name);
                
                // Print decorators
                if !decorator_list.is_empty() {
                    result.push_str(&format!("{}Decorators:\n", self.indent()));
                    self.with_indent(|s| {
                        for decorator in decorator_list {
                            result.push_str(&format!("{}\n", s.visit_expr(decorator)));
                        }
                    });
                }
                
                // Print parameters
                result.push_str(&format!("{}Parameters:\n", self.indent()));
                self.with_indent(|s| {
                    for param in params {
                        result.push_str(&format!("{}\n", s.visit_parameter(param)));
                    }
                });
                
                // Print return annotation if any
                if let Some(ret) = returns {
                    result.push_str(&format!("{}Returns: {}\n", self.indent(), self.visit_expr(ret)));
                }
                
                // Print body
                result.push_str(&format!("{}Body:\n", self.indent()));
                self.with_indent(|s| {
                    for stmt in body {
                        result.push_str(&format!("{}\n", s.visit_stmt(stmt)));
                    }
                });
                
                result
            },
            Stmt::ClassDef { name, bases, keywords, body, decorator_list, .. } => {
                let mut result = format!("{}ClassDef: {}\n", self.indent(), name);
                
                // Print decorators
                if !decorator_list.is_empty() {
                    result.push_str(&format!("{}Decorators:\n", self.indent()));
                    self.with_indent(|s| {
                        for decorator in decorator_list {
                            result.push_str(&format!("{}\n", s.visit_expr(decorator)));
                        }
                    });
                }
                
                // Print bases
                if !bases.is_empty() {
                    result.push_str(&format!("{}Bases:\n", self.indent()));
                    self.with_indent(|s| {
                        for base in bases {
                            result.push_str(&format!("{}\n", s.visit_expr(base)));
                        }
                    });
                }
                
                // Print keywords
                if !keywords.is_empty() {
                    result.push_str(&format!("{}Keywords:\n", self.indent()));
                    self.with_indent(|s| {
                        for (key, value) in keywords {
                            if let Some(key_name) = key {
                                // Regular keyword argument (e.g., metaclass=Meta)
                                result.push_str(&format!("{}{}: {}\n", s.indent(), key_name, s.visit_expr(value)));
                            } else {
                                // **kwargs case
                                result.push_str(&format!("{}**{}\n", s.indent(), s.visit_expr(value)));
                            }
                        }
                    });
                }
                
                // Print body
                result.push_str(&format!("{}Body:\n", self.indent()));
                self.with_indent(|s| {
                    for stmt in body {
                        result.push_str(&format!("{}\n", s.visit_stmt(stmt)));
                    }
                });
                
                result
            },
            Stmt::Return { value, .. } => {
                let mut result = format!("{}Return", self.indent());
                
                if let Some(value) = value {
                    result.push_str(&format!(" {}", self.visit_expr(value)));
                }
                
                result
            },
            Stmt::Delete { targets, .. } => {
                let mut result = format!("{}Delete:\n", self.indent());
                
                self.with_indent(|s| {
                    for target in targets {
                        result.push_str(&format!("{}\n", s.visit_expr(target)));
                    }
                });
                
                result
            },
            Stmt::Assign { targets, value, .. } => {
                let mut result = format!("{}Assign:\n", self.indent());
                
                result.push_str(&format!("{}Targets:\n", self.indent()));
                self.with_indent(|s| {
                    for target in targets {
                        result.push_str(&format!("{}\n", s.visit_expr(target)));
                    }
                });
                
                result.push_str(&format!("{}Value: {}\n", self.indent(), self.visit_expr(value)));
                
                result
            },
            Stmt::AugAssign { target, op, value, .. } => {
                format!(
                    "{}AugAssign: {:?}\n{}Target: {}\n{}Value: {}", 
                    self.indent(), op, 
                    self.indent(), self.visit_expr(target),
                    self.indent(), self.visit_expr(value)
                )
            },
            Stmt::AnnAssign { target, annotation, value, .. } => {
                let mut result = format!("{}AnnAssign:\n", self.indent());
                
                result.push_str(&format!("{}Target: {}\n", self.indent(), self.visit_expr(target)));
                result.push_str(&format!("{}Annotation: {}\n", self.indent(), self.visit_expr(annotation)));
                
                if let Some(value) = value {
                    result.push_str(&format!("{}Value: {}\n", self.indent(), self.visit_expr(value)));
                }
                
                result
            },
            Stmt::For { target, iter, body, orelse, .. } => {
                let mut result = format!("{}For:\n", self.indent());
                
                result.push_str(&format!("{}Target: {}\n", self.indent(), self.visit_expr(target)));
                result.push_str(&format!("{}Iter: {}\n", self.indent(), self.visit_expr(iter)));
                
                result.push_str(&format!("{}Body:\n", self.indent()));
                self.with_indent(|s| {
                    for stmt in body {
                        result.push_str(&format!("{}\n", s.visit_stmt(stmt)));
                    }
                });
                
                if !orelse.is_empty() {
                    result.push_str(&format!("{}Else:\n", self.indent()));
                    self.with_indent(|s| {
                        for stmt in orelse {
                            result.push_str(&format!("{}\n", s.visit_stmt(stmt)));
                        }
                    });
                }
                
                result
            },
            // Implement other statement types similarly...
            _ => format!("{}Unimplemented statement type: {:?}", self.indent(), stmt),
        }
    }
    
    fn visit_expr(&mut self, expr: &'ast Expr) -> String {
        match expr {
            Expr::BoolOp { op, values, .. } => {
                let mut result = format!("{}BoolOp: {:?}\n", self.indent(), op);
                
                result.push_str(&format!("{}Values:\n", self.indent()));
                self.with_indent(|s| {
                    for value in values {
                        result.push_str(&format!("{}\n", s.visit_expr(value)));
                    }
                });
                
                result
            },
            Expr::BinOp { left, op, right, .. } => {
                format!(
                    "{}BinOp: {:?}\n{}Left: {}\n{}Right: {}", 
                    self.indent(), op, 
                    self.indent(), self.visit_expr(left),
                    self.indent(), self.visit_expr(right)
                )
            },
            Expr::UnaryOp { op, operand, .. } => {
                format!(
                    "{}UnaryOp: {:?}\n{}Operand: {}", 
                    self.indent(), op, 
                    self.indent(), self.visit_expr(operand)
                )
            },
            Expr::Name { id, ctx, .. } => {
                format!("{}Name: {} (ctx: {:?})", self.indent(), id, ctx)
            },
            Expr::Num { value, .. } => {
                format!("{}Num: {:?}", self.indent(), value)
            },
            Expr::Str { value, .. } => {
                format!("{}Str: \"{}\"", self.indent(), value)
            },
            // Implement other expression types similarly...
            _ => format!("{}Unimplemented expression type: {:?}", self.indent(), expr),
        }
    }
    
    fn visit_except_handler(&mut self, handler: &'ast ExceptHandler) -> String {
        let mut result = format!("{}ExceptHandler:\n", self.indent());
        
        if let Some(typ) = &handler.typ {
            result.push_str(&format!("{}Type: {}\n", self.indent(), self.visit_expr(typ)));
        }
        
        if let Some(name) = &handler.name {
            result.push_str(&format!("{}Name: {}\n", self.indent(), name));
        }
        
        result.push_str(&format!("{}Body:\n", self.indent()));
        self.with_indent(|s| {
            for stmt in &handler.body {
                result.push_str(&format!("{}\n", s.visit_stmt(stmt)));
            }
        });
        
        result
    }
    
    fn visit_comprehension(&mut self, comp: &'ast Comprehension) -> String {
        let mut result = format!("{}Comprehension:\n", self.indent());
        
        result.push_str(&format!("{}Target: {}\n", self.indent(), self.visit_expr(&comp.target)));
        result.push_str(&format!("{}Iter: {}\n", self.indent(), self.visit_expr(&comp.iter)));
        
        if !comp.ifs.is_empty() {
            result.push_str(&format!("{}Ifs:\n", self.indent()));
            self.with_indent(|s| {
                for if_expr in &comp.ifs {
                    result.push_str(&format!("{}\n", s.visit_expr(if_expr)));
                }
            });
        }
        
        if comp.is_async {
            result.push_str(&format!("{}Async: true\n", self.indent()));
        }
        
        result
    }
    
    fn visit_alias(&mut self, alias: &'ast Alias) -> String {
        let mut result = format!("{}Alias: {}", self.indent(), alias.name);
        
        if let Some(asname) = &alias.asname {
            result.push_str(&format!(" as {}", asname));
        }
        
        result
    }
    
    fn visit_parameter(&mut self, param: &'ast Parameter) -> String {
        let mut result = format!("{}Parameter: {}", self.indent(), param.name);
        
        if let Some(typ) = &param.typ {
            result.push_str(&format!(" (type: {})", self.visit_expr(typ)));
        }
        
        if let Some(default) = &param.default {
            result.push_str(&format!(" = {}", self.visit_expr(default)));
        }
        
        result
    }
}

// Define a trait for nodes that can be visited
pub trait Visitable<'ast, T> {
    fn accept(&'ast self, visitor: &mut dyn Visitor<'ast, T>) -> T;
}

// Implement the trait for each AST node type
impl<'ast, T> Visitable<'ast, T> for Module {
    fn accept(&'ast self, visitor: &mut dyn Visitor<'ast, T>) -> T {
        visitor.visit_module(self)
    }
}

impl<'ast, T> Visitable<'ast, T> for Stmt {
    fn accept(&'ast self, visitor: &mut dyn Visitor<'ast, T>) -> T {
        visitor.visit_stmt(self)
    }
}

impl<'ast, T> Visitable<'ast, T> for Expr {
    fn accept(&'ast self, visitor: &mut dyn Visitor<'ast, T>) -> T {
        visitor.visit_expr(self)
    }
}

impl<'ast, T> Visitable<'ast, T> for ExceptHandler {
    fn accept(&'ast self, visitor: &mut dyn Visitor<'ast, T>) -> T {
        visitor.visit_except_handler(self)
    }
}

impl<'ast, T> Visitable<'ast, T> for Comprehension {
    fn accept(&'ast self, visitor: &mut dyn Visitor<'ast, T>) -> T {
        visitor.visit_comprehension(self)
    }
}

impl<'ast, T> Visitable<'ast, T> for Alias {
    fn accept(&'ast self, visitor: &mut dyn Visitor<'ast, T>) -> T {
        visitor.visit_alias(self)
    }
}

impl<'ast, T> Visitable<'ast, T> for Parameter {
    fn accept(&'ast self, visitor: &mut dyn Visitor<'ast, T>) -> T {
        visitor.visit_parameter(self)
    }
}