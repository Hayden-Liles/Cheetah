use crate::ast::{Expr, Number, NameConstant, Operator, UnaryOperator, CmpOperator};
use crate::compiler::types::{Type, TypeError};
use crate::typechecker::environment::TypeEnvironment;
use crate::typechecker::TypeResult;

/// Type inference for expressions
pub struct TypeInference;

impl TypeInference {
    /// Infer the type of an expression
    pub fn infer_expr(env: &TypeEnvironment, expr: &Expr) -> TypeResult<Type> {
        match expr {
            Expr::Num { value, .. } => Ok(match value {
                Number::Integer(_) => Type::Int,
                Number::Float(_) => Type::Float,
                Number::Complex { .. } => Type::Float, // Simplification
            }),

            Expr::Str { .. } => Ok(Type::String),

            Expr::Bytes { .. } => Ok(Type::Bytes),

            Expr::NameConstant { value, .. } => Ok(match value {
                NameConstant::True | NameConstant::False => Type::Bool,
                NameConstant::None => Type::None,
            }),

            Expr::List { elts, .. } => {
                if elts.is_empty() {
                    Ok(Type::List(Box::new(Type::Any)))
                } else {
                    // Try to infer a common type for all elements
                    let mut element_types = Vec::with_capacity(elts.len());

                    for elt in elts {
                        element_types.push(Self::infer_expr(env, elt)?);
                    }

                    // Find a common type that all elements can be converted to
                    let common_type = Self::find_common_type(&element_types)?;
                    Ok(Type::List(Box::new(common_type)))
                }
            },

            Expr::Tuple { elts, .. } => {
                let mut element_types = Vec::with_capacity(elts.len());

                for elt in elts {
                    element_types.push(Self::infer_expr(env, elt)?);
                }

                Ok(Type::Tuple(element_types))
            },

            Expr::Dict { keys, values, .. } => {
                if keys.is_empty() || values.is_empty() {
                    // Empty dict or dict with None keys (dict unpacking)
                    Ok(Type::Dict(Box::new(Type::Any), Box::new(Type::Any)))
                } else {
                    // Try to infer common types for keys and values
                    let mut key_types = Vec::with_capacity(keys.len());
                    let mut value_types = Vec::with_capacity(values.len());

                    for (key_opt, value) in keys.iter().zip(values.iter()) {
                        if let Some(key) = key_opt {
                            key_types.push(Self::infer_expr(env, key)?);
                        }
                        value_types.push(Self::infer_expr(env, value)?);
                    }

                    // Find common types
                    let key_type = if key_types.is_empty() {
                        Type::Any
                    } else {
                        Self::find_common_type(&key_types)?
                    };

                    let value_type = if value_types.is_empty() {
                        Type::Any
                    } else {
                        Self::find_common_type(&value_types)?
                    };

                    Ok(Type::Dict(Box::new(key_type), Box::new(value_type)))
                }
            },

            Expr::Set { elts, .. } => {
                if elts.is_empty() {
                    Ok(Type::Set(Box::new(Type::Any)))
                } else {
                    // Try to infer a common type for all elements
                    let mut element_types = Vec::with_capacity(elts.len());

                    for elt in elts {
                        element_types.push(Self::infer_expr(env, elt)?);
                    }

                    // Find a common type that all elements can be converted to
                    let common_type = Self::find_common_type(&element_types)?;
                    Ok(Type::Set(Box::new(common_type)))
                }
            },

            Expr::Name { id, .. } => {
                // Look up the variable in the environment
                if let Some(ty) = env.lookup_variable(id) {
                    Ok(ty.clone())
                } else if let Some(ty) = env.lookup_function(id) {
                    Ok(ty.clone())
                } else if let Some(ty) = env.lookup_class(id) {
                    Ok(ty.clone())
                } else {
                    Err(TypeError::UndefinedVariable(id.clone()))
                }
            },

            Expr::BinOp { left, op, right, .. } => {
                let left_type = Self::infer_expr(env, left)?;
                let right_type = Self::infer_expr(env, right)?;

                Self::infer_binary_op(&left_type, op, &right_type)
            },

            Expr::UnaryOp { op, operand, .. } => {
                let operand_type = Self::infer_expr(env, operand)?;

                Self::infer_unary_op(op, &operand_type)
            },

            Expr::BoolOp { op, values, .. } => {
                // Check that all operands are boolean-compatible
                for value in values {
                    let value_type = Self::infer_expr(env, value)?;
                    if !value_type.can_coerce_to(&Type::Bool) {
                        return Err(TypeError::IncompatibleTypes {
                            expected: Type::Bool,
                            got: value_type,
                            operation: format!("{:?}", op),
                        });
                    }
                }

                // Boolean operations always return a boolean
                Ok(Type::Bool)
            },

            Expr::Compare { left, ops, comparators, .. } => {
                let left_type = Self::infer_expr(env, left)?;

                // Check each comparison operation
                for (op, right) in ops.iter().zip(comparators.iter()) {
                    let right_type = Self::infer_expr(env, right)?;

                    // Check if the comparison is valid
                    Self::check_comparison(&left_type, op, &right_type)?;
                }

                // Comparisons always return a boolean
                Ok(Type::Bool)
            },

            Expr::Call { func, args, keywords, .. } => {
                // Special case for built-in functions
                if let Expr::Name { id, .. } = &**func {
                    match id.as_str() {
                        "len" => {
                            // len() returns an integer
                            if args.len() == 1 {
                                // Check that the argument is a container
                                let arg_type = Self::infer_expr(env, &args[0])?;
                                if arg_type.is_indexable() {
                                    return Ok(Type::Int);
                                }
                            }
                            return Ok(Type::Int); // Simplification: always return Int for len()
                        },
                        "str" => {
                            // str() converts to string
                            return Ok(Type::String);
                        },
                        "int" => {
                            // int() converts to integer
                            return Ok(Type::Int);
                        },
                        "float" => {
                            // float() converts to float
                            return Ok(Type::Float);
                        },
                        "bool" => {
                            // bool() converts to boolean
                            return Ok(Type::Bool);
                        },
                        "list" => {
                            // list() creates a list
                            if args.len() == 1 {
                                let arg_type = Self::infer_expr(env, &args[0])?;
                                if let Type::List(elem_type) = arg_type {
                                    return Ok(Type::List(elem_type));
                                }
                                // For other types, create a list of Any
                                return Ok(Type::List(Box::new(Type::Any)));
                            }
                            return Ok(Type::List(Box::new(Type::Any)));
                        },
                        "dict" => {
                            // dict() creates a dictionary
                            return Ok(Type::Dict(Box::new(Type::Any), Box::new(Type::Any)));
                        },
                        "set" => {
                            // set() creates a set
                            if args.len() == 1 {
                                let arg_type = Self::infer_expr(env, &args[0])?;
                                if let Type::Set(elem_type) = arg_type {
                                    return Ok(Type::Set(elem_type));
                                }
                                // For other types, create a set of Any
                                return Ok(Type::Set(Box::new(Type::Any)));
                            }
                            return Ok(Type::Set(Box::new(Type::Any)));
                        },
                        "tuple" => {
                            // tuple() creates a tuple
                            if args.len() == 1 {
                                let arg_type = Self::infer_expr(env, &args[0])?;
                                if let Type::Tuple(elem_types) = arg_type {
                                    return Ok(Type::Tuple(elem_types));
                                }
                                // For other types, create an empty tuple
                                return Ok(Type::Tuple(vec![]));
                            }
                            return Ok(Type::Tuple(vec![]));
                        },
                        "print" => {
                            // print() returns None
                            return Ok(Type::None);
                        },
                        "range" => {
                            // range() returns an iterable of integers
                            return Ok(Type::List(Box::new(Type::Int)));
                        },
                        _ => {
                            // For other function names, proceed with normal function call handling
                        }
                    }
                }

                // Infer the type of the function
                let func_type = Self::infer_expr(env, func)?;

                // Check if the type is callable
                if !func_type.is_callable() {
                    return Err(TypeError::NotCallable(func_type));
                }

                // Infer the types of the arguments
                let mut arg_types = Vec::with_capacity(args.len());
                for arg in args {
                    arg_types.push(Self::infer_expr(env, arg)?);
                }

                // Handle keyword arguments (simplified for now)
                for (_, value) in keywords {
                    let _ = Self::infer_expr(env, value)?;
                }

                // Get the return type of the function call
                func_type.get_call_return_type(&arg_types)
            },

            Expr::Attribute { value, attr, .. } => {
                let value_type = Self::infer_expr(env, value)?;

                // Check if the value is a class or has attributes
                match &value_type {
                    Type::Class { name, methods, fields, .. } => {
                        // Check if the attribute is a method
                        if let Some(method_type) = methods.get(attr) {
                            Ok(*method_type.clone())
                        }
                        // Check if the attribute is a field
                        else if let Some(field_type) = fields.get(attr) {
                            Ok(field_type.clone())
                        }
                        else {
                            Err(TypeError::UndefinedMember {
                                class_name: name.clone(),
                                member: attr.clone(),
                            })
                        }
                    },
                    _ => Err(TypeError::NotAClass {
                        expr_type: value_type,
                        member: attr.clone(),
                    }),
                }
            },

            Expr::Subscript { value, slice, .. } => {
                let value_type = Self::infer_expr(env, value)?;
                let slice_type = Self::infer_expr(env, slice)?;

                // Check if the value is indexable
                if !value_type.is_indexable() {
                    return Err(TypeError::NotIndexable(value_type));
                }

                // For simplicity, we'll assume any indexing with a non-integer type might be a slice
                if !matches!(slice_type, Type::Int) {
                    // Slicing a container returns the same container type
                    match &value_type {
                        Type::List(elem_type) => return Ok(Type::List(elem_type.clone())),
                        Type::Tuple(_) => return Ok(value_type.clone()),
                        Type::String => return Ok(Type::String),
                        Type::Bytes => return Ok(Type::Bytes),
                        _ => {}
                    }
                }

                // Get the element type
                value_type.get_indexed_type(&slice_type)
            },

            Expr::Lambda { args, body, .. } => {
                // Create parameter types (simplified: all Any for now)
                let param_types = vec![Type::Any; args.len()];
                let param_names = args.iter().map(|param| param.name.clone()).collect();
                let default_values = args.iter().map(|param| param.default.is_some()).collect();

                // Infer the return type
                let return_type = Self::infer_expr(env, body)?;

                Ok(Type::Function {
                    param_types,
                    param_names,
                    has_varargs: args.iter().any(|p| p.is_vararg),
                    has_kwargs: args.iter().any(|p| p.is_kwarg),
                    default_values,
                    return_type: Box::new(return_type),
                })
            },

            Expr::IfExp { test, body, orelse, .. } => {
                // Check that the test is boolean-compatible
                let test_type = Self::infer_expr(env, test)?;
                if !test_type.can_coerce_to(&Type::Bool) {
                    return Err(TypeError::IncompatibleTypes {
                        expected: Type::Bool,
                        got: test_type,
                        operation: "if expression condition".to_string(),
                    });
                }

                // Infer types of the branches
                let then_type = Self::infer_expr(env, body)?;
                let else_type = Self::infer_expr(env, orelse)?;

                // Find a common type for both branches
                Self::find_common_type(&[then_type, else_type])
            },

            // For other expression types, return Unknown for now
            _ => Ok(Type::Unknown),
        }
    }

