// print_call.rs - Implementation of the print() function call

use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::types::Type;
use crate::compiler::expr::ExprCompiler;
use inkwell::values::BasicValueEnum;

impl<'ctx> CompilationContext<'ctx> {
    /// Compile a call to the print() function
    pub fn compile_print_call(&mut self, args: &[Expr]) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Handle the case with no arguments
        if args.is_empty() {
            // Get the print_newline function
            let print_newline_fn = match self.module.get_function("print_newline") {
                Some(f) => f,
                None => return Err("print_newline function not found".to_string()),
            };

            // Call print_newline
            let _call_site_value = self.builder.build_call(
                print_newline_fn,
                &[],
                "print_newline_call"
            ).unwrap();

            // Return void
            return Ok((self.llvm_context.i64_type().const_zero().into(), Type::None));
        }

        // Process each argument
        for (_i, arg) in args.iter().enumerate() {
            // Compile the argument
            let (arg_val, arg_type) = self.compile_expr(arg)?;

            // Determine which print function to call based on the argument type
            match arg_type {
                Type::Int => {
                    // Get the print_int function
                    let print_int_fn = match self.module.get_function("print_int") {
                        Some(f) => f,
                        None => return Err("print_int function not found".to_string()),
                    };

                    // Call print_int
                    self.builder.build_call(
                        print_int_fn,
                        &[arg_val.into()],
                        "print_int_call"
                    ).unwrap();
                },
                Type::Float => {
                    // Get the print_float function
                    let print_float_fn = match self.module.get_function("print_float") {
                        Some(f) => f,
                        None => return Err("print_float function not found".to_string()),
                    };

                    // Call print_float
                    self.builder.build_call(
                        print_float_fn,
                        &[arg_val.into()],
                        "print_float_call"
                    ).unwrap();
                },
                Type::Bool => {
                    // Get the print_bool function
                    let print_bool_fn = match self.module.get_function("print_bool") {
                        Some(f) => f,
                        None => return Err("print_bool function not found".to_string()),
                    };

                    // Call print_bool
                    self.builder.build_call(
                        print_bool_fn,
                        &[arg_val.into()],
                        "print_bool_call"
                    ).unwrap();
                },
                Type::String => {
                    // Get the print_string function
                    let print_string_fn = match self.module.get_function("print_string") {
                        Some(f) => f,
                        None => return Err("print_string function not found".to_string()),
                    };

                    // Call print_string
                    self.builder.build_call(
                        print_string_fn,
                        &[arg_val.into()],
                        "print_string_call"
                    ).unwrap();
                },
                _ => {
                    // For other types, convert to string first
                    // Try to convert the value to a string using int_to_string as a fallback
                    let int_to_string_fn = match self.module.get_function("int_to_string") {
                        Some(f) => f,
                        None => return Err("int_to_string function not found".to_string()),
                    };

                    // Convert the value to an integer and then to a string
                    let str_val = if arg_val.is_int_value() {
                        // Call int_to_string directly
                        let call_site_value = self.builder.build_call(
                            int_to_string_fn,
                            &[arg_val.into()],
                            "to_string_result"
                        ).unwrap();

                        call_site_value.try_as_basic_value().left()
                            .ok_or_else(|| "Failed to convert to string".to_string())?
                    } else {
                        // Default to a placeholder string for unsupported types
                        let placeholder = format!("<{:?}>", arg_type);
                        let placeholder_bytes = placeholder.as_bytes();
                        let placeholder_str = self.llvm_context.const_string(placeholder_bytes, false);
                        let global_str = self.module.add_global(placeholder_str.get_type(), None, "placeholder_str");
                        global_str.set_initializer(&placeholder_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        str_ptr.into()
                    };

                    // Get the print_string function
                    let print_string_fn = match self.module.get_function("print_string") {
                        Some(f) => f,
                        None => return Err("print_string function not found".to_string()),
                    };

                    // Call print_string with the converted value
                    self.builder.build_call(
                        print_string_fn,
                        &[str_val.into()],
                        "print_string_call"
                    ).unwrap();
                }
            }
        }

        // Add a newline at the end of the print call
        // Get the print_newline function
        let print_newline_fn = match self.module.get_function("print_newline") {
            Some(f) => f,
            None => return Err("print_newline function not found".to_string()),
        };

        // Call print_newline
        self.builder.build_call(
            print_newline_fn,
            &[],
            "print_newline_call"
        ).unwrap();

        // Return void
        Ok((self.llvm_context.i64_type().const_zero().into(), Type::None))
    }
}