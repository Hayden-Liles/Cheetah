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

    // Special task for handling range() calls
    ProcessRangeCall {
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

// Helper methods for the non-recursive implementation
impl<'ctx> CompilationContext<'ctx> {
    pub fn get_list_element(
        &self,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Get the list_get function
        let list_get_fn = match self.module.get_function("list_get") {
            Some(f) => f,
            None => return Err("list_get function not found".to_string()),
        };

        // Call list_get to get an item from the list
        let call_site_value = self.builder.build_call(
            list_get_fn,
            &[list_ptr.into(), index.into()],
            "list_get"
        ).unwrap();

        let item_ptr = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get item from list".to_string())?;

        // Check if the item is a pointer value
        if item_ptr.is_pointer_value() {
            Ok(item_ptr.into_pointer_value())
        } else {
            // If it's not a pointer value, allocate memory for it and store it
            let item_alloca = self.builder.build_alloca(
                item_ptr.get_type(),
                "list_item_alloca"
            ).unwrap();
            self.builder.build_store(item_alloca, item_ptr).unwrap();
            Ok(item_alloca)
        }
    }

    // Helper method to get a function from a pointer
    fn get_function_from_ptr(
        &self,
        _func_ptr: inkwell::values::PointerValue<'ctx>
    ) -> Option<inkwell::values::FunctionValue<'ctx>> {
        // In a real implementation, we would extract the function pointer from the function object
        // For now, we'll just assume the pointer is the function itself

        // Get the function by name (simplified implementation)
        // In a real implementation, we would extract the function name from the function object
        // and then look up the function in the module

        // For now, just return the first function in the module as a placeholder
        self.module.get_first_function()
    }
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
                        Expr::Name { id, ctx, .. } => {
                            // Handle variable references
                            match ctx {
                                crate::ast::ExprContext::Load => {
                                    // Get the variable pointer from the current scope
                                    let var_ptr = match self.get_variable_ptr(id) {
                                        Some(ptr) => ptr,
                                        None => return Err(format!("Variable '{}' not found", id)),
                                    };

                                    // Get the variable type
                                    let var_type = match self.lookup_variable_type(id) {
                                        Some(ty) => ty.clone(),
                                        None => return Err(format!("Type for variable '{}' not found", id)),
                                    };

                                    // Load the value from the pointer
                                    let llvm_type = self.get_llvm_type(&var_type);
                                    let value = self.builder.build_load(llvm_type, var_ptr, &format!("load_{}", id)).unwrap();

                                    result_stack.push(ExprResult { value, ty: var_type });
                                },
                                _ => {
                                    return Err(format!("Unsupported name context: {:?}", ctx));
                                }
                            }
                        },
                        Expr::Str { value, .. } => {
                            // Compile string literal
                            // Create a global string constant
                            let string_val = self.builder.build_global_string_ptr(value, "str_const").unwrap();
                            let string_ptr = string_val.as_pointer_value();
                            result_stack.push(ExprResult { value: string_ptr.into(), ty: Type::String });
                        },
                        Expr::NameConstant { value, .. } => {
                            // Compile name constant (True, False, None)
                            let (value, ty) = self.compile_name_constant(value)?;
                            result_stack.push(ExprResult { value, ty });
                        },
                        Expr::UnaryOp { op, operand, .. } => {
                            // First, add the task to process the unary operation after the operand is evaluated
                            work_stack.push_front(ExprTask::ProcessUnaryOp {
                                op: op.clone(),
                                operand_idx: result_stack.len(),
                            });

                            // Then, evaluate the operand
                            work_stack.push_front(ExprTask::Evaluate(operand));
                        },
                        Expr::List { elts, ctx, .. } => {
                            // Check that we're in a load context
                            if !matches!(ctx, crate::ast::ExprContext::Load) {
                                return Err(format!("Unsupported list context: {:?}", ctx));
                            }

                            // First, add the task to process the list after all elements are evaluated
                            work_stack.push_front(ExprTask::ProcessList {
                                elements_count: elts.len(),
                            });

                            // Then, evaluate each element in reverse order
                            for elt in elts.iter().rev() {
                                work_stack.push_front(ExprTask::Evaluate(elt));
                            }
                        },
                        Expr::Tuple { elts, ctx, .. } => {
                            // Check that we're in a load context
                            if !matches!(ctx, crate::ast::ExprContext::Load) {
                                return Err(format!("Unsupported tuple context: {:?}", ctx));
                            }

                            // First, add the task to process the tuple after all elements are evaluated
                            work_stack.push_front(ExprTask::ProcessTuple {
                                elements_count: elts.len(),
                            });

                            // Then, evaluate each element in reverse order
                            for elt in elts.iter().rev() {
                                work_stack.push_front(ExprTask::Evaluate(elt));
                            }
                        },
                        Expr::Compare { left, ops, comparators, .. } => {
                            // Ensure we have the same number of operators and comparators
                            if ops.len() != comparators.len() {
                                return Err(format!("Mismatched number of operators ({}) and comparators ({})", ops.len(), comparators.len()));
                            }

                            // For a simple comparison with one operator (a < b)
                            if ops.len() == 1 {
                                // First, add the task to process the comparison after the operands are evaluated
                                work_stack.push_front(ExprTask::ProcessComparison {
                                    op: ops[0].clone(),
                                });

                                // Then, evaluate the right operand (will be processed first, pushed to stack second)
                                work_stack.push_front(ExprTask::Evaluate(&comparators[0]));

                                // Finally, evaluate the left operand (will be processed second, pushed to stack first)
                                work_stack.push_front(ExprTask::Evaluate(left));
                            } else {
                                // For chained comparisons (a < b < c), we need to evaluate them as (a < b) and (b < c)
                                // This is a simplified implementation that doesn't handle short-circuiting
                                return Err("Chained comparisons not yet implemented in non-recursive mode".to_string());
                            }
                        },
                        Expr::Subscript { value, slice, ctx, .. } => {
                            // Check that we're in a load context
                            if !matches!(ctx, crate::ast::ExprContext::Load) {
                                return Err(format!("Unsupported subscript context: {:?}", ctx));
                            }

                            // First, add the task to process the subscript after the value and slice are evaluated
                            work_stack.push_front(ExprTask::ProcessSubscript {
                                value_idx: result_stack.len(),
                                slice_idx: result_stack.len() + 1,
                            });

                            // Then, evaluate the slice (will be processed first, pushed to stack second)
                            work_stack.push_front(ExprTask::Evaluate(slice));

                            // Finally, evaluate the value (will be processed second, pushed to stack first)
                            work_stack.push_front(ExprTask::Evaluate(value));
                        },
                        Expr::Call { func, args, keywords, .. } => {
                            // Handle special built-in functions
                            if let Expr::Name { id, .. } = &**func {
                                match id.as_str() {
                                    "len" => {
                                        // Special handling for len() function
                                        work_stack.push_front(ExprTask::ProcessLenCall {
                                            args,
                                        });
                                        continue;
                                    },
                                    "print" => {
                                        // Special handling for print() function
                                        work_stack.push_front(ExprTask::ProcessPrintCall {
                                            args,
                                        });
                                        continue;
                                    },
                                    "range" => {
                                        // Special handling for range() function
                                        work_stack.push_front(ExprTask::ProcessRangeCall {
                                            args,
                                        });
                                        continue;
                                    },
                                    _ => {}
                                }
                            }

                            // Regular function call
                            // First, add the task to prepare the function call
                            work_stack.push_front(ExprTask::PrepareFunctionCall {
                                func,
                                args,
                                keywords,
                            });
                        },
                        Expr::Dict { keys, values, .. } => {
                            // Check that we have the same number of keys and values
                            if keys.len() != values.len() {
                                return Err(format!("Mismatched number of keys ({}) and values ({})", keys.len(), values.len()));
                            }

                            // First, add the task to process the dictionary after all keys and values are evaluated
                            work_stack.push_front(ExprTask::ProcessDict {
                                elements_count: keys.len(),
                            });

                            // Then, evaluate each key-value pair in reverse order
                            // (so they're processed in the correct order)
                            for i in (0..keys.len()).rev() {
                                // Evaluate the value (will be processed first, pushed to stack second)
                                work_stack.push_front(ExprTask::Evaluate(&values[i]));

                                // Evaluate the key (will be processed second, pushed to stack first)
                                if let Some(key) = &keys[i] {
                                    work_stack.push_front(ExprTask::Evaluate(key));
                                } else {
                                    return Err("Dictionary keys cannot be None".to_string());
                                }
                            }
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
                ExprTask::ProcessUnaryOp { op, operand_idx } => {
                    // Get the operand from the result stack
                    if result_stack.len() <= operand_idx {
                        return Err(format!("Invalid operand index for unary operation: {} (stack size = {})", operand_idx, result_stack.len()));
                    }

                    let operand_result = &result_stack[operand_idx];

                    // Process the unary operation based on the operator type
                    let (result_value, result_type) = match op {
                        UnaryOperator::Not => {
                            // Boolean NOT operation
                            // Convert non-boolean operands to boolean first
                            let (bool_val, _) = if operand_result.ty != Type::Bool {
                                // Convert to boolean using the same logic as in the original implementation
                                match operand_result.ty {
                                    Type::Int => {
                                        let operand_val = operand_result.value.into_int_value();
                                        let zero = self.llvm_context.i64_type().const_int(0, false);
                                        let cmp = self.builder.build_int_compare(inkwell::IntPredicate::NE,
                                                                                operand_val,
                                                                                zero,
                                                                                "int_to_bool").unwrap();
                                        (cmp.into(), Type::Bool)
                                    },
                                    Type::Float => {
                                        let operand_val = operand_result.value.into_float_value();
                                        let zero = self.llvm_context.f64_type().const_float(0.0);
                                        let cmp = self.builder.build_float_compare(inkwell::FloatPredicate::ONE,
                                                                                  operand_val,
                                                                                  zero,
                                                                                  "float_to_bool").unwrap();
                                        (cmp.into(), Type::Bool)
                                    },
                                    Type::String => {
                                        // For strings, check if the string is non-empty
                                        // This is a simplified implementation
                                        let bool_val = self.llvm_context.bool_type().const_int(1, false);
                                        (bool_val.into(), Type::Bool)
                                    },
                                    _ => {
                                        // For other types, assume they're truthy
                                        let bool_val = self.llvm_context.bool_type().const_int(1, false);
                                        (bool_val.into(), Type::Bool)
                                    }
                                }
                            } else {
                                (operand_result.value, operand_result.ty.clone())
                            };

                            let operand_val = bool_val.into_int_value();
                            let result = self.builder.build_not(operand_val, "not_result").unwrap();
                            (result.into(), Type::Bool)
                        },
                        UnaryOperator::USub => {
                            // Numeric negation
                            match operand_result.ty {
                                Type::Int => {
                                    let operand_val = operand_result.value.into_int_value();
                                    let zero = self.llvm_context.i64_type().const_int(0, false);
                                    let result = self.builder.build_int_sub(zero, operand_val, "neg_result").unwrap();
                                    (result.into(), Type::Int)
                                },
                                Type::Float => {
                                    let operand_val = operand_result.value.into_float_value();
                                    let result = self.builder.build_float_neg(operand_val, "neg_result").unwrap();
                                    (result.into(), Type::Float)
                                },
                                _ => return Err(format!("Negation requires numeric operand, got {:?}", operand_result.ty)),
                            }
                        },
                        UnaryOperator::UAdd => {
                            // Unary plus (no-op for numeric types)
                            match operand_result.ty {
                                Type::Int | Type::Float => {
                                    (operand_result.value, operand_result.ty.clone())
                                },
                                _ => return Err(format!("Unary plus requires numeric operand, got {:?}", operand_result.ty)),
                            }
                        },
                        UnaryOperator::Invert => {
                            // Bitwise NOT (only for integers)
                            if operand_result.ty != Type::Int {
                                return Err(format!("Bitwise NOT requires integer operand, got {:?}", operand_result.ty));
                            }

                            let operand_val = operand_result.value.into_int_value();
                            let result = self.builder.build_not(operand_val, "invert_result").unwrap();
                            (result.into(), Type::Int)
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
                ExprTask::ProcessList { elements_count } => {
                    // Get the elements from the result stack
                    if result_stack.len() < elements_count {
                        return Err(format!("Not enough elements for list: stack size = {}, expected {}", result_stack.len(), elements_count));
                    }

                    // Get the element type (use the first element's type or default to Int if empty)
                    let element_type = if elements_count > 0 {
                        result_stack[result_stack.len() - elements_count].ty.clone()
                    } else {
                        Type::Int // Default type for empty lists
                    };

                    // Create an empty list
                    let list_ptr = self.build_empty_list("list_result")?;

                    // Get the list_append function
                    let list_append_fn = match self.module.get_function("list_append") {
                        Some(f) => f,
                        None => return Err("list_append function not found".to_string()),
                    };

                    // Add each element to the list
                    let start_idx = result_stack.len() - elements_count;
                    for i in 0..elements_count {
                        let element = &result_stack[start_idx + i];

                        // Convert the element to the common type if needed
                        let element_val = if element.ty != element_type {
                            self.convert_type(element.value, &element.ty, &element_type)?
                        } else {
                            element.value
                        };

                        // Append the element to the list
                        self.builder.build_call(
                            list_append_fn,
                            &[list_ptr.into(), element_val.into()],
                            &format!("list_append_{}", i)
                        ).unwrap();
                    }

                    // Remove the elements from the result stack
                    for _ in 0..elements_count {
                        result_stack.pop();
                    }

                    // Push the list onto the result stack
                    result_stack.push(ExprResult {
                        value: list_ptr.into(),
                        ty: Type::List(Box::new(element_type)),
                    });
                },
                ExprTask::ProcessTuple { elements_count } => {
                    // Get the elements from the result stack
                    if result_stack.len() < elements_count {
                        return Err(format!("Not enough elements for tuple: stack size = {}, expected {}", result_stack.len(), elements_count));
                    }

                    // Collect the element types
                    let mut element_types = Vec::with_capacity(elements_count);
                    let mut element_values = Vec::with_capacity(elements_count);
                    let start_idx = result_stack.len() - elements_count;

                    for i in 0..elements_count {
                        let element = &result_stack[start_idx + i];
                        element_types.push(element.ty.clone());
                        element_values.push(element.value);
                    }

                    // Create a tuple struct type
                    let llvm_types: Vec<_> = element_types.iter()
                        .map(|ty| self.get_llvm_type(ty))
                        .collect();

                    let tuple_type = self.llvm_context.struct_type(&llvm_types, false);

                    // Allocate memory for the tuple
                    let tuple_ptr = self.builder.build_alloca(tuple_type, "tuple_alloca").unwrap();

                    // Store each element in the tuple
                    for i in 0..elements_count {
                        let element_ptr = self.builder.build_struct_gep(
                            tuple_type,
                            tuple_ptr,
                            i as u32,
                            &format!("tuple_element_{}_ptr", i)
                        ).unwrap();

                        self.builder.build_store(element_ptr, element_values[i]).unwrap();
                    }

                    // Remove the elements from the result stack
                    for _ in 0..elements_count {
                        result_stack.pop();
                    }

                    // Push the tuple onto the result stack
                    result_stack.push(ExprResult {
                        value: tuple_ptr.into(),
                        ty: Type::Tuple(element_types),
                    });
                },
                ExprTask::ProcessComparison { op } => {
                    // Get the operands from the result stack
                    if result_stack.len() < 2 {
                        return Err(format!("Not enough operands for comparison: stack size = {}", result_stack.len()));
                    }

                    // The operands should be the last two items on the stack
                    // The right operand is on top (last pushed), the left operand is below it
                    let right_idx = result_stack.len() - 1;
                    let left_idx = right_idx - 1;

                    let right_result = &result_stack[right_idx];
                    let left_result = &result_stack[left_idx];

                    // Process the comparison operation
                    let (result_value, result_type) = self.compile_comparison(
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
                ExprTask::ProcessSubscript { value_idx, slice_idx } => {
                    // Get the value and slice from the result stack
                    if result_stack.len() <= slice_idx || result_stack.len() <= value_idx {
                        return Err(format!("Invalid indices for subscript: value_idx = {}, slice_idx = {}, stack size = {}",
                                           value_idx, slice_idx, result_stack.len()));
                    }

                    let value_result = &result_stack[value_idx];
                    let slice_result = &result_stack[slice_idx];

                    // Process the subscript operation based on the value and slice types
                    let (result_value, result_type) = match &value_result.ty {
                        Type::List(element_type) => {
                            // Handle list subscript
                            match &slice_result.ty {
                                Type::Int => {
                                    // Get the index
                                    let index = slice_result.value.into_int_value();

                                    // Get the list element at the specified index
                                    let list_ptr = value_result.value.into_pointer_value();
                                    let element_ptr = self.get_list_element(list_ptr, index)?;

                                    // Load the element
                                    let element_llvm_type = self.get_llvm_type(element_type);
                                    let element_value = self.builder.build_load(element_llvm_type, element_ptr, "list_element").unwrap();

                                    (element_value, *element_type.clone())
                                },
                                _ => return Err(format!("List subscript requires integer index, got {:?}", slice_result.ty)),
                            }
                        },
                        Type::Tuple(element_types) => {
                            // Handle tuple subscript
                            match &slice_result.ty {
                                Type::Int => {
                                    // Get the index as a constant
                                    if let Some(index) = slice_result.value.into_int_value().get_zero_extended_constant() {
                                        let index = index as usize;

                                        // Check if the index is in bounds
                                        if index >= element_types.len() {
                                            return Err(format!("Tuple index out of range: {} (tuple size = {})", index, element_types.len()));
                                        }

                                        // Get the tuple element type
                                        let element_type = &element_types[index];

                                        // Get the tuple element at the specified index
                                        let tuple_value = value_result.value;

                                        // Check if the tuple is a pointer or a struct value
                                        let tuple_ptr = if tuple_value.is_pointer_value() {
                                            tuple_value.into_pointer_value()
                                        } else {
                                            // If it's a struct value, we need to allocate memory for it
                                            let tuple_type = self.llvm_context.struct_type(
                                                &element_types.iter().map(|ty| self.get_llvm_type(ty)).collect::<Vec<_>>(),
                                                false
                                            );
                                            let alloca = self.builder.build_alloca(tuple_type, "tuple_alloca").unwrap();
                                            self.builder.build_store(alloca, tuple_value).unwrap();
                                            alloca
                                        };

                                        // Get the tuple type
                                        let tuple_type = self.llvm_context.struct_type(
                                            &element_types.iter().map(|ty| self.get_llvm_type(ty)).collect::<Vec<_>>(),
                                            false
                                        );

                                        let element_ptr = self.builder.build_struct_gep(
                                            tuple_type,
                                            tuple_ptr,
                                            index as u32,
                                            &format!("tuple_element_{}_ptr", index)
                                        ).unwrap();

                                        // Load the element
                                        let element_llvm_type = self.get_llvm_type(element_type);
                                        let element_value = self.builder.build_load(element_llvm_type, element_ptr, &format!("tuple_element_{}", index)).unwrap();

                                        (element_value, element_type.clone())
                                    } else {
                                        return Err("Tuple subscript requires constant index".to_string());
                                    }
                                },
                                _ => return Err(format!("Tuple subscript requires integer index, got {:?}", slice_result.ty)),
                            }
                        },
                        Type::String => {
                            // Handle string subscript
                            match &slice_result.ty {
                                Type::Int => {
                                    // Get the index
                                    let index = slice_result.value.into_int_value();

                                    // Get the character at the specified index
                                    // This is a simplified implementation that doesn't handle out-of-bounds access
                                    let string_ptr = value_result.value.into_pointer_value();
                                    let char_ptr = unsafe {
                                        self.builder.build_in_bounds_gep(
                                            self.llvm_context.i8_type(),
                                            string_ptr,
                                            &[index],
                                            "char_ptr"
                                        ).unwrap()
                                    };

                                    // Load the character
                                    let char_value = self.builder.build_load(self.llvm_context.i8_type(), char_ptr, "char").unwrap();

                                    // Create a new string with just this character
                                    let array_type = self.llvm_context.i8_type().array_type(2);
                                    let char_str_ptr = self.builder.build_alloca(array_type, "char_str").unwrap();

                                    // Store the character at index 0
                                    let char_element_ptr = unsafe {
                                        self.builder.build_in_bounds_gep(
                                            array_type,
                                            char_str_ptr,
                                            &[self.llvm_context.i32_type().const_zero(), self.llvm_context.i32_type().const_zero()],
                                            "char_element_ptr"
                                        ).unwrap()
                                    };
                                    self.builder.build_store(char_element_ptr, char_value).unwrap();

                                    // Add null terminator at index 1
                                    let null_ptr = unsafe {
                                        self.builder.build_in_bounds_gep(
                                            array_type,
                                            char_str_ptr,
                                            &[self.llvm_context.i32_type().const_zero(), self.llvm_context.i32_type().const_int(1, false)],
                                            "null_ptr"
                                        ).unwrap()
                                    };
                                    self.builder.build_store(null_ptr, self.llvm_context.i8_type().const_zero()).unwrap();

                                    (char_str_ptr.into(), Type::String)
                                },
                                _ => return Err(format!("String subscript requires integer index, got {:?}", slice_result.ty)),
                            }
                        },
                        Type::Dict(_key_type, value_type) => {
                            // Handle dictionary subscript
                            // Get the dict_get function
                            let dict_get_fn = match self.module.get_function("dict_get") {
                                Some(f) => f,
                                None => return Err("dict_get function not found".to_string()),
                            };

                            // Call dict_get to get the value from the dictionary
                            let dict_ptr = value_result.value.into_pointer_value();

                            // Special handling for different key types
                            let key_ptr = match &slice_result.ty {
                                Type::String => {
                                    // For string keys, we can use the pointer directly
                                    if slice_result.value.is_pointer_value() {
                                        slice_result.value
                                    } else {
                                        return Err(format!("Expected pointer value for string key"));
                                    }
                                },
                                Type::Int => {
                                    // For integer keys, we need to allocate memory and store the value
                                    let key_alloca = self.builder.build_alloca(
                                        slice_result.value.get_type(),
                                        "dict_key_temp"
                                    ).unwrap();
                                    self.builder.build_store(key_alloca, slice_result.value).unwrap();
                                    key_alloca.into()
                                },
                                _ => {
                                    // For other types, we need to allocate memory and store the value
                                    let key_alloca = self.builder.build_alloca(
                                        slice_result.value.get_type(),
                                        "dict_key_temp"
                                    ).unwrap();
                                    self.builder.build_store(key_alloca, slice_result.value).unwrap();
                                    key_alloca.into()
                                }
                            };

                            let call_site_value = self.builder.build_call(
                                dict_get_fn,
                                &[dict_ptr.into(), key_ptr.into()],
                                "dict_get_result"
                            ).unwrap();

                            let value_ptr = call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to get value from dictionary".to_string())?;

                            (value_ptr, *value_type.clone())
                        },
                        _ => return Err(format!("Cannot subscript value of type {:?}", value_result.ty)),
                    };

                    // Remove the value and slice from the result stack
                    // Note: We need to be careful about the order of removal since indices will shift
                    // We need to remove the higher index first
                    if slice_idx > value_idx {
                        result_stack.remove(slice_idx);
                        result_stack.remove(value_idx);
                    } else {
                        result_stack.remove(value_idx);
                        result_stack.remove(slice_idx);
                    }

                    // Push the result onto the result stack
                    result_stack.push(ExprResult {
                        value: result_value,
                        ty: result_type,
                    });
                },
                ExprTask::PrepareFunctionCall { func, args, keywords } => {
                    // First, evaluate the function expression
                    work_stack.push_front(ExprTask::ProcessFunctionCallArgs {
                        func_idx: result_stack.len(),
                        args,
                        processed_args: 0,
                        keywords,
                    });

                    // Then, evaluate the function
                    work_stack.push_front(ExprTask::Evaluate(func));
                },
                ExprTask::ProcessFunctionCallArgs { func_idx, args, processed_args, keywords } => {
                    // Check if we've processed all arguments
                    if processed_args >= args.len() {
                        // All arguments processed, now handle keywords if any
                        if !keywords.is_empty() {
                            // Process keywords
                            work_stack.push_front(ExprTask::ProcessFunctionCallKeywords {
                                func_idx,
                                args_count: args.len(),
                                keywords,
                                processed_keywords: 0,
                            });
                        } else {
                            // No keywords, finalize the function call
                            // Get the function name if it's a direct name reference
                            // For now, we'll just use a placeholder
                            let func_name = None;

                            work_stack.push_front(ExprTask::FinalizeFunctionCall {
                                func_idx,
                                args_count: args.len(),
                                keywords_count: 0,
                                func_name,
                            });
                        }
                    } else {
                        // Process the next argument
                        work_stack.push_front(ExprTask::ProcessFunctionCallArgs {
                            func_idx,
                            args,
                            processed_args: processed_args + 1,
                            keywords,
                        });

                        // Evaluate the argument
                        work_stack.push_front(ExprTask::Evaluate(&args[processed_args]));
                    }
                },
                ExprTask::ProcessFunctionCallKeywords { func_idx, args_count, keywords, processed_keywords } => {
                    // Check if we've processed all keywords
                    if processed_keywords >= keywords.len() {
                        // All keywords processed, finalize the function call
                        // Get the function name if it's a direct name reference
                        // For now, we'll just use a placeholder
                        let func_name = None;

                        work_stack.push_front(ExprTask::FinalizeFunctionCall {
                            func_idx,
                            args_count,
                            keywords_count: keywords.len(),
                            func_name,
                        });
                    } else {
                        // Process the next keyword
                        let (_name, value) = &keywords[processed_keywords];

                        work_stack.push_front(ExprTask::ProcessFunctionCallKeywords {
                            func_idx,
                            args_count,
                            keywords,
                            processed_keywords: processed_keywords + 1,
                        });

                        // Evaluate the keyword value
                        work_stack.push_front(ExprTask::Evaluate(value));
                    }
                },
                ExprTask::FinalizeFunctionCall { func_idx, args_count, keywords_count, func_name } => {
                    // Get the function and arguments from the result stack
                    if result_stack.len() < func_idx + args_count + keywords_count + 1 {
                        return Err(format!("Not enough values for function call: stack size = {}, expected at least {}",
                                           result_stack.len(), func_idx + args_count + keywords_count + 1));
                    }

                    // Handle special functions
                    if let Some(name) = func_name {
                        match name.as_str() {
                            "len" => {
                                // Special handling for len() function
                                if args_count != 1 {
                                    return Err(format!("len() takes exactly one argument ({} given)", args_count));
                                }

                                // Get the argument
                                let arg_idx = func_idx + 1;
                                let arg_result = &result_stack[arg_idx];

                                // Get the length based on the argument type
                                let (result_value, result_type) = match &arg_result.ty {
                                    Type::List(_) => {
                                        // Get the list_len function
                                        let list_len_fn = match self.module.get_function("list_len") {
                                            Some(f) => f,
                                            None => return Err("list_len function not found".to_string()),
                                        };

                                        // Call list_len
                                        let list_ptr = arg_result.value.into_pointer_value();
                                        let call_site_value = self.builder.build_call(
                                            list_len_fn,
                                            &[list_ptr.into()],
                                            "list_len_result"
                                        ).unwrap();

                                        let len_value = call_site_value.try_as_basic_value().left()
                                            .ok_or_else(|| "Failed to get list length".to_string())?;

                                        (len_value, Type::Int)
                                    },
                                    Type::String => {
                                        // Get the string_len function
                                        let string_len_fn = match self.module.get_function("string_len") {
                                            Some(f) => f,
                                            None => return Err("string_len function not found".to_string()),
                                        };

                                        // Call string_len
                                        let string_ptr = arg_result.value.into_pointer_value();
                                        let call_site_value = self.builder.build_call(
                                            string_len_fn,
                                            &[string_ptr.into()],
                                            "string_len_result"
                                        ).unwrap();

                                        let len_value = call_site_value.try_as_basic_value().left()
                                            .ok_or_else(|| "Failed to get string length".to_string())?;

                                        (len_value, Type::Int)
                                    },
                                    Type::Dict(_, _) => {
                                        // Get the dict_len function
                                        let dict_len_fn = match self.module.get_function("dict_len") {
                                            Some(f) => f,
                                            None => return Err("dict_len function not found".to_string()),
                                        };

                                        // Call dict_len
                                        let dict_ptr = arg_result.value.into_pointer_value();
                                        let call_site_value = self.builder.build_call(
                                            dict_len_fn,
                                            &[dict_ptr.into()],
                                            "dict_len_result"
                                        ).unwrap();

                                        let len_value = call_site_value.try_as_basic_value().left()
                                            .ok_or_else(|| "Failed to get dict length".to_string())?;

                                        (len_value, Type::Int)
                                    },
                                    Type::Tuple(elements) => {
                                        // For tuples, the length is known at compile time
                                        let len_value = self.llvm_context.i64_type().const_int(elements.len() as u64, false);
                                        (len_value.into(), Type::Int)
                                    },
                                    _ => return Err(format!("Object of type '{}' has no len()", arg_result.ty)),
                                };

                                // Remove the function and argument from the result stack
                                result_stack.remove(arg_idx);
                                result_stack.remove(func_idx);

                                // Push the result onto the result stack
                                result_stack.push(ExprResult {
                                    value: result_value,
                                    ty: result_type,
                                });

                                continue;
                            },
                            "print" => {
                                // Special handling for print() function
                                // Get the arguments
                                let mut arg_values = Vec::with_capacity(args_count);
                                let start_idx = func_idx + 1;

                                for i in 0..args_count {
                                    let arg_result = &result_stack[start_idx + i];
                                    arg_values.push(arg_result.value);
                                }

                                // Call print for each argument
                                for (i, arg_value) in arg_values.iter().enumerate() {
                                    let arg_type = &result_stack[start_idx + i].ty;

                                    // Convert the argument to a string if needed
                                    let (string_ptr, _) = match arg_type {
                                        Type::String => {
                                            // Already a string, just use it
                                            (arg_value.into_pointer_value(), Type::String)
                                        },
                                        Type::Int => {
                                            // Convert int to string
                                            let int_to_string_fn = match self.module.get_function("int_to_string") {
                                                Some(f) => f,
                                                None => return Err("int_to_string function not found".to_string()),
                                            };

                                            let int_value = arg_value.into_int_value();
                                            let call_site_value = self.builder.build_call(
                                                int_to_string_fn,
                                                &[int_value.into()],
                                                &format!("int_to_string_{}", i)
                                            ).unwrap();

                                            let string_ptr = call_site_value.try_as_basic_value().left()
                                                .ok_or_else(|| "Failed to convert int to string".to_string())?;

                                            (string_ptr.into_pointer_value(), Type::String)
                                        },
                                        Type::Float => {
                                            // Convert float to string
                                            let float_to_string_fn = match self.module.get_function("float_to_string") {
                                                Some(f) => f,
                                                None => return Err("float_to_string function not found".to_string()),
                                            };

                                            let float_value = arg_value.into_float_value();
                                            let call_site_value = self.builder.build_call(
                                                float_to_string_fn,
                                                &[float_value.into()],
                                                &format!("float_to_string_{}", i)
                                            ).unwrap();

                                            let string_ptr = call_site_value.try_as_basic_value().left()
                                                .ok_or_else(|| "Failed to convert float to string".to_string())?;

                                            (string_ptr.into_pointer_value(), Type::String)
                                        },
                                        Type::Bool => {
                                            // Convert bool to string
                                            let bool_value = arg_value.into_int_value();
                                            let true_str = self.builder.build_global_string_ptr("True", "true_str").unwrap();
                                            let false_str = self.builder.build_global_string_ptr("False", "false_str").unwrap();

                                            let string_ptr = self.builder.build_select(
                                                bool_value,
                                                true_str.as_pointer_value(),
                                                false_str.as_pointer_value(),
                                                "bool_to_string"
                                            ).unwrap();

                                            (string_ptr.into_pointer_value(), Type::String)
                                        },
                                        _ => {
                                            // For other types, use a generic representation
                                            let generic_str = self.builder.build_global_string_ptr("<object>", "generic_str").unwrap();
                                            (generic_str.as_pointer_value(), Type::String)
                                        },
                                    };

                                    // Call print_string
                                    let print_string_fn = match self.module.get_function("print_string") {
                                        Some(f) => f,
                                        None => return Err("print_string function not found".to_string()),
                                    };

                                    self.builder.build_call(
                                        print_string_fn,
                                        &[string_ptr.into()],
                                        &format!("print_result_{}", i)
                                    ).unwrap();

                                    // Print a space between arguments (except for the last one)
                                    if i < arg_values.len() - 1 {
                                        let space_str = self.builder.build_global_string_ptr(" ", "space_str").unwrap();
                                        self.builder.build_call(
                                            print_string_fn,
                                            &[space_str.as_pointer_value().into()],
                                            "print_space"
                                        ).unwrap();
                                    }
                                }

                                // Print a newline at the end
                                let print_string_fn = match self.module.get_function("print_string") {
                                    Some(f) => f,
                                    None => return Err("print_string function not found".to_string()),
                                };

                                let newline_str = self.builder.build_global_string_ptr("\n", "newline_str").unwrap();
                                self.builder.build_call(
                                    print_string_fn,
                                    &[newline_str.as_pointer_value().into()],
                                    "print_newline"
                                ).unwrap();

                                // Remove the function and arguments from the result stack
                                for _ in 0..args_count {
                                    result_stack.remove(start_idx);
                                }
                                result_stack.remove(func_idx);

                                // Push None as the result
                                let none_value = self.llvm_context.i64_type().const_int(0, false);
                                result_stack.push(ExprResult {
                                    value: none_value.into(),
                                    ty: Type::None,
                                });

                                continue;
                            },
                            _ => {}
                        }
                    }

                    // Regular function call
                    // Get the function value and type
                    let func_value = result_stack[func_idx].value;
                    let func_type = result_stack[func_idx].ty.clone();

                    // Check if the function type is callable
                    if let Type::Function { param_types, return_type, .. } = &func_type {
                        // Check if the number of arguments matches
                        if args_count != param_types.len() {
                            return Err(format!("Function takes {} arguments but {} were given", param_types.len(), args_count));
                        }

                        // Get the function pointer
                        let func_ptr = if func_value.is_pointer_value() {
                            func_value.into_pointer_value()
                        } else {
                            return Err("Function value is not a pointer".to_string());
                        };

                        // Get the LLVM function value
                        let func = match self.get_function_from_ptr(func_ptr) {
                            Some(f) => f,
                            None => return Err("Failed to get function from pointer".to_string()),
                        };

                        // Prepare the arguments
                        let mut arg_values = Vec::with_capacity(args_count);
                        let start_idx = func_idx + 1;

                        for i in 0..args_count {
                            let arg_result = &result_stack[start_idx + i];
                            let param_type = &param_types[i];

                            // Convert the argument to the parameter type if needed
                            let arg_value = if arg_result.ty != *param_type {
                                self.convert_type(arg_result.value, &arg_result.ty, param_type)?
                            } else {
                                arg_result.value
                            };

                            arg_values.push(arg_value.into());
                        }

                        // Call the function
                        let call_site_value = self.builder.build_call(
                            func,
                            &arg_values,
                            "function_call_result"
                        ).unwrap();

                        // Get the return value
                        let return_value = if **return_type == Type::None {
                            // For None return type, use a dummy value
                            self.llvm_context.i64_type().const_int(0, false).into()
                        } else {
                            // For other return types, get the actual return value
                            call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to get function return value".to_string())?
                        };

                        // Remove the function and arguments from the result stack
                        for _ in 0..args_count {
                            result_stack.remove(start_idx);
                        }
                        result_stack.remove(func_idx);

                        // Push the result onto the result stack
                        result_stack.push(ExprResult {
                            value: return_value,
                            ty: *return_type.clone(),
                        });
                    } else {
                        return Err(format!("Cannot call object of type {:?}", func_type));
                    }
                },
                ExprTask::ProcessLenCall { args } => {
                    // Check that we have exactly one argument
                    if args.len() != 1 {
                        return Err(format!("len() takes exactly one argument ({} given)", args.len()));
                    }

                    // Evaluate the argument
                    let (arg_val, arg_type) = self.compile_expr(&args[0])?;

                    // Get the length based on the argument type
                    let (result_value, result_type) = match arg_type {
                        Type::List(_) => {
                            // Get the list_len function
                            let list_len_fn = match self.module.get_function("list_len") {
                                Some(f) => f,
                                None => return Err("list_len function not found".to_string()),
                            };

                            // Call list_len
                            let list_ptr = arg_val.into_pointer_value();
                            let call_site_value = self.builder.build_call(
                                list_len_fn,
                                &[list_ptr.into()],
                                "list_len_result"
                            ).unwrap();

                            let len_value = call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to get list length".to_string())?;

                            (len_value, Type::Int)
                        },
                        Type::String => {
                            // Get the string_len function
                            let string_len_fn = match self.module.get_function("string_len") {
                                Some(f) => f,
                                None => return Err("string_len function not found".to_string()),
                            };

                            // Call string_len
                            let string_ptr = arg_val.into_pointer_value();
                            let call_site_value = self.builder.build_call(
                                string_len_fn,
                                &[string_ptr.into()],
                                "string_len_result"
                            ).unwrap();

                            let len_value = call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to get string length".to_string())?;

                            (len_value, Type::Int)
                        },
                        Type::Dict(_, _) => {
                            // Get the dict_len function
                            let dict_len_fn = match self.module.get_function("dict_len") {
                                Some(f) => f,
                                None => return Err("dict_len function not found".to_string()),
                            };

                            // Call dict_len
                            let dict_ptr = arg_val.into_pointer_value();
                            let call_site_value = self.builder.build_call(
                                dict_len_fn,
                                &[dict_ptr.into()],
                                "dict_len_result"
                            ).unwrap();

                            let len_value = call_site_value.try_as_basic_value().left()
                                .ok_or_else(|| "Failed to get dict length".to_string())?;

                            (len_value, Type::Int)
                        },
                        Type::Tuple(elements) => {
                            // For tuples, the length is known at compile time
                            let len_value = self.llvm_context.i64_type().const_int(elements.len() as u64, false);
                            (len_value.into(), Type::Int)
                        },
                        _ => return Err(format!("Object of type '{}' has no len()", arg_type)),
                    };

                    // Push the result onto the result stack
                    result_stack.push(ExprResult {
                        value: result_value,
                        ty: result_type,
                    });
                },
                ExprTask::ProcessDict { elements_count } => {
                    // Check if we have enough elements on the stack
                    if result_stack.len() < elements_count * 2 {
                        return Err(format!("Not enough elements for dictionary creation: stack size = {}, expected at least {}",
                                           result_stack.len(), elements_count * 2));
                    }

                    // Get the dict_with_capacity function
                    let dict_with_capacity_fn = match self.module.get_function("dict_with_capacity") {
                        Some(f) => f,
                        None => return Err("dict_with_capacity function not found".to_string()),
                    };

                    // Create a dictionary with the given capacity
                    let len_value = self.llvm_context.i64_type().const_int(elements_count as u64, false);
                    let call_site_value = self.builder.build_call(
                        dict_with_capacity_fn,
                        &[len_value.into()],
                        "dict_with_capacity"
                    ).unwrap();

                    let dict_ptr = call_site_value.try_as_basic_value().left()
                        .ok_or_else(|| "Failed to create dictionary with capacity".to_string())?;

                    // Get the dict_set function
                    let dict_set_fn = match self.module.get_function("dict_set") {
                        Some(f) => f,
                        None => return Err("dict_set function not found".to_string()),
                    };

                    // Determine the key and value types
                    let mut key_type = Type::Unknown;
                    let mut value_type = Type::Unknown;

                    // Add each key-value pair to the dictionary
                    for i in 0..elements_count {
                        // Get the key and value from the result stack
                        // The stack has pairs of (key, value) in reverse order
                        let value_idx = result_stack.len() - 1 - (i * 2);
                        let key_idx = value_idx - 1;

                        let value_result = &result_stack[value_idx];
                        let key_result = &result_stack[key_idx];

                        // Update the key and value types if they're unknown
                        if matches!(key_type, Type::Unknown) {
                            key_type = key_result.ty.clone();
                        }
                        if matches!(value_type, Type::Unknown) {
                            value_type = value_result.ty.clone();
                        }

                        // Special handling for different key types
                        let key_ptr = match &key_result.ty {
                            Type::String => {
                                // For string keys, we can use the pointer directly
                                if key_result.value.is_pointer_value() {
                                    key_result.value
                                } else {
                                    return Err(format!("Expected pointer value for string key"));
                                }
                            },
                            Type::Int => {
                                // For integer keys, we need to allocate memory and store the value
                                let key_alloca = self.builder.build_alloca(
                                    key_result.value.get_type(),
                                    &format!("dict_key_temp_{}", i)
                                ).unwrap();
                                self.builder.build_store(key_alloca, key_result.value).unwrap();
                                key_alloca.into()
                            },
                            _ => {
                                // For other types, we need to allocate memory and store the value
                                let key_alloca = self.builder.build_alloca(
                                    key_result.value.get_type(),
                                    &format!("dict_key_temp_{}", i)
                                ).unwrap();
                                self.builder.build_store(key_alloca, key_result.value).unwrap();
                                key_alloca.into()
                            }
                        };

                        // Special handling for different value types
                        let value_ptr = match &value_result.ty {
                            Type::String => {
                                // For string values, we can use the pointer directly
                                if value_result.value.is_pointer_value() {
                                    value_result.value
                                } else {
                                    return Err(format!("Expected pointer value for string value"));
                                }
                            },
                            _ => {
                                // For other types, we need to allocate memory and store the value
                                let value_alloca = self.builder.build_alloca(
                                    value_result.value.get_type(),
                                    &format!("dict_value_temp_{}", i)
                                ).unwrap();
                                self.builder.build_store(value_alloca, value_result.value).unwrap();
                                value_alloca.into()
                            }
                        };

                        // Call dict_set to add the key-value pair to the dictionary
                        self.builder.build_call(
                            dict_set_fn,
                            &[
                                dict_ptr.into_pointer_value().into(),
                                key_ptr.into(),
                                value_ptr.into(),
                            ],
                            &format!("dict_set_{}", i)
                        ).unwrap();
                    }

                    // Remove all the keys and values from the result stack
                    for _ in 0..elements_count * 2 {
                        result_stack.pop();
                    }

                    // Push the dictionary onto the result stack
                    result_stack.push(ExprResult {
                        value: dict_ptr,
                        ty: Type::Dict(Box::new(key_type), Box::new(value_type)),
                    });
                },
                ExprTask::ProcessRangeCall { args } => {
                    // Handle range() function call
                    // range(stop) or range(start, stop) or range(start, stop, step)

                    // Get the number of arguments
                    let args_count = args.len();

                    // Check that we have at least one argument
                    if args_count == 0 {
                        return Err("range() requires at least one argument".to_string());
                    }

                    // Get the arguments
                    let (start, stop, step) = match args_count {
                        1 => {
                            // range(stop) - start = 0, step = 1
                            let (stop_val, _) = self.compile_expr(&args[0])?;
                            (0, stop_val.into_int_value().get_zero_extended_constant().unwrap_or(0) as i64, 1)
                        },
                        2 => {
                            // range(start, stop) - step = 1
                            let (start_val, _) = self.compile_expr(&args[0])?;
                            let (stop_val, _) = self.compile_expr(&args[1])?;
                            (start_val.into_int_value().get_zero_extended_constant().unwrap_or(0) as i64,
                             stop_val.into_int_value().get_zero_extended_constant().unwrap_or(0) as i64,
                             1)
                        },
                        3 => {
                            // range(start, stop, step)
                            let (start_val, _) = self.compile_expr(&args[0])?;
                            let (stop_val, _) = self.compile_expr(&args[1])?;
                            let (step_val, _) = self.compile_expr(&args[2])?;
                            (start_val.into_int_value().get_zero_extended_constant().unwrap_or(0) as i64,
                             stop_val.into_int_value().get_zero_extended_constant().unwrap_or(0) as i64,
                             step_val.into_int_value().get_zero_extended_constant().unwrap_or(1) as i64)
                        },
                        _ => {
                            return Err(format!("range() takes at most 3 arguments ({} given)", args_count));
                        }
                    };

                    // Create a list to hold the range values
                    let list_ptr = self.build_empty_list("range_list")?;

                    // Get the list_append function
                    let list_append_fn = match self.module.get_function("list_append") {
                        Some(f) => f,
                        None => return Err("list_append function not found".to_string()),
                    };

                    // Calculate the number of elements in the range
                    let num_elements = if step > 0 {
                        (stop - start + step - 1) / step
                    } else if step < 0 {
                        (start - stop - step - 1) / (-step)
                    } else {
                        return Err("range() step argument must not be zero".to_string());
                    };

                    // Limit the number of elements to avoid excessive memory usage
                    let max_elements = 1000;
                    let num_elements = std::cmp::min(num_elements, max_elements);

                    // Add each value to the list
                    for i in 0..num_elements {
                        let value = start + (i as i64 * step);

                        // Create an integer constant for the value
                        let int_value = self.llvm_context.i64_type().const_int(value as u64, false);

                        // Allocate memory for the value
                        let value_alloca = self.builder.build_alloca(
                            self.llvm_context.i64_type(),
                            &format!("range_value_{}", i)
                        ).unwrap();

                        // Store the value in the allocated memory
                        self.builder.build_store(value_alloca, int_value).unwrap();

                        // Append the value to the list
                        self.builder.build_call(
                            list_append_fn,
                            &[list_ptr.into(), value_alloca.into()],
                            &format!("list_append_{}", i)
                        ).unwrap();
                    }

                    // Push the list onto the result stack
                    result_stack.push(ExprResult {
                        value: list_ptr.into(),
                        ty: Type::List(Box::new(Type::Int)),
                    });
                },
                ExprTask::ProcessPrintCall { args } => {
                    // Evaluate each argument and print it
                    for (i, arg) in args.iter().enumerate() {
                        // Evaluate the argument
                        let (arg_val, arg_type) = self.compile_expr(arg)?;

                        // Convert the argument to a string if needed
                        let (string_ptr, _) = match arg_type {
                            Type::String => {
                                // Already a string, just use it
                                (arg_val.into_pointer_value(), Type::String)
                            },
                            Type::Int => {
                                // Convert int to string
                                let int_to_string_fn = match self.module.get_function("int_to_string") {
                                    Some(f) => f,
                                    None => return Err("int_to_string function not found".to_string()),
                                };

                                let int_value = arg_val.into_int_value();
                                let call_site_value = self.builder.build_call(
                                    int_to_string_fn,
                                    &[int_value.into()],
                                    &format!("int_to_string_{}", i)
                                ).unwrap();

                                let string_ptr = call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to convert int to string".to_string())?;

                                (string_ptr.into_pointer_value(), Type::String)
                            },
                            Type::Float => {
                                // Convert float to string
                                let float_to_string_fn = match self.module.get_function("float_to_string") {
                                    Some(f) => f,
                                    None => return Err("float_to_string function not found".to_string()),
                                };

                                let float_value = arg_val.into_float_value();
                                let call_site_value = self.builder.build_call(
                                    float_to_string_fn,
                                    &[float_value.into()],
                                    &format!("float_to_string_{}", i)
                                ).unwrap();

                                let string_ptr = call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| "Failed to convert float to string".to_string())?;

                                (string_ptr.into_pointer_value(), Type::String)
                            },
                            Type::Bool => {
                                // Convert bool to string
                                let bool_value = arg_val.into_int_value();
                                let true_str = self.builder.build_global_string_ptr("True", "true_str").unwrap();
                                let false_str = self.builder.build_global_string_ptr("False", "false_str").unwrap();

                                let string_ptr = self.builder.build_select(
                                    bool_value,
                                    true_str.as_pointer_value(),
                                    false_str.as_pointer_value(),
                                    "bool_to_string"
                                ).unwrap();

                                (string_ptr.into_pointer_value(), Type::String)
                            },
                            _ => {
                                // For other types, use a generic representation
                                let generic_str = self.builder.build_global_string_ptr("<object>", "generic_str").unwrap();
                                (generic_str.as_pointer_value(), Type::String)
                            },
                        };

                        // Call print_string
                        let print_string_fn = match self.module.get_function("print_string") {
                            Some(f) => f,
                            None => return Err("print_string function not found".to_string()),
                        };

                        self.builder.build_call(
                            print_string_fn,
                            &[string_ptr.into()],
                            &format!("print_result_{}", i)
                        ).unwrap();

                        // Print a space between arguments (except for the last one)
                        if i < args.len() - 1 {
                            let space_str = self.builder.build_global_string_ptr(" ", "space_str").unwrap();
                            self.builder.build_call(
                                print_string_fn,
                                &[space_str.as_pointer_value().into()],
                                "print_space"
                            ).unwrap();
                        }
                    }

                    // Print a newline at the end
                    let print_string_fn = match self.module.get_function("print_string") {
                        Some(f) => f,
                        None => return Err("print_string function not found".to_string()),
                    };

                    let newline_str = self.builder.build_global_string_ptr("\n", "newline_str").unwrap();
                    self.builder.build_call(
                        print_string_fn,
                        &[newline_str.as_pointer_value().into()],
                        "print_newline"
                    ).unwrap();

                    // Push None as the result
                    let none_value = self.llvm_context.i64_type().const_int(0, false);
                    result_stack.push(ExprResult {
                        value: none_value.into(),
                        ty: Type::None,
                    });
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
