use crate::ast::{Module, Stmt, Expr, BoolOperator, Operator, UnaryOperator, CmpOperator};
use crate::visitor::Visitor;

pub struct CodeFormatter {
    indent_level: usize,
    indent_size: usize,
    output: String,
}

impl CodeFormatter {
    pub fn new(indent_size: usize) -> Self {
        CodeFormatter {
            indent_level: 0,
            indent_size,
            output: String::new(),
        }
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }

    fn indent(&self) -> String {
        " ".repeat(self.indent_level * self.indent_size)
    }

    fn write(&mut self, text: &str) {
        self.output.push_str(text);
    }

    fn write_indented(&mut self, text: &str) {
        self.output.push_str(&self.indent());
        self.output.push_str(text);
    }

    fn write_line(&mut self, text: &str) {
        self.write_indented(text);
        self.output.push('\n');
    }

    fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn format_operator(&self, op: &Operator) -> &'static str {
        match op {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mult => "*",
            Operator::MatMult => "@",
            Operator::Div => "/",
            Operator::FloorDiv => "//",
            Operator::Mod => "%",
            Operator::Pow => "**",
            Operator::LShift => "<<",
            Operator::RShift => ">>",
            Operator::BitOr => "|",
            Operator::BitXor => "^",
            Operator::BitAnd => "&",
        }
    }

    fn format_unary_operator(&self, op: &UnaryOperator) -> &'static str {
        match op {
            UnaryOperator::Invert => "~",
            UnaryOperator::Not => "not ",
            UnaryOperator::UAdd => "+",
            UnaryOperator::USub => "-",
        }
    }

    fn format_bool_operator(&self, op: &BoolOperator) -> &'static str {
        match op {
            BoolOperator::And => "and",
            BoolOperator::Or => "or",
        }
    }

    fn format_cmp_operator(&self, op: &CmpOperator) -> &'static str {
        match op {
            CmpOperator::Eq => "==",
            CmpOperator::NotEq => "!=",
            CmpOperator::Lt => "<",
            CmpOperator::LtE => "<=",
            CmpOperator::Gt => ">",
            CmpOperator::GtE => ">=",
            CmpOperator::Is => "is",
            CmpOperator::IsNot => "is not",
            CmpOperator::In => "in",
            CmpOperator::NotIn => "not in",
        }
    }
}

