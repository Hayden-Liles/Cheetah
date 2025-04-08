use crate::ast::{BoolOperator, CmpOperator, Expr, ExprContext, NameConstant, Number, Operator, UnaryOperator};
use crate::compiler::context::CompilationContext;
use crate::compiler::types::Type;
use crate::compiler::types::is_reference_type;
use inkwell::values::BasicValueEnum;

/// Extension trait for handling expression code generation
pub trait ExprCompiler<'ctx> {
    fn build_empty_list(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_list(&self, elements: Vec<BasicValueEnum<'ctx>>, element_type: &Type) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_empty_tuple(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_tuple(&self, elements: Vec<BasicValueEnum<'ctx>>, element_types: &[Type]) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_empty_dict(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_dict(&self, keys: Vec<BasicValueEnum<'ctx>>, values: Vec<BasicValueEnum<'ctx>>, key_type: &Type, value_type: &Type) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_empty_set(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_set(&self, elements: Vec<BasicValueEnum<'ctx>>, element_type: &Type) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_list_get_item(&self, list_ptr: inkwell::values::PointerValue<'ctx>, index: inkwell::values::IntValue<'ctx>) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_dict_get_item(&self, dict_ptr: inkwell::values::PointerValue<'ctx>, key: BasicValueEnum<'ctx>, key_type: &Type) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_string_get_char(&self, str_ptr: inkwell::values::PointerValue<'ctx>, index: inkwell::values::IntValue<'ctx>) -> Result<BasicValueEnum<'ctx>, String>;
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
                        let value = self.builder.build_load(llvm_type, ptr, id).unwrap();
                        Ok((value, var_type.clone()))
                    } else {
                        Err(format!("Variable '{}' has no allocated storage", id))
                    }
                } else {
                    Err(format!("Undefined variable: {}", id))
                }
            },
            
            Expr::Str { value, .. } => {
                // Create a global string constant
                let str_type = self.llvm_context.i8_type().array_type(value.len() as u32 + 1);
                
                // Create the string constant (null-terminated)
                let mut chars: Vec<u8> = value.bytes().collect();
                chars.push(0); // Null terminator
                
                // Create a global variable to hold the string constant
                let global_str = self.module.add_global(str_type, None, "str_const");
                global_str.set_constant(true);
                global_str.set_initializer(&self.llvm_context.const_string(&chars, true));
                
                // Get a pointer to the string (fixed deprecated method)
                let str_ptr = self.builder.build_pointer_cast(
                    global_str.as_pointer_value(),
                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                    "str_ptr"
                ).unwrap();
                
                // Return the string pointer and String type
                Ok((str_ptr.into(), Type::String))
            },
            
            Expr::BoolOp { op, values, .. } => {
                if values.is_empty() {
                    return Err("Empty boolean operation".to_string());
                }
                
                // Compile the first value
                let (first_val, first_type) = self.compile_expr(&values[0])?;
                
                // Convert to boolean if needed
                let bool_type = Type::Bool;
                let mut current_val = if first_type != bool_type {
                    self.convert_type(first_val, &first_type, &bool_type)?.into_int_value()
                } else {
                    first_val.into_int_value()
                };
                
                // If there's only one value, just return it as a boolean
                if values.len() == 1 {
                    return Ok((current_val.into(), bool_type));
                }
                
                // Current function
                let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                
                // Create a phi node to gather results from different paths
                let result_ptr = self.builder.build_alloca(self.llvm_context.bool_type(), "bool_result").unwrap();
                
                // Store the initial value
                self.builder.build_store(result_ptr, current_val).unwrap();
                
                // Create merge block where all paths will converge
                let mut merge_block = self.llvm_context.append_basic_block(current_function, "bool_merge");
                
                // Process the rest of the values with short-circuit evaluation
                for (i, value_expr) in values.iter().skip(1).enumerate() {
                    // Create blocks for short-circuit and next value evaluation
                    let next_value_block = self.llvm_context.append_basic_block(current_function, &format!("next_value_{}", i));
                    let short_circuit_block = self.llvm_context.append_basic_block(current_function, &format!("short_circuit_{}", i));
                    
                    // Branch based on the boolean operator
                    match op {
                        BoolOperator::And => {
                            // For AND, if current value is false, short-circuit
                            self.builder.build_conditional_branch(current_val, next_value_block, short_circuit_block).unwrap();
                        },
                        BoolOperator::Or => {
                            // For OR, if current value is true, short-circuit
                            self.builder.build_conditional_branch(current_val, short_circuit_block, next_value_block).unwrap();
                        },
                    }
                    
                    // Compile the next value
                    self.builder.position_at_end(next_value_block);
                    let (next_val, next_type) = self.compile_expr(value_expr)?;
                    
                    // Convert to boolean if needed
                    let next_bool = if next_type != bool_type {
                        self.convert_type(next_val, &next_type, &bool_type)?.into_int_value()
                    } else {
                        next_val.into_int_value()
                    };
                    
                    // Store the result and branch to merge
                    self.builder.build_store(result_ptr, next_bool).unwrap();
                    self.builder.build_unconditional_branch(merge_block).unwrap();
                    
                    // Handle short-circuit case
                    self.builder.position_at_end(short_circuit_block);
                    
                    // In short-circuit case, value remains the same (false for AND, true for OR)
                    // We already stored the value at the beginning, so no need to store again
                    self.builder.build_unconditional_branch(merge_block).unwrap();
                    
                    // Continue at the merge block for the next iteration
                    self.builder.position_at_end(merge_block);
                    
                    // Load the result for the next iteration
                    current_val = self.builder.build_load(self.llvm_context.bool_type(), result_ptr, "bool_op_result").unwrap().into_int_value();
                    
                    // Create a new merge block for the next iteration (if not the last one)
                    if i < values.len() - 2 {
                        let new_merge_block = self.llvm_context.append_basic_block(current_function, &format!("bool_merge_{}", i+1));
                        merge_block = new_merge_block;
                    }
                }
                
                // The final value is our result
                Ok((current_val.into(), bool_type))
            },
            
            Expr::Call { func, args, keywords, .. } => {
                // For function calls, we need to handle various cases:
                // 1. Direct function references
                // 2. Function pointers
                // 3. Method calls
    
                // Different handling based on the function expression
                match func.as_ref() {
                    Expr::Name { id, .. } => {
                        // Direct function reference by name
                        if let Some(func_value) = self.functions.get(id) {
                            // Compile all argument expressions
                            let mut arg_values = Vec::with_capacity(args.len());
                            let mut arg_types = Vec::with_capacity(args.len());
                            
                            for arg in args {
                                let (arg_val, arg_type) = self.compile_expr(arg)?;
                                arg_values.push(arg_val);
                                arg_types.push(arg_type);
                            }
                            
                            // Handle keyword arguments
                            if !keywords.is_empty() {
                                return Err("Keyword arguments not yet implemented".to_string());
                            }
                            
                            // Convert to inkwell's BasicMetadataValueEnum
                            let call_args: Vec<_> = arg_values
                                .iter()
                                .map(|&v| v.into())
                                .collect();
                            
                            // Build the call instruction
                            let call = self.builder.build_call(*func_value, &call_args, "call").unwrap();
                            
                            // Get the return value if there is one
                            if let Some(ret_val) = call.try_as_basic_value().left() {
                                // Look up the function's return type
                                // For now, we'll just return the value with a placeholder type
                                Ok((ret_val, Type::Int)) // Simplified; should get actual return type
                            } else {
                                // Function returns void
                                Ok((self.llvm_context.i32_type().const_zero().into(), Type::Void))
                            }
                        } else {
                            Err(format!("Undefined function: {}", id))
                        }
                    },
                    _ => {
                        // For now, only support direct function references
                        Err("Indirect function calls not yet implemented".to_string())
                    }
                }
            },
            
            Expr::IfExp { test, body, orelse, .. } => {
                // Compile the test expression
                let (test_val, test_type) = self.compile_expr(test)?;
                
                // Convert to boolean if needed
                let cond_val = if test_type != Type::Bool {
                    self.convert_type(test_val, &test_type, &Type::Bool)?.into_int_value()
                } else {
                    test_val.into_int_value()
                };
                
                // Create basic blocks for then, else, and merge
                let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                let then_block = self.llvm_context.append_basic_block(current_function, "if_then");
                let else_block = self.llvm_context.append_basic_block(current_function, "if_else");
                let merge_block = self.llvm_context.append_basic_block(current_function, "if_merge");
                
                // Branch based on the condition
                self.builder.build_conditional_branch(cond_val, then_block, else_block).unwrap();
                
                // Compile the then expression
                self.builder.position_at_end(then_block);
                let (then_val, then_type) = self.compile_expr(body)?;
                let then_block = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_block).unwrap();
                
                // Compile the else expression
                self.builder.position_at_end(else_block);
                let (else_val, else_type) = self.compile_expr(orelse)?;
                let else_block = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_block).unwrap();
                
                // Determine the result type
                let result_type = if then_type == else_type {
                    then_type.clone()
                } else {
                    // Try to find a common type that both can be converted to
                    match self.get_common_type(&then_type, &else_type) {
                        Ok(common_type) => common_type,
                        Err(_) => return Err(format!("Incompatible types in if expression: {:?} and {:?}", then_type, else_type)),
                    }
                };
                
                // Convert both values to the result type if needed
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
                
                // Create the phi node - fixed error by using llvm_type directly
                let llvm_type = self.get_llvm_type(&result_type);
                let phi = self.builder.build_phi(llvm_type, "if_result").unwrap();
                
                // Add the incoming values
                phi.add_incoming(&[
                    (&then_val, then_block),
                    (&else_val, else_block),
                ]);
                
                Ok((phi.as_basic_value(), result_type))
            },
            
            // For the remaining expressions, we'll return a placeholder error since they're not yet implemented
            Expr::List { .. } => Err("List expressions not yet implemented".to_string()),
            Expr::Tuple { .. } => Err("Tuple expressions not yet implemented".to_string()),
            Expr::Dict { .. } => Err("Dictionary expressions not yet implemented".to_string()),
            Expr::Set { .. } => Err("Set expressions not yet implemented".to_string()),
            Expr::Attribute { .. } => Err("Attribute access not yet implemented".to_string()),
            Expr::Subscript { .. } => Err("Subscript expressions not yet implemented".to_string()),
            
            // Handle other expression types with appropriate placeholder errors
            _ => Err(format!("Unsupported expression type: {:?}", expr)),
        }
    }
    
    // Placeholder methods for collection operations (to be implemented with runtime support)
    // These would be defined in your CompilationContext impl block
    
    fn build_empty_list(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = name;
        Err("List operations require runtime support (not yet implemented)".to_string())
    }
    
    fn build_list(
        &self, 
        elements: Vec<BasicValueEnum<'ctx>>, 
        element_type: &Type
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = elements;
        let _ = element_type;
        Err("List operations require runtime support (not yet implemented)".to_string())
    }
    
    fn build_empty_tuple(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = name;
        Err("Tuple operations require runtime support (not yet implemented)".to_string())
    }
    
    fn build_tuple(
        &self, 
        elements: Vec<BasicValueEnum<'ctx>>, 
        element_types: &[Type]
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = elements;
        let _ = element_types;
        Err("Tuple operations require runtime support (not yet implemented)".to_string())
    }
    
    fn build_empty_dict(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = name;
        Err("Dict operations require runtime support (not yet implemented)".to_string())
    }
    
    fn build_dict(
        &self,
        keys: Vec<BasicValueEnum<'ctx>>,
        values: Vec<BasicValueEnum<'ctx>>,
        key_type: &Type,
        value_type: &Type
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = keys;
        let _ = values;
        let _ = key_type;
        let _ = value_type;
        Err("Dict operations require runtime support (not yet implemented)".to_string())
    }
    
    fn build_empty_set(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = name;
        Err("Set operations require runtime support (not yet implemented)".to_string())
    }
    
    fn build_set(
        &self,
        elements: Vec<BasicValueEnum<'ctx>>,
        element_type: &Type
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = elements;
        let _ = element_type;
        Err("Set operations require runtime support (not yet implemented)".to_string())
    }
    
    fn build_list_get_item(
        &self,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = list_ptr;
        let _ = index;
        Err("List operations require runtime support (not yet implemented)".to_string())
    }
    
    fn build_dict_get_item(
        &self,
        dict_ptr: inkwell::values::PointerValue<'ctx>,
        key: BasicValueEnum<'ctx>,
        key_type: &Type
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = dict_ptr;
        let _ = key;
        let _ = key_type;
        Err("Dict operations require runtime support (not yet implemented)".to_string())
    }
    
    fn build_string_get_char(
        &self,
        str_ptr: inkwell::values::PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let _ = str_ptr;
        let _ = index;
        Err("String operations require runtime support (not yet implemented)".to_string())
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
                // Check if variable already exists
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
                    // Variable doesn't exist yet, allocate storage for it
                    let ptr = self.allocate_variable(id.clone(), value_type);
                    
                    // Store the value to the newly created variable
                    self.builder.build_store(ptr, value).unwrap();
                    Ok(())
                }
            },
            
            // Handle other assignment targets (subscripts, attributes, etc.)
            _ => Err(format!("Unsupported assignment target: {:?}", target)),
        }
    }
}