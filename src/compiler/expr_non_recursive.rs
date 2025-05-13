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

                        println!("Looking up variable: {}", id);

                        // First, try to find the variable in the current scope stack
                        if let Some(var_ptr) =
                            self.scope_stack.get_variable_respecting_declarations(id)
                        {
                            if let Some(var_type) =
                                self.scope_stack.get_type_respecting_declarations(id)
                            {
                                let is_nonlocal =
                                    if let Some(current_scope) = self.scope_stack.current_scope() {
                                        current_scope.is_nonlocal(id)
                                    } else {
                                        false
                                    };

                                let var_val = if is_nonlocal {
                                    let llvm_type = self.get_llvm_type(&var_type);
                                    self.builder
                                        .build_load(llvm_type, *var_ptr, &format!("load_{}", id))
                                        .unwrap()
                                } else {
                                    let llvm_type = self.get_llvm_type(&var_type);
                                    self.builder
                                        .build_load(llvm_type, *var_ptr, &format!("load_{}", id))
                                        .unwrap()
                                };

                                println!("Found variable '{}' in scope stack with type: {:?}", id, var_type);
                                result_stack.push(ExprResult {
                                    value: var_val,
                                    ty: var_type,
                                });
                            } else {
                                return Err(format!("Variable found but type unknown: {}", id));
                            }
                        }
                        // Next, try to find the variable in the global variables
                        else if let Some(var_ptr) = self.variables.get(id) {
                            if let Some(var_type) = self.type_env.get(id) {
                                let llvm_type = self.get_llvm_type(var_type);

                                let var_val = self
                                    .builder
                                    .build_load(llvm_type, *var_ptr, &format!("load_{}", id))
                                    .unwrap();

                                self.ensure_block_has_terminator();

                                println!("Found variable '{}' in global variables with type: {:?}", id, var_type);
                                result_stack.push(ExprResult {
                                    value: var_val,
                                    ty: var_type.clone(),
                                });
                            } else {
                                return Err(format!(
                                    "Global variable found but type unknown: {}",
                                    id
                                ));
                            }
                        }
                        // Special case for nested list comprehensions - try to find the variable in any scope
                        else {
                            // Try to find the variable in any scope, not just respecting declarations
                            // This is needed for nested list comprehensions where variables from outer
                            // comprehensions need to be accessible in inner comprehensions
                            if let Some(var_ptr) = self.scope_stack.get_variable(id) {
                                if let Some(var_type) = self.scope_stack.get_type(id) {
                                    let llvm_type = self.get_llvm_type(var_type);

                                    let var_val = self.builder
                                        .build_load(llvm_type, *var_ptr, &format!("load_{}", id))
                                        .unwrap();

                                    println!("Found variable '{}' in any scope with type: {:?}", id, var_type);
                                    result_stack.push(ExprResult {
                                        value: var_val,
                                        ty: var_type.clone(),
                                    });
                                } else {
                                    return Err(format!("Variable found but type unknown: {}", id));
                                }
                            } else {
                                return Err(format!("Undefined variable: {}", id));
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

                    Expr::Call { .. } => {
                        // We'll handle all calls the same way

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

                    let (result_value, result_type) = self.compile_binary_op(
                        left_result.value,
                        &left_result.ty,
                        op,
                        right_result.value,
                        &right_result.ty,
                    )?;

                    result_stack.remove(right_idx);
                    result_stack.remove(left_idx);

                    result_stack.push(ExprResult {
                        value: result_value,
                        ty: result_type,
                    });
                }
                ExprTask::ProcessUnaryOp { op, operand_idx } => {
                    if operand_idx >= result_stack.len() {
                        return Err("Invalid result stack index for unary operation".to_string());
                    }

                    let operand_result = &result_stack[operand_idx];

                    let (result_value, result_type) = match op {
                        UnaryOperator::Not => {
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
                    let phi = self.builder.build_phi(llvm_type, "if_result").unwrap();

                    phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);

                    result_stack.remove(test_idx);

                    result_stack.push(ExprResult {
                        value: phi.as_basic_value(),
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
                    let mut element_types = Vec::with_capacity(elements_count);
                    let mut common_element_type = Type::Unknown;

                    for _ in 0..elements_count {
                        let idx = result_stack.len() - 1;
                        let element = result_stack.remove(idx);
                        elements.push((element.value, element.ty.clone()));
                        element_types.push(element.ty.clone());

                        if common_element_type == Type::Unknown {
                            common_element_type = element.ty;
                        } else if common_element_type != element.ty {
                            common_element_type = Type::Any;
                        }
                    }

                    elements.reverse();
                    element_types.reverse();

                    let list_ptr = self.build_list(elements, &common_element_type)?;

                    result_stack.push(ExprResult {
                        value: list_ptr.into(),
                        ty: Type::List(Box::new(common_element_type)),
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

                                    let range_1_fn = match self.module.get_function("range_1") {
                                        Some(f) => f,
                                        None => {
                                            return Err("range_1 function not found".to_string())
                                        }
                                    };

                                    let call_site_value = self
                                        .builder
                                        .build_call(range_1_fn, &[len_val.into()], "range_1_result")
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

                <Self as ExprCompiler>::compile_expr_original(self, expr)
            }
            _ => <Self as ExprCompiler>::compile_expr_original(self, expr),
        }
    }
}
