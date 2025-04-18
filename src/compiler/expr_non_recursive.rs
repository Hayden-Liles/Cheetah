// Non-recursive implementation of the expression compiler
// This implementation avoids deep recursion by using an explicit work stack

use crate::ast::{BoolOperator, CmpOperator, Expr, Operator, UnaryOperator};
use crate::compiler::context::CompilationContext;
use crate::compiler::types::Type;
use crate::compiler::expr::{ExprCompiler, BinaryOpCompiler};
use inkwell::values::BasicValueEnum;
use std::collections::VecDeque;

// This trait is used to extend the CompilationContext with non-recursive expression compilation
pub trait ExprNonRecursive<'ctx> {
    fn compile_expr_non_recursive(&mut self, expr: &crate::ast::Expr) -> Result<(BasicValueEnum<'ctx>, crate::compiler::types::Type), String>;
}

// Task for the work stack
#[derive(Clone, Debug)]
enum ExprTask<'a, 'ctx> {
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

    // Process function call keywords - evaluate each keyword argument
    ProcessFunctionCallKeywords {
        func_idx: usize,
        args_count: usize,
        keywords: &'a [(Option<String>, Box<Expr>)],
        processed_keywords: usize,
    },

    // Finalize a function call after evaluating all arguments
    FinalizeFunctionCall {
        func_idx: usize,
        args_count: usize,
        keywords_count: usize,
        func_name: Option<String>, // Store the function name if available for special handling
    },

    // Special task for handling len() calls
    ProcessLenCall {
        args: &'a [Box<Expr>],
    },

    // Special task for handling print() calls
    ProcessPrintCall {
        args: &'a [Box<Expr>],
    },

    // Prepare a list comprehension
    PrepareListComprehension {
        elt: &'a Expr,
        generators: &'a [crate::ast::Comprehension],
    },

    // Prepare a dictionary comprehension
    PrepareDictComprehension {
        key: &'a Expr,
        value: &'a Expr,
        generators: &'a [crate::ast::Comprehension],
    },

    // Process a function call - evaluate the function and arguments
    PrepareFunctionCall {
        func: &'a Expr,
        args: &'a [Box<Expr>],
        keywords: &'a [(Option<String>, Box<Expr>)],
    },

    // Process function call arguments - evaluate each argument
    ProcessFunctionCallArgs {
        func_idx: usize,
        args: &'a [Box<Expr>],
        processed_args: usize,
        keywords: &'a [(Option<String>, Box<Expr>)],
    },

    // Process an attribute access
    ProcessAttribute {
        attr: String,
    },

    // Process a subscript operation
    ProcessSubscript {
        value_idx: usize,
        slice_idx: usize,
    },

    // Process a tuple creation
    ProcessTuple {
        elements_count: usize,
    },

    // Process a list creation
    ProcessList {
        elements_count: usize,
    },

    // Process a dictionary creation
    ProcessDict {
        elements_count: usize,
    },

    // Process a set creation
    ProcessSet {
        elements_count: usize,
    },

    // Process a dictionary comprehension key
    EvaluateDictCompKey {
        key: &'a Expr,
        value: &'a Expr,
        generators: &'a [crate::ast::Comprehension],
        gen_idx: usize,
    },

    // Process a dictionary comprehension value
    EvaluateDictCompValue {
        key: &'a Expr,
        value: &'a Expr,
        generators: &'a [crate::ast::Comprehension],
        gen_idx: usize,
        key_idx: usize,
    },

    // Add a key-value pair to a dictionary comprehension
    AddPairToDictComp {
        key: &'a Expr,
        value: &'a Expr,
        generators: &'a [crate::ast::Comprehension],
        gen_idx: usize,
        key_idx: usize,
        value_idx: usize,
    },

    // Process the next generator or finalize a dictionary comprehension
    ProcessNextDictGeneratorOrFinalize {
        key: &'a Expr,
        value: &'a Expr,
        generators: &'a [crate::ast::Comprehension],
        gen_idx: usize,
        dict_ptr: Option<inkwell::values::PointerValue<'ctx>>,
        key_type: Option<Type>,
        value_type: Option<Type>,
    },

    // Finalize dictionary comprehension
    FinalizeDictComprehension {
        dict_ptr: inkwell::values::PointerValue<'ctx>,
        key_type: Type,
        value_type: Type,
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
                        // Add other expression types here
                        _ => {
                            return Err(format!("Unsupported expression type: {:?}", expr));
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
                ExprTask::ProcessFunctionCallKeywords { .. } => {
                    // This is a simplified implementation
                    return Err("ProcessFunctionCallKeywords not fully implemented".to_string());
                },
                _ => {
                    // Handle other task types
                    return Err(format!("Unsupported task type: {:?}", task));
                }
            }
        }

        // The final result should be the only item on the result stack
        if result_stack.len() != 1 {
            return Err(format!("Expected 1 result, but got {} results", result_stack.len()));
        }

        let final_result = result_stack.pop().unwrap();
        Ok((final_result.value, final_result.ty))
    }
}
