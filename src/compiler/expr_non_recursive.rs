// Non-recursive implementation of the expression compiler
// This implementation avoids deep recursion by using an explicit work stack

use crate::ast::{BoolOperator, CmpOperator, Expr, Operator, UnaryOperator};
use crate::compiler::context::CompilationContext;
use crate::compiler::types::Type;
use crate::compiler::expr::{ExprCompiler, BinaryOpCompiler, ComparisonCompiler};
use inkwell::values::BasicValueEnum;
use std::collections::VecDeque;

// This trait is used to extend the CompilationContext with non-recursive expression compilation
pub trait ExprNonRecursive<'ctx> {
    fn compile_expr_non_recursive(&mut self, expr: &crate::ast::Expr) -> Result<(BasicValueEnum<'ctx>, crate::compiler::types::Type), String>;

    // This is a helper method for the non-recursive implementation
    fn compile_expr_original(&mut self, expr: &crate::ast::Expr) -> Result<(BasicValueEnum<'ctx>, crate::compiler::types::Type), String>;
}

// Task for the work stack
enum ExprTask<'a> {
    // Evaluate an expression and push the result to the result stack
    Evaluate(&'a Expr),

    // Process a binary operation with operands from the result stack
    ProcessBinaryOp {
        op: Operator,
    },

    // Process a unary operation with operand from the result stack
    ProcessUnaryOp {
        op: UnaryOperator,
        operand_idx: usize,
    },

    // Process a comparison with operands from the result stack
    ProcessComparison {
        op: CmpOperator,
    },

    // Process a boolean operation with operands from the result stack
    ProcessBoolOp {
        op: BoolOperator,
    },

    // Process an if expression
    ProcessIfExpression {
        then_block: inkwell::basic_block::BasicBlock<'a>,
        else_block: inkwell::basic_block::BasicBlock<'a>,
        merge_block: inkwell::basic_block::BasicBlock<'a>,
        body: Box<Expr>,
        orelse: Box<Expr>,
    },
}

// Result of an expression evaluation
struct ExprResult<'ctx> {
    value: BasicValueEnum<'ctx>,
    ty: Type,
}