    /// Infer the type of a binary operation
    pub fn infer_binary_op(left_type: &Type, op: &Operator, right_type: &Type) -> TypeResult<Type> {
        match op {
            Operator::Add => {
                // Addition works for numbers and strings
                match (left_type, right_type) {
                    // Numeric addition
                    (Type::Int, Type::Int) => Ok(Type::Int),
                    (Type::Int, Type::Float) | (Type::Float, Type::Int) | (Type::Float, Type::Float) => Ok(Type::Float),

                    // String concatenation
                    (Type::String, Type::String) => Ok(Type::String),

                    // List concatenation
                    (Type::List(left_elem), Type::List(right_elem)) => {
                        // Find a common element type
                        let common_elem = Type::unify(left_elem, right_elem)
                            .ok_or_else(|| TypeError::IncompatibleTypes {
                                expected: left_type.clone(),
                                got: right_type.clone(),
                                operation: "list concatenation".to_string(),
                            })?;

                        Ok(Type::List(Box::new(common_elem)))
                    },

                    // Tuple concatenation
                    (Type::Tuple(left_elems), Type::Tuple(right_elems)) => {
                        let mut result_elems = left_elems.clone();
                        result_elems.extend(right_elems.clone());
                        Ok(Type::Tuple(result_elems))
                    },

                    _ => Err(TypeError::InvalidOperator {
                        operator: "+".to_string(),
                        left_type: left_type.clone(),
                        right_type: Some(right_type.clone()),
                    }),
                }
            },

            Operator::Sub | Operator::Mult | Operator::Div | Operator::FloorDiv |
            Operator::Mod | Operator::Pow => {
                // These operations work only for numbers
                match (left_type, right_type) {
                    (Type::Int, Type::Int) => Ok(Type::Int),
                    (Type::Int, Type::Float) | (Type::Float, Type::Int) | (Type::Float, Type::Float) => Ok(Type::Float),
                    _ => Err(TypeError::InvalidOperator {
                        operator: format!("{:?}", op),
                        left_type: left_type.clone(),
                        right_type: Some(right_type.clone()),
                    }),
                }
            },

            Operator::BitOr | Operator::BitXor | Operator::BitAnd |
            Operator::LShift | Operator::RShift => {
                // Bitwise operations work only for integers
                match (left_type, right_type) {
                    (Type::Int, Type::Int) => Ok(Type::Int),
                    _ => Err(TypeError::InvalidOperator {
                        operator: format!("{:?}", op),
                        left_type: left_type.clone(),
                        right_type: Some(right_type.clone()),
                    }),
                }
            },

            Operator::MatMult => {
                // Matrix multiplication (not fully implemented)
                Err(TypeError::InvalidOperator {
                    operator: "@".to_string(),
                    left_type: left_type.clone(),
                    right_type: Some(right_type.clone()),
                })
            },
        }
    }

