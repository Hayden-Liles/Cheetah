use crate::ast::Operator;
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::BinaryOpCompiler;
use crate::compiler::types::Type;

impl<'ctx> BinaryOpCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_binary_op(
        &mut self,
        left: inkwell::values::BasicValueEnum<'ctx>,
        left_type: &Type,
        op: Operator,
        right: inkwell::values::BasicValueEnum<'ctx>,
        right_type: &Type,
    ) -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String> {
        // Fast path for unboxed Int arithmetic
        if left_type == &Type::Int && right_type == &Type::Int {
            // purely unboxed i64 math
            let l = left.into_int_value();
            let r = right.into_int_value();
            let result = match op {
                Operator::Add => self.builder.build_int_add(l, r, "add")?.into(),
                Operator::Sub => self.builder.build_int_sub(l, r, "sub")?.into(),
                Operator::Mult => self.builder.build_int_mul(l, r, "mul")?.into(),
                Operator::Div => self.builder.build_int_signed_div(l, r, "div")?.into(),
                _ => {
                    // Fallback to BoxedAny path for other operators
                    return self.compile_boxed_binary_op(left, left_type, op, right, right_type);
                }
            };
            return Ok((result, Type::Int));
        }

        // Fallback to BoxedAny path for other types
        self.compile_boxed_binary_op(left, left_type, op, right, right_type)
    }
}

impl<'ctx> CompilationContext<'ctx> {
    // Helper method for BoxedAny binary operations
    fn compile_boxed_binary_op(
        &mut self,
        left: inkwell::values::BasicValueEnum<'ctx>,
        left_type: &Type,
        op: Operator,
        right: inkwell::values::BasicValueEnum<'ctx>,
        right_type: &Type,
    ) -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String> {
        // With BoxedAny, we need to convert types before operations
        let (left_boxed, _) = self.maybe_box(left, left_type)?;
        let (right_boxed, _) = self.maybe_box(right, right_type)?;

        // Make sure we have pointers to BoxedAny values
        if !left_boxed.is_pointer_value() || !right_boxed.is_pointer_value() {
            return Err("Expected BoxedAny pointers for binary operation".to_string());
        }

        let left_ptr = left_boxed.into_pointer_value();
        let right_ptr = right_boxed.into_pointer_value();

        // Determine which BoxedAny operation to call based on the operator
        match op {
            Operator::Add => {
                // Get the boxed_any_add function
                let boxed_any_add_fn = self.module.get_function("boxed_any_add")
                    .ok_or_else(|| "boxed_any_add function not found".to_string())?;

                // Call boxed_any_add to perform the addition
                let call_site_value = self.builder.build_call(
                    boxed_any_add_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_add"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny addition".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::Sub => {
                // Get the boxed_any_subtract function
                let boxed_any_subtract_fn = self.module.get_function("boxed_any_subtract")
                    .ok_or_else(|| "boxed_any_subtract function not found".to_string())?;

                // Call boxed_any_subtract to perform the subtraction
                let call_site_value = self.builder.build_call(
                    boxed_any_subtract_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_subtract"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny subtraction".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::Mult => {
                // Get the boxed_any_multiply function
                let boxed_any_multiply_fn = self.module.get_function("boxed_any_multiply")
                    .ok_or_else(|| "boxed_any_multiply function not found".to_string())?;

                // Call boxed_any_multiply to perform the multiplication
                let call_site_value = self.builder.build_call(
                    boxed_any_multiply_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_multiply"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny multiplication".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::Div => {
                // Get the boxed_any_divide function
                let boxed_any_divide_fn = self.module.get_function("boxed_any_divide")
                    .ok_or_else(|| "boxed_any_divide function not found".to_string())?;

                // Call boxed_any_divide to perform the division
                let call_site_value = self.builder.build_call(
                    boxed_any_divide_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_divide"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny division".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::FloorDiv => {
                // Get the boxed_any_floor_div function
                let boxed_any_floor_div_fn = self.module.get_function("boxed_any_floor_div")
                    .ok_or_else(|| "boxed_any_floor_div function not found".to_string())?;

                // Call boxed_any_floor_div to perform the floor division
                let call_site_value = self.builder.build_call(
                    boxed_any_floor_div_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_floor_div"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny floor division".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::Mod => {
                // Get the boxed_any_modulo function
                let boxed_any_modulo_fn = self.module.get_function("boxed_any_modulo")
                    .ok_or_else(|| "boxed_any_modulo function not found".to_string())?;

                // Call boxed_any_modulo to perform the modulo operation
                let call_site_value = self.builder.build_call(
                    boxed_any_modulo_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_modulo"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny modulo operation".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::Pow => {
                // Get the boxed_any_power function
                let boxed_any_power_fn = self.module.get_function("boxed_any_power")
                    .ok_or_else(|| "boxed_any_power function not found".to_string())?;

                // Call boxed_any_power to perform the power operation
                let call_site_value = self.builder.build_call(
                    boxed_any_power_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_power"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny power operation".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::LShift => {
                // Get the boxed_any_lshift function
                let boxed_any_lshift_fn = self.module.get_function("boxed_any_lshift")
                    .ok_or_else(|| "boxed_any_lshift function not found".to_string())?;

                // Call boxed_any_lshift to perform the left shift operation
                let call_site_value = self.builder.build_call(
                    boxed_any_lshift_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_lshift"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny left shift operation".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::RShift => {
                // Get the boxed_any_rshift function
                let boxed_any_rshift_fn = self.module.get_function("boxed_any_rshift")
                    .ok_or_else(|| "boxed_any_rshift function not found".to_string())?;

                // Call boxed_any_rshift to perform the right shift operation
                let call_site_value = self.builder.build_call(
                    boxed_any_rshift_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_rshift"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny right shift operation".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::BitOr => {
                // Get the boxed_any_bitor function
                let boxed_any_bitor_fn = self.module.get_function("boxed_any_bitor")
                    .ok_or_else(|| "boxed_any_bitor function not found".to_string())?;

                // Call boxed_any_bitor to perform the bitwise OR operation
                let call_site_value = self.builder.build_call(
                    boxed_any_bitor_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_bitor"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny bitwise OR operation".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::BitXor => {
                // Get the boxed_any_bitxor function
                let boxed_any_bitxor_fn = self.module.get_function("boxed_any_bitxor")
                    .ok_or_else(|| "boxed_any_bitxor function not found".to_string())?;

                // Call boxed_any_bitxor to perform the bitwise XOR operation
                let call_site_value = self.builder.build_call(
                    boxed_any_bitxor_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_bitxor"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny bitwise XOR operation".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            Operator::BitAnd => {
                // Get the boxed_any_bitand function
                let boxed_any_bitand_fn = self.module.get_function("boxed_any_bitand")
                    .ok_or_else(|| "boxed_any_bitand function not found".to_string())?;

                // Call boxed_any_bitand to perform the bitwise AND operation
                let call_site_value = self.builder.build_call(
                    boxed_any_bitand_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "boxed_bitand"
                )?;

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny bitwise AND operation".to_string())?;

                // All BoxedAny operations return Type::Any
                Ok((result, Type::Any))
            },
            // For other operators, we'll need to implement them as needed
            _ => Err(format!("Binary operator {:?} not yet implemented for BoxedAny", op))
        }
    }
}
