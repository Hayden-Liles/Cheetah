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

        // print_list
        if m.get_function("print_list").is_none() {
            let t = ctx.void_type().fn_type(&[ctx.ptr_type(AddressSpace::default()).into()], false);
            m.add_function("print_list", t, None);
        }

        // print_dict
        if m.get_function("print_dict").is_none() {
            let t = ctx.void_type().fn_type(&[ctx.ptr_type(AddressSpace::default()).into()], false);
            m.add_function("print_dict", t, None);
        }

        // print_any
        if m.get_function("print_any").is_none() {
            let t = ctx.void_type().fn_type(&[ctx.ptr_type(AddressSpace::default()).into()], false);
            m.add_function("print_any", t, None);
        }

        // Bind the high‑level `print` to print_string (for overloading simplicity)
        if let Some(f) = m.get_function("print_string") {
            self.functions.insert("print".to_string(), f);
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

    /// Helper function to print a list value
    fn print_list_value(
        &mut self,
        list_ptr: PointerValue<'ctx>,
        element_type: &Type,
        print_str_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String> {
        // Get list_len function
        let list_len_fn = self.module.get_function("list_len")
            .ok_or("list_len function not found".to_string())?;

        // Get list_get function
        let list_get_fn = self.module.get_function("list_get")
            .ok_or("list_get function not found".to_string())?;

        // Print opening bracket
        let open_bracket = self.make_cstr("open_bracket", b"[\0");
        let _ = self.builder.build_call(print_str_fn, &[open_bracket.into()], "print_open_bracket");

        // Get list length
        let len_call = self.builder.build_call(
            list_len_fn,
            &[list_ptr.into()],
            "list_len_call"
        ).unwrap();

        let list_len = len_call.try_as_basic_value().left()
            .ok_or("Failed to get list length".to_string())?;

        let list_len_int = list_len.into_int_value();

        // Create basic blocks for the loop
        let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let loop_cond_block = self.llvm_context.append_basic_block(current_function, "list_print_cond");
        let loop_body_block = self.llvm_context.append_basic_block(current_function, "list_print_body");
        let loop_exit_block = self.llvm_context.append_basic_block(current_function, "list_print_exit");

        // Create index variable
        let index_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), "list_print_index").unwrap();
        self.builder.build_store(index_ptr, self.llvm_context.i64_type().const_zero()).unwrap();

        // Jump to loop condition
        self.builder.build_unconditional_branch(loop_cond_block).unwrap();

        // Loop condition block
        self.builder.position_at_end(loop_cond_block);
        let index = self.builder.build_load(self.llvm_context.i64_type(), index_ptr, "list_index").unwrap();
        let index_int = index.into_int_value();
        let cond = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            index_int,
            list_len_int,
            "list_index_cond"
        ).unwrap();
        self.builder.build_conditional_branch(cond, loop_body_block, loop_exit_block).unwrap();

        // Loop body block
        self.builder.position_at_end(loop_body_block);

        // Print comma if not the first element
        let is_first = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            index_int,
            self.llvm_context.i64_type().const_zero(),
            "is_first_element"
        ).unwrap();

        let comma_block = self.llvm_context.append_basic_block(current_function, "print_comma");
        let element_block = self.llvm_context.append_basic_block(current_function, "print_element");

        self.builder.build_conditional_branch(is_first, element_block, comma_block).unwrap();

        // Print comma block
        self.builder.position_at_end(comma_block);
        let comma_space = self.make_cstr("comma_space", b", \0");
        let _ = self.builder.build_call(print_str_fn, &[comma_space.into()], "print_comma_space");
        self.builder.build_unconditional_branch(element_block).unwrap();

        // Print element block
        self.builder.position_at_end(element_block);

        // Get list element
        let get_call = self.builder.build_call(
            list_get_fn,
            &[list_ptr.into(), index_int.into()],
            "list_get_call"
        ).unwrap();

        // --- real element printing based on element_type ---
        let elem_val = get_call.try_as_basic_value().left()
            .ok_or("Failed to get list element".to_string())?;
        match element_type {
            Type::Int => {
                // maybe boxed or direct integer
                let int_val = if elem_val.is_pointer_value() {
                    let ptr = elem_val.into_pointer_value();
                    self.builder.build_load(self.llvm_context.i64_type(), ptr, "load_int_elem")
                        .unwrap().into_int_value()
                } else {
                    elem_val.into_int_value()
                };
                let print_int_fn = self.module.get_function("print_int")
                    .ok_or("print_int not found".to_string())?;
                let _ = self.builder.build_call(print_int_fn, &[int_val.into()], "print_int_elem");
            }
            Type::Float => {
                let float_val = if elem_val.is_pointer_value() {
                    let ptr = elem_val.into_pointer_value();
                    self.builder.build_load(self.llvm_context.f64_type(), ptr, "load_float_elem")
                        .unwrap().into_float_value()
                } else {
                    elem_val.into_float_value()
                };
                let print_flt_fn = self.module.get_function("print_float")
                    .ok_or("print_float not found".to_string())?;
                let _ = self.builder.build_call(print_flt_fn, &[float_val.into()], "print_float_elem");
            }
            Type::Bool => {
                let bool_val = if elem_val.is_pointer_value() {
                    let ptr = elem_val.into_pointer_value();
                    self.builder.build_load(self.llvm_context.bool_type(), ptr, "load_bool_elem")
                        .unwrap().into_int_value()
                } else {
                    elem_val.into_int_value() // 0 or 1
                };
                let print_bool_fn = self.module.get_function("print_bool")
                    .ok_or("print_bool not found".to_string())?;
                let _ = self.builder.build_call(print_bool_fn, &[bool_val.into()], "print_bool_elem");
            }
            Type::String => {
                let str_ptr = elem_val.into_pointer_value();
                let _ = self.builder.build_call(print_str_fn, &[str_ptr.into()], "print_str_elem");
            }
            Type::List(_) => {
                let print_list_fn = self.module.get_function("print_list")
                    .ok_or("print_list not found".to_string())?;
                let list_ptr = elem_val.into_pointer_value();
                let _ = self.builder.build_call(print_list_fn, &[list_ptr.into()], "print_list_elem");
            }
            Type::Dict(_, _) => {
                let print_dict_fn = self.module.get_function("print_dict")
                    .ok_or("print_dict not found".to_string())?;
                let dict_ptr = elem_val.into_pointer_value();
                let _ = self.builder.build_call(print_dict_fn, &[dict_ptr.into()], "print_dict_elem");
            }
            Type::None => {
                let none_str = self.make_cstr("none_literal", b"None\0");
                let _ = self.builder.build_call(print_str_fn, &[none_str.into()], "print_none_elem");
            }
            Type::Any | Type::Unknown => {
                // delegate to the generic runtime printer
                let print_any_fn = self.module.get_function("print_any")
                    .ok_or("print_any not found".to_string())?;
                let ptr = elem_val.into_pointer_value();
                let _ = self.builder.build_call(print_any_fn, &[ptr.into()], "print_any_elem");
            }
            _ => {
                // fallback placeholder
                let ph = self.make_cstr("ph", format!("<{:?}>\0", element_type).as_bytes());
                let _ = self.builder.build_call(print_str_fn, &[ph.into()], "print_ph_elem");
            }
        }

        // Increment index
        let next_index = self.builder.build_int_add(
            index_int,
            self.llvm_context.i64_type().const_int(1, false),
            "next_index"
        ).unwrap();
        self.builder.build_store(index_ptr, next_index).unwrap();

        // Jump back to condition
        self.builder.build_unconditional_branch(loop_cond_block).unwrap();

        // Exit block
        self.builder.position_at_end(loop_exit_block);

        // Print closing bracket
        let close_bracket = self.make_cstr("close_bracket", b"]\0");
        let _ = self.builder.build_call(print_str_fn, &[close_bracket.into()], "print_close_bracket");

        Ok(())
    }

    /// Compile a call to the print() function
    pub fn compile_print_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        let print_str_fn   = self.module.get_function("print_string")
            .ok_or("print_string not found".to_string())?;
        let print_int_fn   = self.module.get_function("print_int")
            .ok_or("print_int not found".to_string())?;
        let print_flt_fn   = self.module.get_function("print_float")
            .ok_or("print_float not found".to_string())?;
        let print_bool_fn  = self.module.get_function("print_bool")
            .ok_or("print_bool not found".to_string())?;
        let println_fn     = self.module.get_function("println_string")
            .ok_or("println_string not found".to_string())?;

        for (i, arg) in args.iter().enumerate() {
            let (val, ty) = self.compile_expr(arg)?;
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
                Type::List(inner) => {
                    // inline compile‑time list printing
                    self.print_list_value(
                        val.into_pointer_value(),
                        &*inner,
                        print_str_fn,
                    )?;
                }
                Type::Dict(_,_) => {
                    let print_dict_fn = self.module.get_function("print_dict").unwrap();
                    let _ = self.builder.build_call(print_dict_fn, &[val.into_pointer_value().into()], "print_dict_call");
                }
                Type::None => {
                    let none_str = self.make_cstr("none_literal", b"None\0");
                    let _ = self.builder.build_call(print_str_fn, &[none_str.into()], "print_none");
                }
                other => {
                    // Fallback: print a placeholder for unsupported types
                    let placeholder = format!("<{:?}>\0", other);
                    let ptr = self.make_cstr("ph", placeholder.as_bytes());
                    let _ = self.builder.build_call(print_str_fn, &[ptr.into()], "print_ph");
                }
            }

            // space between args
            if i + 1 < args.len() {
                let ptr = self.make_cstr("sp", b" \0");
                let _ = self.builder.build_call(print_str_fn, &[ptr.into()], "print_sp");
            }
        }

        // final newline
        let nl = self.make_cstr("nl", b"\n\0");
        let _ = self.builder.build_call(println_fn, &[nl.into()], "print_nl");

        Ok((self.llvm_context.i64_type().const_zero().into(), Type::None))
    }
}