    /// Infer the type of a unary operation
    fn infer_unary_op(op: &UnaryOperator, operand_type: &Type) -> TypeResult<Type> {
        match op {
            UnaryOperator::UAdd | UnaryOperator::USub => {
                // Unary plus and minus work only for numbers
                match operand_type {
                    Type::Int => Ok(Type::Int),
                    Type::Float => Ok(Type::Float),
                    _ => Err(TypeError::InvalidOperator {
                        operator: format!("{:?}", op),
                        left_type: operand_type.clone(),
                        right_type: None,
                    }),
                }
            },

            UnaryOperator::Not => {
                // Not operator works on any type but always returns a boolean
                Ok(Type::Bool)
            },

            UnaryOperator::Invert => {
                // Bitwise inversion works only for integers
                match operand_type {
                    Type::Int => Ok(Type::Int),
                    _ => Err(TypeError::InvalidOperator {
                        operator: "~".to_string(),
                        left_type: operand_type.clone(),
                        right_type: None,
                    }),
                }
            },
        }
    }

    /// Check if a comparison operation is valid
    fn check_comparison(left_type: &Type, op: &CmpOperator, right_type: &Type) -> TypeResult<()> {
        match op {
            CmpOperator::Eq | CmpOperator::NotEq => {
                // Equality comparisons work for any types
                Ok(())
            },

            CmpOperator::Lt | CmpOperator::LtE | CmpOperator::Gt | CmpOperator::GtE => {
                // Ordering comparisons work for numbers and strings
                match (left_type, right_type) {
                    (Type::Int, Type::Int) |
                    (Type::Int, Type::Float) |
                    (Type::Float, Type::Int) |
                    (Type::Float, Type::Float) |
                    (Type::String, Type::String) => Ok(()),

                    _ => Err(TypeError::InvalidOperator {
                        operator: format!("{:?}", op),
                        left_type: left_type.clone(),
                        right_type: Some(right_type.clone()),
                    }),
                }
            },

            CmpOperator::Is | CmpOperator::IsNot => {
                // Identity comparisons work for any types
                Ok(())
            },

            CmpOperator::In | CmpOperator::NotIn => {
                // Membership tests work for containers
                if right_type.is_indexable() {
                    Ok(())
                } else {
                    Err(TypeError::InvalidOperator {
                        operator: format!("{:?}", op),
                        left_type: left_type.clone(),
                        right_type: Some(right_type.clone()),
                    })
                }
            },
        }
    }

    /// Find a common type that all given types can be converted to
    pub fn find_common_type(types: &[Type]) -> TypeResult<Type> {
        if types.is_empty() {
            return Ok(Type::Any);
        }

        if types.len() == 1 {
            return Ok(types[0].clone());
        }

        let mut result = types[0].clone();

        for ty in &types[1..] {
            if let Some(common) = Type::unify(&result, ty) {
                result = common;
            } else {
                return Err(TypeError::IncompatibleTypes {
                    expected: result,
                    got: ty.clone(),
                    operation: "type unification".to_string(),
                });
            }
        }

        Ok(result)
    }
}
