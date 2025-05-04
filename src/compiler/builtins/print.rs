use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::ExprCompiler;
use crate::compiler::types::Type;
use inkwell::AddressSpace;
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::IntPredicate;

impl<'ctx> CompilationContext<'ctx> {
    /// Register print functions and bind `print`
    pub fn register_print_function(&mut self) {
        let ctx = self.llvm_context;
        let m = &mut self.module;

        // print_string
        if m.get_function("print_string").is_none() {
            let ty = ctx.ptr_type(AddressSpace::default()).into();
            m.add_function("print_string", ctx.void_type().fn_type(&[ty], false), None);
        }
        // print_int
        if m.get_function("print_int").is_none() {
            m.add_function("print_int", ctx.void_type().fn_type(&[ctx.i64_type().into()], false), None);
        }
        // print_float
        if m.get_function("print_float").is_none() {
            m.add_function("print_float", ctx.void_type().fn_type(&[ctx.f64_type().into()], false), None);
        }
        // print_bool
        if m.get_function("print_bool").is_none() {
            m.add_function("print_bool", ctx.void_type().fn_type(&[ctx.bool_type().into()], false), None);
        }
    }

    /// Create a global C string and return i8* pointer
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

    /// Compile a call to print(), supporting None, primitives, lists, and tuples
    pub fn compile_print_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        let print_str = self.module.get_function("print_string").ok_or("print_string not found")?;
        let print_int = self.module.get_function("print_int").ok_or("print_int not found")?;
        let print_flt = self.module.get_function("print_float").ok_or("print_float not found")?;
        let print_bool = self.module.get_function("print_bool").ok_or("print_bool not found")?;
        let println_fn = self.module.get_function("println_string").ok_or("println_string not found")?;

        // For string quoting
        let quote = self.make_cstr("sq", b"'\0");
        let none_lit = self.make_cstr("none", b"None\0");
        let space = self.make_cstr("sp", b" \0");

        for (i, arg) in args.iter().enumerate() {
            let (val, ty) = self.compile_expr(arg)?;
            if i > 0 {
                self.builder.build_call(print_str, &[space.into()], "print_space").unwrap();
            }
            match ty {
                Type::None => {
                    self.builder.build_call(print_str, &[none_lit.into()], "print_none").unwrap();
                }
                Type::String => {
                    // print ' + content + '
                    self.builder.build_call(print_str, &[quote.into()], "print_quote_l").unwrap();
                    let str_ptr = self.builder.build_pointer_cast(
                        val.into_pointer_value(),
                        self.llvm_context.ptr_type(AddressSpace::default()),
                        "str_ptr",
                    ).unwrap();
                    self.builder.build_call(print_str, &[str_ptr.into()], "print_str").unwrap();
                    self.builder.build_call(print_str, &[quote.into()], "print_quote_r").unwrap();
                }
                Type::Int => {
                    self.builder.build_call(print_int, &[val.into()], "print_int").unwrap();
                }
                Type::Float => {
                    self.builder.build_call(print_flt, &[val.into()], "print_float").unwrap();
                }
                Type::Bool => {
                    self.builder.build_call(print_bool, &[val.into()], "print_bool").unwrap();
                }
                Type::List(elem_ty) => {
                    self.print_list(val.into_pointer_value(), &*elem_ty)?;
                }
                Type::Tuple(elem_tys) => {
                    self.print_tuple(val.into_pointer_value(), &elem_tys)?;
                }
                other => {
                    // fallback
                    let ph = self.make_cstr("ph", format!("<{:?}>\0", other).as_bytes());
                    self.builder.build_call(print_str, &[ph.into()], "print_ph").unwrap();
                }
            }
        }