impl<'ctx> ExprNonRecursive<'ctx> for CompilationContext<'ctx> {
    // Non-recursive implementation of compile_expr
    fn compile_expr_non_recursive(&mut self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Create work stack and result stack
        let mut work_stack: VecDeque<ExprTask> = VecDeque::new();
        let mut result_stack: Vec<ExprResult<'ctx>> = Vec::new();

        // Start by evaluating the top-level expression
        work_stack.push_back(ExprTask::Evaluate(expr));

        // Process tasks until the work stack is empty
        while let Some(task) = work_stack.pop_front() {
            match task {
                ExprTask::Evaluate(expr) => {
                    // Handle different expression types
                    match expr {
                        Expr::Num { value, .. } => {
                            // Compile number literal
                            let (value, ty) = self.compile_number(value)?;
                            result_stack.push(ExprResult { value, ty });
                        },
                        Expr::NameConstant { value, .. } => {
                            // Compile name constant (True, False, None)
                            let (value, ty) = self.compile_name_constant(value)?;
                            result_stack.push(ExprResult { value, ty });
                        },
                        Expr::BinOp { left, op, right, .. } => {
                            // First, add the task to process the binary operation after the operands are evaluated
                            work_stack.push_front(ExprTask::ProcessBinaryOp {
                                op: op.clone(),
                            });

                            // Then, evaluate the right operand (will be processed first, pushed to stack second)
                            work_stack.push_front(ExprTask::Evaluate(right));

                            // Finally, evaluate the left operand (will be processed second, pushed to stack first)
                            work_stack.push_front(ExprTask::Evaluate(left));
                        },
                        Expr::UnaryOp { op, operand, .. } => {
                            // Push tasks to evaluate the operand and then process the unary operation
                            let operand_idx = result_stack.len();

                            work_stack.push_front(ExprTask::ProcessUnaryOp {
                                op: op.clone(),
                                operand_idx,
                            });
                            work_stack.push_front(ExprTask::Evaluate(operand));
                        },
                        Expr::Compare { left, ops, comparators, .. } => {
                            // For comparisons like a < b < c, we need to evaluate each operand
                            // and then process each comparison in sequence

                            if ops.is_empty() || comparators.is_empty() || ops.len() != comparators.len() {
                                return Err("Invalid comparison expression".to_string());
                            }

                            // For a single comparison (a < b), we can use the simple approach
                            if ops.len() == 1 {
                                // First, add the task to process the comparison after the operands are evaluated
                                work_stack.push_front(ExprTask::ProcessComparison {
                                    op: ops[0].clone(),
                                });

                                // Then, evaluate the right operand (will be processed first, pushed to stack second)
                                work_stack.push_front(ExprTask::Evaluate(&comparators[0]));

                                // Finally, evaluate the left operand (will be processed second, pushed to stack first)
                                work_stack.push_front(ExprTask::Evaluate(left));
                            } else if ops.len() == 2 {
                                // For chained comparisons like a < b < c, we'll handle this differently
                                // We'll evaluate each part separately and combine with AND operations

                                // For a < b < c, we need to evaluate (a < b) and (b < c)
                                // First, create a boolean operation to AND the results
                                work_stack.push_front(ExprTask::ProcessBoolOp {
                                    op: BoolOperator::And,
                                });

                                // Second comparison: b < c
                                work_stack.push_front(ExprTask::ProcessComparison {
                                    op: ops[1].clone(),
                                });
                                work_stack.push_front(ExprTask::Evaluate(&comparators[1])); // c
                                work_stack.push_front(ExprTask::Evaluate(&comparators[0])); // b

                                // First comparison: a < b
                                work_stack.push_front(ExprTask::ProcessComparison {
                                    op: ops[0].clone(),
                                });
                                work_stack.push_front(ExprTask::Evaluate(&comparators[0])); // b
                                work_stack.push_front(ExprTask::Evaluate(left)); // a
                            } else {
                                // For longer comparison chains, we need a more general approach
                                // We'll build a series of AND operations for each comparison

                                // Start with the last comparison and work backwards
                                // This ensures the operations are processed in the correct order

                                // For each pair of comparisons, we need to AND them together
                                for i in (1..ops.len()).rev() {
                                    // Add the AND operation for this pair
                                    if i < ops.len() - 1 {
                                        work_stack.push_front(ExprTask::ProcessBoolOp {
                                            op: BoolOperator::And,
                                        });
                                    }

                                    // Add the comparison operation
                                    work_stack.push_front(ExprTask::ProcessComparison {
                                        op: ops[i].clone(),
                                    });

                                    // Add the operands for this comparison
                                    work_stack.push_front(ExprTask::Evaluate(&comparators[i])); // right operand
                                    work_stack.push_front(ExprTask::Evaluate(&comparators[i-1])); // left operand
                                }

                                // Finally, add the first comparison
                                work_stack.push_front(ExprTask::ProcessBoolOp {
                                    op: BoolOperator::And,
                                });

                                work_stack.push_front(ExprTask::ProcessComparison {
                                    op: ops[0].clone(),
                                });
                                work_stack.push_front(ExprTask::Evaluate(&comparators[0])); // right operand
                                work_stack.push_front(ExprTask::Evaluate(left)); // left operand
                            }
                        },
                        Expr::BoolOp { op, values, .. } => {
                            if values.is_empty() {
                                return Err("Empty boolean operation".to_string());
                            }

                            if values.len() == 1 {
                                // If there's only one value, just evaluate it
                                work_stack.push_front(ExprTask::Evaluate(&values[0]));
                            } else if values.len() == 2 {
                                // For two values, we can use the simple approach
                                // First, add the task to process the boolean operation
                                work_stack.push_front(ExprTask::ProcessBoolOp {
                                    op: op.clone(),
                                });

                                // Then, evaluate the right operand (will be processed first, pushed to stack second)
                                work_stack.push_front(ExprTask::Evaluate(&values[1]));

                                // Finally, evaluate the left operand (will be processed second, pushed to stack first)
                                work_stack.push_front(ExprTask::Evaluate(&values[0]));
                            } else {
                                // For more than two values, we need to chain the operations
                                // For example, a and b and c becomes (a and b) and c

                                // Start with the last two values and work backwards
                                // This ensures the operations are processed in the correct order

                                // Process the last pair of values
                                let last_idx = values.len() - 1;
                                let second_last_idx = last_idx - 1;

                                // Add the boolean operation for the last pair
                                work_stack.push_front(ExprTask::ProcessBoolOp {
                                    op: op.clone(),
                                });

                                // Add the last two values
                                work_stack.push_front(ExprTask::Evaluate(&values[last_idx]));
                                work_stack.push_front(ExprTask::Evaluate(&values[second_last_idx]));

                                // Process the remaining values in reverse order
                                for i in (0..second_last_idx).rev() {
                                    // Add the boolean operation for this pair
                                    work_stack.push_front(ExprTask::ProcessBoolOp {
                                        op: op.clone(),
                                    });

                                    // Add the current value
                                    work_stack.push_front(ExprTask::Evaluate(&values[i]));

                                    // The second operand will be the result of the previous operation
                                    // which will be on top of the result stack
                                }
                            }
                        },
                        Expr::Name { id, .. } => {
                            // We don't need to check if the variable is global or nonlocal here
                            // because get_variable_respecting_declarations handles that for us

                            // Get the variable respecting global and nonlocal declarations
                            if let Some(var_ptr) = self.scope_stack.get_variable_respecting_declarations(id) {
                                // Get the variable type
                                if let Some(var_type) = self.scope_stack.get_type_respecting_declarations(id) {
                                    // Load the variable value
                                    let var_val = self.builder.build_load(
                                        self.get_llvm_type(&var_type),
                                        *var_ptr,
                                        &format!("load_{}", id)
                                    ).unwrap();

                                    result_stack.push(ExprResult { value: var_val, ty: var_type });
                                } else {
                                    return Err(format!("Variable found but type unknown: {}", id));
                                }
                            } else {
                                // Variable not found
                                return Err(format!("Undefined variable: {}", id));
                            }
                        },
                        Expr::IfExp { test, body, orelse, .. } => {
                            // For if expressions like `x if condition else y`, we need to evaluate the condition
                            // and then either the body or the orelse expression

                            // Create basic blocks for the then, else, and merge parts
                            let current_block = self.builder.get_insert_block().unwrap();
                            let current_function = current_block.get_parent().unwrap();
                            let then_block = self.llvm_context.append_basic_block(current_function, "if_then");
                            let else_block = self.llvm_context.append_basic_block(current_function, "if_else");
                            let merge_block = self.llvm_context.append_basic_block(current_function, "if_merge");

                            // First, add the task to process the if expression after the test is evaluated
                            // The test result will be at the top of the stack
                            work_stack.push_front(ExprTask::ProcessIfExpression {
                                then_block,
                                else_block,
                                merge_block,
                                body: body.clone(),
                                orelse: orelse.clone(),
                            });

                            // Then, evaluate the test condition
                            // This will be processed first, and its result will be on the stack
                            // when the ProcessIfExpression task is executed
                            work_stack.push_front(ExprTask::Evaluate(test));
                        },

                        // For other expression types, fall back to the original recursive implementation
                        _ => {
                            let (value, ty) = <Self as ExprCompiler>::compile_expr_original(self, expr)?;
                            result_stack.push(ExprResult { value, ty });
                        }
                    }
                },
                ExprTask::ProcessBinaryOp { op } => {
                    // Get the operands from the result stack
                    if result_stack.len() < 2 {
                        return Err(format!("Not enough operands for binary operation: stack size = {}", result_stack.len()));
                    }

                    // The operands should be the last two items on the stack
                    // The right operand is on top (last pushed), the left operand is below it
                    let right_idx = result_stack.len() - 1;
                    let left_idx = right_idx - 1;

                    let right_result = &result_stack[right_idx];
                    let left_result = &result_stack[left_idx];

                    // Process the binary operation
                    let (result_value, result_type) = self.compile_binary_op(
                        left_result.value,
                        &left_result.ty,
                        op,
                        right_result.value,
                        &right_result.ty
                    )?;

                    // Remove the operands from the result stack
                    // Note: We need to remove the right operand first since it has a higher index
                    result_stack.remove(right_idx);
                    result_stack.remove(left_idx);

                    // Push the result onto the result stack
                    result_stack.push(ExprResult {
                        value: result_value,
                        ty: result_type,
                    });
                },
                ExprTask::ProcessUnaryOp { op, operand_idx } => {
                    // Get the operand from the result stack
                    if operand_idx >= result_stack.len() {
                        return Err("Invalid result stack index for unary operation".to_string());
                    }

                    let operand_result = &result_stack[operand_idx];

                    // Process the unary operation
                    let (result_value, result_type) = match op {
                        UnaryOperator::Not => {
                            // Convert to bool if needed
                            let bool_val = if !matches!(operand_result.ty, Type::Bool) {
                                self.convert_type(operand_result.value, &operand_result.ty, &Type::Bool)?
                            } else {
                                operand_result.value
                            };

                            let result = self.builder.build_not(bool_val.into_int_value(), "not").unwrap();
                            (result.into(), Type::Bool)
                        },
                        UnaryOperator::USub => {
                            match operand_result.ty {
                                Type::Int => {
                                    let int_val = operand_result.value.into_int_value();
                                    let result = self.builder.build_int_neg(int_val, "neg").unwrap();
                                    (result.into(), Type::Int)
                                },
                                Type::Float => {
                                    let float_val = operand_result.value.into_float_value();
                                    let result = self.builder.build_float_neg(float_val, "neg").unwrap();
                                    (result.into(), Type::Float)
                                },
                                _ => return Err(format!("Cannot negate value of type {:?}", operand_result.ty)),
                            }
                        },
                        UnaryOperator::UAdd => {
                            // Unary plus is a no-op
                            (operand_result.value, operand_result.ty.clone())
                        },
                        UnaryOperator::Invert => {
                            // Bitwise NOT (~)
                            match operand_result.ty {
                                Type::Int => {
                                    let int_val = operand_result.value.into_int_value();
                                    let result = self.builder.build_not(int_val, "invert").unwrap();
                                    (result.into(), Type::Int)
                                },
                                _ => return Err(format!("Cannot bitwise invert value of type {:?}", operand_result.ty)),
                            }
                        },
                    };

                    // Remove the operand from the result stack
                    result_stack.remove(operand_idx);

                    // Push the result onto the result stack
                    result_stack.push(ExprResult {
                        value: result_value,
                        ty: result_type,
                    });
                },
                ExprTask::ProcessComparison { op } => {
                    // Get the operands from the result stack
                    if result_stack.len() < 2 {
                        return Err(format!("Not enough operands for comparison operation: stack size = {}", result_stack.len()));
                    }

                    // The operands should be the last two items on the stack
                    // The right operand is on top (last pushed), the left operand is below it
                    let right_idx = result_stack.len() - 1;
                    let left_idx = right_idx - 1;

                    // Clone the values to avoid borrowing issues
                    let left_value = result_stack[left_idx].value;
                    let left_type = result_stack[left_idx].ty.clone();
                    let right_value = result_stack[right_idx].value;
                    let right_type = result_stack[right_idx].ty.clone();

                    // Process the comparison operation
                    let (result_value, result_type) = self.compile_comparison(
                        left_value,
                        &left_type,
                        op,
                        right_value,
                        &right_type
                    )?;

                    // Remove the operands from the result stack
                    // Note: We need to remove the right operand first since it has a higher index
                    if right_idx > left_idx {
                        result_stack.remove(right_idx);
                        result_stack.remove(left_idx);
                    } else {
                        result_stack.remove(left_idx);
                        result_stack.remove(right_idx);
                    }

                    // Push the result onto the result stack
                    result_stack.push(ExprResult {
                        value: result_value,
                        ty: result_type,
                    });
                },
                ExprTask::ProcessBoolOp { op } => {
                    // Process boolean operations (and, or)
                    // We need at least two values on the stack
                    if result_stack.len() < 2 {
                        return Err(format!("Not enough operands for boolean operation: stack size = {}", result_stack.len()));
                    }

                    // Get the last two values from the stack
                    let right_idx = result_stack.len() - 1;
                    let left_idx = right_idx - 1;

                    let right_result = &result_stack[right_idx];
                    let left_result = &result_stack[left_idx];

                    let right_value = right_result.value;
                    let right_type = right_result.ty.clone();
                    let left_value = left_result.value;
                    let left_type = left_result.ty.clone();

                    // Convert to boolean if needed
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

                    // Apply the boolean operation
                    let mut current_value = left_bool;

                    match op {
                        BoolOperator::And => {
                            // Short-circuit evaluation for 'and'
                            let cond_block = self.builder.get_insert_block().unwrap();
                            let current_function = cond_block.get_parent().unwrap();

                            let then_block = self.llvm_context.append_basic_block(current_function, "and_then");
                            let merge_block = self.llvm_context.append_basic_block(current_function, "and_merge");

                            // Branch based on the current value
                            self.builder.build_conditional_branch(
                                current_value.into_int_value(),
                                then_block,
                                merge_block
                            ).unwrap();

                            // Build the then block (evaluate the right operand)
                            self.builder.position_at_end(then_block);
                            let then_value = right_bool;
                            self.builder.build_unconditional_branch(merge_block).unwrap();
                            let then_block = self.builder.get_insert_block().unwrap();

                            // Build the merge block with phi node
                            self.builder.position_at_end(merge_block);
                            let phi = self.builder.build_phi(self.llvm_context.bool_type(), "and_result").unwrap();

                            phi.add_incoming(&[
                                (&self.llvm_context.bool_type().const_int(0, false), cond_block),
                                (&then_value.into_int_value(), then_block),
                            ]);

                            current_value = phi.as_basic_value();
                        },
                        BoolOperator::Or => {
                            // Short-circuit evaluation for 'or'
                            let cond_block = self.builder.get_insert_block().unwrap();
                            let current_function = cond_block.get_parent().unwrap();

                            let else_block = self.llvm_context.append_basic_block(current_function, "or_else");
                            let merge_block = self.llvm_context.append_basic_block(current_function, "or_merge");

                            // Branch based on the current value
                            self.builder.build_conditional_branch(
                                current_value.into_int_value(),
                                merge_block,
                                else_block
                            ).unwrap();

                            // Build the else block (evaluate the right operand)
                            self.builder.position_at_end(else_block);
                            let else_value = right_bool;
                            self.builder.build_unconditional_branch(merge_block).unwrap();
                            let else_block = self.builder.get_insert_block().unwrap();

                            // Build the merge block with phi node
                            self.builder.position_at_end(merge_block);
                            let phi = self.builder.build_phi(self.llvm_context.bool_type(), "or_result").unwrap();

                            phi.add_incoming(&[
                                (&self.llvm_context.bool_type().const_int(1, false), cond_block),
                                (&else_value.into_int_value(), else_block),
                            ]);

                            current_value = phi.as_basic_value();
                        },
                    }

                    // Remove the operands from the stack
                    result_stack.remove(right_idx);
                    result_stack.remove(left_idx);

                    // Push the result onto the result stack
                    result_stack.push(ExprResult {
                        value: current_value,
                        ty: Type::Bool,
                    });
                },
                ExprTask::ProcessIfExpression { then_block, else_block, merge_block, body, orelse } => {
                    // The test condition should be the last item on the result stack
                    if result_stack.is_empty() {
                        return Err("No test condition found for if expression".to_string());
                    }

                    let test_idx = result_stack.len() - 1;
                    let test_result = &result_stack[test_idx];
                    let test_val = test_result.value;
                    let test_type = test_result.ty.clone();

                    // Convert the test value to a boolean if needed
                    let cond_val = if test_type != Type::Bool {
                        self.convert_type(test_val, &test_type, &Type::Bool)?.into_int_value()
                    } else {
                        test_val.into_int_value()
                    };

                    // Create the conditional branch
                    self.builder.build_conditional_branch(cond_val, then_block, else_block).unwrap();

                    // Compile the then block
                    self.builder.position_at_end(then_block);
                    let (then_val, then_type) = self.compile_expr(&body)?;
                    self.builder.build_unconditional_branch(merge_block).unwrap();
                    let then_block = self.builder.get_insert_block().unwrap(); // Get the updated block

                    // Compile the else block
                    self.builder.position_at_end(else_block);
                    let (else_val, else_type) = self.compile_expr(&orelse)?;
                    self.builder.build_unconditional_branch(merge_block).unwrap();
                    let else_block = self.builder.get_insert_block().unwrap(); // Get the updated block

                    // Determine the result type (unify the then and else types)
                    let result_type = if then_type == else_type {
                        then_type.clone()
                    } else if then_type.can_coerce_to(&else_type) {
                        else_type.clone()
                    } else if else_type.can_coerce_to(&then_type) {
                        then_type.clone()
                    } else {
                        return Err(format!("Incompatible types in if expression: {:?} and {:?}", then_type, else_type));
                    };

                    // Convert the values to the result type if needed
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

                    // Create a merge block with phi node
                    self.builder.position_at_end(merge_block);

                    // Create the phi node
                    let llvm_type = self.get_llvm_type(&result_type);
                    let phi = self.builder.build_phi(llvm_type, "if_result").unwrap();

                    // Add the incoming values
                    phi.add_incoming(&[
                        (&then_val, then_block),
                        (&else_val, else_block),
                    ]);

                    // Remove the test value from the result stack
                    result_stack.remove(test_idx);

                    // Push the result onto the result stack
                    result_stack.push(ExprResult {
                        value: phi.as_basic_value(),
                        ty: result_type,
                    });
                },
                // Add more task types as needed
            }
        }

        // The final result should be the only item on the result stack
        if result_stack.len() != 1 {
            return Err(format!("Expected 1 result, but got {} results", result_stack.len()));
        }

        let final_result = result_stack.pop().unwrap();
        Ok((final_result.value, final_result.ty))
    }

    // This is a placeholder for the original implementation
    // In a real implementation, this would be the original recursive method
    fn compile_expr_original(&mut self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // For now, we'll implement a simple version that handles basic expressions
        // to avoid circular references
        match expr {
            Expr::Num { value, .. } => self.compile_number(value),
            Expr::NameConstant { value, .. } => self.compile_name_constant(value),
            _ => Err(format!("Unsupported expression type in fallback implementation: {:?}", expr)),
        }
    }
}
