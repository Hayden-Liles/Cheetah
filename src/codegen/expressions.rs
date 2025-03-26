use inkwell::values::{BasicValueEnum, BasicValue};
use crate::ast::{Expr, ExprContext, BoolOperator, Operator, UnaryOperator, CmpOperator, NameConstant};
use super::context::CompilationContext;
use super::error::CodegenError;
use super::types::number_to_llvm_value;

pub fn compile_expr<'ctx>(
    context: &mut CompilationContext<'ctx>,
    expr: &Expr
) -> Result<BasicValueEnum<'ctx>, CodegenError> {
    match expr {
        Expr::Num { value, line, column } => {
            // Convert AST number to LLVM value
            Ok(number_to_llvm_value(context.context, value))
        },
        
        Expr::Str { value, line, column } => {
            // Create a global string constant
            let string_ptr = context.builder.build_global_string_ptr(value, "str_const")
                .as_pointer_value();
            
            Ok(string_ptr.into())
        },
        
        Expr::NameConstant { value, line, column } => {
            match value {
                NameConstant::True => Ok(context.context.bool_type().const_int(1, false).into()),
                NameConstant::False => Ok(context.context.bool_type().const_int(0, false).into()),
                NameConstant::None => {
                    // None is represented as a null pointer
                    let null_ptr = context.context
                        .i8_type()
                        .ptr_type(Default::default())
                        .const_null();
                    
                    Ok(null_ptr.into())
                }
            }
        },
        
        Expr::Name { id, ctx, line, column } => {
            // Look up the variable in the context
            let var_info = context.lookup_variable(id)?;
            
            // Load the value if we're in a Load context
            match ctx {
                ExprContext::Load => {
                    let load_instr = context.builder.build_load(var_info.ty, var_info.ptr, id);
                    Ok(load_instr)
                },
                ExprContext::Store => {
                    // For store context, return the pointer, not the value
                    Ok(var_info.ptr.into())
                },
                ExprContext::Del => {
                    return Err(CodegenError::unsupported_feature("Delete expression context"));
                }
            }
        },
        
        Expr::BinOp { left, op, right, line, column } => {
            let lhs = compile_expr(context, left)?;
            let rhs = compile_expr(context, right)?;
            
            // Make sure the types match
            if lhs.get_type() != rhs.get_type() {
                return Err(CodegenError::type_error(&format!(
                    "Mismatched types in binary operation: {:?} and {:?}",
                    lhs.get_type(),
                    rhs.get_type()
                )));
            }
            
            match (lhs, rhs) {
                // Integer operations
                (BasicValueEnum::IntValue(lhs), BasicValueEnum::IntValue(rhs)) => {
                    let result = match op {
                        Operator::Add => context.builder.build_int_add(lhs, rhs, "add_tmp"),
                        Operator::Sub => context.builder.build_int_sub(lhs, rhs, "sub_tmp"),
                        Operator::Mult => context.builder.build_int_mul(lhs, rhs, "mul_tmp"),
                        Operator::Div => context.builder.build_int_signed_div(lhs, rhs, "div_tmp"),
                        Operator::FloorDiv => context.builder.build_int_signed_div(lhs, rhs, "floordiv_tmp"),
                        Operator::Mod => context.builder.build_int_signed_rem(lhs, rhs, "mod_tmp"),
                        Operator::BitAnd => context.builder.build_and(lhs, rhs, "and_tmp"),
                        Operator::BitOr => context.builder.build_or(lhs, rhs, "or_tmp"),
                        Operator::BitXor => context.builder.build_xor(lhs, rhs, "xor_tmp"),
                        Operator::LShift => context.builder.build_left_shift(lhs, rhs, "lshift_tmp"),
                        Operator::RShift => context.builder.build_right_shift(lhs, rhs, true, "rshift_tmp"),
                        Operator::Pow => {
                            return Err(CodegenError::unsupported_feature("Integer power operation"));
                        },
                        _ => {
                            return Err(CodegenError::unsupported_feature(
                                &format!("Unsupported integer operation: {:?}", op)
                            ));
                        }
                    };
                    
                    Ok(result.into())
                },
                
                // Float operations
                (BasicValueEnum::FloatValue(lhs), BasicValueEnum::FloatValue(rhs)) => {
                    let result = match op {
                        Operator::Add => context.builder.build_float_add(lhs, rhs, "add_tmp"),
                        Operator::Sub => context.builder.build_float_sub(lhs, rhs, "sub_tmp"),
                        Operator::Mult => context.builder.build_float_mul(lhs, rhs, "mul_tmp"),
                        Operator::Div => context.builder.build_float_div(lhs, rhs, "div_tmp"),
                        Operator::FloorDiv => {
                            // Floor division for floats requires calling the floor function on the result
                            let div = context.builder.build_float_div(lhs, rhs, "floordiv_tmp");
                            // We'd need to call a floor intrinsic here
                            return Err(CodegenError::unsupported_feature("Float floor division"));
                        },
                        Operator::Mod => {
                            // Float modulo is more complex and usually implemented via stdlib
                            return Err(CodegenError::unsupported_feature("Float modulo operation"));
                        },
                        Operator::Pow => {
                            // Need to call a pow function from math lib
                            return Err(CodegenError::unsupported_feature("Float power operation"));
                        },
                        _ => {
                            return Err(CodegenError::unsupported_feature(
                                &format!("Unsupported float operation: {:?}", op)
                            ));
                        }
                    };
                    
                    Ok(result.into())
                },
                
                _ => {
                    Err(CodegenError::type_error(&format!(
                        "Unsupported operand types for binary operation: {:?}",
                        op
                    )))
                }
            }
        },
        
        Expr::UnaryOp { op, operand, line, column } => {
            let value = compile_expr(context, operand)?;
            
            match (op, value) {
                // Integer unary operations
                (UnaryOperator::USub, BasicValueEnum::IntValue(int_val)) => {
                    let zero = context.context.i64_type().const_zero();
                    let result = context.builder.build_int_sub(zero, int_val, "neg_tmp");
                    Ok(result.into())
                },
                (UnaryOperator::UAdd, BasicValueEnum::IntValue(int_val)) => {
                    // Unary plus doesn't change the value
                    Ok(int_val.into())
                },
                (UnaryOperator::Invert, BasicValueEnum::IntValue(int_val)) => {
                    let result = context.builder.build_not(int_val, "invert_tmp");
                    Ok(result.into())
                },
                
                // Float unary operations
                (UnaryOperator::USub, BasicValueEnum::FloatValue(float_val)) => {
                    let result = context.builder.build_float_neg(float_val, "neg_tmp");
                    Ok(result.into())
                },
                (UnaryOperator::UAdd, BasicValueEnum::FloatValue(float_val)) => {
                    // Unary plus doesn't change the value
                    Ok(float_val.into())
                },
                
                // Boolean not
                (UnaryOperator::Not, val) => {
                    // Convert to boolean first if needed
                    let bool_val = match val {
                        BasicValueEnum::IntValue(int_val) => {
                            let zero = context.context.i64_type().const_zero();
                            context.builder.build_int_compare(
                                inkwell::IntPredicate::NE,
                                int_val,
                                zero,
                                "bool_cast"
                            )
                        },
                        BasicValueEnum::FloatValue(float_val) => {
                            let zero = context.context.f64_type().const_zero();
                            context.builder.build_float_compare(
                                inkwell::FloatPredicate::ONE,
                                float_val,
                                zero,
                                "bool_cast"
                            )
                        },
                        _ => {
                            return Err(CodegenError::type_error(
                                "Cannot apply 'not' to this type"
                            ));
                        }
                    };
                    
                    let result = context.builder.build_not(bool_val, "not_tmp");
                    Ok(result.into())
                },
                
                _ => {
                    Err(CodegenError::type_error(&format!(
                        "Unsupported operand type for unary operation: {:?}",
                        op
                    )))
                }
            }
        },
        
        Expr::BoolOp { op, values, line, column } => {
            if values.is_empty() {
                return Err(CodegenError::expression_error("Empty boolean operation"));
            }
            
            // Short-circuit evaluation for 'and' and 'or'
            let current_fn = context.current_function()?;
            
            // Create basic blocks for short-circuit evaluation
            let entry_block = context.builder.get_insert_block().ok_or_else(|| {
                CodegenError::internal_error("No current block")
            })?;
            
            // Evaluate the first value
            let first_value = compile_expr(context, &values[0])?;
            
            // If there's only one value, return it
            if values.len() == 1 {
                return Ok(first_value);
            }
            
            // Convert the first value to a boolean if needed
            let first_bool = match first_value {
                BasicValueEnum::IntValue(int_val) => {
                    let zero = context.context.i64_type().const_zero();
                    context.builder.build_int_compare(
                        inkwell::IntPredicate::NE,
                        int_val,
                        zero,
                        "bool_cast"
                    )
                },
                _ => {
                    return Err(CodegenError::type_error(
                        "Boolean operations only support integer values for now"
                    ));
                }
            };
            
            // Create basic blocks for remaining values
            let mut phi_blocks = vec![context.builder.get_insert_block().unwrap()];
            let mut phi_values = vec![first_bool.into()];
            
            // Create continuing block
            let continue_block = context.context.append_basic_block(current_fn, "continue");
            
            // Create final block
            let final_block = context.context.append_basic_block(current_fn, "bool_op_final");
            
            match op {
                BoolOperator::And => {
                    // Short-circuit: if first value is false, skip the rest
                    context.builder.build_conditional_branch(
                        first_bool,
                        continue_block,
                        final_block
                    );
                },
                BoolOperator::Or => {
                    // Short-circuit: if first value is true, skip the rest
                    context.builder.build_conditional_branch(
                        first_bool,
                        final_block,
                        continue_block
                    );
                }
            }
            
            // Process remaining values with short-circuit
            context.builder.position_at_end(continue_block);
            
            for (i, value) in values.iter().enumerate().skip(1) {
                let is_last = i == values.len() - 1;
                
                let value_result = compile_expr(context, value)?;
                let bool_val = match value_result {
                    BasicValueEnum::IntValue(int_val) => {
                        let zero = context.context.i64_type().const_zero();
                        context.builder.build_int_compare(
                            inkwell::IntPredicate::NE,
                            int_val,
                            zero,
                            "bool_cast"
                        )
                    },
                    _ => {
                        return Err(CodegenError::type_error(
                            "Boolean operations only support integer values for now"
                        ));
                    }
                };
                
                phi_blocks.push(context.builder.get_insert_block().unwrap());
                phi_values.push(bool_val.into());
                
                if !is_last {
                    // Create next continue block
                    let next_continue = context.context.append_basic_block(current_fn, "continue");
                    
                    match op {
                        BoolOperator::And => {
                            // AND: continue to next value only if current value is true
                            context.builder.build_conditional_branch(
                                bool_val,
                                next_continue,
                                final_block
                            );
                        },
                        BoolOperator::Or => {
                            // OR: exit early if current value is true
                            context.builder.build_conditional_branch(
                                bool_val,
                                final_block,
                                next_continue
                            );
                        }
                    }
                    
                    context.builder.position_at_end(next_continue);
                } else {
                    // Last value, unconditional branch to final
                    context.builder.build_unconditional_branch(final_block);
                }
            }
            
            // Final block with PHI node
            context.builder.position_at_end(final_block);
            
            let phi = context.builder.build_phi(context.context.bool_type(), "bool_op_result");
            
            for (value, block) in phi_values.iter().zip(phi_blocks.iter()) {
                if let BasicValueEnum::IntValue(int_val) = value {
                    phi.add_incoming(&[(&int_val, block)]);
                }
            }
            
            Ok(phi.as_basic_value())
        },
        
        // Many more expression types to implement...
        
        _ => Err(CodegenError::unsupported_feature(
            &format!("Unsupported expression type: {:?}", expr)
        )),
    }
}