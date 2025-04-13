// print_call.rs - Implementation of the print() built-in function

use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::types::Type;
use crate::compiler::expr::ExprCompiler;
use inkwell::values::BasicValueEnum;

impl<'ctx> CompilationContext<'ctx> {
    /// Register the print function
    pub fn register_print_function(&mut self) {
        let context = self.llvm_context;
        let module = &mut self.module;

        // Create the print function type for strings
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        let fn_type = context.void_type().fn_type(&[ptr_type.into()], false);

        // Add the function to the module
        let function = module.add_function("print_string", fn_type, None);

        // Register the function in our context
        self.functions.insert("print_string".to_string(), function);

        // Also register the println_string function
        let println_fn = match module.get_function("println_string") {
            Some(f) => f,
            None => {
                // Create the println_string function if it doesn't exist
                let println_type = context.void_type().fn_type(&[ptr_type.into()], false);
                module.add_function("println_string", println_type, None)
            }
        };

        // Register the println_string function
        self.functions.insert("println_string".to_string(), println_fn);

        // Register the print_int function
        let print_int_fn = match module.get_function("print_int") {
            Some(f) => f,
            None => {
                // Create the print_int function if it doesn't exist
                let print_int_type = context.void_type().fn_type(&[context.i64_type().into()], false);
                module.add_function("print_int", print_int_type, None)
            }
        };

        // Register the print_int function
        self.functions.insert("print_int".to_string(), print_int_fn);

        // Register the print_float function
        let print_float_fn = match module.get_function("print_float") {
            Some(f) => f,
            None => {
                // Create the print_float function if it doesn't exist
                let print_float_type = context.void_type().fn_type(&[context.f64_type().into()], false);
                module.add_function("print_float", print_float_type, None)
            }
        };

        // Register the print_float function
        self.functions.insert("print_float".to_string(), print_float_fn);

        // Register the print_bool function
        let print_bool_fn = match module.get_function("print_bool") {
            Some(f) => f,
            None => {
                // Create the print_bool function if it doesn't exist
                let print_bool_type = context.void_type().fn_type(&[context.bool_type().into()], false);
                module.add_function("print_bool", print_bool_type, None)
            }
        };

        // Register the print_bool function
        self.functions.insert("print_bool".to_string(), print_bool_fn);

        // Register the generic print function (we'll use print_string as the default)
        self.functions.insert("print".to_string(), function);
    }

    /// Compile a call to the print() function
    pub fn compile_print_call(&mut self, args: &[Expr]) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Check that we have at least one argument
        if args.is_empty() {
            // Print a single newline if no arguments
            let newline_str = self.llvm_context.const_string(b"\n", false);
            let global_str = self.module.add_global(newline_str.get_type(), None, "newline_str");
            global_str.set_initializer(&newline_str);

            let str_ptr = self.builder.build_pointer_cast(
                global_str.as_pointer_value(),
                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                "str_ptr"
            ).unwrap();

            // Use print_string instead of println_string to avoid double newlines
            let print_fn = self.module.get_function("print_string").unwrap();
            self.builder.build_call(print_fn, &[str_ptr.into()], "print_call").unwrap();

            // Return void
            return Ok((self.llvm_context.i64_type().const_int(0, false).into(), Type::None));
        }

