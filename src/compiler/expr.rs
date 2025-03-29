use crate::ast::{Expr, Number, NameConstant, Operator, UnaryOperator, CmpOperator};
use crate::compiler::context::CompilationContext;
use crate::compiler::types::Type;
use crate::compiler::types::is_reference_type;
use inkwell::values::BasicValueEnum;

/// Extension trait for handling expression code generation
pub trait ExprCompiler<'ctx> {
    /// Compile an expression and return the resulting LLVM value with its type
    fn compile_expr(&mut self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String>;
    
    /// Compile a numeric literal
    fn compile_number(&mut self, num: &Number) -> Result<(BasicValueEnum<'ctx>, Type), String>;
    
    /// Compile a name constant (True, False, None)
    fn compile_name_constant(&mut self, constant: &NameConstant) -> Result<(BasicValueEnum<'ctx>, Type), String>;
}

pub trait AssignmentCompiler<'ctx> {
    /// Compile an assignment expression
    fn compile_assignment(&mut self, target: &Expr, value: BasicValueEnum<'ctx>, 
                        value_type: &Type) -> Result<(), String>;
}

/// Extension trait for handling binary operations with type conversions
pub trait BinaryOpCompiler<'ctx> {
    /// Compile a binary operation with type conversion if needed
    fn compile_binary_op(&mut self, left: inkwell::values::BasicValueEnum<'ctx>, left_type: &Type,
                       op: Operator, right: inkwell::values::BasicValueEnum<'ctx>, right_type: &Type)
                       -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String>;
}

/// Extension trait for handling comparison operations with type conversions
pub trait ComparisonCompiler<'ctx> {
    /// Compile a comparison operation with type conversion if needed
    fn compile_comparison(&mut self, left: inkwell::values::BasicValueEnum<'ctx>, left_type: &Type,
                        op: CmpOperator, right: inkwell::values::BasicValueEnum<'ctx>, right_type: &Type)
                        -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String>;
}

impl<'ctx> ExprCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_expr(&mut self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        match expr {
            Expr::Num { value, .. } => self.compile_number(value),
            Expr::NameConstant { value, .. } => self.compile_name_constant(value),
            Expr::BinOp { left, op, right, .. } => {
                // Compile both operands
                let (left_val, left_type) = self.compile_expr(left)?;
                let (right_val, right_type) = self.compile_expr(right)?;
                
                // Use our binary operation compiler
                self.compile_binary_op(left_val, &left_type, op.clone(), right_val, &right_type)
            },
            Expr::UnaryOp { op, operand, .. } => {
                // Compile the operand
                let (operand_val, operand_type) = self.compile_expr(operand)?;
                
                // Handle different unary operators
                match op {
                    UnaryOperator::Not => {
                        // Convert to bool if needed
                        let bool_val = if !matches!(operand_type, Type::Bool) {
                            self.convert_type(operand_val, &operand_type, &Type::Bool)?
                        } else {
                            operand_val
                        };
                        
                        let result = self.builder.build_not(bool_val.into_int_value(), "not").unwrap();
                        Ok((result.into(), Type::Bool))
                    },
                    UnaryOperator::USub => {
                        match operand_type {
                            Type::Int => {
                                let int_val = operand_val.into_int_value();
                                let result = self.builder.build_int_neg(int_val, "neg").unwrap();
                                Ok((result.into(), Type::Int))
                            },
                            Type::Float => {
                                let float_val = operand_val.into_float_value();
                                let result = self.builder.build_float_neg(float_val, "neg").unwrap();
                                Ok((result.into(), Type::Float))
                            },
                            _ => Err(format!("Cannot negate value of type {:?}", operand_type)),
                        }
                    },
                    UnaryOperator::UAdd => {
                        // Unary plus is a no-op
                        Ok((operand_val, operand_type))
                    },
                    UnaryOperator::Invert => {
                        // Bitwise NOT (~)
                        match operand_type {
                            Type::Int => {
                                let int_val = operand_val.into_int_value();
                                let result = self.builder.build_not(int_val, "invert").unwrap();
                                Ok((result.into(), Type::Int))
                            },
                            _ => Err(format!("Cannot bitwise invert value of type {:?}", operand_type)),
                        }
                    },
                }
            },
            Expr::Compare { left, ops, comparators, .. } => {
                if ops.is_empty() || comparators.is_empty() {
                    return Err("Empty comparison".to_string());
                }
                
                // Compile the left operand
                let (left_val, left_type) = self.compile_expr(left)?;
                
                // For each comparison operator and right operand
                let mut current_val = left_val;
                let mut current_type = left_type.clone();
                let mut result_val: Option<BasicValueEnum<'ctx>> = None;
                
                for (op, right) in ops.iter().zip(comparators.iter()) {
                    // Compile the right operand
                    let (right_val, right_type) = self.compile_expr(right)?;
                    
                    // Perform the comparison using our comparison compiler
                    let (cmp_result, _) = self.compile_comparison(current_val, &current_type, 
                                                               op.clone(), right_val, &right_type)?;
                    
                    // For chained comparisons (a < b < c), we need to AND the results
                    if let Some(prev_result) = result_val {
                        let and_result = self.builder.build_and(
                            prev_result.into_int_value(),
                            cmp_result.into_int_value(),
                            "and_cmp"
                        ).unwrap();
                        result_val = Some(and_result.into());
                    } else {
                        result_val = Some(cmp_result);
                    }
                    
                    // For the next comparison, the left operand is the current right operand
                    current_val = right_val;
                    current_type = right_type;
                }
                
                Ok((result_val.unwrap(), Type::Bool))
            },
            Expr::Name { id, .. } => {
                // Look up variable type
                if let Some(var_type) = self.lookup_variable_type(id) {
                    // Look up variable storage location
                    if let Some(ptr) = self.get_variable_ptr(id) {
                        // Get the LLVM type for the variable
                        let llvm_type = self.get_llvm_type(var_type);
                        
                        // Load the variable's value with the correct method signature
                        // Note the three arguments: type, pointer, name
                        let value = self.builder.build_load(llvm_type, ptr, id).unwrap();
                        Ok((value, var_type.clone()))
                    } else {
                        Err(format!("Variable '{}' has no allocated storage", id))
                    }
                } else {
                    Err(format!("Undefined variable: {}", id))
                }
            },
            // Handle other expression types
            _ => Err(format!("Unsupported expression type: {:?}", expr)),
        }
    }
    
    fn compile_number(&mut self, num: &Number) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        match num {
            Number::Integer(value) => {
                let int_type = self.llvm_context.i64_type();
                let int_value = int_type.const_int(*value as u64, true);
                Ok((int_value.into(), Type::Int))
            },
            Number::Float(value) => {
                let float_type = self.llvm_context.f64_type();
                let float_value = float_type.const_float(*value);
                Ok((float_value.into(), Type::Float))
            },
            Number::Complex { real, imag } => {
                // For complex numbers, you might create a struct with real and imaginary parts
                let float_type = self.llvm_context.f64_type();
                let struct_type = self.llvm_context.struct_type(&[
                    float_type.into(),
                    float_type.into(),
                ], false);
                
                let real_value = float_type.const_float(*real);
                let imag_value = float_type.const_float(*imag);
                
                let complex_value = struct_type.const_named_struct(&[
                    real_value.into(),
                    imag_value.into(),
                ]);
                
                Ok((complex_value.into(), Type::Float)) // Simplified for now
            },
        }
    }
    
    fn compile_name_constant(&mut self, constant: &NameConstant) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        match constant {
            NameConstant::True => {
                let bool_type = self.llvm_context.bool_type();
                let bool_value = bool_type.const_int(1, false);
                Ok((bool_value.into(), Type::Bool))
            },
            NameConstant::False => {
                let bool_type = self.llvm_context.bool_type();
                let bool_value = bool_type.const_int(0, false);
                Ok((bool_value.into(), Type::Bool))
            },
            NameConstant::None => {
                let ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let null_value = ptr_type.const_null();
                Ok((null_value.into(), Type::None))
            },
        }
    }
}

