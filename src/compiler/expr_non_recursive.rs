// Non-recursive implementation of the expression compiler
// This implementation avoids deep recursion by using an explicit work stack

use crate::ast::{BoolOperator, CmpOperator, Expr, Operator, UnaryOperator};
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::{BinaryOpCompiler, ComparisonCompiler, ExprCompiler};
use crate::compiler::types::Type;
use inkwell::values::BasicValueEnum;
use std::collections::VecDeque;

// This trait is used to extend the CompilationContext with non-recursive expression compilation
pub trait ExprNonRecursive<'ctx> {
    fn compile_expr_non_recursive(
        &mut self,
        expr: &crate::ast::Expr,
    ) -> Result<(BasicValueEnum<'ctx>, crate::compiler::types::Type), String>;

    fn compile_expr_original(
        &mut self,
        expr: &crate::ast::Expr,
    ) -> Result<(BasicValueEnum<'ctx>, crate::compiler::types::Type), String>;

    fn compile_expr_fallback(
        &mut self,
        expr: &crate::ast::Expr,
    ) -> Result<(BasicValueEnum<'ctx>, crate::compiler::types::Type), String>;
}

// Task for the work stack
enum ExprTask<'a> {
    Evaluate(&'a Expr),

    ProcessBinaryOp {
        op: Operator,
    },

    ProcessUnaryOp {
        op: UnaryOperator,
        operand_idx: usize,
    },

    ProcessComparison {
        op: CmpOperator,
    },

    ProcessBoolOp {
        op: BoolOperator,
    },

    ProcessIfExpression {
        then_block: inkwell::basic_block::BasicBlock<'a>,
        else_block: inkwell::basic_block::BasicBlock<'a>,
        merge_block: inkwell::basic_block::BasicBlock<'a>,
        body: Box<Expr>,
        orelse: Box<Expr>,
    },

    ProcessTuple {
        elements_count: usize,
    },

    ProcessList {
        elements_count: usize,
    },

    ProcessDict {
        elements_count: usize,
    },

    ProcessSet {
        elements_count: usize,
    },

    ProcessAttribute {
        attr: String,
    },

    ProcessMethodCall {
        method_name: String,
        args_count: usize,
    },
}

// Result of an expression evaluation
struct ExprResult<'ctx> {
    value: BasicValueEnum<'ctx>,
    ty: Type,
}

impl<'ctx> ExprNonRecursive<'ctx> for CompilationContext<'ctx> {
    fn compile_expr_non_recursive(
        &mut self,
        expr: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        let mut work_stack: VecDeque<ExprTask> = VecDeque::new();
        let mut result_stack: Vec<ExprResult<'ctx>> = Vec::new();

        work_stack.push_back(ExprTask::Evaluate(expr));

        while let Some(task) = work_stack.pop_front() {
            match task {
                ExprTask::Evaluate(expr) => match expr {
                    Expr::Num { value, .. } => {
                        let (value, ty) = self.compile_number(value)?;
                        result_stack.push(ExprResult { value, ty });
                    }
                    Expr::NameConstant { value, .. } => {
                        let (value, ty) = self.compile_name_constant(value)?;
                        result_stack.push(ExprResult { value, ty });
                    }
                    Expr::BinOp {
                        left, op, right, ..
                    } => {
                        work_stack.push_front(ExprTask::ProcessBinaryOp { op: op.clone() });

                        work_stack.push_front(ExprTask::Evaluate(right));

                        work_stack.push_front(ExprTask::Evaluate(left));
                    }
                    Expr::UnaryOp { op, operand, .. } => {
                        let operand_idx = result_stack.len();

                        work_stack.push_front(ExprTask::ProcessUnaryOp {
                            op: op.clone(),
                            operand_idx,
                        });
                        work_stack.push_front(ExprTask::Evaluate(operand));
                    }
                    Expr::Compare {
                        left,
                        ops,
                        comparators,
                        ..
                    } => {
                        if ops.is_empty()
                            || comparators.is_empty()
                            || ops.len() != comparators.len()
                        {
                            return Err("Invalid comparison expression".to_string());
                        }

                        if ops.len() == 1 {
                            work_stack
                                .push_front(ExprTask::ProcessComparison { op: ops[0].clone() });

                            work_stack.push_front(ExprTask::Evaluate(&comparators[0]));

                            work_stack.push_front(ExprTask::Evaluate(left));
                        } else if ops.len() == 2 {
                            work_stack.push_front(ExprTask::ProcessBoolOp {
                                op: BoolOperator::And,
                            });

                            work_stack
                                .push_front(ExprTask::ProcessComparison { op: ops[1].clone() });
                            work_stack.push_front(ExprTask::Evaluate(&comparators[1]));
                            work_stack.push_front(ExprTask::Evaluate(&comparators[0]));

                            work_stack
                                .push_front(ExprTask::ProcessComparison { op: ops[0].clone() });
                            work_stack.push_front(ExprTask::Evaluate(&comparators[0]));
                            work_stack.push_front(ExprTask::Evaluate(left));
                        } else {
                            for i in (1..ops.len()).rev() {
                                if i < ops.len() - 1 {
                                    work_stack.push_front(ExprTask::ProcessBoolOp {
                                        op: BoolOperator::And,
                                    });
                                }

                                work_stack
                                    .push_front(ExprTask::ProcessComparison { op: ops[i].clone() });

                                work_stack.push_front(ExprTask::Evaluate(&comparators[i]));
                                work_stack.push_front(ExprTask::Evaluate(&comparators[i - 1]));
                            }

                            work_stack.push_front(ExprTask::ProcessBoolOp {
                                op: BoolOperator::And,
                            });

                            work_stack
                                .push_front(ExprTask::ProcessComparison { op: ops[0].clone() });
                            work_stack.push_front(ExprTask::Evaluate(&comparators[0]));
                            work_stack.push_front(ExprTask::Evaluate(left));
                        }
                    }
                    Expr::BoolOp { op, values, .. } => {
                        if values.is_empty() {
                            return Err("Empty boolean operation".to_string());
                        }

                        if values.len() == 1 {
                            work_stack.push_front(ExprTask::Evaluate(&values[0]));
                        } else if values.len() == 2 {
                            work_stack.push_front(ExprTask::ProcessBoolOp { op: op.clone() });

                            work_stack.push_front(ExprTask::Evaluate(&values[1]));

                            work_stack.push_front(ExprTask::Evaluate(&values[0]));
                        } else {
                            let last_idx = values.len() - 1;
                            let second_last_idx = last_idx - 1;

                            work_stack.push_front(ExprTask::ProcessBoolOp { op: op.clone() });

                            work_stack.push_front(ExprTask::Evaluate(&values[last_idx]));
                            work_stack.push_front(ExprTask::Evaluate(&values[second_last_idx]));

                            for i in (0..second_last_idx).rev() {
                                work_stack.push_front(ExprTask::ProcessBoolOp { op: op.clone() });

                                work_stack.push_front(ExprTask::Evaluate(&values[i]));
                            }
                        }
                    }
                    Expr::Name { id, .. } => {
                        self.ensure_block_has_terminator();

                        // Try to load the variable using the new load_var helper
                        match self.load_var(id) {
                            Ok((val, ty)) => {
                                // Variable exists, push the loaded value onto the stack
                                result_stack.push(ExprResult {
                                    value: val,
                                    ty,
                                });
                            },
                            Err(_) => {
                                // If the variable doesn't exist, create a new BoxedAny None value
                                // This mimics Python's behavior of creating variables on first use

                                // Get the boxed_any_none function
                                let boxed_any_none_fn = self.module.get_function("boxed_any_none")
                                    .ok_or_else(|| "boxed_any_none function not found".to_string())?;

                                // Call boxed_any_none to create a None value
                                let call_site_value = self.builder.build_call(
                                    boxed_any_none_fn,
                                    &[],
                                    &format!("none_for_{}", id)
                                ).unwrap();

                                let none_val = call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| format!("Failed to create None value for {}", id))?;

                                // Store the None value using store_var
                                self.store_var(id, none_val, &Type::Any)?;

                                result_stack.push(ExprResult {
                                    value: none_val,
                                    ty: Type::Any,
                                });
                            }
                        }
                    }
                    Expr::IfExp {
                        test, body, orelse, ..
                    } => {
                        let current_block = self.builder.get_insert_block().unwrap();
                        let current_function = current_block.get_parent().unwrap();
                        let then_block = self
                            .llvm_context
                            .append_basic_block(current_function, "if_then");
                        let else_block = self
                            .llvm_context
                            .append_basic_block(current_function, "if_else");
                        let merge_block = self
                            .llvm_context
                            .append_basic_block(current_function, "if_merge");

                        work_stack.push_front(ExprTask::ProcessIfExpression {
                            then_block,
                            else_block,
                            merge_block,
                            body: body.clone(),
                            orelse: orelse.clone(),
                        });

                        work_stack.push_front(ExprTask::Evaluate(test));
                    }

                    Expr::ListComp { .. } => {
                        let (value, ty) = self.compile_expr_fallback(expr)?;
                        result_stack.push(ExprResult { value, ty });
                    }

                    Expr::Str { value, .. } => {
                        let const_str = self.llvm_context.const_string(value.as_bytes(), true);

                        let str_type = const_str.get_type();

                        let global_str = self.module.add_global(str_type, None, "str_const");
                        global_str.set_constant(true);
                        global_str.set_initializer(&const_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        result_stack.push(ExprResult {
                            value: str_ptr.into(),
                            ty: Type::String,
                        });
                    }

                    Expr::Tuple { elts, .. } => {
                        let elements_count = elts.len();
                        work_stack.push_front(ExprTask::ProcessTuple { elements_count });

                        for elt in elts.iter().rev() {
                            work_stack.push_front(ExprTask::Evaluate(elt));
                        }
                    }

                    Expr::List { elts, .. } => {
                        let elements_count = elts.len();
                        work_stack.push_front(ExprTask::ProcessList { elements_count });

                        for elt in elts.iter().rev() {
                            work_stack.push_front(ExprTask::Evaluate(elt));
                        }
                    }

                    Expr::Dict { keys, values, .. } => {
                        let elements_count = keys.len();
                        work_stack.push_front(ExprTask::ProcessDict { elements_count });

                        for i in (0..keys.len()).rev() {
                            if let Some(key) = &keys[i] {
                                work_stack.push_front(ExprTask::Evaluate(&values[i]));
                                work_stack.push_front(ExprTask::Evaluate(key));
                            }
                        }
                    }

                    Expr::Set { elts, .. } => {
                        let elements_count = elts.len();
                        work_stack.push_front(ExprTask::ProcessSet { elements_count });

                        for elt in elts.iter().rev() {
                            work_stack.push_front(ExprTask::Evaluate(elt));
                        }
                    }

                    Expr::Subscript { value, slice, .. } => {
                        let (value_val, ty) = self.compile_subscript_non_recursive(value, slice)?;
                        result_stack.push(ExprResult {
                            value: value_val,
                            ty,
                        });
                    }

                    Expr::Attribute { value, attr, .. } => {
                        // We'll handle all attribute access the same way
                        if false {
                            // This branch is never taken, just here to keep the code structure
                        } else {
                            work_stack.push_front(ExprTask::ProcessAttribute { attr: attr.clone() });
                            work_stack.push_front(ExprTask::Evaluate(value));
                        }
                    }

                    Expr::Call { func, args, .. } => {
                        // Check if this is a method call (e.g., obj.method())
                        if let Expr::Attribute { value, attr, .. } = func.as_ref() {
                            // This is a method call, so we need to evaluate the object first,
                            // then the arguments, and finally call the method
                            let args_count = args.len();

                            // Push the method call task
                            work_stack.push_front(ExprTask::ProcessMethodCall {
                                method_name: attr.clone(),
                                args_count,
                            });

                            // Push the arguments in reverse order
                            for arg in args.iter().rev() {
                                work_stack.push_front(ExprTask::Evaluate(arg));
                            }

                            // Push the object
                            work_stack.push_front(ExprTask::Evaluate(value));

                            continue;
                        }

                        // Check if this is a call to a built-in function like print
                        if let Expr::Name { id, .. } = func.as_ref() {
                            if id == "print" {
                                // Use the specialized print function handler
                                let args_slice: Vec<Expr> = args.iter().map(|arg| (**arg).clone()).collect();

                                // Debug output to help diagnose the issue
                                println!("Compiling print call with {} arguments", args_slice.len());

                                let (print_val, print_type) = self.compile_print_call(&args_slice)?;
                                result_stack.push(ExprResult {
                                    value: print_val,
                                    ty: print_type,
                                });
                                continue;
                            } else if id == "len" {
                                let args_slice: Vec<Expr> = args.iter().map(|arg| (**arg).clone()).collect();
                                let (len_val, len_type) = self.compile_len_call(&args_slice)?;
                                result_stack.push(ExprResult {
                                    value: len_val,
                                    ty: len_type,
                                });
                                continue;
                            } else if id == "min" {
                                let args_slice: Vec<Expr> = args.iter().map(|arg| (**arg).clone()).collect();
                                let (min_val, min_type) = self.compile_min_call(&args_slice)?;
                                result_stack.push(ExprResult {
                                    value: min_val,
                                    ty: min_type,
                                });
                                continue;
                            } else if id == "max" {
                                let args_slice: Vec<Expr> = args.iter().map(|arg| (**arg).clone()).collect();
                                let (max_val, max_type) = self.compile_max_call(&args_slice)?;
                                result_stack.push(ExprResult {
                                    value: max_val,
                                    ty: max_type,
                                });
                                continue;
                            }
                        }

                        // For other calls, use the fallback
                        let (call_val, call_type) = self.compile_expr_fallback(expr)?;
                        result_stack.push(ExprResult {
                            value: call_val,
                            ty: call_type,
                        });
                    }

                    Expr::DictComp { .. } => {
                        let (dict_val, dict_type) = self.compile_expr_fallback(expr)?;
                        result_stack.push(ExprResult {
                            value: dict_val,
                            ty: dict_type,
                        });
                    }

                    _ => {
                        let (value, ty) = self.compile_expr_fallback(expr)?;
                        result_stack.push(ExprResult { value, ty });
                    }
                },
                ExprTask::ProcessBinaryOp { op } => {
                    if result_stack.len() < 2 {
                        return Err(format!(
                            "Not enough operands for binary operation: stack size = {}",
                            result_stack.len()
                        ));
                    }

                    let right_idx = result_stack.len() - 1;
                    let left_idx = right_idx - 1;

                    let right_result = &result_stack[right_idx];
                    let left_result = &result_stack[left_idx];


                    // Convert primitive types to BoxedAny pointers if needed
                    let (left_val, left_type) = match left_result.ty {
                        Type::Int => {
                            // Convert Int to BoxedAny
                            let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                                .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                            let call_site_value = self.builder.build_call(
                                boxed_any_from_int_fn,
                                &[left_result.value.into()],
                                "int_to_boxed"
                            ).unwrap();

                            let boxed_val = call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to convert Int to BoxedAny".to_string())?;

                            (boxed_val, Type::Any)
                        },
                        Type::Float => {
                            // Convert Float to BoxedAny
                            let boxed_any_from_float_fn = self.module.get_function("boxed_any_from_float")
                                .ok_or_else(|| "boxed_any_from_float function not found".to_string())?;

                            let call_site_value = self.builder.build_call(
                                boxed_any_from_float_fn,
                                &[left_result.value.into()],
                                "float_to_boxed"
                            ).unwrap();

                            let boxed_val = call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to convert Float to BoxedAny".to_string())?;

                            (boxed_val, Type::Any)
                        },
                        Type::Bool => {
                            // Convert Bool to BoxedAny
                            let boxed_any_from_bool_fn = self.module.get_function("boxed_any_from_bool")
                                .ok_or_else(|| "boxed_any_from_bool function not found".to_string())?;

                            let call_site_value = self.builder.build_call(
                                boxed_any_from_bool_fn,
                                &[left_result.value.into()],
                                "bool_to_boxed"
                            ).unwrap();

                            let boxed_val = call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to convert Bool to BoxedAny".to_string())?;

                            (boxed_val, Type::Any)
                        },
                        _ => (left_result.value, left_result.ty.clone()),
                    };

                    let (right_val, right_type) = match right_result.ty {
                        Type::Int => {
                            // Convert Int to BoxedAny
                            let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                                .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                            let call_site_value = self.builder.build_call(
                                boxed_any_from_int_fn,
                                &[right_result.value.into()],
                                "int_to_boxed"
                            ).unwrap();

                            let boxed_val = call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to convert Int to BoxedAny".to_string())?;

                            (boxed_val, Type::Any)
                        },
                        Type::Float => {
                            // Convert Float to BoxedAny
                            let boxed_any_from_float_fn = self.module.get_function("boxed_any_from_float")
                                .ok_or_else(|| "boxed_any_from_float function not found".to_string())?;

                            let call_site_value = self.builder.build_call(
                                boxed_any_from_float_fn,
                                &[right_result.value.into()],
                                "float_to_boxed"
                            ).unwrap();

                            let boxed_val = call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to convert Float to BoxedAny".to_string())?;

                            (boxed_val, Type::Any)
                        },
                        Type::Bool => {
                            // Convert Bool to BoxedAny
                            let boxed_any_from_bool_fn = self.module.get_function("boxed_any_from_bool")
                                .ok_or_else(|| "boxed_any_from_bool function not found".to_string())?;

                            let call_site_value = self.builder.build_call(
                                boxed_any_from_bool_fn,
                                &[right_result.value.into()],
                                "bool_to_boxed"
                            ).unwrap();

                            let boxed_val = call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to convert Bool to BoxedAny".to_string())?;

                            (boxed_val, Type::Any)
                        },
                        _ => (right_result.value, right_result.ty.clone()),
                    };

                    // With BoxedAny, all binary operations return BoxedAny pointers
                    let expected_result_type = Type::Any;

                    let (boxed_result, _) = self.compile_binary_op(
                        left_val,
                        &left_type,
                        op,
                        right_val,
                        &right_type,
                    )?;

                    // Convert the BoxedAny result back to the expected type
                    let result_value = match expected_result_type {
                        Type::Int => {
                            // Convert BoxedAny to Int
                            let boxed_any_to_int_fn = self.module.get_function("boxed_any_to_int")
                                .ok_or_else(|| "boxed_any_to_int function not found".to_string())?;

                            let call_site_value = self.builder.build_call(
                                boxed_any_to_int_fn,
                                &[boxed_result.into()],
                                "boxed_to_int"
                            ).unwrap();

                            call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to convert BoxedAny to Int".to_string())?
                        },
                        Type::Float => {
                            // Convert BoxedAny to Float
                            let boxed_any_to_float_fn = self.module.get_function("boxed_any_to_float")
                                .ok_or_else(|| "boxed_any_to_float function not found".to_string())?;

                            let call_site_value = self.builder.build_call(
                                boxed_any_to_float_fn,
                                &[boxed_result.into()],
                                "boxed_to_float"
                            ).unwrap();

                            call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to convert BoxedAny to Float".to_string())?
                        },
                        Type::Bool => {
                            // Convert BoxedAny to Bool
                            let boxed_any_to_bool_fn = self.module.get_function("boxed_any_to_bool")
                                .ok_or_else(|| "boxed_any_to_bool function not found".to_string())?;

                            let call_site_value = self.builder.build_call(
                                boxed_any_to_bool_fn,
                                &[boxed_result.into()],
                                "boxed_to_bool"
                            ).unwrap();

                            call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to convert BoxedAny to Bool".to_string())?
                        },
                        _ => boxed_result,
                    };

                    result_stack.remove(right_idx);
                    result_stack.remove(left_idx);


                    result_stack.push(ExprResult {
                        value: result_value,
                        ty: expected_result_type,
                    });
                }
                ExprTask::ProcessUnaryOp { op, operand_idx } => {
                    if operand_idx >= result_stack.len() {
                        return Err("Invalid result stack index for unary operation".to_string());
                    }

                    let operand_result = &result_stack[operand_idx];

                    let (result_value, result_type) = match op {
                        UnaryOperator::Not => {
                            match operand_result.ty {
                                Type::Any => {
                                    // For Any type, use boxed_any_not function
                                    let boxed_any_not_fn = self.module.get_function("boxed_any_not")
                                        .ok_or_else(|| "boxed_any_not function not found".to_string())?;

                                    let call_site_value = self.builder.build_call(
                                        boxed_any_not_fn,
                                        &[operand_result.value.into()],
                                        "boxed_not_result"
                                    ).unwrap();

                                    let result = call_site_value.try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to negate BoxedAny value".to_string())?;

                                    (result, Type::Any)
                                },
                                _ => {
                                    let bool_val = if !matches!(operand_result.ty, Type::Bool) {
                                        self.convert_type(
                                            operand_result.value,
                                            &operand_result.ty,
                                            &Type::Bool,
                                        )?
                                    } else {
                                        operand_result.value
                                    };

                                    let result = self
                                        .builder
                                        .build_not(bool_val.into_int_value(), "not")
                                        .unwrap();
                                    (result.into(), Type::Bool)
                                }
                            }
                        }
                        UnaryOperator::USub => match operand_result.ty {
                            Type::Int => {
                                let int_val = operand_result.value.into_int_value();
                                let result = self.builder.build_int_neg(int_val, "neg").unwrap();
                                (result.into(), Type::Int)
                            }
                            Type::Float => {
                                let float_val = operand_result.value.into_float_value();
                                let result =
                                    self.builder.build_float_neg(float_val, "neg").unwrap();
                                (result.into(), Type::Float)
                            }
                            Type::Any => {
                                // For Any type, use boxed_any_negate function
                                let boxed_any_negate_fn = self.module.get_function("boxed_any_negate")
                                    .ok_or_else(|| "boxed_any_negate function not found".to_string())?;

                                let call_site_value = self.builder.build_call(
                                    boxed_any_negate_fn,
                                    &[operand_result.value.into()],
                                    "boxed_negate_result"
                                ).unwrap();

                                let result = call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to negate BoxedAny value".to_string())?;

                                (result, Type::Any)
                            }
                            _ => {
                                return Err(format!(
                                    "Cannot negate value of type {:?}",
                                    operand_result.ty
                                ))
                            }
                        },
                        UnaryOperator::UAdd => (operand_result.value, operand_result.ty.clone()),
                        UnaryOperator::Invert => match operand_result.ty {
                            Type::Int => {
                                let int_val = operand_result.value.into_int_value();
                                let result = self.builder.build_not(int_val, "invert").unwrap();
                                (result.into(), Type::Int)
                            }
                            _ => {
                                return Err(format!(
                                    "Cannot bitwise invert value of type {:?}",
                                    operand_result.ty
                                ))
                            }
                        },
                    };

                    result_stack.remove(operand_idx);

                    result_stack.push(ExprResult {
                        value: result_value,
                        ty: result_type,
                    });
                }
                ExprTask::ProcessComparison { op } => {
                    if result_stack.len() < 2 {
                        return Err(format!(
                            "Not enough operands for comparison operation: stack size = {}",
                            result_stack.len()
                        ));
                    }

                    let right_idx = result_stack.len() - 1;
                    let left_idx = right_idx - 1;

                    let left_value = result_stack[left_idx].value;
                    let left_type = result_stack[left_idx].ty.clone();
                    let right_value = result_stack[right_idx].value;
                    let right_type = result_stack[right_idx].ty.clone();

                    let (result_value, result_type) = self.compile_comparison(
                        left_value,
                        &left_type,
                        op,
                        right_value,
                        &right_type,
                        false, // We need a boolean result for comparison operations
                    )?;

                    if right_idx > left_idx {
                        result_stack.remove(right_idx);
                        result_stack.remove(left_idx);
                    } else {
                        result_stack.remove(left_idx);
                        result_stack.remove(right_idx);
                    }

                    result_stack.push(ExprResult {
                        value: result_value,
                        ty: result_type,
                    });
                }
                ExprTask::ProcessBoolOp { op } => {
                    if result_stack.len() < 2 {
                        return Err(format!(
                            "Not enough operands for boolean operation: stack size = {}",
                            result_stack.len()
                        ));
                    }

                    let right_idx = result_stack.len() - 1;
                    let left_idx = right_idx - 1;

                    let right_result = &result_stack[right_idx];
                    let left_result = &result_stack[left_idx];

                    let right_value = right_result.value;
                    let right_type = right_result.ty.clone();
                    let left_value = left_result.value;
                    let left_type = left_result.ty.clone();

                    let left_bool = if left_type != Type::Bool {
                        self.convert_type(left_value, &left_type, &Type::Bool)?
                    } else {
                        left_value
                    };

                    let right_bool = if right_type != Type::Bool {
                        self.convert_type(right_value, &right_type, &Type::Bool)?
                    } else {
                        right_value
                    };

                    let mut current_value = left_bool;

                    match op {
                        BoolOperator::And => {
                            let cond_block = self.builder.get_insert_block().unwrap();
                            let current_function = cond_block.get_parent().unwrap();

                            let then_block = self
                                .llvm_context
                                .append_basic_block(current_function, "and_then");
                            let merge_block = self
                                .llvm_context
                                .append_basic_block(current_function, "and_merge");

                            self.builder
                                .build_conditional_branch(
                                    current_value.into_int_value(),
                                    then_block,
                                    merge_block,
                                )
                                .unwrap();

                            self.builder.position_at_end(then_block);
                            let then_value = right_bool;
                            self.builder
                                .build_unconditional_branch(merge_block)
                                .unwrap();
                            let then_block = self.builder.get_insert_block().unwrap();

                            self.builder.position_at_end(merge_block);
                            let phi = self
                                .builder
                                .build_phi(self.llvm_context.bool_type(), "and_result")
                                .unwrap();

                            phi.add_incoming(&[
                                (
                                    &self.llvm_context.bool_type().const_int(0, false),
                                    cond_block,
                                ),
                                (&then_value.into_int_value(), then_block),
                            ]);

                            current_value = phi.as_basic_value();
                        }
                        BoolOperator::Or => {
                            let cond_block = self.builder.get_insert_block().unwrap();
                            let current_function = cond_block.get_parent().unwrap();

                            let else_block = self
                                .llvm_context
                                .append_basic_block(current_function, "or_else");
                            let merge_block = self
                                .llvm_context
                                .append_basic_block(current_function, "or_merge");

                            self.builder
                                .build_conditional_branch(
                                    current_value.into_int_value(),
                                    merge_block,
                                    else_block,
                                )
                                .unwrap();

                            self.builder.position_at_end(else_block);
                            let else_value = right_bool;
                            self.builder
                                .build_unconditional_branch(merge_block)
                                .unwrap();
                            let else_block = self.builder.get_insert_block().unwrap();

                            self.builder.position_at_end(merge_block);
                            let phi = self
                                .builder
                                .build_phi(self.llvm_context.bool_type(), "or_result")
                                .unwrap();

                            phi.add_incoming(&[
                                (
                                    &self.llvm_context.bool_type().const_int(1, false),
                                    cond_block,
                                ),
                                (&else_value.into_int_value(), else_block),
                            ]);

                            current_value = phi.as_basic_value();
                        }
                    }

                    result_stack.remove(right_idx);
                    result_stack.remove(left_idx);

                    result_stack.push(ExprResult {
                        value: current_value,
                        ty: Type::Bool,
                    });
                }
                ExprTask::ProcessIfExpression {
                    then_block,
                    else_block,
                    merge_block,
                    body,
                    orelse,
                } => {
                    if result_stack.is_empty() {
                        return Err("No test condition found for if expression".to_string());
                    }

                    let test_idx = result_stack.len() - 1;
                    let test_result = &result_stack[test_idx];
                    let test_val = test_result.value;
                    let test_type = test_result.ty.clone();

                    let cond_val = if test_type != Type::Bool {
                        self.convert_type(test_val, &test_type, &Type::Bool)?
                            .into_int_value()
                    } else {
                        test_val.into_int_value()
                    };

                    self.ensure_block_has_terminator();

                    self.builder
                        .build_conditional_branch(cond_val, then_block, else_block)
                        .unwrap();

                    self.builder.position_at_end(then_block);
                    let (then_val, then_type) = self.compile_expr(&body)?;

                    self.ensure_block_has_terminator();
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .unwrap();
                    let then_block = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(else_block);
                    let (else_val, else_type) = self.compile_expr(&orelse)?;

                    self.ensure_block_has_terminator();
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .unwrap();
                    let else_block = self.builder.get_insert_block().unwrap();

                    let result_type = if then_type == else_type {
                        then_type.clone()
                    } else if then_type.can_coerce_to(&else_type) {
                        else_type.clone()
                    } else if else_type.can_coerce_to(&then_type) {
                        then_type.clone()
                    } else {
                        return Err(format!(
                            "Incompatible types in if expression: {:?} and {:?}",
                            then_type, else_type
                        ));
                    };

                    let then_val = if then_type != result_type {
                        self.convert_type(then_val, &then_type, &result_type)?
                    } else {
                        then_val
                    };

                    let else_val = if else_type != result_type {
                        self.convert_type(else_val, &else_type, &result_type)?
                    } else {
                        else_val
                    };

                    self.builder.position_at_end(merge_block);

                    self.ensure_block_has_terminator();

                    let llvm_type = self.get_llvm_type(&result_type);

                    // Allocate a variable to store the result
                    let result_ptr = self.builder.build_alloca(llvm_type, "if_result_ptr").unwrap();

                    // Create a PHI node to select the appropriate value
                    let phi = self.builder.build_phi(llvm_type, "if_result").unwrap();
                    phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);

                    // Store the result in the allocated variable
                    self.builder.build_store(result_ptr, phi.as_basic_value()).unwrap();

                    // Load the result from the allocated variable
                    let result_val = self.builder.build_load(llvm_type, result_ptr, "if_result_val").unwrap();

                    result_stack.remove(test_idx);

                    result_stack.push(ExprResult {
                        value: result_val,
                        ty: result_type,
                    });
                }
                ExprTask::ProcessTuple { elements_count } => {
                    if result_stack.len() < elements_count {
                        return Err(format!(
                            "Not enough elements for tuple: expected {}, got {}",
                            elements_count,
                            result_stack.len()
                        ));
                    }

                    let mut elements = Vec::with_capacity(elements_count);
                    let mut element_types = Vec::with_capacity(elements_count);

                    for _ in 0..elements_count {
                        let idx = result_stack.len() - 1;
                        let element = result_stack.remove(idx);
                        elements.push(element.value);
                        element_types.push(element.ty);
                    }

                    elements.reverse();
                    element_types.reverse();

                    let tuple_ptr = self.build_tuple(elements, &element_types)?;

                    result_stack.push(ExprResult {
                        value: tuple_ptr.into(),
                        ty: Type::Tuple(element_types),
                    });
                }
                ExprTask::ProcessList { elements_count } => {
                    if result_stack.len() < elements_count {
                        return Err(format!(
                            "Not enough elements for list: expected {}, got {}",
                            elements_count,
                            result_stack.len()
                        ));
                    }

                    let mut elements = Vec::with_capacity(elements_count);
                    let mut element_type = Type::Unknown;

                    for _ in 0..elements_count {
                        let idx = result_stack.len() - 1;
                        let element = result_stack.remove(idx);
                        elements.push(element.value);

                        if element_type == Type::Unknown {
                            element_type = element.ty;
                        } else if element_type != element.ty {
                            element_type = Type::Any;
                        }
                    }

                    elements.reverse();

                    let list_ptr = self.build_list(elements, &element_type)?;

                    result_stack.push(ExprResult {
                        value: list_ptr.into(),
                        ty: Type::List(Box::new(element_type)),
                    });
                }
                ExprTask::ProcessDict { elements_count } => {
                    if result_stack.len() < elements_count * 2 {
                        return Err(format!(
                            "Not enough elements for dict: expected {}, got {}",
                            elements_count * 2,
                            result_stack.len()
                        ));
                    }

                    let mut keys = Vec::with_capacity(elements_count);
                    let mut values = Vec::with_capacity(elements_count);
                    let mut key_type = Type::Unknown;
                    let mut value_type = Type::Unknown;

                    for _ in 0..elements_count {
                        let value_idx = result_stack.len() - 1;
                        let value = result_stack.remove(value_idx);
                        values.push(value.value);

                        if value_type == Type::Unknown {
                            value_type = value.ty;
                        } else if value_type != value.ty {
                            value_type = Type::Any;
                        }

                        let key_idx = result_stack.len() - 1;
                        let key = result_stack.remove(key_idx);
                        keys.push(key.value);

                        if key_type == Type::Unknown {
                            key_type = key.ty;
                        } else if key_type != key.ty {
                            key_type = Type::Any;
                        }
                    }

                    keys.reverse();
                    values.reverse();

                    let dict_ptr = self.build_dict(keys, values, &key_type, &value_type)?;

                    result_stack.push(ExprResult {
                        value: dict_ptr.into(),
                        ty: Type::Dict(Box::new(key_type), Box::new(value_type)),
                    });
                }
                ExprTask::ProcessSet { elements_count } => {
                    if result_stack.len() < elements_count {
                        return Err(format!(
                            "Not enough elements for set: expected {}, got {}",
                            elements_count,
                            result_stack.len()
                        ));
                    }

                    let mut elements = Vec::with_capacity(elements_count);
                    let mut element_type = Type::Unknown;

                    for _ in 0..elements_count {
                        let idx = result_stack.len() - 1;
                        let element = result_stack.remove(idx);
                        elements.push(element.value);

                        if element_type == Type::Unknown {
                            element_type = element.ty;
                        } else if element_type != element.ty {
                            element_type = Type::Any;
                        }
                    }

                    elements.reverse();

                    let set_ptr = self.build_set(elements, &element_type)?;

                    result_stack.push(ExprResult {
                        value: set_ptr.into(),
                        ty: Type::Set(Box::new(element_type)),
                    });
                }
                ExprTask::ProcessAttribute { attr } => {
                    if result_stack.is_empty() {
                        return Err("No value found for attribute access".to_string());
                    }

                    let value_idx = result_stack.len() - 1;
                    let value_result = result_stack.remove(value_idx);

                    let (attr_val, attr_type) = match value_result.ty {
                        Type::Dict(_, _) => match attr.as_str() {
                            "keys" | "values" | "items" | "get" | "pop" | "clear" | "update" => {
                                let placeholder = self.llvm_context.i32_type().const_int(0, false);
                                (placeholder.into(), Type::function(vec![], Type::Any))
                            }
                            _ => {
                                return Err(format!("Unknown attribute '{}' for dictionary", attr))
                            }
                        },
                        Type::List(_) | Type::Unknown => match attr.as_str() {
                            "append" | "pop" | "clear" | "extend" | "insert" | "remove"
                            | "sort" => {
                                // Return a function that will be called with the argument
                                let list_ptr = value_result.value.into_pointer_value();

                                // Create a placeholder function value
                                let i32_type = self.llvm_context.i32_type();
                                let placeholder = i32_type.const_int(0, false);

                                // The function type is (Any) -> None since we don't know the element type
                                let fn_type = Type::function(vec![Type::Any], Type::None);

                                // Get the element type
                                let element_type_for_call = if let Type::List(element_type) = &value_result.ty {
                                    if matches!(*element_type.as_ref(), Type::Unknown) {
                                        Box::new(Type::Any)
                                    } else {
                                        element_type.clone()
                                    }
                                } else {
                                    Box::new(Type::Any)
                                };

                                // Store the list pointer in a global variable so we can access it later
                                let global_name = format!("list_for_append_{}", self.get_unique_id());
                                let global = self.module.add_global(
                                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                    None,
                                    &global_name,
                                );
                                global.set_initializer(&self.llvm_context.ptr_type(inkwell::AddressSpace::default()).const_null());
                                global.set_linkage(inkwell::module::Linkage::Private);
                                self.builder.build_store(global.as_pointer_value(), list_ptr).unwrap();

                                // Store the method name in the context for later use
                                self.set_pending_method_call(global_name, attr.clone(), element_type_for_call);

                                (placeholder.into(), fn_type)
                            }
                            _ => return Err(format!("Unknown attribute '{}' for list", attr)),
                        },
                        Type::String => match attr.as_str() {
                            "upper" | "lower" | "strip" | "split" | "join" | "replace" => {
                                let placeholder = self.llvm_context.i32_type().const_int(0, false);
                                (placeholder.into(), Type::function(vec![], Type::Any))
                            }
                            _ => return Err(format!("Unknown attribute '{}' for string", attr)),
                        },
                        Type::Class { methods, .. } => {
                            if let Some(method_type) = methods.get(&attr) {
                                let placeholder = self.llvm_context.i32_type().const_int(0, false);
                                (placeholder.into(), (**method_type).clone())
                            } else {
                                let placeholder = self.llvm_context.i32_type().const_int(0, false);
                                (placeholder.into(), Type::Any)
                            }
                        }


                        _ => {
                            return Err(format!(
                                "Cannot access attribute '{}' on value of type {:?}",
                                attr, value_result.ty
                            ))
                        }
                    };

                    result_stack.push(ExprResult {
                        value: attr_val,
                        ty: attr_type,
                    });
                }
                ExprTask::ProcessMethodCall { method_name, args_count } => {
                    if result_stack.len() < args_count + 1 {
                        return Err(format!(
                            "Not enough arguments for method call: expected {} arguments plus object, got {}",
                            args_count,
                            result_stack.len()
                        ));
                    }

                    // Get the arguments
                    let mut arg_values = Vec::with_capacity(args_count);

                    for _ in 0..args_count {
                        let idx = result_stack.len() - 1;
                        let arg = result_stack.remove(idx);

                        // Convert primitive types to BoxedAny if needed
                        let boxed_arg = match arg.ty {
                            Type::Int => {
                                // Convert Int to BoxedAny
                                let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                                    .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                                let call_site_value = self.builder.build_call(
                                    boxed_any_from_int_fn,
                                    &[arg.value.into()],
                                    "int_to_boxed"
                                ).unwrap();

                                call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to convert Int to BoxedAny".to_string())?
                            },
                            Type::Float => {
                                // Convert Float to BoxedAny
                                let boxed_any_from_float_fn = self.module.get_function("boxed_any_from_float")
                                    .ok_or_else(|| "boxed_any_from_float function not found".to_string())?;

                                let call_site_value = self.builder.build_call(
                                    boxed_any_from_float_fn,
                                    &[arg.value.into()],
                                    "float_to_boxed"
                                ).unwrap();

                                call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to convert Float to BoxedAny".to_string())?
                            },
                            Type::String => {
                                // Convert String to BoxedAny
                                let boxed_any_from_string_fn = self.module.get_function("boxed_any_from_string")
                                    .ok_or_else(|| "boxed_any_from_string function not found".to_string())?;

                                let call_site_value = self.builder.build_call(
                                    boxed_any_from_string_fn,
                                    &[arg.value.into()],
                                    "string_to_boxed"
                                ).unwrap();

                                call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to convert String to BoxedAny".to_string())?
                            },
                            Type::Bool => {
                                // Convert Bool to BoxedAny
                                let boxed_any_from_bool_fn = self.module.get_function("boxed_any_from_bool")
                                    .ok_or_else(|| "boxed_any_from_bool function not found".to_string())?;

                                let call_site_value = self.builder.build_call(
                                    boxed_any_from_bool_fn,
                                    &[arg.value.into()],
                                    "bool_to_boxed"
                                ).unwrap();

                                call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to convert Bool to BoxedAny".to_string())?
                            },
                            Type::None => {
                                // Convert None to BoxedAny
                                let boxed_any_none_fn = self.module.get_function("boxed_any_none")
                                    .ok_or_else(|| "boxed_any_none function not found".to_string())?;

                                let call_site_value = self.builder.build_call(
                                    boxed_any_none_fn,
                                    &[],
                                    "none_to_boxed"
                                ).unwrap();

                                call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to convert None to BoxedAny".to_string())?
                            },
                            _ => arg.value,
                        };

                        arg_values.push(boxed_arg);
                    }

                    // Reverse the arguments to restore left-to-right order
                    arg_values.reverse();

                    // Get the object
                    let obj_idx = result_stack.len() - 1;
                    let obj = result_stack.remove(obj_idx);

                    // Make sure the object is a BoxedAny pointer
                    if !obj.value.is_pointer_value() {
                        return Err("Expected pointer value for object in method call".to_string());
                    }

                    // Create a method name string constant
                    let method_name_str = self.llvm_context.const_string(method_name.as_bytes(), true);
                    let unique_id = self.get_unique_id();
                    let method_name_global = self.module.add_global(
                        method_name_str.get_type(),
                        None,
                        &format!("method_name_{}", unique_id)
                    );
                    method_name_global.set_constant(true);
                    method_name_global.set_initializer(&method_name_str);
                    let method_name_ptr = self.builder.build_pointer_cast(
                        method_name_global.as_pointer_value(),
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "method_name_ptr"
                    ).unwrap();

                    // Create an array of BoxedAny pointers for the arguments
                    let ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                    let args_array = if args_count > 0 {
                        // Allocate space for the arguments
                        self.builder.build_array_alloca(
                            ptr_type,
                            self.llvm_context.i32_type().const_int(args_count as u64, false),
                            "args_array"
                        ).unwrap()
                    } else {
                        // If there are no arguments, just create a null pointer
                        ptr_type.const_null()
                    };

                    // Store each argument in the array
                    for (i, arg) in arg_values.iter().enumerate() {
                        let arg_ptr = unsafe {
                            self.builder.build_gep(
                                ptr_type,
                                args_array,
                                &[self.llvm_context.i32_type().const_int(i as u64, false)],
                                &format!("arg_ptr_{}", i)
                            ).unwrap()
                        };
                        self.builder.build_store(arg_ptr, *arg).unwrap();
                    }

                    // Cast the array to a pointer to pointer
                    let args_ptr = self.builder.build_pointer_cast(
                        args_array,
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "args_ptr"
                    ).unwrap();

                    // Get the boxed_any_call_method function
                    let boxed_any_call_method_fn = self.module.get_function("boxed_any_call_method")
                        .ok_or_else(|| "boxed_any_call_method function not found".to_string())?;

                    // Call boxed_any_call_method
                    let call_site_value = self.builder.build_call(
                        boxed_any_call_method_fn,
                        &[
                            obj.value.into(),
                            method_name_ptr.into(),
                            args_ptr.into(),
                            self.llvm_context.i32_type().const_int(args_count as u64, false).into(),
                        ],
                        &format!("call_method_{}", method_name)
                    ).unwrap();

                    let result = call_site_value.try_as_basic_value().left()
                        .ok_or_else(|| format!("Failed to call method {}", method_name))?;

                    // Push the result onto the stack
                    result_stack.push(ExprResult {
                        value: result,
                        ty: Type::Any,
                    });
                }
            }
        }

        if result_stack.len() != 1 {
            return Err(format!(
                "Expected 1 result, but got {} results",
                result_stack.len()
            ));
        }

        let final_result = result_stack.pop().unwrap();

        Ok((final_result.value, final_result.ty))
    }

    fn compile_expr_original(
        &mut self,
        expr: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        match expr {
            Expr::Num { value, .. } => self.compile_number(value),
            Expr::NameConstant { value, .. } => self.compile_name_constant(value),
            _ => Err(format!(
                "Unsupported expression type in fallback implementation: {:?}",
                expr
            )),
        }
    }

    fn compile_expr_fallback(
        &mut self,
        expr: &crate::ast::Expr,
    ) -> Result<(BasicValueEnum<'ctx>, crate::compiler::types::Type), String> {
        match expr {

            Expr::ListComp {
                elt, generators, ..
            } => self.compile_list_comprehension(elt, generators),
            Expr::Call { func, args, .. } => {
                if let Expr::Name { id, .. } = func.as_ref() {
                    if id == "range" && args.len() == 1 {
                        if let Expr::Call {
                            func: len_func,
                            args: len_args,
                            ..
                        } = args[0].as_ref()
                        {
                            if let Expr::Name { id: len_id, .. } = len_func.as_ref() {
                                if len_id == "len" && len_args.len() == 1 {
                                    let args_slice: Vec<Expr> =
                                        len_args.iter().map(|arg| (**arg).clone()).collect();
                                    let (len_val, _) = self.compile_len_call(&args_slice)?;

                                    // Convert the integer to a BoxedAny pointer
                                    let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                                        .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                                    let boxed_len_val = self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[len_val.into()],
                                        "boxed_len"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?;

                                    // Use boxed_range_1 instead of range_1
                                    let boxed_range_1_fn = match self.module.get_function("boxed_range_1") {
                                        Some(f) => f,
                                        None => {
                                            return Err("boxed_range_1 function not found".to_string())
                                        }
                                    };

                                    let call_site_value = self
                                        .builder
                                        .build_call(boxed_range_1_fn, &[boxed_len_val.into()], "boxed_range_1_result")
                                        .unwrap();

                                    let range_val = call_site_value
                                        .try_as_basic_value()
                                        .left()
                                        .ok_or_else(|| "Failed to get range value".to_string())?;

                                    return Ok((range_val, Type::Int));
                                }
                            }
                        }
                    }
                }

                // Handle function calls with BoxedAny parameters
                if let Expr::Name { id, .. } = func.as_ref() {
                    // Special case for range with constant integer arguments
                    if id == "range" {
                        // Get the boxed_any_from_int function for converting integers to BoxedAny pointers
                        let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                            .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                        match args.len() {
                            1 => {
                                if let Expr::Num { value, .. } = args[0].as_ref() {
                                    if let crate::ast::Number::Integer(n) = value {
                                        // Convert the integer to a BoxedAny pointer
                                        let int_val = self.llvm_context.i64_type().const_int(*n as u64, false);

                                        let boxed_int_val = self.builder.build_call(
                                            boxed_any_from_int_fn,
                                            &[int_val.into()],
                                            "boxed_int"
                                        ).unwrap().try_as_basic_value().left()
                                            .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?;

                                        // Use boxed_range_1 instead of range_1
                                        let boxed_range_1_fn = match self.module.get_function("boxed_range_1") {
                                            Some(f) => f,
                                            None => {
                                                return Err("boxed_range_1 function not found".to_string())
                                            }
                                        };

                                        let call_site_value = self
                                            .builder
                                            .build_call(boxed_range_1_fn, &[boxed_int_val.into()], "boxed_range_1_result")
                                            .unwrap();

                                        let range_val = call_site_value
                                            .try_as_basic_value()
                                            .left()
                                            .ok_or_else(|| "Failed to get range value".to_string())?;

                                        return Ok((range_val, Type::Int));
                                    }
                                }
                            },
                            2 => {
                                // Handle range(start, stop)
                                if let (Expr::Num { value: start_value, .. }, Expr::Num { value: stop_value, .. }) =
                                    (args[0].as_ref(), args[1].as_ref()) {
                                    if let (crate::ast::Number::Integer(start), crate::ast::Number::Integer(stop)) =
                                        (start_value, stop_value) {
                                        // Convert the integers to BoxedAny pointers
                                        let start_val = self.llvm_context.i64_type().const_int(*start as u64, false);
                                        let stop_val = self.llvm_context.i64_type().const_int(*stop as u64, false);

                                        let boxed_start_val = self.builder.build_call(
                                            boxed_any_from_int_fn,
                                            &[start_val.into()],
                                            "boxed_start"
                                        ).unwrap().try_as_basic_value().left()
                                            .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?;

                                        let boxed_stop_val = self.builder.build_call(
                                            boxed_any_from_int_fn,
                                            &[stop_val.into()],
                                            "boxed_stop"
                                        ).unwrap().try_as_basic_value().left()
                                            .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?;

                                        // Use boxed_range_2 instead of range_2
                                        let boxed_range_2_fn = match self.module.get_function("boxed_range_2") {
                                            Some(f) => f,
                                            None => {
                                                return Err("boxed_range_2 function not found".to_string())
                                            }
                                        };

                                        let call_site_value = self
                                            .builder
                                            .build_call(
                                                boxed_range_2_fn,
                                                &[boxed_start_val.into(), boxed_stop_val.into()],
                                                "boxed_range_2_result"
                                            )
                                            .unwrap();

                                        let range_val = call_site_value
                                            .try_as_basic_value()
                                            .left()
                                            .ok_or_else(|| "Failed to get range value".to_string())?;

                                        return Ok((range_val, Type::Int));
                                    }
                                }
                            },
                            3 => {
                                // Handle range(start, stop, step)
                                if let (Expr::Num { value: start_value, .. },
                                       Expr::Num { value: stop_value, .. },
                                       Expr::Num { value: step_value, .. }) =
                                    (args[0].as_ref(), args[1].as_ref(), args[2].as_ref()) {
                                    if let (crate::ast::Number::Integer(start),
                                           crate::ast::Number::Integer(stop),
                                           crate::ast::Number::Integer(step)) =
                                        (start_value, stop_value, step_value) {
                                        // Convert the integers to BoxedAny pointers
                                        let start_val = self.llvm_context.i64_type().const_int(*start as u64, false);
                                        let stop_val = self.llvm_context.i64_type().const_int(*stop as u64, false);
                                        let step_val = self.llvm_context.i64_type().const_int(*step as u64, false);

                                        let boxed_start_val = self.builder.build_call(
                                            boxed_any_from_int_fn,
                                            &[start_val.into()],
                                            "boxed_start"
                                        ).unwrap().try_as_basic_value().left()
                                            .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?;

                                        let boxed_stop_val = self.builder.build_call(
                                            boxed_any_from_int_fn,
                                            &[stop_val.into()],
                                            "boxed_stop"
                                        ).unwrap().try_as_basic_value().left()
                                            .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?;

                                        let boxed_step_val = self.builder.build_call(
                                            boxed_any_from_int_fn,
                                            &[step_val.into()],
                                            "boxed_step"
                                        ).unwrap().try_as_basic_value().left()
                                            .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?;

                                        // Use boxed_range_3 instead of range_3
                                        let boxed_range_3_fn = match self.module.get_function("boxed_range_3") {
                                            Some(f) => f,
                                            None => {
                                                return Err("boxed_range_3 function not found".to_string())
                                            }
                                        };

                                        let call_site_value = self
                                            .builder
                                            .build_call(
                                                boxed_range_3_fn,
                                                &[boxed_start_val.into(), boxed_stop_val.into(), boxed_step_val.into()],
                                                "boxed_range_3_result"
                                            )
                                            .unwrap();

                                        let range_val = call_site_value
                                            .try_as_basic_value()
                                            .left()
                                            .ok_or_else(|| "Failed to get range value".to_string())?;

                                        return Ok((range_val, Type::Int));
                                    }
                                }
                            },
                            _ => {}
                        }
                    }

                    // Special case for range with variable arguments
                    if id == "range" {
                        // Get the boxed_any_from_int function for converting integers to BoxedAny pointers
                        let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                            .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                        match args.len() {
                            1 => {
                                // Compile the argument
                                let (arg_val, arg_type) = self.compile_expr(&args[0])?;

                                // Convert to BoxedAny if needed
                                let boxed_arg = if arg_type == Type::Any {
                                    arg_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_arg_val = if arg_type != Type::Int {
                                        self.convert_type(arg_val, &arg_type, &Type::Int)?.into_int_value()
                                    } else if arg_val.is_pointer_value() {
                                        self.builder
                                            .build_load(self.llvm_context.i64_type(), arg_val.into_pointer_value(), "range_arg")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        arg_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_arg_val.into()],
                                        "boxed_arg"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                // Use boxed_range_1
                                let boxed_range_1_fn = match self.module.get_function("boxed_range_1") {
                                    Some(f) => f,
                                    None => {
                                        return Err("boxed_range_1 function not found".to_string())
                                    }
                                };

                                let call_site_value = self.builder.build_call(
                                    boxed_range_1_fn,
                                    &[boxed_arg.into()],
                                    "boxed_range_1_result"
                                ).unwrap();

                                let range_val = call_site_value
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or_else(|| "Failed to get range value".to_string())?;

                                return Ok((range_val, Type::Int));
                            },
                            2 => {
                                // Compile the arguments
                                let (start_val, start_type) = self.compile_expr(&args[0])?;
                                let (stop_val, stop_type) = self.compile_expr(&args[1])?;

                                // Convert to BoxedAny if needed
                                let boxed_start_val = if start_type == Type::Any {
                                    start_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_start_val = if start_type != Type::Int {
                                        self.convert_type(start_val, &start_type, &Type::Int)?.into_int_value()
                                    } else if start_val.is_pointer_value() {
                                        self.builder
                                            .build_load(self.llvm_context.i64_type(), start_val.into_pointer_value(), "range_start")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        start_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_start_val.into()],
                                        "boxed_start"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                let boxed_stop_val = if stop_type == Type::Any {
                                    stop_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_stop_val = if stop_type != Type::Int {
                                        self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                                    } else if stop_val.is_pointer_value() {
                                        self.builder
                                            .build_load(self.llvm_context.i64_type(), stop_val.into_pointer_value(), "range_stop")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        stop_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_stop_val.into()],
                                        "boxed_stop"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                // Use boxed_range_2
                                let boxed_range_2_fn = match self.module.get_function("boxed_range_2") {
                                    Some(f) => f,
                                    None => {
                                        return Err("boxed_range_2 function not found".to_string())
                                    }
                                };

                                let call_site_value = self.builder.build_call(
                                    boxed_range_2_fn,
                                    &[boxed_start_val.into(), boxed_stop_val.into()],
                                    "boxed_range_2_result"
                                ).unwrap();

                                let range_val = call_site_value
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or_else(|| "Failed to get range value".to_string())?;

                                return Ok((range_val, Type::Int));
                            },
                            3 => {
                                // Compile the arguments
                                let (start_val, start_type) = self.compile_expr(&args[0])?;
                                let (stop_val, stop_type) = self.compile_expr(&args[1])?;
                                let (step_val, step_type) = self.compile_expr(&args[2])?;

                                // Convert to BoxedAny if needed
                                let boxed_start_val = if start_type == Type::Any {
                                    start_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_start_val = if start_type != Type::Int {
                                        self.convert_type(start_val, &start_type, &Type::Int)?.into_int_value()
                                    } else if start_val.is_pointer_value() {
                                        self.builder
                                            .build_load(self.llvm_context.i64_type(), start_val.into_pointer_value(), "range_start")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        start_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_start_val.into()],
                                        "boxed_start"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                let boxed_stop_val = if stop_type == Type::Any {
                                    stop_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_stop_val = if stop_type != Type::Int {
                                        self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                                    } else if stop_val.is_pointer_value() {
                                        self.builder
                                            .build_load(self.llvm_context.i64_type(), stop_val.into_pointer_value(), "range_stop")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        stop_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_stop_val.into()],
                                        "boxed_stop"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                let boxed_step_val = if step_type == Type::Any {
                                    step_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_step_val = if step_type != Type::Int {
                                        self.convert_type(step_val, &step_type, &Type::Int)?.into_int_value()
                                    } else if step_val.is_pointer_value() {
                                        self.builder
                                            .build_load(self.llvm_context.i64_type(), step_val.into_pointer_value(), "range_step")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        step_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_step_val.into()],
                                        "boxed_step"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                // Use boxed_range_3
                                let boxed_range_3_fn = match self.module.get_function("boxed_range_3") {
                                    Some(f) => f,
                                    None => {
                                        return Err("boxed_range_3 function not found".to_string())
                                    }
                                };

                                let call_site_value = self.builder.build_call(
                                    boxed_range_3_fn,
                                    &[boxed_start_val.into(), boxed_stop_val.into(), boxed_step_val.into()],
                                    "boxed_range_3_result"
                                ).unwrap();

                                let range_val = call_site_value
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or_else(|| "Failed to get range value".to_string())?;

                                return Ok((range_val, Type::Int));
                            },
                            _ => {
                                return Err(format!("Invalid number of arguments for range: expected 1, 2, or 3, got {}", args.len()));
                            }
                        }
                    }

                    // Compile the arguments for other function calls
                    let mut arg_values = Vec::with_capacity(args.len());

                    for arg in args {
                        let (arg_val, arg_type) = self.compile_expr(arg)?;

                        // Convert primitive types to BoxedAny if needed
                        let boxed_arg = match arg_type {
                            Type::Int => {
                                // Convert Int to BoxedAny
                                let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                                    .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                                let call_site_value = self.builder.build_call(
                                    boxed_any_from_int_fn,
                                    &[arg_val.into()],
                                    "int_to_boxed"
                                ).unwrap();

                                call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to convert Int to BoxedAny".to_string())?
                            },
                            Type::Float => {
                                // Convert Float to BoxedAny
                                let boxed_any_from_float_fn = self.module.get_function("boxed_any_from_float")
                                    .ok_or_else(|| "boxed_any_from_float function not found".to_string())?;

                                let call_site_value = self.builder.build_call(
                                    boxed_any_from_float_fn,
                                    &[arg_val.into()],
                                    "float_to_boxed"
                                ).unwrap();

                                call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to convert Float to BoxedAny".to_string())?
                            },
                            Type::Bool => {
                                // Convert Bool to BoxedAny
                                let boxed_any_from_bool_fn = self.module.get_function("boxed_any_from_bool")
                                    .ok_or_else(|| "boxed_any_from_bool function not found".to_string())?;

                                let call_site_value = self.builder.build_call(
                                    boxed_any_from_bool_fn,
                                    &[arg_val.into()],
                                    "bool_to_boxed"
                                ).unwrap();

                                call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to convert Bool to BoxedAny".to_string())?
                            },
                            _ => arg_val,
                        };

                        arg_values.push(boxed_arg);
                    }

                    // Look for the function
                    let qualified_name = if let Some(current_function) = self.current_function {
                        let fn_name = current_function.get_name().to_string_lossy();
                        if fn_name.contains('.') {
                            format!("{}.{}", fn_name, id)
                        } else {
                            id.clone()
                        }
                    } else {
                        id.clone()
                    };

                    println!("Looking for nested function: {}", qualified_name);

                    let func_value = if let Some(&f) = self.functions.get(&qualified_name) {
                        f
                    } else if let Some(&f) = self.functions.get(id) {
                        f
                    } else {
                        return Err(format!("Function not found: {}", id));
                    };

                    // Call the function
                    let call_site_value = self.builder.build_call(
                        func_value,
                        &arg_values.iter().map(|v| (*v).into()).collect::<Vec<_>>(),
                        &format!("call_{}", id)
                    ).unwrap();

                    let result = call_site_value.try_as_basic_value().left()
                        .ok_or_else(|| format!("Failed to call function {}", id))?;

                    // Return the result as Type::Any
                    Ok((result, Type::Any))
                } else {
                    // For other function types, use the original implementation
                    <Self as ExprCompiler>::compile_expr_original(self, expr)
                }
            }
            _ => <Self as ExprCompiler>::compile_expr_original(self, expr),
        }
    }
}