        // For each argument, compile it and call the appropriate print function
        for (i, arg) in args.iter().enumerate() {
            // Compile the argument
            let (arg_val, arg_type) = self.compile_expr(arg)?;

            // Determine which print function to use based on the argument type
            match arg_type {
                Type::String => {
                    // Always use print_string to avoid extra newlines
                    let print_fn = self.module.get_function("print_string").unwrap();

                    // Make sure the argument is a pointer
                    let arg_ptr = if arg_val.is_pointer_value() {
                        arg_val.into_pointer_value()
                    } else {
                        // If it's not a pointer, allocate memory and store the value
                        let ptr = self.builder.build_alloca(arg_val.get_type(), "string_arg").unwrap();
                        self.builder.build_store(ptr, arg_val).unwrap();
                        ptr
                    };

                    // Call the print function
                    self.builder.build_call(print_fn, &[arg_ptr.into()], "print_call").unwrap();

                    // If this is the last argument, print a newline
                    if i == args.len() - 1 {
                        let newline_str = self.llvm_context.const_string(b"\n", true);  // true for null termination
                        let global_str = self.module.add_global(newline_str.get_type(), None, "newline_str");
                        global_str.set_initializer(&newline_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        self.builder.build_call(print_fn, &[str_ptr.into()], "print_newline").unwrap();
                    } else {
                        // If not the last argument, print a single space character (no newlines)
                        // Create a clean space string with no extra characters
                        let space_str = self.llvm_context.const_string(b" ", true);  // true for null termination
                        let global_str = self.module.add_global(space_str.get_type(), None, "space_str");
                        global_str.set_initializer(&space_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        self.builder.build_call(print_fn, &[str_ptr.into()], "print_space").unwrap();
                    }
                },
                Type::Int => {
                    // Get the print_int function
                    let print_fn = self.module.get_function("print_int").unwrap();

                    // Call the print function
                    self.builder.build_call(print_fn, &[arg_val.into()], "print_call").unwrap();

                    // If this is not the last argument, print a space
                    if i < args.len() - 1 {
                        let space_str = self.llvm_context.const_string(b" ", false);
                        let global_str = self.module.add_global(space_str.get_type(), None, "space_str");
                        global_str.set_initializer(&space_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder.build_call(print_str_fn, &[str_ptr.into()], "print_space").unwrap();
                    } else {
                        // If this is the last argument, print a newline
                        // Use a clean newline string without any extra whitespace
                        let newline_str = self.llvm_context.const_string(b"\n", true);  // true for null termination
                        let global_str = self.module.add_global(newline_str.get_type(), None, "newline_str");
                        global_str.set_initializer(&newline_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder.build_call(print_str_fn, &[str_ptr.into()], "print_newline").unwrap();
                    }
                },
                Type::Float => {
                    // Get the print_float function
                    let print_fn = self.module.get_function("print_float").unwrap();

                    // Call the print function
                    self.builder.build_call(print_fn, &[arg_val.into()], "print_call").unwrap();

                    // If this is not the last argument, print a space
                    if i < args.len() - 1 {
                        let space_str = self.llvm_context.const_string(b" ", false);
                        let global_str = self.module.add_global(space_str.get_type(), None, "space_str");
                        global_str.set_initializer(&space_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder.build_call(print_str_fn, &[str_ptr.into()], "print_space").unwrap();
                    } else {
                        // If this is the last argument, print a newline
                        let newline_str = self.llvm_context.const_string(b"\n", false);
                        let global_str = self.module.add_global(newline_str.get_type(), None, "newline_str");
                        global_str.set_initializer(&newline_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder.build_call(print_str_fn, &[str_ptr.into()], "print_newline").unwrap();
                    }
                },
                Type::Bool => {
                    // Get the print_bool function
                    let print_fn = self.module.get_function("print_bool").unwrap();

                    // Convert to bool if needed
                    let bool_val = if arg_val.is_int_value() {
                        let int_val = arg_val.into_int_value();
                        self.builder.build_int_compare(
                            inkwell::IntPredicate::NE,
                            int_val,
                            self.llvm_context.i64_type().const_int(0, false),
                            "bool_val"
                        ).unwrap()
                    } else {
                        // If it's not an int, we need to convert it
                        return Err(format!("Cannot convert {:?} to bool", arg_type));
                    };

                    // Call the print function
                    self.builder.build_call(print_fn, &[bool_val.into()], "print_call").unwrap();

                    // If this is not the last argument, print a space
                    if i < args.len() - 1 {
                        let space_str = self.llvm_context.const_string(b" ", false);
                        let global_str = self.module.add_global(space_str.get_type(), None, "space_str");
                        global_str.set_initializer(&space_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder.build_call(print_str_fn, &[str_ptr.into()], "print_space").unwrap();
                    } else {
                        // If this is the last argument, print a newline
                        let newline_str = self.llvm_context.const_string(b"\n", false);
                        let global_str = self.module.add_global(newline_str.get_type(), None, "newline_str");
                        global_str.set_initializer(&newline_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder.build_call(print_str_fn, &[str_ptr.into()], "print_newline").unwrap();
                    }
                },
                _ => {
                    // For other types, convert to string first
                    // This is a simplified implementation - in a real compiler, you'd handle all types

                    // For now, just print a placeholder
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

                    // Always use print_string to avoid extra newlines
                    let print_fn = self.module.get_function("print_string").unwrap();
                    self.builder.build_call(print_fn, &[str_ptr.into()], "print_call").unwrap();

                    // If this is the last argument, print a newline
                    if i == args.len() - 1 {
                        let newline_str = self.llvm_context.const_string(b"\n", false);
                        let global_str = self.module.add_global(newline_str.get_type(), None, "newline_str");
                        global_str.set_initializer(&newline_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        self.builder.build_call(print_fn, &[str_ptr.into()], "print_newline").unwrap();
                    } else {
                        // If not the last argument, print a space
                        let space_str = self.llvm_context.const_string(b" ", false);
                        let global_str = self.module.add_global(space_str.get_type(), None, "space_str");
                        global_str.set_initializer(&space_str);

                        let str_ptr = self.builder.build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr"
                        ).unwrap();

                        self.builder.build_call(print_fn, &[str_ptr.into()], "print_space").unwrap();
                    }
                }
            }
        }

        // Return void
        Ok((self.llvm_context.i64_type().const_int(0, false).into(), Type::None))
    }
}