impl<'ast> Visitor<'ast, ()> for CodeFormatter {
    fn visit_module(&mut self, module: &'ast Module) -> () {
        for (i, stmt) in module.body.iter().enumerate() {
            self.visit_stmt(stmt);
            
            // Add blank line between top-level statements, except for consecutive
            // imports or consecutive simple statements
            if i < module.body.len() - 1 {
                // Use as_ref() to get a reference to the inner Stmt
                match (stmt.as_ref(), module.body[i + 1].as_ref()) {
                    (Stmt::Import { .. }, Stmt::Import { .. }) => {},
                    (Stmt::ImportFrom { .. }, Stmt::ImportFrom { .. }) => {},
                    (Stmt::Import { .. }, Stmt::ImportFrom { .. }) => {},
                    (Stmt::ImportFrom { .. }, Stmt::Import { .. }) => {},
                    
                    (Stmt::Expr { .. }, Stmt::Expr { .. }) => {},
                    (Stmt::Assign { .. }, Stmt::Assign { .. }) => {},
                    (Stmt::AugAssign { .. }, Stmt::AugAssign { .. }) => {},
                    
                    // Add TWO newlines after function or class definitions
                    (Stmt::FunctionDef { .. }, _) | (Stmt::ClassDef { .. }, _) => {
                        self.write("\n\n");
                    },
                    
                    // Default case - add one newline
                    _ => self.write("\n"),
                }
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) -> () {
        match stmt {
            Stmt::FunctionDef { name, params, body, decorator_list, returns, line: _line, column: _column, is_async: _is_async } => {
                // Write decorators
                for decorator in decorator_list {
                    self.write_indented("@");
                    self.visit_expr(&**decorator); // Dereference Box<Expr>
                    self.write("\n");
                }
                
                // Write function definition
                self.write_indented("def ");
                self.write(name);
                self.write("(");
                
                // Write parameters
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    
                    self.write(&param.name);
                    
                    if let Some(typ) = &param.typ {
                        self.write(": ");
                        self.visit_expr(&**typ);
                    }
                    
                    if let Some(default) = &param.default {
                        self.write(" = ");
                        self.visit_expr(&**default);
                    }
                }
                
                self.write(")");
                
                // Write return type annotation
                if let Some(ret) = returns {
                    self.write(" -> ");
                    self.visit_expr(&**ret);
                }
                
                self.write(":\n");
                
                // Write function body
                self.increase_indent();
                
                if body.is_empty() {
                    self.write_line("pass");
                } else {
                    for stmt in body {
                        self.visit_stmt(&**stmt);
                    }
                }
                
                self.decrease_indent();
            },
            Stmt::ClassDef { name, bases, keywords, body, decorator_list, line: _line, column: _column } => {
                // Write decorators
                for decorator in decorator_list {
                    self.write_indented("@");
                    self.visit_expr(&**decorator);
                    self.write("\n");
                }
                
                // Write class definition
                self.write_indented("class ");
                self.write(name);
                
                if !bases.is_empty() || !keywords.is_empty() {
                    self.write("(");
                    
                    // Write base classes
                    for (i, base) in bases.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.visit_expr(&**base);
                    }
                    
                    // Write keyword arguments
                    if !bases.is_empty() && !keywords.is_empty() {
                        self.write(", ");
                    }
                    
                    for (i, (key, value)) in keywords.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.write(key);
                        self.write("=");
                        self.visit_expr(&**value);
                    }
                    
                    self.write(")");
                }
                
                self.write(":\n");
                
                // Write class body
                self.increase_indent();
                
                if body.is_empty() {
                    self.write_line("pass");
                } else {
                    for stmt in body {
                        self.visit_stmt(&**stmt);
                    }
                }
                
                self.decrease_indent();
            },
            Stmt::Return { value, line: _, column: _ } => {
                self.write_indented("return");
                
                if let Some(value) = value {
                    self.write(" ");
                    self.visit_expr(&**value);
                }
                
                self.write("\n");
            },
            Stmt::Delete { targets, line: _, column: _ } => {
                self.write_indented("del ");
                
                for (i, target) in targets.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.visit_expr(&**target);
                }
                
                self.write("\n");
            },
            Stmt::Assign { targets, value, line: _, column: _ } => {
                self.write_indented("");
                
                for (i, target) in targets.iter().enumerate() {
                    if i > 0 {
                        self.write(" = ");
                    }
                    self.visit_expr(&**target);
                }
                
                self.write(" = ");
                self.visit_expr(&**value);
                self.write("\n");
            },
            Stmt::AugAssign { target, op, value, line: _, column: _ } => {
                self.write_indented("");
                self.visit_expr(&**target);
                self.write(" ");
                self.write(self.format_operator(op));
                self.write("= ");
                self.visit_expr(&**value);
                self.write("\n");
            },
            Stmt::AnnAssign { target, annotation, value, line: _, column: _ } => {
                self.write_indented("");
                self.visit_expr(&**target);
                self.write(": ");
                self.visit_expr(&**annotation);
                
                if let Some(value) = value {
                    self.write(" = ");
                    self.visit_expr(&**value);
                }
                
                self.write("\n");
            },
            Stmt::For { target, iter, body, orelse, line: _, column: _, is_async: _is_async } => {
                self.write_indented("for ");
                self.visit_expr(&**target);
                self.write(" in ");
                self.visit_expr(&**iter);
                self.write(":\n");
                
                self.increase_indent();
                
                if body.is_empty() {
                    self.write_line("pass");
                } else {
                    for stmt in body {
                        self.visit_stmt(&**stmt);
                    }
                }
                
                self.decrease_indent();
                
                if !orelse.is_empty() {
                    self.write_line("else:");
                    self.increase_indent();
                    
                    for stmt in orelse {
                        self.visit_stmt(&**stmt);
                    }
                    
                    self.decrease_indent();
                }
            },
            Stmt::While { test, body, orelse, line: _, column: _ } => {
                self.write_indented("while ");
                self.visit_expr(&**test);
                self.write(":\n");
                
                self.increase_indent();
                
                if body.is_empty() {
                    self.write_line("pass");
                } else {
                    for stmt in body {
                        self.visit_stmt(&**stmt);
                    }
                }
                
                self.decrease_indent();
                
                if !orelse.is_empty() {
                    self.write_line("else:");
                    self.increase_indent();
                    
                    for stmt in orelse {
                        self.visit_stmt(&**stmt);
                    }
                    
                    self.decrease_indent();
                }
            },
            Stmt::If { test, body, orelse, line: _, column: _ } => {
                self.write_indented("if ");
                self.visit_expr(&**test);
                self.write(":\n");
                
                self.increase_indent();
                
                if body.is_empty() {
                    self.write_line("pass");
                } else {
                    for stmt in body {
                        self.visit_stmt(&**stmt);
                    }
                }
                
                self.decrease_indent();
                
                // Handle elif blocks
                if orelse.len() == 1 {
                    // Need to use as_ref() to get a reference to the inner Stmt
                    if let Stmt::If { .. } = orelse[0].as_ref() {
                        self.write_indented("el");
                        self.visit_stmt(&*orelse[0]);
                        return;
                    }
                }
                
                if !orelse.is_empty() {
                    self.write_line("else:");
                    self.increase_indent();
                    
                    for stmt in orelse {
                        self.visit_stmt(&**stmt);
                    }
                    
                    self.decrease_indent();
                }
            },
            Stmt::With { items, body, line: _, column: _, is_async: _ } => {
                self.write_indented("with ");
                
                for (i, (item, target)) in items.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    
                    self.visit_expr(&**item);
                    
                    if let Some(target) = target {
                        self.write(" as ");
                        self.visit_expr(&**target);
                    }
                }
                