impl<'ctx> BinaryOpCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_binary_op(&mut self, left: inkwell::values::BasicValueEnum<'ctx>, left_type: &Type,
        op: Operator, right: inkwell::values::BasicValueEnum<'ctx>, right_type: &Type)
        -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String> {
        // Get the common type for this operation
        let common_type = self.get_common_type(left_type, right_type)?;
        
        // Convert operands to common type if needed
        let left_converted = if left_type != &common_type {
            self.convert_type(left, left_type, &common_type)?
        } else {
            left
        };
        
        let right_converted = if right_type != &common_type {
            self.convert_type(right, right_type, &common_type)?
        } else {
            right
        };
        
        // Perform the operation on converted values
        match op {
            Operator::Add => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self.builder.build_int_add(left_int, right_int, "int_add").unwrap();
                    Ok((result.into(), Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    let result = self.builder.build_float_add(left_float, right_float, "float_add").unwrap();
                    Ok((result.into(), Type::Float))
                },
                Type::String => {
                    // String concatenation would require runtime support
                    Err("String concatenation not yet implemented".to_string())
                },
                _ => Err(format!("Addition not supported for type {:?}", common_type)),
            },
            
            Operator::Sub => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self.builder.build_int_sub(left_int, right_int, "int_sub").unwrap();
                    Ok((result.into(), Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    let result = self.builder.build_float_sub(left_float, right_float, "float_sub").unwrap();
                    Ok((result.into(), Type::Float))
                },
                _ => Err(format!("Subtraction not supported for type {:?}", common_type)),
            },
            
            Operator::Mult => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self.builder.build_int_mul(left_int, right_int, "int_mul").unwrap();
                    Ok((result.into(), Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    let result = self.builder.build_float_mul(left_float, right_float, "float_mul").unwrap();
                    Ok((result.into(), Type::Float))
                },
                _ => Err(format!("Multiplication not supported for type {:?}", common_type)),
            },
            
            Operator::Div => match common_type {
                Type::Int => {
                    // Convert to float for division to avoid integer division issues
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    
                    // Check for division by zero
                    let zero = self.llvm_context.i64_type().const_zero();
                    let is_zero = self.builder.build_int_compare(
                        inkwell::IntPredicate::EQ,
                        right_int,
                        zero,
                        "is_zero"
                    ).unwrap();
                    
                    // Create basic blocks for division by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let div_bb = self.llvm_context.append_basic_block(current_function, "div");
                    let div_by_zero_bb = self.llvm_context.append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");
                    
                    // Branch based on division by zero check
                    self.builder.build_conditional_branch(is_zero, div_by_zero_bb, div_bb).unwrap();
                    
                    // Normal division block
                    self.builder.position_at_end(div_bb);
                    let left_float = self.builder.build_signed_int_to_float(left_int, self.llvm_context.f64_type(), "int_to_float").unwrap();
                    let right_float = self.builder.build_signed_int_to_float(right_int, self.llvm_context.f64_type(), "int_to_float").unwrap();
                    let div_result = self.builder.build_float_div(left_float, right_float, "float_div").unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();
                    
                    // Division by zero error block
                    self.builder.position_at_end(div_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();
                    
                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.f64_type(), "div_result").unwrap();
                    phi.add_incoming(&[(&div_result, div_bb), (&error_value, div_by_zero_bb)]);
                    
                    Ok((phi.as_basic_value(), Type::Float))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    
                    // Check for division by zero
                    let zero = self.llvm_context.f64_type().const_float(0.0);
                    let is_zero = self.builder.build_float_compare(
                        inkwell::FloatPredicate::OEQ,
                        right_float,
                        zero,
                        "is_zero"
                    ).unwrap();
                    
                    // Create basic blocks for division by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let div_bb = self.llvm_context.append_basic_block(current_function, "div");
                    let div_by_zero_bb = self.llvm_context.append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");
                    
                    // Branch based on division by zero check
                    self.builder.build_conditional_branch(is_zero, div_by_zero_bb, div_bb).unwrap();
                    
                    // Normal division block
                    self.builder.position_at_end(div_bb);
                    let div_result = self.builder.build_float_div(left_float, right_float, "float_div").unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();
                    
                    // Division by zero error block
                    self.builder.position_at_end(div_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();
                    
                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.f64_type(), "div_result").unwrap();
                    phi.add_incoming(&[(&div_result, div_bb), (&error_value, div_by_zero_bb)]);
                    
                    Ok((phi.as_basic_value(), Type::Float))
                },
                _ => Err(format!("Division not supported for type {:?}", common_type)),
            },
            
            Operator::FloorDiv => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    
                    // Check for division by zero
                    let zero = self.llvm_context.i64_type().const_zero();
                    let is_zero = self.builder.build_int_compare(
                        inkwell::IntPredicate::EQ,
                        right_int,
                        zero,
                        "is_zero"
                    ).unwrap();
                    
                    // Create basic blocks for division by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let div_bb = self.llvm_context.append_basic_block(current_function, "div");
                    let div_by_zero_bb = self.llvm_context.append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");
                    
                    // Branch based on division by zero check
                    self.builder.build_conditional_branch(is_zero, div_by_zero_bb, div_bb).unwrap();
                    
                    // Normal division block
                    self.builder.position_at_end(div_bb);
                    let div_result = self.builder.build_int_signed_div(left_int, right_int, "int_div").unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();
                    
                    // Division by zero error block
                    self.builder.position_at_end(div_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.i64_type().const_zero();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();
                    
                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.i64_type(), "div_result").unwrap();
                    phi.add_incoming(&[(&div_result, div_bb), (&error_value, div_by_zero_bb)]);
                    
                    Ok((phi.as_basic_value(), Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    
                    // Check for division by zero
                    let zero = self.llvm_context.f64_type().const_float(0.0);
                    let is_zero = self.builder.build_float_compare(
                        inkwell::FloatPredicate::OEQ,
                        right_float,
                        zero,
                        "is_zero"
                    ).unwrap();
                    
                    // Create basic blocks for division by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let div_bb = self.llvm_context.append_basic_block(current_function, "div");
                    let div_by_zero_bb = self.llvm_context.append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");
                    
                    // Branch based on division by zero check
                    self.builder.build_conditional_branch(is_zero, div_by_zero_bb, div_bb).unwrap();
                    
                    // Normal division block
                    self.builder.position_at_end(div_bb);
                    let div_result = self.builder.build_float_div(left_float, right_float, "float_div").unwrap();
                    let floor_result = self.builder.build_call(
                        self.module.get_function("llvm.floor.f64").unwrap_or_else(|| {
                            // Create floor function declaration if it doesn't exist
                            let f64_type = self.llvm_context.f64_type();
                            let function_type = f64_type.fn_type(&[f64_type.into()], false);
                            self.module.add_function("llvm.floor.f64", function_type, None)
                        }),
                        &[div_result.into()],
                        "floor_div"
                    ).unwrap();
                    let floor_result = floor_result.try_as_basic_value().left().unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();
                    
                    // Division by zero error block
                    self.builder.position_at_end(div_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();
                    
                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.f64_type(), "div_result").unwrap();
                    phi.add_incoming(&[(&floor_result, div_bb), (&error_value, div_by_zero_bb)]);
                    
                    Ok((phi.as_basic_value(), Type::Float))
                },
                _ => Err(format!("Floor division not supported for type {:?}", common_type)),
            },
            
            Operator::Mod => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    
                    // Check for modulo by zero
                    let zero = self.llvm_context.i64_type().const_zero();
                    let is_zero = self.builder.build_int_compare(
                        inkwell::IntPredicate::EQ,
                        right_int,
                        zero,
                        "is_zero"
                    ).unwrap();
                    
                    // Create basic blocks for modulo by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let mod_bb = self.llvm_context.append_basic_block(current_function, "mod");
                    let mod_by_zero_bb = self.llvm_context.append_basic_block(current_function, "mod_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");
                    
                    // Branch based on modulo by zero check
                    self.builder.build_conditional_branch(is_zero, mod_by_zero_bb, mod_bb).unwrap();
                    
                    // Normal modulo block
                    self.builder.position_at_end(mod_bb);
                    let mod_result = self.builder.build_int_signed_rem(left_int, right_int, "int_mod").unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_bb = self.builder.get_insert_block().unwrap();
                    
                    // Modulo by zero error block
                    self.builder.position_at_end(mod_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.i64_type().const_zero();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_by_zero_bb = self.builder.get_insert_block().unwrap();
                    
                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.i64_type(), "mod_result").unwrap();
                    phi.add_incoming(&[(&mod_result, mod_bb), (&error_value, mod_by_zero_bb)]);
                    
                    Ok((phi.as_basic_value(), Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    
                    // Check for modulo by zero
                    let zero = self.llvm_context.f64_type().const_float(0.0);
                    let is_zero = self.builder.build_float_compare(
                        inkwell::FloatPredicate::OEQ,
                        right_float,
                        zero,
                        "is_zero"
                    ).unwrap();
                    
                    // Create basic blocks for modulo by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let mod_bb = self.llvm_context.append_basic_block(current_function, "mod");
                    let mod_by_zero_bb = self.llvm_context.append_basic_block(current_function, "mod_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");
                    
                    // Branch based on modulo by zero check
                    self.builder.build_conditional_branch(is_zero, mod_by_zero_bb, mod_bb).unwrap();
                    
                    // Normal modulo block
                    self.builder.position_at_end(mod_bb);
                    let mod_result = self.builder.build_call(
                        self.module.get_function("fmod").unwrap_or_else(|| {
                            // Create fmod function declaration if it doesn't exist
                            let f64_type = self.llvm_context.f64_type();
                            let function_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
                            self.module.add_function("fmod", function_type, None)
                        }),
                        &[left_float.into(), right_float.into()],
                        "float_mod"
                    ).unwrap();
                    let mod_result = mod_result.try_as_basic_value().left().unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_bb = self.builder.get_insert_block().unwrap();
                    
                    // Modulo by zero error block
                    self.builder.position_at_end(mod_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_by_zero_bb = self.builder.get_insert_block().unwrap();
                    
                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.f64_type(), "mod_result").unwrap();
                    phi.add_incoming(&[(&mod_result, mod_bb), (&error_value, mod_by_zero_bb)]);
                    
                    Ok((phi.as_basic_value(), Type::Float))
                },
                _ => Err(format!("Modulo not supported for type {:?}", common_type)),
            },
            
            Operator::Pow => match common_type {
                Type::Int => {
                    // For integer exponentiation, we'll use a runtime function or convert to float
                    let left_float = self.convert_type(left_converted, &Type::Int, &Type::Float)?;
                    let right_float = self.convert_type(right_converted, &Type::Int, &Type::Float)?;
                    
                    let pow_result = self.builder.build_call(
                        self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
                            // Create pow function declaration if it doesn't exist
                            let f64_type = self.llvm_context.f64_type();
                            let function_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
                            self.module.add_function("llvm.pow.f64", function_type, None)
                        }),
                        &[left_float.into_float_value().into(), right_float.into_float_value().into()],
                        "float_pow"
                    ).unwrap();
                    
                    // Convert back to integer
                    let pow_float = pow_result.try_as_basic_value().left().unwrap();
                    let pow_int = self.convert_type(pow_float, &Type::Float, &Type::Int)?;
                    
                    Ok((pow_int, Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    
                    let pow_result = self.builder.build_call(
                        self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
                            // Create pow function declaration if it doesn't exist
                            let f64_type = self.llvm_context.f64_type();
                            let function_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
                            self.module.add_function("llvm.pow.f64", function_type, None)
                        }),
                        &[left_float.into(), right_float.into()],
                        "float_pow"
                    ).unwrap();
                    
                    let pow_float = pow_result.try_as_basic_value().left().unwrap();
                    
                    Ok((pow_float, Type::Float))
                },
                _ => Err(format!("Power operation not supported for type {:?}", common_type)),
            },
            
            // Implement other binary operators (bitwise, etc.) as needed
            _ => Err(format!("Binary operator {:?} not implemented", op)),
        }
    }
}

