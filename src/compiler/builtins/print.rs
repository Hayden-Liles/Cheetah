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

        // We no longer bind print to a specific function
        // The compile_print_call method will dispatch to the appropriate function based on type
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

    /// Helper function to print a dictionary value
    fn print_dict_value(
        &mut self,
        dict_ptr: PointerValue<'ctx>,
        key_type: &Type,
        value_type: &Type,
        print_str_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String> {
        // 1) Print "{"
        let open_brace = self.make_cstr("open_brace", b"{\0");
        let _ = self.builder.build_call(print_str_fn, &[open_brace.into()], "print_open_brace");

        // 2) Get the keys list
        let dict_keys_fn = self.module.get_function("dict_keys")
            .ok_or("dict_keys not found".to_string())?;
        let keys_call = self.builder.build_call(
            dict_keys_fn,
            &[dict_ptr.into()],
            "dict_keys_call",
        ).unwrap();

        let keys = keys_call
            .try_as_basic_value().left()
            .ok_or("Failed to get dictionary keys".to_string())?
            .into_pointer_value();

        // 3) Get its length
        let list_len_fn = self.module.get_function("list_len")
            .ok_or("list_len function not found".to_string())?;
        let len_call = self.builder.build_call(
            list_len_fn,
            &[keys.into()],
            "keys_len_call",
        ).unwrap();

        let list_len = len_call
            .try_as_basic_value().left()
            .ok_or("Failed to get keys length".to_string())?;

        let list_len_int = list_len.into_int_value();

        // 4) Loop over i in [0..len):
        // Create basic blocks for the loop
        let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let loop_cond_block = self.llvm_context.append_basic_block(current_function, "dict_print_cond");
        let loop_body_block = self.llvm_context.append_basic_block(current_function, "dict_print_body");
        let loop_exit_block = self.llvm_context.append_basic_block(current_function, "dict_print_exit");

        // Create index variable
        let index_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), "dict_print_index").unwrap();
        self.builder.build_store(index_ptr, self.llvm_context.i64_type().const_zero()).unwrap();

        // Jump to loop condition
        self.builder.build_unconditional_branch(loop_cond_block).unwrap();

        // Loop condition block
        self.builder.position_at_end(loop_cond_block);
        let index = self.builder.build_load(self.llvm_context.i64_type(), index_ptr, "dict_index").unwrap();
        let index_int = index.into_int_value();
        let cond = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            index_int,
            list_len_int,
            "dict_index_cond"
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

        // Get list_get function
        let list_get_fn = self.module.get_function("list_get")
            .ok_or("list_get function not found".to_string())?;

        // Get dict_get function
        let dict_get_fn = self.module.get_function("dict_get")
            .ok_or("dict_get function not found".to_string())?;

        // a) fetch key = list_get(keys, i)
        let key_call = self.builder.build_call(
            list_get_fn,
            &[keys.into(), index_int.into()],
            "list_get_key_call"
        ).unwrap();

        let key_val = key_call.try_as_basic_value().left()
            .ok_or("Failed to get dictionary key".to_string())?;

        // b) inline‐print key (match on key_type)
        match key_type {
            Type::Int => {
                // maybe boxed or direct integer
                let int_val = if key_val.is_pointer_value() {
                    let ptr = key_val.into_pointer_value();
                    self.builder.build_load(self.llvm_context.i64_type(), ptr, "load_int_key")
                        .unwrap().into_int_value()
                } else {
                    key_val.into_int_value()
                };
                let print_int_fn = self.module.get_function("print_int")
                    .ok_or("print_int not found".to_string())?;
                let _ = self.builder.build_call(print_int_fn, &[int_val.into()], "print_int_key");
            }
            Type::Float => {
                let float_val = if key_val.is_pointer_value() {
                    let ptr = key_val.into_pointer_value();
                    self.builder.build_load(self.llvm_context.f64_type(), ptr, "load_float_key")
                        .unwrap().into_float_value()
                } else {
                    key_val.into_float_value()
                };
                let print_flt_fn = self.module.get_function("print_float")
                    .ok_or("print_float not found".to_string())?;
                let _ = self.builder.build_call(print_flt_fn, &[float_val.into()], "print_float_key");
            }
            Type::Bool => {
                let bool_val = if key_val.is_pointer_value() {
                    let ptr = key_val.into_pointer_value();
                    self.builder.build_load(self.llvm_context.bool_type(), ptr, "load_bool_key")
                        .unwrap().into_int_value()
                } else {
                    key_val.into_int_value() // 0 or 1
                };
                let print_bool_fn = self.module.get_function("print_bool")
                    .ok_or("print_bool not found".to_string())?;
                let _ = self.builder.build_call(print_bool_fn, &[bool_val.into()], "print_bool_key");
            }
            Type::String => {
                let str_ptr = key_val.into_pointer_value();
                let _ = self.builder.build_call(print_str_fn, &[str_ptr.into()], "print_str_key");
            }
            _ => {
                // fallback placeholder
                let ph = self.make_cstr("ph_key", format!("<{:?}>\0", key_type).as_bytes());
                let _ = self.builder.build_call(print_str_fn, &[ph.into()], "print_ph_key");
            }
        }

        // c) print ": "
        let colon_space = self.make_cstr("colon_space", b": \0");
        let _ = self.builder.build_call(print_str_fn, &[colon_space.into()], "print_colon_space");

        // d) fetch val = dict_get(dict_ptr, key)
        let val_call = self.builder.build_call(
            dict_get_fn,
            &[dict_ptr.into(), key_val.into()],
            "dict_get_val_call"
        ).unwrap();

        let val_ptr = val_call.try_as_basic_value().left()
            .ok_or("Failed to get dictionary value".to_string())?
            .into_pointer_value();

        // e) inline‐print val (match on value_type)
        match value_type {
            Type::Int => {
                // maybe boxed or direct integer
                let int_val = self.builder.build_load(self.llvm_context.i64_type(), val_ptr, "load_int_val")
                    .unwrap().into_int_value();
                let print_int_fn = self.module.get_function("print_int")
                    .ok_or("print_int not found".to_string())?;
                let _ = self.builder.build_call(print_int_fn, &[int_val.into()], "print_int_val");
            }
            Type::Float => {
                let float_val = self.builder.build_load(self.llvm_context.f64_type(), val_ptr, "load_float_val")
                    .unwrap().into_float_value();
                let print_flt_fn = self.module.get_function("print_float")
                    .ok_or("print_float not found".to_string())?;
                let _ = self.builder.build_call(print_flt_fn, &[float_val.into()], "print_float_val");
            }
            Type::Bool => {
                let bool_val = self.builder.build_load(self.llvm_context.bool_type(), val_ptr, "load_bool_val")
                    .unwrap().into_int_value();
                let print_bool_fn = self.module.get_function("print_bool")
                    .ok_or("print_bool not found".to_string())?;
                let _ = self.builder.build_call(print_bool_fn, &[bool_val.into()], "print_bool_val");
            }
            Type::String => {
                let _ = self.builder.build_call(print_str_fn, &[val_ptr.into()], "print_str_val");
            }
            Type::List(inner) => {
                self.print_list_value(val_ptr, &*inner, print_str_fn)?;
            }
            Type::Dict(k, v) => {
                self.print_dict_value(val_ptr, &*k, &*v, print_str_fn)?;
            }
            Type::None => {
                let none_str = self.make_cstr("none_literal_val", b"None\0");
                let _ = self.builder.build_call(print_str_fn, &[none_str.into()], "print_none_val");
            }
            _ => {
                // For Any or other dynamic types, call the generic runtime catch-all
                let print_any_fn = self.module.get_function("print_any")
                    .ok_or("print_any not found".to_string())?;
                let _ = self.builder.build_call(print_any_fn, &[val_ptr.into()], "print_any_val");
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

        // 5) Print "}"
        let close_brace = self.make_cstr("close_brace", b"}\0");
        let _ = self.builder.build_call(print_str_fn, &[close_brace.into()], "print_close_brace");

        Ok(())
    }

    /// Helper function to print a list value
    fn print_list_value(
        &mut self,
        list_ptr: PointerValue<'ctx>,
        element_type: &Type,
        print_str_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String> {
        // Helpers
        let list_len_fn = self.module.get_function("list_len")
            .ok_or("list_len not found".to_string())?;
        let list_get_fn = self.module.get_function("list_get")
            .ok_or("list_get not found".to_string())?;

        // Print "["
        let open = self.make_cstr("open_bracket", b"[\0");
        let _ = self.builder.build_call(print_str_fn, &[open.into()], "print_open_bracket");

        // len = list_len(list_ptr)
        let len_call = self.builder
            .build_call(list_len_fn, &[list_ptr.into()], "list_len_call")
            .unwrap();
        let len_val = len_call.try_as_basic_value().left()
            .ok_or("Failed to get list length".to_string())?
            .into_int_value();

        // Set up loop
        let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let cond_bb = self.llvm_context.append_basic_block(current_fn, "list_print_cond");
        let body_bb = self.llvm_context.append_basic_block(current_fn, "list_print_body");
        let exit_bb = self.llvm_context.append_basic_block(current_fn, "list_print_exit");

        let idx_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), "idx").unwrap();
        self.builder.build_store(idx_ptr, self.llvm_context.i64_type().const_zero()).unwrap();
        self.builder.build_unconditional_branch(cond_bb).unwrap();

        // cond block
        self.builder.position_at_end(cond_bb);
        let idx = self.builder
            .build_load(self.llvm_context.i64_type(), idx_ptr, "idx")
            .unwrap()
            .into_int_value();
        let cmp = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            idx,
            len_val,
            "cmp"
        ).unwrap();
        self.builder.build_conditional_branch(cmp, body_bb, exit_bb).unwrap();

        // body block
        self.builder.position_at_end(body_bb);
        let get_call = self.builder
            .build_call(list_get_fn, &[list_ptr.into(), idx.into()], "get_elem")
            .unwrap();
        let elem_ptr = get_call.try_as_basic_value().left()
            .ok_or("Failed to get list element".to_string())?
            .into_pointer_value();

        // Print element based on static type
        match element_type {
            Type::Int => {
                let int_val = self.builder.build_load(self.llvm_context.i64_type(), elem_ptr, "load_int")
                    .unwrap()
                    .into_int_value();
                let print_int_fn = self.module.get_function("print_int")
                    .ok_or("print_int not found".to_string())?;
                let _ = self.builder.build_call(print_int_fn, &[int_val.into()], "print_int");
            }
            Type::Float => {
                let flt_val = self.builder.build_load(self.llvm_context.f64_type(), elem_ptr, "load_flt")
                    .unwrap()
                    .into_float_value();
                let print_flt_fn = self.module.get_function("print_float")
                    .ok_or("print_float not found".to_string())?;
                let _ = self.builder.build_call(print_flt_fn, &[flt_val.into()], "print_flt");
            }
            Type::Bool => {
                let bool_val = self.builder.build_load(self.llvm_context.bool_type(), elem_ptr, "load_bool")
                    .unwrap()
                    .into_int_value();
                let print_bool_fn = self.module.get_function("print_bool")
                    .ok_or("print_bool not found".to_string())?;
                let _ = self.builder.build_call(print_bool_fn, &[bool_val.into()], "print_bool");
            }
            Type::String => {
                // elem_ptr is already a PointerValue
                let _ = self.builder.build_call(print_str_fn, &[elem_ptr.into()], "print_str");
            }
            Type::List(inner) => {
                self.print_list_value(elem_ptr, &*inner, print_str_fn)?;
            }
            Type::Dict(k, v) => {
                self.print_dict_value(elem_ptr, &*k, &*v, print_str_fn)?;
            }
            _ => {
                // dynamic fallback
                let print_any_fn = self.module.get_function("print_any")
                    .ok_or("print_any not found".to_string())?;
                let _ = self.builder.build_call(print_any_fn, &[elem_ptr.into()], "print_any");
            }
        }

        // comma
        let comma = self.make_cstr("comma", b", \0");
        let _ = self.builder.build_call(print_str_fn, &[comma.into()], "print_comma");

        // idx += 1
        let next_idx = self.builder.build_int_add(
            idx,
            self.llvm_context.i64_type().const_int(1, false),
            "next_idx"
        ).unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(cond_bb).unwrap();

        // exit block: "]"
        self.builder.position_at_end(exit_bb);
        let close = self.make_cstr("close_bracket", b"]\0");
        let _ = self.builder.build_call(print_str_fn, &[close.into()], "print_close_bracket");

        Ok(())
    }

    /// Compile a call to the print() function
    pub fn compile_print_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        let print_str_fn  = self.module.get_function("print_string")
            .ok_or("print_string not found".to_string())?;
        let print_int_fn  = self.module.get_function("print_int")
            .ok_or("print_int not found".to_string())?;
        let print_flt_fn  = self.module.get_function("print_float")
            .ok_or("print_float not found".to_string())?;
        let print_bool_fn = self.module.get_function("print_bool")
            .ok_or("print_bool not found".to_string())?;
        let print_any_fn  = self.module.get_function("print_any")
            .ok_or("print_any not found".to_string())?;
        let println_fn    = self.module.get_function("println_string")
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
                    // inline IR recursion
                    self.print_list_value(val.into_pointer_value(), &*inner, print_str_fn)?;
                }
                Type::Dict(k, v) => {
                    // inline IR recursion
                    self.print_dict_value(val.into_pointer_value(), &*k, &*v, print_str_fn)?;
                }
                Type::None => {
                    let none_str = self.make_cstr("none_literal", b"None\0");
                    let _ = self.builder.build_call(print_str_fn, &[none_str.into()], "print_none");
                }
                other => {
                    // fallback for truly dynamic
                    let placeholder = format!("<{:?}>\0", other);
                    let ptr = self.make_cstr("ph", placeholder.as_bytes());
                    let _ = self.builder.build_call(print_str_fn, &[ptr.into()], "print_ph");
                }
            }

            // space between args
            if i + 1 < args.len() {
                let sp = self.make_cstr("sp", b" \0");
                let _ = self.builder.build_call(print_str_fn, &[sp.into()], "print_sp");
            }
        }

        // final newline
        let nl = self.make_cstr("nl", b"\n\0");
        let _ = self.builder.build_call(println_fn, &[nl.into()], "print_nl");

        Ok((self.llvm_context.i64_type().const_zero().into(), Type::None))
    }

}
