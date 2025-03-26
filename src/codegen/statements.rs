use inkwell::values::BasicValueEnum;
use crate::ast::{Stmt, Expr, ExprContext};
use super::context::CompilationContext;
use super::error::CodegenError;
use super::expressions::compile_expr;
use super::variables::{define_variable, assign_variable};
use super::functions::{declare_function, define_function, compile_call};

pub fn compile_stmt<'ctx>(
    context: &mut CompilationContext<'ctx>,
    stmt: &Stmt
) -> Result<(), CodegenError> {
    match stmt {
        Stmt::Expr { value, .. } => {
            // Expression statement - compile it but discard the result
            compile_expr(context, value)?;
            Ok(())
        },
        
        Stmt::Assign { targets, value, .. } => {
            // Compile the value first
            let value_result = compile_expr(context, value)?;
            
            // Assign it to each target
            for target in targets {
                match target.as_ref() {
                    Expr::Name { id, ctx, .. } => {
                        match ctx {
                            ExprContext::Store => {
                                // Check if variable already exists
                                let var_exists = context.variables.contains_key(id);
                                
                                if var_exists {
                                    // Existing variable
                                    assign_variable(context, target, value_result)?;
                                } else {
                                    // New variable
                                    define_variable(
                                        context, 
                                        id, 
                                        value_result.get_type(),
                                        Some(value_result),
                                        true // Mutable by default
                                    )?;
                                }
                            },
                            _ => {
                                return Err(CodegenError::type_error(
                                    "Assignment target must be in store context"
                                ));
                            }
                        }
                    },
                    
                    // Handle other types of assignment targets
                    _ => assign_variable(context, target, value_result)?,
                }
            }
            
            Ok(())
        },
        
        Stmt::AugAssign { target, op, value, .. } => {
            // Get the current value of the target
            let target_value = compile_expr(context, target)?;
            
            // Get the value to add/sub/etc.
            let value_result = compile_expr(context, value)?;
            
            // Create a BinOp expression to combine them
            let bin_op = Expr::BinOp { 
                left: Box::new(target.as_ref().clone()),
                op: op.clone(),
                right: value.clone(),
                line: target.get_line(),
                column: target.get_column(),
            };
            
            // Compute the combined value
            let result = compile_expr(context, &bin_op)?;
            
            // Assign it back to the target
            assign_variable(context, target, result)?;
            
            Ok(())
        },
        
        Stmt::FunctionDef { name, params, body, returns, is_async, .. } => {
            if *is_async {
                return Err(CodegenError::unsupported_feature("Async functions"));
            }
            
            // Determine return type from annotation
            let return_type = if let Some(ret_expr) = returns {
                match ret_expr.as_ref() {
                    Expr::Name { id, .. } => {
                        Some(super::types::map_type(context.context, id)?)
                    },
                    _ => {
                        return Err(CodegenError::type_error(
                            "Complex return type annotations not supported yet"
                        ));
                    }
                }
            } else {
                None
            };
            
            // Declare the function
            let function = declare_function(context, name, params, return_type, false)?;
            
            // Define the function body
            define_function(context, function, params, body)?;
            
            Ok(())
        },
        
        Stmt::Return { value, .. } => {
            // Make sure we're in a function
            let function = context.current_function()?;
            
            // Check return type
            let return_type = function.get_type().get_return_type();
            
            match (value, return_type) {
                (Some(expr), Some(_)) => {
                    // Return with value
                    let return_value = compile_expr(context, expr)?;
                    context.builder.build_return(Some(&return_value));
                },
                (None, None) => {
                    // Void return
                    context.builder.build_return(None);
                },
                (Some(_), None) => {
                    return Err(CodegenError::type_error(
                        "Cannot return a value from a void function"
                    ));
                },
                (None, Some(_)) => {
                    return Err(CodegenError::type_error(
                        "Must return a value from a non-void function"
                    ));
                }
            }
            
            Ok(())
        },
        
        Stmt::If { test, body, orelse, .. } => {
            // Get the current function
            let function = context.current_function()?;
            
            // Create basic blocks
            let then_block = context.context.append_basic_block(function, "then");
            let else_block = context.context.append_basic_block(function, "else");
            let merge_block = context.context.append_basic_block(function, "if.end");
            
            // Compile the condition
            let condition = compile_expr(context, test)?;
            
            // Convert to a boolean if needed
            let condition_bool = match condition {
                BasicValueEnum::IntValue(int_val) => {
                    let zero = context.context.i64_type().const_zero();
                    context.builder.build_int_compare(
                        inkwell::IntPredicate::NE,
                        int_val,
                        zero,
                        "if.cond"
                    )
                },
                BasicValueEnum::FloatValue(float_val) => {
                    let zero = context.context.f64_type().const_zero();
                    context.builder.build_float_compare(
                        inkwell::FloatPredicate::ONE,
                        float_val,
                        zero,
                        "if.cond"
                    )
                },
                _ => {
                    return Err(CodegenError::type_error(
                        "If condition must be a boolean expression"
                    ));
                }
            };
            
            // Branch based on condition
            context.builder.build_conditional_branch(condition_bool, then_block, else_block);
            
            // Build 'then' block
            context.builder.position_at_end(then_block);
            for stmt in body {
                compile_stmt(context, stmt)?;
            }
            
            // Branch to merge block if we haven't already returned
            if !context.builder.get_insert_block().unwrap().get_terminator().is_some() {
                context.builder.build_unconditional_branch(merge_block);
            }
            
            // Build 'else' block
            context.builder.position_at_end(else_block);
            for stmt in orelse {
                compile_stmt(context, stmt)?;
            }
            
            // Branch to merge block if we haven't already returned
            if !context.builder.get_insert_block().unwrap().get_terminator().is_some() {
                context.builder.build_unconditional_branch(merge_block);
            }
            
            // Continue with merge block
            context.builder.position_at_end(merge_block);
            
            Ok(())
        },
        
        Stmt::While { test, body, orelse, .. } => {
            // Get the current function
            let function = context.current_function()?;
            
            // Create basic blocks
            let cond_block = context.context.append_basic_block(function, "while.cond");
            let body_block = context.context.append_basic_block(function, "while.body");
            let else_block = context.context.append_basic_block(function, "while.else");
            let end_block = context.context.append_basic_block(function, "while.end");
            
            // Jump to condition block
            context.builder.build_unconditional_branch(cond_block);
            
            // Build condition block
            context.builder.position_at_end(cond_block);
            let condition = compile_expr(context, test)?;
            
            // Convert to a boolean if needed
            let condition_bool = match condition {
                BasicValueEnum::IntValue(int_val) => {
                    let zero = context.context.i64_type().const_zero();
                    context.builder.build_int_compare(
                        inkwell::IntPredicate::NE,
                        int_val,
                        zero,
                        "while.cond"
                    )
                },
                _ => {
                    return Err(CodegenError::type_error(
                        "While condition must be a boolean expression"
                    ));
                }
            };
            
            // Branch based on condition
            context.builder.build_conditional_branch(condition_bool, body_block, else_block);
            
            // Set up loop context
            context.enter_loop(cond_block, end_block);
            
            // Build body block
            context.builder.position_at_end(body_block);
            for stmt in body {
                compile_stmt(context, stmt)?;
            }
            
            // Jump back to condition after body
            if !context.builder.get_insert_block().unwrap().get_terminator().is_some() {
                context.builder.build_unconditional_branch(cond_block);
            }
            
            // Exit loop context
            context.exit_loop();
            
            // Build else block
            context.builder.position_at_end(else_block);
            for stmt in orelse {
                compile_stmt(context, stmt)?;
            }
            
            // Jump to end after else
            if !context.builder.get_insert_block().unwrap().get_terminator().is_some() {
                context.builder.build_unconditional_branch(end_block);
            }
            
            // Continue with end block
            context.builder.position_at_end(end_block);
            
            Ok(())
        },
        
        Stmt::Break { .. } => {
            // Make sure we're in a loop
            let break_block = context.break_block()?;
            
            // Branch to the break block
            context.builder.build_unconditional_branch(break_block);
            
            Ok(())
        },
        
        Stmt::Continue { .. } => {
            // Make sure we're in a loop
            let continue_block = context.continue_block()?;
            
            // Branch to the continue block
            context.builder.build_unconditional_branch(continue_block);
            
            Ok(())
        },
        
        // Many more statements to implement...
        
        _ => Err(CodegenError::unsupported_feature(
            &format!("Unsupported statement type: {:?}", stmt)
        )),
    }
}