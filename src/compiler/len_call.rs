// len_call.rs - Implementation of the len() built-in function

use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::types::Type;
use crate::compiler::expr::ExprCompiler;
use inkwell::values::BasicValueEnum;

impl<'ctx> CompilationContext<'ctx> {
    /// Compile a call to the len() function
    pub fn compile_len_call(&mut self, args: &[Expr]) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Check that we have exactly one argument
        if args.len() != 1 {
            return Err(format!("len() takes exactly one argument ({} given)", args.len()));
        }

        // Compile the argument
        let (arg_val, arg_type) = self.compile_expr(&args[0])?;

        // Check if the argument is a string or list
        match arg_type {
            Type::String => {
                // Get the string_len function
                let string_len_fn = match self.module.get_function("string_len") {
                    Some(f) => f,
                    None => return Err("string_len function not found".to_string()),
                };

                // Make sure the argument is a pointer
                let arg_ptr = if arg_val.is_pointer_value() {
                    arg_val.into_pointer_value()
                } else {
                    // If it's not a pointer, allocate memory and store the value
                    let ptr = self.builder.build_alloca(arg_val.get_type(), "string_arg").unwrap();
                    self.builder.build_store(ptr, arg_val).unwrap();
                    ptr
                };

                // Call the string_len function
                let call_site_value = self.builder.build_call(
                    string_len_fn,
                    &[arg_ptr.into()],
                    "string_len_result"
                ).unwrap();

                // Get the result
                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to get string length".to_string())?;

                Ok((result, Type::Int))
            },
            Type::List(_) => {
                // Get the list_len function
                let list_len_fn = match self.module.get_function("list_len") {
                    Some(f) => f,
                    None => return Err("list_len function not found".to_string()),
                };

                // Make sure the argument is a pointer
                let arg_ptr = if arg_val.is_pointer_value() {
                    arg_val.into_pointer_value()
                } else {
                    // If it's not a pointer, allocate memory and store the value
                    let ptr = self.builder.build_alloca(arg_val.get_type(), "list_arg").unwrap();
                    self.builder.build_store(ptr, arg_val).unwrap();
                    ptr
                };

                // Call the list_len function
                let call_site_value = self.builder.build_call(
                    list_len_fn,
                    &[arg_ptr.into()],
                    "list_len_result"
                ).unwrap();

                // Get the result
                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to get list length".to_string())?;

                Ok((result, Type::Int))
            },
            // Handle function parameters that might be strings or lists
            Type::Any => {
                // Try to treat the value as a string first
                if let Ok(result) = self.try_get_string_length(arg_val) {
                    return Ok((result, Type::Int));
                }

                // Then try to treat it as a list
                if let Ok(result) = self.try_get_list_length(arg_val) {
                    return Ok((result, Type::Int));
                }

                // If neither works, return an error
                Err("Cannot determine length of Any type".to_string())
            },
            _ => Err(format!("Object of type '{:?}' has no len()", arg_type)),
        }
    }

    /// Try to get the length of a string
    fn try_get_string_length(&self, value: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        // Get the string_len function
        let string_len_fn = match self.module.get_function("string_len") {
            Some(f) => f,
            None => return Err("string_len function not found".to_string()),
        };

        // Make sure the value is a pointer
        let value_ptr = if value.is_pointer_value() {
            value.into_pointer_value()
        } else {
            return Err("Value is not a pointer".to_string());
        };

        // Call the string_len function
        let call_site_value = self.builder.build_call(
            string_len_fn,
            &[value_ptr.into()],
            "string_len_result"
        ).unwrap();

        // Get the result
        call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get string length".to_string())
    }

    /// Try to get the length of a list
    fn try_get_list_length(&self, value: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        // Get the list_len function
        let list_len_fn = match self.module.get_function("list_len") {
            Some(f) => f,
            None => return Err("list_len function not found".to_string()),
        };

        // Make sure the value is a pointer
        let value_ptr = if value.is_pointer_value() {
            value.into_pointer_value()
        } else {
            return Err("Value is not a pointer".to_string());
        };

        // Call the list_len function
        let call_site_value = self.builder.build_call(
            list_len_fn,
            &[value_ptr.into()],
            "list_len_result"
        ).unwrap();

        // Get the result
        call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get list length".to_string())
    }
}