                self.write(":\n");
                
                self.increase_indent();
                
                if body.is_empty() {
                    self.write_line("pass");
                } else {
                    for stmt in body {
                        self.visit_stmt(&**stmt);
                    }
                }
                
                self.decrease_indent();
            },
            Stmt::Raise { exc, cause, line: _, column: _ } => {
                self.write_indented("raise");
                
                if let Some(exc) = exc {
                    self.write(" ");
                    self.visit_expr(&**exc);
                    
                    if let Some(cause) = cause {
                        self.write(" from ");
                        self.visit_expr(&**cause);
                    }
                }
                
                self.write("\n");
            },
            Stmt::Try { body, handlers, orelse, finalbody, line: _, column: _ } => {
                self.write_line("try:");
                
                self.increase_indent();
                
                if body.is_empty() {
                    self.write_line("pass");
                } else {
                    for stmt in body {
                        self.visit_stmt(&**stmt);
                    }
                }
                
                self.decrease_indent();
                
                for handler in handlers {
                    self.write_indented("except");
                    
                    if let Some(typ) = &handler.typ {
                        self.write(" ");
                        self.visit_expr(&**typ);
                        
                        if let Some(name) = &handler.name {
                            self.write(" as ");
                            self.write(name);
                        }
                    }
                    
                    self.write(":\n");
                    
                    self.increase_indent();
                    
                    if handler.body.is_empty() {
                        self.write_line("pass");
                    } else {
                        for stmt in &handler.body {
                            self.visit_stmt(&**stmt);
                        }
                    }
                    
                    self.decrease_indent();
                }
                
                if !orelse.is_empty() {
                    self.write_line("else:");
                    self.increase_indent();
                    
                    for stmt in orelse {
                        self.visit_stmt(&**stmt);
                    }
                    
                    self.decrease_indent();
                }
                
                if !finalbody.is_empty() {
                    self.write_line("finally:");
                    self.increase_indent();
                    
                    for stmt in finalbody {
                        self.visit_stmt(&**stmt);
                    }
                    
                    self.decrease_indent();
                }
            },
            Stmt::Assert { test, msg, line: _, column: _ } => {
                self.write_indented("assert ");
                self.visit_expr(&**test);
                
                if let Some(msg) = msg {
                    self.write(", ");
                    self.visit_expr(&**msg);
                }
                
                self.write("\n");
            },
            Stmt::Import { names, line: _, column: _ } => {
                self.write_indented("import ");
                
                for (i, alias) in names.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    
                    self.write(&alias.name);
                    
                    if let Some(asname) = &alias.asname {
                        self.write(" as ");
                        self.write(asname);
                    }
                }
                
                self.write("\n");
            },
            Stmt::ImportFrom { module, names, level, line: _, column: _ } => {
                self.write_indented("from ");
                
                // Write relative import dots
                for _ in 0..*level {
                    self.write(".");
                }
                
                if let Some(module) = module {
                    self.write(module);
                }
                
                self.write(" import ");
                
                if names.len() == 1 && names[0].name == "*" {
                    self.write("*");
                } else {
                    for (i, alias) in names.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        
                        self.write(&alias.name);
                        
                        if let Some(asname) = &alias.asname {
                            self.write(" as ");
                            self.write(asname);
                        }
                    }
                }
                
                self.write("\n");
            },
            Stmt::Global { names, line: _, column: _ } => {
                self.write_indented("global ");
                
                for (i, name) in names.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    
                    self.write(name);
                }
                
                self.write("\n");
            },
            Stmt::Nonlocal { names, line: _, column: _ } => {
                self.write_indented("nonlocal ");
                
                for (i, name) in names.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    
                    self.write(name);
                }
                
                self.write("\n");
            },
            Stmt::Expr { value, line: _, column: _ } => {
                self.write_indented("");
                self.visit_expr(&**value);
                self.write("\n");
            },
            Stmt::Pass { line: _, column: _ } => {
                self.write_line("pass");
            },
            Stmt::Break { line: _, column: _ } => {
                self.write_line("break");
            },
            Stmt::Continue { line: _, column: _ } => {
                self.write_line("continue");
            },
            Stmt::Match { subject, cases, line: _, column: _ } => {
                self.write_indented("match ");
                self.visit_expr(&**subject);
                self.write(":\n");
                
                self.increase_indent();
                
                for (pattern, guard, body) in cases {
                    self.write_indented("case ");
                    self.visit_expr(&**pattern);
                    
                    if let Some(guard_expr) = guard {
                        self.write(" if ");
                        self.visit_expr(&**guard_expr);
                    }
                    
                    self.write(":\n");
                    
                    self.increase_indent();
                    
                    if body.is_empty() {
                        self.write_line("pass");
                    } else {
                        for stmt in body {
                            self.visit_stmt(&**stmt);
                        }
                    }
                    
                    self.decrease_indent();
                }
                
                self.decrease_indent();
            }
        }
    }

    fn visit_expr(&mut self, expr: &'ast Expr) -> () {
        match expr {
            Expr::BoolOp { op, values, line: _, column: _ } => {
                let op_str = self.format_bool_operator(op);
                
                self.write("(");
                
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        self.write(" ");
                        self.write(op_str);
                        self.write(" ");
                    }
                    
                    self.visit_expr(&**value);
                }
                
                self.write(")");
            },
            Expr::BinOp { left, op, right, line: _, column: _ } => {
                self.write("(");
                self.visit_expr(&**left);
                self.write(" ");
                self.write(self.format_operator(op));
                self.write(" ");
                self.visit_expr(&**right);
                self.write(")");
            },
            Expr::UnaryOp { op, operand, line: _, column: _ } => {
                self.write("(");
                self.write(self.format_unary_operator(op));
                self.visit_expr(&**operand);
                self.write(")");
            },
            Expr::Lambda { args, body, line: _, column: _ } => {
                self.write("lambda ");
                
                for (i, param) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    
                    self.write(&param.name);
                    
                    if let Some(default) = &param.default {
                        self.write("=");
                        self.visit_expr(&**default);
                    }
                }
                
                self.write(": ");
                self.visit_expr(&**body);
            },
            Expr::IfExp { test, body, orelse, line: _, column: _ } => {
                self.write("(");
                self.visit_expr(&**body);
                self.write(" if ");
                self.visit_expr(&**test);
                self.write(" else ");
                self.visit_expr(&**orelse);
                self.write(")");
            },
            Expr::Dict { keys, values, line: _, column: _ } => {
                self.write("{");
                
                for (i, (key, value)) in keys.iter().zip(values.iter()).enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    
                    if let Some(key) = key {
                        self.visit_expr(&**key);
                        self.write(": ");
                        self.visit_expr(&**value);
                    } else {
                        // Dictionary unpacking with **
                        self.write("**");
                        self.visit_expr(&**value);
                    }
                }
                
                self.write("}");
            },
            Expr::Set { elts, line: _, column: _ } => {
                if elts.is_empty() {
                    self.write("set()");
                } else {
                    self.write("{");
                    
                    for (i, elt) in elts.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        
                        self.visit_expr(&**elt);
                    }
                    
                    self.write("}");
                }
            },
            Expr::ListComp { elt, generators, line: _, column: _ } => {
                self.write("[");
                self.visit_expr(&**elt);
                
                for comp in generators {
                    self.write(" for ");
                    self.visit_expr(&comp.target);
                    self.write(" in ");
                    self.visit_expr(&comp.iter);
                    
                    for if_expr in &comp.ifs {
                        self.write(" if ");
                        self.visit_expr(&**if_expr);
                    }
                }
                
                self.write("]");
            },
            Expr::SetComp { elt, generators, line: _, column: _ } => {
                self.write("{");
                self.visit_expr(&**elt);
                
                for comp in generators {
                    self.write(" for ");
                    self.visit_expr(&comp.target);
                    self.write(" in ");
                    self.visit_expr(&comp.iter);
                    
                    for if_expr in &comp.ifs {
                        self.write(" if ");
                        self.visit_expr(&**if_expr);
                    }
                }
                
                self.write("}");
            },
            Expr::DictComp { key, value, generators, line: _, column: _ } => {
                self.write("{");
                self.visit_expr(&**key);
                self.write(": ");
                self.visit_expr(&**value);
                
                for comp in generators {
                    self.write(" for ");
                    self.visit_expr(&comp.target);
                    self.write(" in ");
                    self.visit_expr(&comp.iter);
                    
                    for if_expr in &comp.ifs {
                        self.write(" if ");
                        self.visit_expr(&**if_expr);
                    }
                }
                
                self.write("}");
            },
            Expr::GeneratorExp { elt, generators, line: _, column: _ } => {
                self.write("(");
                self.visit_expr(&**elt);
                
                for comp in generators {
                    self.write(" for ");
                    self.visit_expr(&comp.target);
                    self.write(" in ");
                    self.visit_expr(&comp.iter);
                    
                    for if_expr in &comp.ifs {
                        self.write(" if ");
                        self.visit_expr(&**if_expr);
                    }
                }
                
                self.write(")");
            },
            Expr::Await { value, line: _, column: _ } => {
                self.write("await ");
                self.visit_expr(&**value);
            },
            Expr::Yield { value, line: _, column: _ } => {
                self.write("yield");
                
                if let Some(value) = value {
                    self.write(" ");
                    self.visit_expr(&**value);
                }
            },
            Expr::YieldFrom { value, line: _, column: _ } => {
                self.write("yield from ");
                self.visit_expr(&**value);
            },
            Expr::Compare { left, ops, comparators, line: _, column: _ } => {
                self.visit_expr(&**left);
                
                for (op, comparator) in ops.iter().zip(comparators.iter()) {
                    self.write(" ");
                    self.write(self.format_cmp_operator(op));
                    self.write(" ");
                    self.visit_expr(&**comparator);
                }
            },
            Expr::Call { func, args, keywords, line: _, column: _ } => {
                self.visit_expr(&**func);
                self.write("(");
                
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    
                    if let Expr::Starred { value, .. } = &**arg {
                        self.write("*");
                        self.visit_expr(&**value);
                    } else {
                        self.visit_expr(&**arg);
                    }
                }
                
                if !args.is_empty() && !keywords.is_empty() {
                    self.write(", ");
                }
                
                for (i, (key, value)) in keywords.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    
                    if let Some(key) = key {
                        self.write(key);
                        self.write("=");
                        self.visit_expr(&**value);
                    } else {
                        self.write("**");
                        self.visit_expr(&**value);
                    }
                }
                
                self.write(")");
            },
            Expr::Num { value, line: _, column: _ } => {
                match value {
                    crate::ast::Number::Integer(i) => self.write(&i.to_string()),
                    crate::ast::Number::Float(f) => self.write(&f.to_string()),
                    crate::ast::Number::Complex { real, imag } => {
                        self.write(&format!("{}+{}j", real, imag));
                    }
                }
            },
            Expr::Str { value, line: _, column: _ } => {
                self.write(&format!("\"{}\"", value));
            },
            Expr::FormattedValue { value, conversion, format_spec, line: _, column: _ } => {
                // This is used inside f-strings
                self.write("{");
                self.visit_expr(&**value);
                
                if *conversion != '\0' {
                    self.write(&format!("!{}", conversion));
                }
                
                if let Some(spec) = format_spec {
                    self.write(":");
                    self.visit_expr(&**spec);
                }
                
                self.write("}");
            },
            Expr::JoinedStr { values, line: _, column: _ } => {
                self.write("f\"");
                
                for value in values {
                    self.visit_expr(&**value);
                }
                
                self.write("\"");
            },
            Expr::Bytes { value, line: _, column: _ } => {
                self.write("b\"");
                // Simplified - in a real formatter you'd properly escape binary data
                for byte in value {
                    if *byte >= 32 && *byte <= 126 {
                        self.write(&(*byte as char).to_string());
                    } else {
                        self.write(&format!("\\x{:02x}", byte));
                    }
                }
                self.write("\"");
            },
            Expr::NameConstant { value, line: _, column: _ } => {
                match value {
                    crate::ast::NameConstant::None => self.write("None"),
                    crate::ast::NameConstant::True => self.write("True"),
                    crate::ast::NameConstant::False => self.write("False"),
                }
            },
            Expr::Ellipsis { line: _, column: _ } => {
                self.write("...");
            },
            Expr::Constant { value, line: _, column: _ } => {
                match value {
                    crate::ast::Constant::Num(num) => {
                        match num {
                            crate::ast::Number::Integer(i) => self.write(&i.to_string()),
                            crate::ast::Number::Float(f) => self.write(&f.to_string()),
                            crate::ast::Number::Complex { real, imag } => {
                                self.write(&format!("{}+{}j", real, imag));
                            }
                        }
                    },
                    crate::ast::Constant::Str(s) => self.write(&format!("\"{}\"", s)),
                    crate::ast::Constant::Bytes(bytes) => {
                        self.write("b\"");
                        for byte in bytes {
                            if *byte >= 32 && *byte <= 126 {
                                self.write(&(*byte as char).to_string());
                            } else {
                                self.write(&format!("\\x{:02x}", byte));
                            }
                        }
                        self.write("\"");
                    },
                    crate::ast::Constant::NameConstant(nc) => match nc {
                        crate::ast::NameConstant::None => self.write("None"),
                        crate::ast::NameConstant::True => self.write("True"),
                        crate::ast::NameConstant::False => self.write("False"),
                    },
                    crate::ast::Constant::Ellipsis => self.write("..."),
                }
            },
            Expr::Attribute { value, attr, ctx: _, line: _, column: _ } => {
                self.visit_expr(&**value);
                self.write(".");
                self.write(attr);
            },
            Expr::Subscript { value, slice, ctx: _, line: _, column: _ } => {
                self.visit_expr(&**value);
                self.write("[");
                self.visit_expr(&**slice);
                self.write("]");
            },
            Expr::Starred { value, ctx: _, line: _, column: _ } => {
                self.write("*");
                self.visit_expr(&**value);
            },
            Expr::Name { id, ctx: _, line: _, column: _ } => {
                self.write(id);
            },
            Expr::List { elts, ctx: _, line: _, column: _ } => {
                self.write("[");
                
                for (i, elt) in elts.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    
                    self.visit_expr(&**elt);
                }
                
                self.write("]");
            },
            Expr::Tuple { elts, ctx: _, line: _, column: _ } => {
                if elts.is_empty() {
                    self.write("()");
                } else if elts.len() == 1 {
                    self.visit_expr(&*elts[0]);
                    self.write(",");
                } else {
                    self.write("(");
                    
                    for (i, elt) in elts.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        
                        self.visit_expr(&**elt);
                    }
                    
                    self.write(")");
                }
            },
            Expr::NamedExpr { target, value, line: _, column: _ } => {
                self.write("(");
                self.visit_expr(target);
                self.write(" := ");
                self.visit_expr(value);
                self.write(")");
            },
        }
    }

    fn visit_except_handler(&mut self, _handler: &'ast crate::ast::ExceptHandler) -> () {
        // Handled in visit_stmt for Try
    }

    fn visit_comprehension(&mut self, _comp: &'ast crate::ast::Comprehension) -> () {
        // Handled in visit_expr for the various comprehension types
    }

    fn visit_alias(&mut self, _alias: &'ast crate::ast::Alias) -> () {
        // Handled in visit_stmt for Import and ImportFrom
    }

    fn visit_parameter(&mut self, _param: &'ast crate::ast::Parameter) -> () {
        // Handled in visit_stmt for FunctionDef and visit_expr for Lambda
    }
}