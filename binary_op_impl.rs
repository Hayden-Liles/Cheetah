impl<'ctx> BinaryOpCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_binary_op(
        &mut self,
        left: inkwell::values::BasicValueEnum<'ctx>,
        left_type: &Type,
        op: Operator,
        right: inkwell::values::BasicValueEnum<'ctx>,
        right_type: &Type,
    ) -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String> {
        // With BoxedAny, we don't need to convert types before operations
        // The BoxedAny operations will handle type conversions internally

        // Make sure we have pointers to BoxedAny values
        if !left.is_pointer_value() || !right.is_pointer_value() {
            return Err("Expected BoxedAny pointers for binary operation".to_string());
        }

        let left_ptr = left.into_pointer_value();
        let right_ptr = right.into_pointer_value();

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
                ).unwrap();

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny addition".to_string())?;

                // Determine the result type based on the operand types
                let result_type = if left_type == &Type::Float || right_type == &Type::Float {
                    Type::Float
                } else if left_type == &Type::String || right_type == &Type::String {
                    Type::String
                } else {
                    Type::Int
                };

                Ok((result, result_type))
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
                ).unwrap();

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny subtraction".to_string())?;

                // Determine the result type based on the operand types
                let result_type = if left_type == &Type::Float || right_type == &Type::Float {
                    Type::Float
                } else {
                    Type::Int
                };

                Ok((result, result_type))
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
                ).unwrap();

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny multiplication".to_string())?;

                // Determine the result type based on the operand types
                let result_type = if left_type == &Type::Float || right_type == &Type::Float {
                    Type::Float
                } else if left_type == &Type::String && right_type == &Type::Int {
                    Type::String
                } else if left_type == &Type::List(Box::new(Type::Any)) && right_type == &Type::Int {
                    Type::List(Box::new(Type::Any))
                } else {
                    Type::Int
                };

                Ok((result, result_type))
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
                ).unwrap();

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to perform BoxedAny division".to_string())?;

                // Division always returns a float in Python
                Ok((result, Type::Float))
            },

            // For other operators, we'll need to implement them as needed
            _ => Err(format!("Binary operator {:?} not yet implemented for BoxedAny", op))
        }
    }
}
