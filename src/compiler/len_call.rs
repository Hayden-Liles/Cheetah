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

        // Check if the argument is a string
        match arg_type {
            Type::String => {
                // Get the string_len function
                let string_len_fn = match self.module.get_function("string_len") {
                    Some(f) => f,
                    None => return Err("string_len function not found".to_string()),
                };

                // Call the string_len function
                let call_site_value = self.builder.build_call(
                    string_len_fn,
                    &[arg_val.into()],
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

                // Call the list_len function
                let call_site_value = self.builder.build_call(
                    list_len_fn,
                    &[arg_val.into()],
                    "list_len_result"
                ).unwrap();

                // Get the result
                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to get list length".to_string())?;

                Ok((result, Type::Int))
            },
            _ => Err(format!("Object of type '{:?}' has no len()", arg_type)),
        }
    }
}
