// print.rs - Registration and compilation of the print() built-in

use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::ExprCompiler;
use crate::compiler::types::Type;
use inkwell::AddressSpace;
use inkwell::values::{BasicValueEnum, PointerValue};

impl<'ctx> CompilationContext<'ctx> {
    /// Register print_string, print_int, print_float, print_bool, println_string, and bind `print`
    pub fn register_print_function(&mut self) {
        let ctx = self.llvm_context;
        let m = &mut self.module;

        // print_string
        if m.get_function("print_string").is_none() {
            let t = ctx.void_type().fn_type(&[ctx.ptr_type(AddressSpace::default()).into()], false);
            m.add_function("print_string", t, None);
        }
        // print_int
        if m.get_function("print_int").is_none() {
            let t = ctx.void_type().fn_type(&[ctx.i64_type().into()], false);
            m.add_function("print_int", t, None);
        }
        // print_float
        if m.get_function("print_float").is_none() {
            let t = ctx.void_type().fn_type(&[ctx.f64_type().into()], false);
            m.add_function("print_float", t, None);
        }
        // print_bool
        if m.get_function("print_bool").is_none() {
            let t = ctx.void_type().fn_type(&[ctx.bool_type().into()], false);
            m.add_function("print_bool", t, None);
        }
        // println_string
        if m.get_function("println_string").is_none() {
            let t = ctx.void_type().fn_type(&[ctx.ptr_type(AddressSpace::default()).into()], false);
            m.add_function("println_string", t, None);
        }

        // print_boxed_any
        if m.get_function("print_boxed_any").is_none() {
            let t = ctx.void_type().fn_type(&[ctx.ptr_type(AddressSpace::default()).into()], false);
            m.add_function("print_boxed_any", t, None);
        }

        // println_boxed_any
        if m.get_function("println_boxed_any").is_none() {
            let t = ctx.void_type().fn_type(&[ctx.ptr_type(AddressSpace::default()).into()], false);
            m.add_function("println_boxed_any", t, None);
        }

        // Bind the high‑level `print` function based on whether we're using BoxedAny values
        if self.use_boxed_values {
            if let Some(f) = m.get_function("print_boxed_any") {
                self.functions.insert("print".to_string(), f);
            } else if let Some(f) = m.get_function("print_string") {
                self.functions.insert("print".to_string(), f);
            }
        } else {
            if let Some(f) = m.get_function("print_string") {
                self.functions.insert("print".to_string(), f);
            }
        }
    }

