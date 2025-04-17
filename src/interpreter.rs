// interpreter.rs - Simple interpreter for Cheetah language

use crate::ast::{Expr, Module, Stmt};

/// Simple interpreter for Cheetah language
pub struct Interpreter {
    // Environment for variables
    variables: std::collections::HashMap<String, Value>,
}

/// Value type for the interpreter
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<Value>),
    None,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::None => write!(f, "None"),
        }
    }
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        Interpreter {
            variables: std::collections::HashMap::new(),
        }
    }

    /// Interpret a module
    pub fn interpret(&mut self, module: &Module) -> Result<(), String> {
        for stmt in &module.body {
            self.interpret_stmt(stmt)?;
        }
        Ok(())
    }

    /// Interpret a statement
    fn interpret_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expr { value, .. } => {
                self.eval_expr(value)?;
                Ok(())
            }
            Stmt::Assign { targets, value, .. } => {
                let val = self.eval_expr(value)?;
                for target in targets {
                    match target.as_ref() {
                        Expr::Name { id, .. } => {
                            self.variables.insert(id.clone(), val.clone());
                        }
                        _ => return Err(format!("Unsupported assignment target: {:?}", target)),
                    }
                }
                Ok(())
            }
            Stmt::AugAssign { target, op, value, .. } => {
                let right_val = self.eval_expr(value)?;

                match target.as_ref() {
                    Expr::Name { id, .. } => {
                        let left_val = if let Some(val) = self.variables.get(id) {
                            val.clone()
                        } else {
                            return Err(format!("Undefined variable in augmented assignment: {}", id));
                        };

                        // Perform the operation based on the operator
                        let result = match (left_val, op, right_val) {
                            (Value::Int(l), crate::ast::Operator::Add, Value::Int(r)) => Value::Int(l + r),
                            (Value::Int(l), crate::ast::Operator::Sub, Value::Int(r)) => Value::Int(l - r),
                            (Value::Int(l), crate::ast::Operator::Mult, Value::Int(r)) => Value::Int(l * r),
                            (Value::Int(l), crate::ast::Operator::Div, Value::Int(r)) => {
                                if r == 0 {
                                    return Err("Division by zero".to_string());
                                }
                                Value::Int(l / r)
                            },
                            (Value::Float(l), crate::ast::Operator::Add, Value::Float(r)) => Value::Float(l + r),
                            (Value::Float(l), crate::ast::Operator::Sub, Value::Float(r)) => Value::Float(l - r),
                            (Value::Float(l), crate::ast::Operator::Mult, Value::Float(r)) => Value::Float(l * r),
                            (Value::Float(l), crate::ast::Operator::Div, Value::Float(r)) => {
                                if r == 0.0 {
                                    return Err("Division by zero".to_string());
                                }
                                Value::Float(l / r)
                            },
                            (Value::String(l), crate::ast::Operator::Add, Value::String(r)) => Value::String(l + &r),
                            _ => return Err(format!("Unsupported augmented assignment operation")),
                        };

                        // Store the result back in the variable
                        self.variables.insert(id.clone(), result);
                        Ok(())
                    },
                    _ => Err(format!("Unsupported augmented assignment target: {:?}", target)),
                }
            }
            Stmt::For { target, iter, body, .. } => {
                // Evaluate the iterator
                let iter_val = self.eval_expr(iter)?;

                // Get the iterator values
                let values = match iter_val {
                    Value::List(values) => values,
                    _ => return Err(format!("Cannot iterate over non-list value: {:?}", iter_val)),
                };

                // Get the target variable name
                let target_name = match target.as_ref() {
                    Expr::Name { id, .. } => id.clone(),
                    _ => return Err(format!("Unsupported for loop target: {:?}", target)),
                };

                // Execute the loop body for each value
                for value in values {
                    // Set the target variable
                    self.variables.insert(target_name.clone(), value);

                    // Execute the loop body
                    for stmt in body {
                        self.interpret_stmt(stmt)?;
                    }
                }

                Ok(())
            }
            _ => Err(format!("Unsupported statement: {:?}", stmt)),
        }
    }

    /// Evaluate an expression
    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Name { id, .. } => {
                if let Some(val) = self.variables.get(id) {
                    Ok(val.clone())
                } else {
                    Err(format!("Undefined variable: {}", id))
                }
            }
            Expr::Str { value, .. } => Ok(Value::String(value.clone())),
            Expr::Num { value, .. } => match value {
                crate::ast::Number::Integer(i) => Ok(Value::Int(*i)),
                crate::ast::Number::Float(f) => Ok(Value::Float(*f)),
                _ => Err(format!("Unsupported number type: {:?}", value)),
            },
            Expr::Constant { value, .. } => match value {
                crate::ast::Constant::Num(crate::ast::Number::Integer(i)) => Ok(Value::Int(*i)),
                crate::ast::Constant::Num(crate::ast::Number::Float(f)) => Ok(Value::Float(*f)),
                crate::ast::Constant::NameConstant(crate::ast::NameConstant::True) => Ok(Value::Bool(true)),
                crate::ast::Constant::NameConstant(crate::ast::NameConstant::False) => Ok(Value::Bool(false)),
                crate::ast::Constant::NameConstant(crate::ast::NameConstant::None) => Ok(Value::None),
                crate::ast::Constant::Str(s) => Ok(Value::String(s.clone())),
                _ => Err(format!("Unsupported constant: {:?}", value)),
            },
            Expr::Call { func, args, .. } => {
                // Handle built-in functions
                if let Expr::Name { id, .. } = func.as_ref() {
                    match id.as_str() {
                        "print" => {
                            // Evaluate arguments
                            let mut arg_values = Vec::new();
                            for arg in args {
                                arg_values.push(self.eval_expr(arg)?);
                            }

                            // Print the values
                            for (i, val) in arg_values.iter().enumerate() {
                                if i > 0 {
                                    print!(" ");
                                }
                                print!("{}", val);
                            }
                            println!();

                            Ok(Value::None)
                        }
                        "range" => {
                            // Evaluate arguments
                            let mut arg_values = Vec::new();
                            for arg in args {
                                arg_values.push(self.eval_expr(arg)?);
                            }

                            // Create range based on number of arguments
                            match arg_values.len() {
                                1 => {
                                    // range(stop)
                                    if let Value::Int(stop) = arg_values[0] {
                                        let mut values = Vec::new();
                                        for i in 0..stop {
                                            values.push(Value::Int(i));
                                        }
                                        Ok(Value::List(values))
                                    } else {
                                        Err("range() argument must be an integer".to_string())
                                    }
                                }
                                2 => {
                                    // range(start, stop)
                                    if let (Value::Int(start), Value::Int(stop)) = (&arg_values[0], &arg_values[1]) {
                                        let mut values = Vec::new();
                                        for i in *start..*stop {
                                            values.push(Value::Int(i));
                                        }
                                        Ok(Value::List(values))
                                    } else {
                                        Err("range() arguments must be integers".to_string())
                                    }
                                }
                                3 => {
                                    // range(start, stop, step)
                                    if let (Value::Int(start), Value::Int(stop), Value::Int(step)) = (&arg_values[0], &arg_values[1], &arg_values[2]) {
                                        let mut values = Vec::new();
                                        let mut i = *start;
                                        while if *step > 0 { i < *stop } else { i > *stop } {
                                            values.push(Value::Int(i));
                                            i += step;
                                        }
                                        Ok(Value::List(values))
                                    } else {
                                        Err("range() arguments must be integers".to_string())
                                    }
                                }
                                _ => Err(format!("range() takes 1-3 arguments, got {}", arg_values.len())),
                            }
                        }
                        _ => Err(format!("Undefined function: {}", id)),
                    }
                } else {
                    Err(format!("Unsupported function call: {:?}", func))
                }
            }
            Expr::BinOp { left, op, right, .. } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;

                match (left_val, op, right_val) {
                    (Value::Int(l), crate::ast::Operator::Add, Value::Int(r)) => Ok(Value::Int(l + r)),
                    (Value::Int(l), crate::ast::Operator::Sub, Value::Int(r)) => Ok(Value::Int(l - r)),
                    (Value::Int(l), crate::ast::Operator::Mult, Value::Int(r)) => Ok(Value::Int(l * r)),
                    (Value::Int(l), crate::ast::Operator::Div, Value::Int(r)) => {
                        if r == 0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(Value::Int(l / r))
                        }
                    }
                    _ => Err(format!("Unsupported binary operation: {:?} {:?} {:?}", left, op, right)),
                }
            }
            Expr::List { elts, .. } => {
                let mut values = Vec::new();
                for elt in elts {
                    values.push(self.eval_expr(elt)?);
                }
                Ok(Value::List(values))
            },
            Expr::Tuple { elts, .. } => {
                // In our simple interpreter, we'll treat tuples as lists
                let mut values = Vec::new();
                for elt in elts {
                    values.push(self.eval_expr(elt)?);
                }
                Ok(Value::List(values))
            }
            Expr::Subscript { value, slice, .. } => {
                let value = self.eval_expr(value)?;
                let index = self.eval_expr(slice)?;

                match (value, index) {
                    (Value::List(list), Value::Int(idx)) => {
                        let idx = if idx < 0 { (list.len() as i64 + idx) as usize } else { idx as usize };
                        if idx < list.len() {
                            Ok(list[idx].clone())
                        } else {
                            Err(format!("Index out of range: {}", idx))
                        }
                    }
                    _ => Err(format!("Unsupported subscript operation")),
                }
            }
            _ => Err(format!("Unsupported expression: {:?}", expr)),
        }
    }
}