impl<'ctx> ComparisonCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_comparison(&mut self, left: inkwell::values::BasicValueEnum<'ctx>, left_type: &Type,
                        op: CmpOperator, right: inkwell::values::BasicValueEnum<'ctx>, right_type: &Type)
                        -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String> {
        // Special cases for identity comparisons (is, is not)
        if matches!(op, CmpOperator::Is) || matches!(op, CmpOperator::IsNot) {
            // For reference types, compare pointers
            if is_reference_type(left_type) && is_reference_type(right_type) {
                let left_ptr = if left.is_pointer_value() {
                    left.into_pointer_value()
                } else {
                    // Convert to pointer value if needed
                    let left_as_ptr = self.builder.build_bit_cast(
                        left,
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "as_ptr"
                    ).unwrap();
                    left_as_ptr.into_pointer_value()
                };
                
                let right_ptr = if right.is_pointer_value() {
                    right.into_pointer_value()
                } else {
                    // Convert to pointer value if needed
                    let right_as_ptr = self.builder.build_bit_cast(
                        right,
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "as_ptr"
                    ).unwrap();
                    right_as_ptr.into_pointer_value()
                };
                
                // Convert pointers to integers for comparison
                let left_ptr_int = self.builder.build_ptr_to_int(
                    left_ptr,
                    self.llvm_context.i64_type(),
                    "ptr_as_int"
                ).unwrap();
                
                let right_ptr_int = self.builder.build_ptr_to_int(
                    right_ptr,
                    self.llvm_context.i64_type(),
                    "ptr_as_int"
                ).unwrap();
                
                let is_same = self.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    left_ptr_int,
                    right_ptr_int,
                    "is_same"
                ).unwrap();
                
                // For "is not", negate the result
                let result = if matches!(op, CmpOperator::IsNot) {
                    self.builder.build_not(is_same, "is_not_same").unwrap()
                } else {
                    is_same
                };
                
                return Ok((result.into(), Type::Bool));
            }
            
            // For primitive types, just compare values
            return self.compile_comparison(left, left_type, 
                                          if matches!(op, CmpOperator::Is) { CmpOperator::Eq } else { CmpOperator::NotEq }, 
                                          right, right_type);
        }
        
        // Special cases for 'in' and 'not in' operators
        if matches!(op, CmpOperator::In) || matches!(op, CmpOperator::NotIn) {
            // These would require runtime support functions for collections
            return Err(format!("'in' operator not yet implemented for types {:?} and {:?}", left_type, right_type));
        }
        
        // For regular comparisons, get the common type
        let common_type = self.get_common_type(left_type, right_type)?;
        
        // Convert operands to common type if needed
        let left_converted = if left_type != &common_type {
            self.convert_type(left, left_type, &common_type)?
        } else {
            left
        };
        
        let right_converted = if right_type != &common_type {
            self.convert_type(right, right_type, &common_type)?
        } else {
            right
        };
        
        // Perform the comparison on converted values
        match common_type {
            Type::Int => {
                let left_int = left_converted.into_int_value();
                let right_int = right_converted.into_int_value();
                
                let pred = match op {
                    CmpOperator::Eq => inkwell::IntPredicate::EQ,
                    CmpOperator::NotEq => inkwell::IntPredicate::NE,
                    CmpOperator::Lt => inkwell::IntPredicate::SLT,
                    CmpOperator::LtE => inkwell::IntPredicate::SLE,
                    CmpOperator::Gt => inkwell::IntPredicate::SGT,
                    CmpOperator::GtE => inkwell::IntPredicate::SGE,
                    _ => return Err(format!("Comparison operator {:?} not supported for integers", op)),
                };
                
                let result = self.builder.build_int_compare(pred, left_int, right_int, "int_cmp").unwrap();
                Ok((result.into(), Type::Bool))
            },
            
            Type::Float => {
                let left_float = left_converted.into_float_value();
                let right_float = right_converted.into_float_value();
                
                let pred = match op {
                    CmpOperator::Eq => inkwell::FloatPredicate::OEQ,
                    CmpOperator::NotEq => inkwell::FloatPredicate::ONE,
                    CmpOperator::Lt => inkwell::FloatPredicate::OLT,
                    CmpOperator::LtE => inkwell::FloatPredicate::OLE,
                    CmpOperator::Gt => inkwell::FloatPredicate::OGT,
                    CmpOperator::GtE => inkwell::FloatPredicate::OGE,
                    _ => return Err(format!("Comparison operator {:?} not supported for floats", op)),
                };
                
                let result = self.builder.build_float_compare(pred, left_float, right_float, "float_cmp").unwrap();
                Ok((result.into(), Type::Bool))
            },
            
            Type::Bool => {
                let left_bool = left_converted.into_int_value();
                let right_bool = right_converted.into_int_value();
                
                let pred = match op {
                    CmpOperator::Eq => inkwell::IntPredicate::EQ,
                    CmpOperator::NotEq => inkwell::IntPredicate::NE,
                    _ => return Err(format!("Comparison operator {:?} not supported for booleans", op)),
                };
                
                let result = self.builder.build_int_compare(pred, left_bool, right_bool, "bool_cmp").unwrap();
                Ok((result.into(), Type::Bool))
            },
            
            Type::String => {
                // String comparisons would require runtime support functions
                Err(format!("String comparison not yet implemented for operator {:?}", op))
            },
            
            _ => Err(format!("Comparison not supported for type {:?}", common_type)),
        }
    }
}

impl<'ctx> AssignmentCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_assignment(&mut self, target: &Expr, value: BasicValueEnum<'ctx>, 
        value_type: &Type) -> Result<(), String> {
        match target {
            Expr::Name { id, .. } => {
                // Look up variable storage
                if let Some(ptr) = self.get_variable_ptr(id) {
                    // Check if types are compatible
                    if let Some(target_type) = self.lookup_variable_type(id) {
                        // Convert value to target type if needed
                        let converted_value = if target_type != value_type {
                            self.convert_type(value, value_type, target_type)?
                        } else {
                            value
                        };
                        
                        // Store the value to the variable
                        self.builder.build_store(ptr, converted_value).unwrap();
                        Ok(())
                    } else {
                        Err(format!("Variable '{}' has unknown type", id))
                    }
                } else {
                    Err(format!("Variable '{}' has no allocated storage", id))
                }
            },
            
            // Handle other assignment targets (subscripts, attributes, etc.)
            _ => Err(format!("Unsupported assignment target: {:?}", target)),
        }
    }
}