        // newline
        let nl = self.make_cstr("nl", b"\n\0");
        self.builder.build_call(println_fn, &[nl.into()], "print_nl").unwrap();
        Ok((self.llvm_context.i64_type().const_zero().into(), Type::None))
    }

    /// Print a RawList with brackets and comma-sep elements
    fn print_list(&mut self, list_ptr: PointerValue<'ctx>, elem_ty: &Type) -> Result<(), String> {
        let print_str  = self.module.get_function("print_string").unwrap();
        let print_int  = self.module.get_function("print_int").unwrap();
        let print_flt  = self.module.get_function("print_float").unwrap();
        let print_bool = self.module.get_function("print_bool").unwrap();

        // Pre-constructed constants
        let lb    = self.make_cstr("lb",    b"[\0");
        let rb    = self.make_cstr("rb",    b"]\0");
        let comma = self.make_cstr("cm",    b", \0");
        let none_lit = self.make_cstr("none", b"None\0");

        // '['
        self.builder.build_call(print_str, &[lb.into()], "print_lb").unwrap();

        // length = list_len(list_ptr)
        let len_fn = self.module.get_function("list_len").ok_or("list_len not found")?;
        let len_val = self.builder
            .build_call(len_fn, &[list_ptr.into()], "list_len")
            .unwrap()
            .try_as_basic_value().left().unwrap()
            .into_int_value();

        // setup loop indices
        let i64_ty = self.llvm_context.i64_type();
        let zero   = i64_ty.const_zero();
        let one    = i64_ty.const_int(1, false);
        let idx    = self.builder.build_alloca(i64_ty, "idx").unwrap();
        self.builder.build_store(idx, zero).unwrap();

        // basic blocks
        let parent  = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let cond_bb = self.llvm_context.append_basic_block(parent, "l_cond");
        let body_bb = self.llvm_context.append_basic_block(parent, "l_body");
        let done_bb = self.llvm_context.append_basic_block(parent, "l_done");

        // jump into the loop
        self.builder.build_unconditional_branch(cond_bb).unwrap();
        self.builder.position_at_end(cond_bb);
        let cur = self.builder.build_load(i64_ty, idx, "cur").unwrap().into_int_value();
        let cmp = self.builder.build_int_compare(IntPredicate::SLT, cur, len_val, "cmp_len").unwrap();
        self.builder.build_conditional_branch(cmp, body_bb, done_bb).unwrap();

        // loop body
        self.builder.position_at_end(body_bb);
        // comma before all but the first
        let not_first = self.builder.build_int_compare(IntPredicate::NE, cur, zero, "not_first").unwrap();
        let sep_bb     = self.llvm_context.append_basic_block(parent, "l_sep");
        let nos_bb     = self.llvm_context.append_basic_block(parent, "l_nosep");
        self.builder.build_conditional_branch(not_first, sep_bb, nos_bb).unwrap();
        self.builder.position_at_end(sep_bb);
        self.builder.build_call(print_str, &[comma.into()], "print_sep").unwrap();
        self.builder.build_unconditional_branch(nos_bb).unwrap();
        self.builder.position_at_end(nos_bb);

        // elem_ptr = list_get(list_ptr, cur)
        let get_fn = self.module.get_function("list_get").ok_or("list_get not found")?;
        let elem_ptr = self.builder
            .build_call(get_fn, &[list_ptr.into(), cur.into()], "get")
            .unwrap()
            .try_as_basic_value().left().unwrap()
            .into_pointer_value();

        // dispatch on element type
        match elem_ty {
            Type::None => {
                self.builder.build_call(print_str, &[none_lit.into()], "p_none").unwrap();
            }
            Type::Int => {
                // cast i8* → i64* then load
                let int_ptr_ty = i64_ty.ptr_type(AddressSpace::default());
                let int_ptr = self.builder.build_pointer_cast(elem_ptr, int_ptr_ty, "int_ptr").unwrap();
                let ival    = self.builder.build_load(i64_ty, int_ptr, "ival").unwrap().into_int_value();
                self.builder.build_call(print_int, &[ival.into()], "p_int").unwrap();
            }
            Type::Float => {
                let f64_ty = self.llvm_context.f64_type();
                let flt_ptr_ty = f64_ty.ptr_type(AddressSpace::default());
                let fptr = self.builder.build_pointer_cast(elem_ptr, flt_ptr_ty, "fptr").unwrap();
                let fval = self.builder.build_load(f64_ty, fptr, "fval").unwrap().into_float_value();
                self.builder.build_call(print_flt, &[fval.into()], "p_flt").unwrap();
            }
            Type::Bool => {
                // load an i1 directly
                let bool_ptr_ty = self.llvm_context.bool_type().ptr_type(AddressSpace::default());
                let bptr = self.builder.build_pointer_cast(elem_ptr, bool_ptr_ty, "bptr").unwrap();
                let bval = self.builder.build_load(self.llvm_context.bool_type(), bptr, "bval").unwrap().into_int_value();
                self.builder.build_call(print_bool, &[bval.into()], "p_bool").unwrap();
            }
            Type::String => {
                let sq = self.make_cstr("sq", b"'\0");
                // print ' str '
                self.builder.build_call(print_str, &[sq.into()], "p_q1").unwrap();
                let sptr = self.builder.build_pointer_cast(elem_ptr, self.llvm_context.ptr_type(AddressSpace::default()), "sptr").unwrap();
                self.builder.build_call(print_str, &[sptr.into()], "p_str").unwrap();
                self.builder.build_call(print_str, &[sq.into()], "p_q2").unwrap();
            }
            Type::List(inner) => {
                // nested list: treat elem_ptr as raw list pointer
                let lptr = self.builder.build_pointer_cast(elem_ptr, self.llvm_context.ptr_type(AddressSpace::default()), "lptr").unwrap();
                self.print_list(lptr, inner)?;
            }
            Type::Tuple(inner) => {
                let tptr = self.builder.build_pointer_cast(elem_ptr, self.llvm_context.ptr_type(AddressSpace::default()), "tptr").unwrap();
                self.print_tuple(tptr, inner)?;
            }
            other => {
                // fallback
                let ph = self.make_cstr("ph", format!("<{:?}>\0", other).as_bytes());
                self.builder.build_call(print_str, &[ph.into()], "p_ph").unwrap();
            }
        }

        // idx += 1; loop back
        let inc = self.builder.build_int_add(cur, one, "inc").unwrap();
        self.builder.build_store(idx, inc).unwrap();
        self.builder.build_unconditional_branch(cond_bb).unwrap();

        // done: print ']'
        self.builder.position_at_end(done_bb);
        self.builder.build_call(print_str, &[rb.into()], "print_rb").unwrap();

        Ok(())
    }


    /// Print a Tuple with parentheses and comma-sep fields
    fn print_tuple(&mut self, tup: PointerValue<'ctx>, types: &[Type]) -> Result<(), String> {
        let print_str  = self.module.get_function("print_string").unwrap();
        let print_int  = self.module.get_function("print_int").unwrap();
        let print_flt  = self.module.get_function("print_float").unwrap();
        let print_bool = self.module.get_function("print_bool").unwrap();

        let lp        = self.make_cstr("lp", b"(\0");
        let rp        = self.make_cstr("rp", b")\0");
        let comma     = self.make_cstr("cm", b", \0");
        let sq        = self.make_cstr("sq", b"'\0");
        let none_lit  = self.make_cstr("none", b"None\0");

        self.builder.build_call(print_str, &[lp.into()], "print_lp").unwrap();

        // get the LLVM StructType for this tuple
        let struct_ty = match self.get_llvm_type(&Type::Tuple(types.to_vec())) {
            inkwell::types::BasicTypeEnum::StructType(st) => st,
            _ => return Err("Expected tuple struct".into()),
        };

        let i64_ty = self.llvm_context.i64_type();
        let ptr_ty = self.llvm_context.ptr_type(AddressSpace::default());

        for (i, ty) in types.iter().enumerate() {
            if i > 0 {
                self.builder.build_call(print_str, &[comma.into()], "print_comma").unwrap();
            }
            // load the field
            let field_ptr = self.builder
                .build_struct_gep(struct_ty, tup, i as u32, &format!("fp{}", i))
                .unwrap();
            let val = self.builder.build_load(struct_ty.get_field_types()[i], field_ptr, "fv").unwrap();

            match ty {
                Type::None => {
                    self.builder.build_call(print_str, &[none_lit.into()], "tp_none").unwrap();
                }
                Type::String => {
                    self.builder.build_call(print_str, &[sq.into()], "tp_q1").unwrap();
                    let s_ptr = self.builder.build_pointer_cast(val.into_pointer_value(), ptr_ty, "cast_str").unwrap();
                    self.builder.build_call(print_str, &[s_ptr.into()], "tp_str").unwrap();
                    self.builder.build_call(print_str, &[sq.into()], "tp_q2").unwrap();
                }
                Type::Int => {
                    let iv = val.into_int_value();
                    self.builder.build_call(print_int, &[iv.into()], "tp_int").unwrap();
                }
                Type::Float => {
                    let fv = val.into_float_value();
                    self.builder.build_call(print_flt, &[fv.into()], "tp_flt").unwrap();
                }
                Type::Bool => {
                    let bv = val.into_int_value();
                    self.builder.build_call(print_bool, &[bv.into()], "tp_bool").unwrap();
                }
                Type::List(inner) => {
                    let lptr = val.into_pointer_value();
                    self.print_list(lptr, inner)?;
                }
                Type::Tuple(inner) => {
                    let tptr = val.into_pointer_value();
                    self.print_tuple(tptr, inner)?;
                }
                other => {
                    let ph = self.make_cstr("ph", format!("<{:?}>\0", other).as_bytes());
                    self.builder.build_call(print_str, &[ph.into()], "tp_ph").unwrap();
                }
            }
        }

        // single-element tuple needs trailing comma
        if types.len() == 1 {
            let tc = self.make_cstr("tc", b",\0");
            self.builder.build_call(print_str, &[tc.into()], "tp_trailing").unwrap();
        }
        self.builder.build_call(print_str, &[rp.into()], "print_rp").unwrap();
        Ok(())
    }

}
