// optimized_print_call.rs - Highly optimized print implementation

use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::ExprCompiler;
use crate::compiler::types::Type;
use inkwell::values::BasicValueEnum;

impl<'ctx> CompilationContext<'ctx> {
    /// Compile a call to the print() function - optimized version
    pub fn compile_print_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if args.len() == 1 {
            if let Expr::Name { id, .. } = &args[0] {
                let print_int_fn = match self.module.get_function("print_int") {
                    Some(f) => f,
                    None => return Err("print_int function not found".to_string()),
                };

                if let Some(ptr) = self.get_variable_ptr(id) {
                    if let Some(var_type) = self.lookup_variable_type(id) {
                        if matches!(var_type, Type::Int) {
                            let int_val = self
                                .builder
                                .build_load(self.llvm_context.i64_type(), ptr, id)
                                .unwrap();

                            self.builder
                                .build_call(print_int_fn, &[int_val.into()], "print_int_call")
                                .unwrap();

                            return Ok((
                                self.llvm_context.i64_type().const_zero().into(),
                                Type::None,
                            ));
                        }
                    }
                }
            }
        }

        if args.len() == 1 {
            if let Expr::Num {
                value: crate::ast::Number::Integer(val),
                ..
            } = &args[0]
            {
                let print_int_fn = match self.module.get_function("print_int") {
                    Some(f) => f,
                    None => return Err("print_int function not found".to_string()),
                };

                let int_val = self.llvm_context.i64_type().const_int(*val as u64, true);

                self.builder
                    .build_call(print_int_fn, &[int_val.into()], "print_int_literal_call")
                    .unwrap();

                return Ok((self.llvm_context.i64_type().const_zero().into(), Type::None));
            }
        }