    /// Helper to create a global C string and return its pointer
    fn make_cstr(&mut self, name: &str, bytes: &[u8]) -> PointerValue<'ctx> {
        let const_str = self.llvm_context.const_string(bytes, false);
        let global = self.module.add_global(const_str.get_type(), None, name);
        global.set_initializer(&const_str);
        self.builder.build_pointer_cast(
            global.as_pointer_value(),
            self.llvm_context.ptr_type(AddressSpace::default()),
            &format!("{}_ptr", name),
        ).unwrap()
    }

    /// Compile a call to the print() function
    pub fn compile_print_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Debug output to help diagnose the issue
        println!("In compile_print_call with {} arguments", args.len());
        println!("use_boxed_values = {}", self.use_boxed_values);

        // Get the necessary print functions
        let print_str_fn = match self.module.get_function("print_string") {
            Some(f) => {
                println!("Found print_string function");
                f
            },
            None => {
                println!("print_string function not found, registering it");
                let ctx = self.llvm_context;
                let t = ctx.void_type().fn_type(&[ctx.ptr_type(AddressSpace::default()).into()], false);
                self.module.add_function("print_string", t, None)
            }
        };

        let print_int_fn = match self.module.get_function("print_int") {
            Some(f) => {
                println!("Found print_int function");
                f
            },
            None => {
                println!("print_int function not found, registering it");
                let ctx = self.llvm_context;
                let t = ctx.void_type().fn_type(&[ctx.i64_type().into()], false);
                self.module.add_function("print_int", t, None)
            }
        };

        let print_flt_fn = match self.module.get_function("print_float") {
            Some(f) => {
                println!("Found print_float function");
                f
            },
            None => {
                println!("print_float function not found, registering it");
                let ctx = self.llvm_context;
                let t = ctx.void_type().fn_type(&[ctx.f64_type().into()], false);
                self.module.add_function("print_float", t, None)
            }
        };

        let print_bool_fn = match self.module.get_function("print_bool") {
            Some(f) => {
                println!("Found print_bool function");
                f
            },
            None => {
                println!("print_bool function not found, registering it");
                let ctx = self.llvm_context;
                let t = ctx.void_type().fn_type(&[ctx.bool_type().into()], false);
                self.module.add_function("print_bool", t, None)
            }
        };

        let println_fn = match self.module.get_function("println_string") {
            Some(f) => {
                println!("Found println_string function");
                f
            },
            None => {
                println!("println_string function not found, registering it");
                let ctx = self.llvm_context;
                let t = ctx.void_type().fn_type(&[ctx.ptr_type(AddressSpace::default()).into()], false);
                self.module.add_function("println_string", t, None)
            }
        };

        // Get BoxedAny print functions if we're using BoxedAny values
        let print_boxed_any_fn = if self.use_boxed_values {
            match self.module.get_function("print_boxed_any") {
                Some(f) => {
                    println!("Found print_boxed_any function");
                    Some(f)
                },
                None => {
                    println!("print_boxed_any function not found, registering it");
                    let ctx = self.llvm_context;
                    let t = ctx.void_type().fn_type(&[ctx.ptr_type(AddressSpace::default()).into()], false);
                    Some(self.module.add_function("print_boxed_any", t, None))
                }
            }
        } else {
            println!("Not using BoxedAny values, skipping print_boxed_any function");
            None
        };

        // If we have no arguments, just print a newline
        if args.is_empty() {
            let nl = self.make_cstr("nl", b"\n\0");
            let _ = self.builder.build_call(println_fn, &[nl.into()], "print_nl");
            return Ok((self.llvm_context.i64_type().const_zero().into(), Type::None));
        }

        for (i, arg) in args.iter().enumerate() {
            let (val, ty) = self.compile_expr(arg)?;

            if self.use_boxed_values && print_boxed_any_fn.is_some() {
                // If we're using BoxedAny values, convert the value to a BoxedAny and use print_boxed_any
                let boxed_val = match ty {
                    Type::String => {
                        // Convert string to BoxedAny
                        let boxed_any_from_string_fn = self.module.get_function("boxed_any_from_string")
                            .ok_or("boxed_any_from_string function not found".to_string())?;

                        let ptr = val.into_pointer_value();
                        let call = self.builder.build_call(
                            boxed_any_from_string_fn,
                            &[ptr.into()],
                            "string_to_boxed_any"
                        ).unwrap();

                        call.try_as_basic_value().left()
                            .ok_or("Failed to convert string to BoxedAny".to_string())?
                    },
                    Type::Int => {
                        // Convert int to BoxedAny
                        let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                            .ok_or("boxed_any_from_int function not found".to_string())?;

                        let call = self.builder.build_call(
                            boxed_any_from_int_fn,
                            &[val.into()],
                            "int_to_boxed_any"
                        ).unwrap();

                        call.try_as_basic_value().left()
                            .ok_or("Failed to convert int to BoxedAny".to_string())?
                    },
                    Type::Float => {
                        // Convert float to BoxedAny
                        let boxed_any_from_float_fn = self.module.get_function("boxed_any_from_float")
                            .ok_or("boxed_any_from_float function not found".to_string())?;

                        let call = self.builder.build_call(
                            boxed_any_from_float_fn,
                            &[val.into()],
                            "float_to_boxed_any"
                        ).unwrap();

                        call.try_as_basic_value().left()
                            .ok_or("Failed to convert float to BoxedAny".to_string())?
                    },
                    Type::Bool => {
                        // Convert bool to BoxedAny
                        let boxed_any_from_bool_fn = self.module.get_function("boxed_any_from_bool")
                            .ok_or("boxed_any_from_bool function not found".to_string())?;

                        let call = self.builder.build_call(
                            boxed_any_from_bool_fn,
                            &[val.into()],
                            "bool_to_boxed_any"
                        ).unwrap();

                        call.try_as_basic_value().left()
                            .ok_or("Failed to convert bool to BoxedAny".to_string())?
                    },
                    Type::Any => {
                        // Already a BoxedAny, use it directly
                        val
                    },
                    _ => {
                        // For other types, create a placeholder string and convert to BoxedAny
                        let placeholder = format!("<{:?}>\0", ty);
                        let ptr = self.make_cstr("ph", placeholder.as_bytes());

                        let boxed_any_from_string_fn = self.module.get_function("boxed_any_from_string")
                            .ok_or("boxed_any_from_string function not found".to_string())?;

                        let call = self.builder.build_call(
                            boxed_any_from_string_fn,
                            &[ptr.into()],
                            "placeholder_to_boxed_any"
                        ).unwrap();

                        call.try_as_basic_value().left()
                            .ok_or("Failed to convert placeholder to BoxedAny".to_string())?
                    }
                };

                // Call print_boxed_any with the boxed value
                let _ = self.builder.build_call(
                    print_boxed_any_fn.unwrap(),
                    &[boxed_val.into()],
                    "print_boxed_any_call"
                );
            } else {
                // If we're not using BoxedAny values, use the appropriate print function based on type
                match ty {
                    Type::String => {
                        let ptr = val.into_pointer_value();
                        let _ = self.builder.build_call(print_str_fn, &[ptr.into()], "print_str");
                    }
                    Type::Int => {
                        let _ = self.builder.build_call(print_int_fn, &[val.into()], "print_int");
                    }
                    Type::Float => {
                        let _ = self.builder.build_call(print_flt_fn, &[val.into()], "print_flt");
                    }
                    Type::Bool => {
                        let _ = self.builder.build_call(print_bool_fn, &[val.into()], "print_bool");
                    }
                    other => {
                        // Fallback: print a placeholder for unsupported types
                        let placeholder = format!("<{:?}>\0", other);
                        let ptr = self.make_cstr("ph", placeholder.as_bytes());
                        let _ = self.builder.build_call(print_str_fn, &[ptr.into()], "print_ph");
                    }
                }
            }

            // Add space between arguments
            if i + 1 < args.len() {
                if self.use_boxed_values && print_boxed_any_fn.is_some() {
                    // Create a space string as BoxedAny and print it
                    let space_str = self.make_cstr("sp", b" \0");
                    let boxed_any_from_string_fn = self.module.get_function("boxed_any_from_string")
                        .ok_or("boxed_any_from_string function not found".to_string())?;

                    let call = self.builder.build_call(
                        boxed_any_from_string_fn,
                        &[space_str.into()],
                        "space_to_boxed_any"
                    ).unwrap();

                    let boxed_space = call.try_as_basic_value().left()
                        .ok_or("Failed to convert space to BoxedAny".to_string())?;

                    let _ = self.builder.build_call(
                        print_boxed_any_fn.unwrap(),
                        &[boxed_space.into()],
                        "print_space"
                    );
                } else {
                    // Use the regular print_string function
                    let ptr = self.make_cstr("sp", b" \0");
                    let _ = self.builder.build_call(print_str_fn, &[ptr.into()], "print_sp");
                }
            }
        }

        // Add final newline
        if self.use_boxed_values && print_boxed_any_fn.is_some() {
            // Create a newline string as BoxedAny and print it
            let nl = self.make_cstr("nl", b"\n\0");
            let boxed_any_from_string_fn = self.module.get_function("boxed_any_from_string")
                .ok_or("boxed_any_from_string function not found".to_string())?;

            let call = self.builder.build_call(
                boxed_any_from_string_fn,
                &[nl.into()],
                "newline_to_boxed_any"
            ).unwrap();

            let boxed_nl = call.try_as_basic_value().left()
                .ok_or("Failed to convert newline to BoxedAny".to_string())?;

            let _ = self.builder.build_call(
                print_boxed_any_fn.unwrap(),
                &[boxed_nl.into()],
                "print_newline"
            );
        } else {
            // Use the regular println_string function
            let nl = self.make_cstr("nl", b"\n\0");
            let _ = self.builder.build_call(println_fn, &[nl.into()], "print_nl");
        }

        Ok((self.llvm_context.i64_type().const_zero().into(), Type::None))
    }
}