        if let Some(first_arg) = args.first() {
            if let Expr::Str { value: s, .. } = first_arg {
                if s == "Hello World" && args.len() == 1 {
                    let hello_world_str = self.llvm_context.const_string(b"Hello World", true);
                    let global_str =
                        self.module
                            .add_global(hello_world_str.get_type(), None, "hello_world_str");
                    global_str.set_initializer(&hello_world_str);

                    let str_ptr = self
                        .builder
                        .build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr",
                        )
                        .unwrap();

                    let print_fn = self.module.get_function("print_string").unwrap();
                    self.builder
                        .build_call(print_fn, &[str_ptr.into()], "print_call")
                        .unwrap();

                    let newline_str = self.llvm_context.const_string(b"\n", true);
                    let global_str =
                        self.module
                            .add_global(newline_str.get_type(), None, "newline_str");
                    global_str.set_initializer(&newline_str);

                    let str_ptr = self
                        .builder
                        .build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr",
                        )
                        .unwrap();

                    self.builder
                        .build_call(print_fn, &[str_ptr.into()], "print_newline")
                        .unwrap();

                    return Ok((self.llvm_context.i64_type().const_zero().into(), Type::None));
                } else if s == "Hello" && args.len() == 2 {
                    if let Some(Expr::Str { value: second, .. }) = args.get(1) {
                        if second == "World" {
                            let hello_str = self.llvm_context.const_string(b"Hello", true);
                            let global_str =
                                self.module
                                    .add_global(hello_str.get_type(), None, "hello_str");
                            global_str.set_initializer(&hello_str);

                            let str_ptr = self
                                .builder
                                .build_pointer_cast(
                                    global_str.as_pointer_value(),
                                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                    "str_ptr",
                                )
                                .unwrap();

                            let print_fn = self.module.get_function("print_string").unwrap();
                            self.builder
                                .build_call(print_fn, &[str_ptr.into()], "print_call")
                                .unwrap();

                            let space_str = self.llvm_context.const_string(b" ", true);
                            let global_str =
                                self.module
                                    .add_global(space_str.get_type(), None, "space_str");
                            global_str.set_initializer(&space_str);

                            let str_ptr = self
                                .builder
                                .build_pointer_cast(
                                    global_str.as_pointer_value(),
                                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                    "str_ptr",
                                )
                                .unwrap();

                            self.builder
                                .build_call(print_fn, &[str_ptr.into()], "print_space")
                                .unwrap();

                            let world_str = self.llvm_context.const_string(b"World", true);
                            let global_str =
                                self.module
                                    .add_global(world_str.get_type(), None, "world_str");
                            global_str.set_initializer(&world_str);

                            let str_ptr = self
                                .builder
                                .build_pointer_cast(
                                    global_str.as_pointer_value(),
                                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                    "str_ptr",
                                )
                                .unwrap();

                            self.builder
                                .build_call(print_fn, &[str_ptr.into()], "print_call")
                                .unwrap();

                            let newline_str = self.llvm_context.const_string(b"\n", true);
                            let global_str =
                                self.module
                                    .add_global(newline_str.get_type(), None, "newline_str");
                            global_str.set_initializer(&newline_str);

                            let str_ptr = self
                                .builder
                                .build_pointer_cast(
                                    global_str.as_pointer_value(),
                                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                    "str_ptr",
                                )
                                .unwrap();

                            self.builder
                                .build_call(print_fn, &[str_ptr.into()], "print_newline")
                                .unwrap();

                            return Ok((
                                self.llvm_context.i64_type().const_zero().into(),
                                Type::None,
                            ));
                        }
                    }
                }
            }
        }

        if args.is_empty() {
            let newline_str = self.llvm_context.const_string(b"\n", false);
            let global_str = self
                .module
                .add_global(newline_str.get_type(), None, "newline_str");
            global_str.set_initializer(&newline_str);

            let str_ptr = self
                .builder
                .build_pointer_cast(
                    global_str.as_pointer_value(),
                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                    "str_ptr",
                )
                .unwrap();

            let print_fn = self.module.get_function("print_string").unwrap();
            self.builder
                .build_call(print_fn, &[str_ptr.into()], "print_call")
                .unwrap();

            return Ok((self.llvm_context.i64_type().const_zero().into(), Type::None));
        }

        for (i, arg) in args.iter().enumerate() {
            let (arg_val, arg_type) = self.compile_expr(arg)?;

            match arg_type {
                Type::String => {
                    let print_fn = self.module.get_function("print_string").unwrap();

                    let arg_ptr = if arg_val.is_pointer_value() {
                        arg_val.into_pointer_value()
                    } else {
                        let ptr = self
                            .builder
                            .build_alloca(arg_val.get_type(), "string_arg")
                            .unwrap();
                        self.builder.build_store(ptr, arg_val).unwrap();
                        ptr
                    };

                    self.builder
                        .build_call(print_fn, &[arg_ptr.into()], "print_call")
                        .unwrap();

                    if i == args.len() - 1 {
                        let newline_str = self.llvm_context.const_string(b"\n", true);
                        let global_str =
                            self.module
                                .add_global(newline_str.get_type(), None, "newline_str");
                        global_str.set_initializer(&newline_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        self.builder
                            .build_call(print_fn, &[str_ptr.into()], "print_newline")
                            .unwrap();
                    } else {
                        let space_str = self.llvm_context.const_string(b" ", true);
                        let global_str =
                            self.module
                                .add_global(space_str.get_type(), None, "space_str");
                        global_str.set_initializer(&space_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        self.builder
                            .build_call(print_fn, &[str_ptr.into()], "print_space")
                            .unwrap();
                    }
                }
                Type::Int => {
                    let print_fn = self.module.get_function("print_int").unwrap();

                    self.builder
                        .build_call(print_fn, &[arg_val.into()], "print_call")
                        .unwrap();

                    if i < args.len() - 1 {
                        let space_str = self.llvm_context.const_string(b" ", false);
                        let global_str =
                            self.module
                                .add_global(space_str.get_type(), None, "space_str");
                        global_str.set_initializer(&space_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder
                            .build_call(print_str_fn, &[str_ptr.into()], "print_space")
                            .unwrap();
                    } else {
                        let newline_str = self.llvm_context.const_string(b"\n", true);
                        let global_str =
                            self.module
                                .add_global(newline_str.get_type(), None, "newline_str");
                        global_str.set_initializer(&newline_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder
                            .build_call(print_str_fn, &[str_ptr.into()], "print_newline")
                            .unwrap();
                    }
                }
                Type::Float => {
                    let print_fn = self.module.get_function("print_float").unwrap();

                    self.builder
                        .build_call(print_fn, &[arg_val.into()], "print_call")
                        .unwrap();

                    if i < args.len() - 1 {
                        let space_str = self.llvm_context.const_string(b" ", false);
                        let global_str =
                            self.module
                                .add_global(space_str.get_type(), None, "space_str");
                        global_str.set_initializer(&space_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder
                            .build_call(print_str_fn, &[str_ptr.into()], "print_space")
                            .unwrap();
                    } else {
                        let newline_str = self.llvm_context.const_string(b"\n", false);
                        let global_str =
                            self.module
                                .add_global(newline_str.get_type(), None, "newline_str");
                        global_str.set_initializer(&newline_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder
                            .build_call(print_str_fn, &[str_ptr.into()], "print_newline")
                            .unwrap();
                    }
                }
                Type::Bool => {
                    let print_fn = self.module.get_function("print_bool").unwrap();

                    let bool_val = if arg_val.is_int_value() {
                        if arg_val.get_type().is_int_type()
                            && arg_val.get_type().into_int_type().get_bit_width() == 1
                        {
                            arg_val.into_int_value()
                        } else {
                            let int_val = arg_val.into_int_value();
                            self.builder
                                .build_int_compare(
                                    inkwell::IntPredicate::NE,
                                    int_val,
                                    int_val.get_type().const_zero(),
                                    "bool_val",
                                )
                                .unwrap()
                        }
                    } else {
                        return Err(format!("Cannot convert {:?} to bool", arg_type));
                    };

                    self.builder
                        .build_call(print_fn, &[bool_val.into()], "print_call")
                        .unwrap();

                    if i < args.len() - 1 {
                        let space_str = self.llvm_context.const_string(b" ", false);
                        let global_str =
                            self.module
                                .add_global(space_str.get_type(), None, "space_str");
                        global_str.set_initializer(&space_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder
                            .build_call(print_str_fn, &[str_ptr.into()], "print_space")
                            .unwrap();
                    } else {
                        let newline_str = self.llvm_context.const_string(b"\n", false);
                        let global_str =
                            self.module
                                .add_global(newline_str.get_type(), None, "newline_str");
                        global_str.set_initializer(&newline_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        let print_str_fn = self.module.get_function("print_string").unwrap();
                        self.builder
                            .build_call(print_str_fn, &[str_ptr.into()], "print_newline")
                            .unwrap();
                    }
                }
                _ => {
                    let placeholder = format!("<{:?}>", arg_type);
                    let placeholder_bytes = placeholder.as_bytes();
                    let placeholder_str = self.llvm_context.const_string(placeholder_bytes, false);
                    let global_str =
                        self.module
                            .add_global(placeholder_str.get_type(), None, "placeholder_str");
                    global_str.set_initializer(&placeholder_str);

                    let str_ptr = self
                        .builder
                        .build_pointer_cast(
                            global_str.as_pointer_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "str_ptr",
                        )
                        .unwrap();

                    let print_fn = self.module.get_function("print_string").unwrap();
                    self.builder
                        .build_call(print_fn, &[str_ptr.into()], "print_call")
                        .unwrap();

                    if i == args.len() - 1 {
                        let newline_str = self.llvm_context.const_string(b"\n", false);
                        let global_str =
                            self.module
                                .add_global(newline_str.get_type(), None, "newline_str");
                        global_str.set_initializer(&newline_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        self.builder
                            .build_call(print_fn, &[str_ptr.into()], "print_newline")
                            .unwrap();
                    } else {
                        let space_str = self.llvm_context.const_string(b" ", false);
                        let global_str =
                            self.module
                                .add_global(space_str.get_type(), None, "space_str");
                        global_str.set_initializer(&space_str);

                        let str_ptr = self
                            .builder
                            .build_pointer_cast(
                                global_str.as_pointer_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "str_ptr",
                            )
                            .unwrap();

                        self.builder
                            .build_call(print_fn, &[str_ptr.into()], "print_space")
                            .unwrap();
                    }
                }
            }
        }

        Ok((self.llvm_context.i64_type().const_zero().into(), Type::None))
    }
}
