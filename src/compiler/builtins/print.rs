use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::ExprCompiler;
use crate::compiler::types::Type;
use crate::compiler::runtime::list::{TypeTag, get_list_struct_type};
use inkwell::AddressSpace;
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::IntPredicate;

impl<'ctx> CompilationContext<'ctx> {
    /// Helper function to safely cast a pointer or return the original if the cast is a no-op
    fn cast_or_self(
        builder: &inkwell::builder::Builder<'ctx>,
        v: inkwell::values::PointerValue<'ctx>,
        dst_ty: inkwell::types::PointerType<'ctx>,
        name: &str,
    ) -> inkwell::values::PointerValue<'ctx> {
        builder.build_pointer_cast(v, dst_ty, name).unwrap_or(v)
    }

    /// Get the current function from the builder's insertion point
    fn current_fn(&self) -> inkwell::values::FunctionValue<'ctx> {
        self.builder
            .get_insert_block()
            .and_then(|bb| bb.get_parent())
            .expect("builder has no insertion point")
    }

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
        // with opaque pointers the cast is often a no‑op -> use the helper
        Self::cast_or_self(
            &self.builder,
            global.as_pointer_value(),
            self.llvm_context.ptr_type(AddressSpace::default()),
            &format!("{}_ptr", name),
        )
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
                    let str_ptr = Self::cast_or_self(
                        &self.builder,
                        val.into_pointer_value(),
                        self.llvm_context.ptr_type(AddressSpace::default()),
                        "str_ptr",
                    );
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

    /// Helper: print one value whose *static* LLVM type is known.
    /// Handles quoting for strings, etc.
    fn print_value_by_type(
        &mut self,
        opaque_ptr: BasicValueEnum<'ctx>,
        ty: &Type,
        quote: PointerValue<'ctx>,
        none_lit: PointerValue<'ctx>,
    ) -> Result<(), String> {
        let void_ptr_t = self.llvm_context.ptr_type(AddressSpace::default());

        match ty {
            Type::Int => {
                let int_ptr_ty = self.llvm_context.ptr_type(AddressSpace::default());
                let int_ptr = Self::cast_or_self(
                    &self.builder,
                    opaque_ptr.into_pointer_value(),
                    int_ptr_ty,
                    "int_ptr",
                );
                let int_val = self
                    .builder
                    .build_load(self.llvm_context.i64_type(), int_ptr, "int_val")
                    .unwrap();
                let print_int = self
                    .module
                    .get_function("print_int")
                    .ok_or("print_int not found")?;
                self.builder
                    .build_call(print_int, &[int_val.into()], "pi")
                    .unwrap();
            }
            Type::Float => {
                let f_ptr_ty = self.llvm_context.ptr_type(AddressSpace::default());
                let f_ptr = Self::cast_or_self(
                    &self.builder,
                    opaque_ptr.into_pointer_value(),
                    f_ptr_ty,
                    "fptr",
                );
                let f_val = self
                    .builder
                    .build_load(self.llvm_context.f64_type(), f_ptr, "fval")
                    .unwrap();
                let print_flt = self
                    .module
                    .get_function("print_float")
                    .ok_or("print_float not found")?;
                self.builder
                    .build_call(print_flt, &[f_val.into()], "pf")
                    .unwrap();
            }
            Type::Bool => {
                let b_ptr_ty = self.llvm_context.ptr_type(AddressSpace::default());
                let b_ptr = Self::cast_or_self(
                    &self.builder,
                    opaque_ptr.into_pointer_value(),
                    b_ptr_ty,
                    "bptr",
                );
                let b_val = self
                    .builder
                    .build_load(self.llvm_context.bool_type(), b_ptr, "bval")
                    .unwrap();
                let print_bool = self
                    .module
                    .get_function("print_bool")
                    .ok_or("print_bool not found")?;
                self.builder
                    .build_call(print_bool, &[b_val.into()], "pb")
                    .unwrap();
            }
            Type::String => {
                let str_ptr = Self::cast_or_self(
                    &self.builder,
                    opaque_ptr.into_pointer_value(),
                    void_ptr_t,
                    "str_ptr",
                );
                let print_str = self
                    .module
                    .get_function("print_string")
                    .ok_or("print_string not found")?;
                // opening quote
                self.builder
                    .build_call(print_str, &[quote.into()], "q1")
                    .unwrap();
                // body
                self.builder
                    .build_call(print_str, &[str_ptr.into()], "ps")
                    .unwrap();
                // closing quote
                self.builder
                    .build_call(print_str, &[quote.into()], "q2")
                    .unwrap();
            }
            Type::None => {
                let print_str = self
                    .module
                    .get_function("print_string")
                    .ok_or("print_string not found")?;
                self.builder
                    .build_call(print_str, &[none_lit.into()], "pnone")
                    .unwrap();
            }
            Type::List(inner) => {
                let list_ptr_ty = opaque_ptr.into_pointer_value().get_type();
                let list_ptr = Self::cast_or_self(
                    &self.builder,
                    opaque_ptr.into_pointer_value(),
                    list_ptr_ty,
                    "listcast",
                );
                self.print_list(list_ptr, &*inner)?;
            }
            Type::Tuple(elem_tys) => {
                let tup_ptr_ty = self.llvm_context.ptr_type(AddressSpace::default());
                let tup_ptr = Self::cast_or_self(
                    &self.builder,
                    opaque_ptr.into_pointer_value(),
                    tup_ptr_ty,
                    "tupcast",
                );
                self.print_tuple(tup_ptr, elem_tys)?;
            }
            _ => {
                // fallback
                let ph = self.make_cstr("ph2", b"<Any>\0");
                let print_str = self
                    .module
                    .get_function("print_string")
                    .ok_or("print_string not found")?;
                self.builder
                    .build_call(print_str, &[ph.into()], "ph")
                    .unwrap();
            }
        }

        Ok(())
    }

    /// Print a RawList with brackets, comma‑separated elements, and
    /// *per‑element dynamic dispatch* when the list’s element type is `Any`.
    ///
    /// ── `list_ptr` is `*mut RawList` (LLVM pointer value)
    /// ── `elem_ty` is the *static* element type known at compile time
    ///
    fn print_list(
        &mut self,
        list_ptr: PointerValue<'ctx>,
        elem_ty: &Type,
    ) -> Result<(), String> {
        use inkwell::{AddressSpace, IntPredicate};
        use crate::compiler::runtime::list::{get_list_struct_type, TypeTag};
    
        let ctx            = self.llvm_context;
        let i64_t          = ctx.i64_type();
        let i8_t           = ctx.i8_type();
        let void_ptr_t     = ctx.ptr_type(AddressSpace::default());          // i8*
        let void_ptr_ptr_t = void_ptr_t.ptr_type(AddressSpace::default());   // i8**
    
        // RawList layout (length, capacity, data, tags)
        let rawlist_ty = get_list_struct_type(ctx);

        // ——————————————————————————————————————————————————————————
        // Runtime helpers we may call
        // ——————————————————————————————————————————————————————————
        let print_str   = self.module.get_function("print_string").ok_or("print_string not found")?;
        let _print_int  = self.module.get_function("print_int").ok_or("print_int not found")?;
        let _print_flt  = self.module.get_function("print_float").ok_or("print_float not found")?;
        let _print_bool = self.module.get_function("print_bool").ok_or("print_bool not found")?;

        // list_get_tag(list, idx) → u8
        let list_get_tag = self.module.get_function("list_get_tag").unwrap_or_else(|| {
            let fn_ty = i8_t.fn_type(&[void_ptr_t.into(), i64_t.into()], false);
            self.module.add_function("list_get_tag", fn_ty, None)
        });

        // ——————————————————————————————————————————————————————————
        // Handy literals
        // ——————————————————————————————————————————————————————————
        let lbrack   = self.make_cstr("lb", b"[\0");
        let rbrack   = self.make_cstr("rb", b"]\0");
        let comma    = self.make_cstr("cm", b", \0");
        let quote    = self.make_cstr("qt", b"'\0");
        let none_lit = self.make_cstr("none", b"None\0");

        // print “[”
        self.builder.build_call(print_str, &[lbrack.into()], "pr_lb").unwrap();

        // len = list.length
        let len_val = {
            let len_ptr = self
                .builder
                .build_struct_gep(rawlist_ty, list_ptr, 0, "len_ptr").unwrap();
            self.builder
                .build_load(i64_t, len_ptr, "len").unwrap()
                .into_int_value()
        };

        // i = 0
        let idx_ptr = self.builder.build_alloca(i64_t, "idx").unwrap();
        self.builder.build_store(idx_ptr, i64_t.const_zero()).unwrap();

        let cur_fn   = self.current_fn();
        let bb_cond  = ctx.append_basic_block(cur_fn, "list.cond");
        let bb_body  = ctx.append_basic_block(cur_fn, "list.body");
        let bb_after = ctx.append_basic_block(cur_fn, "list.after");
        self.builder.build_unconditional_branch(bb_cond).unwrap();

        // ———————————————————  cond
        self.builder.position_at_end(bb_cond);
        let idx_val = self
            .builder
            .build_load(i64_t, idx_ptr, "idx_val").unwrap()
            .into_int_value();
        let cmp = self
            .builder
            .build_int_compare(IntPredicate::ULT, idx_val, len_val, "cmp").unwrap();
        self.builder.build_conditional_branch(cmp, bb_body, bb_after).unwrap();

        // ———————————————————  body
        self.builder.position_at_end(bb_body);

        // data_ptr = (*list).data      — **load as i8 ** not i8*
        let data_ptr_ptr = self
            .builder
            .build_struct_gep(rawlist_ty, list_ptr, 2, "data_ptr_ptr").unwrap();
        let data_ptr = self
            .builder
            .build_load(void_ptr_ptr_t, data_ptr_ptr, "data_ptr").unwrap()
            .into_pointer_value();

        // elem_ptr = data_ptr[idx]
        let elem_addr = unsafe {
            self.builder
                .build_in_bounds_gep(void_ptr_t, data_ptr, &[idx_val], "elem_addr").unwrap()
        };
        let elem_ptr = self.builder.build_load(void_ptr_t, elem_addr, "elem_ptr").unwrap();


        // ----------------------------------------------------------
        // STATIC path  (homogeneous list, elem_ty != Any)
        // ----------------------------------------------------------
        if elem_ty != &Type::Any {
            self.print_value_by_type(elem_ptr, elem_ty, quote, none_lit)?;
        }
        // ----------------------------------------------------------
        // DYNAMIC path  (elem_ty == Any) – dispatch by TypeTag
        // ----------------------------------------------------------
        else {
            // tag = list_get_tag(list, idx)
            let tag_val = self
                .builder
                .build_call(
                    list_get_tag,
                    &[list_ptr.into(), idx_val.into()],
                    "tagcall",
                )
                .unwrap()
                .try_as_basic_value()
                .left()
                .unwrap()
                .into_int_value();

            // prepare blocks for the switch
            let bb_int    = ctx.append_basic_block(cur_fn, "tag.int");
            let bb_flt    = ctx.append_basic_block(cur_fn, "tag.flt");
            let bb_bool   = ctx.append_basic_block(cur_fn, "tag.bool");
            let bb_str    = ctx.append_basic_block(cur_fn, "tag.str");
            let bb_list   = ctx.append_basic_block(cur_fn, "tag.list");
            let bb_tuple  = ctx.append_basic_block(cur_fn, "tag.tuple");
            let bb_none   = ctx.append_basic_block(cur_fn, "tag.none");
            let bb_deflt  = ctx.append_basic_block(cur_fn, "tag.deflt");

            // switch(tag_val)
            self.builder
                .build_switch(
                    tag_val,
                    bb_deflt,
                    &[
                        (
                            i8_t.const_int(TypeTag::Int as u64, false),
                            bb_int,
                        ),
                        (
                            i8_t.const_int(TypeTag::Float as u64, false),
                            bb_flt,
                        ),
                        (
                            i8_t.const_int(TypeTag::Bool as u64, false),
                            bb_bool,
                        ),
                        (
                            i8_t.const_int(TypeTag::String as u64, false),
                            bb_str,
                        ),
                        (
                            i8_t.const_int(TypeTag::List as u64, false),
                            bb_list,
                        ),
                        (
                            i8_t.const_int(TypeTag::Tuple as u64, false),
                            bb_tuple,
                        ),
                        (
                            i8_t.const_int(TypeTag::None_ as u64, false),
                            bb_none,
                        ),
                    ],
                )
                .unwrap();

            // helper to end each tag‐block by branching to cond
            let branch_back = |builder: &inkwell::builder::Builder<'ctx>| {
                builder.build_unconditional_branch(bb_cond).unwrap();
            };

            // INT
            self.builder.position_at_end(bb_int);
            self.print_value_by_type(elem_ptr, &Type::Int, quote, none_lit)?;
            branch_back(&self.builder);

            // FLOAT
            self.builder.position_at_end(bb_flt);
            self.print_value_by_type(elem_ptr, &Type::Float, quote, none_lit)?;
            branch_back(&self.builder);

            // BOOL
            self.builder.position_at_end(bb_bool);
            self.print_value_by_type(elem_ptr, &Type::Bool, quote, none_lit)?;
            branch_back(&self.builder);

            // STRING
            self.builder.position_at_end(bb_str);
            self.print_value_by_type(elem_ptr, &Type::String, quote, none_lit)?;
            branch_back(&self.builder);

            // LIST   (recurse)
            self.builder.position_at_end(bb_list);
            let list_ptr_cast = Self::cast_or_self(
                &self.builder,
                elem_ptr.into_pointer_value(),
                list_ptr.get_type(),
                "cast_list",
            );
            self.print_list(list_ptr_cast, &Type::Any)?;
            branch_back(&self.builder);

            // TUPLE  (recurse)
            self.builder.position_at_end(bb_tuple);
            let tup_ptr_ty = self
                .llvm_context
                .ptr_type(AddressSpace::default()); // tuple prints take *void anyway
            let tup_ptr = Self::cast_or_self(
                &self.builder,
                elem_ptr.into_pointer_value(),
                tup_ptr_ty,
                "cast_tup",
            );
            self.print_tuple(tup_ptr, &vec![])?; // element types unknown; tuple printer handles Any
            branch_back(&self.builder);

            // NONE
            self.builder.position_at_end(bb_none);
            self.builder
                .build_call(print_str, &[none_lit.into()], "pnone")
                .unwrap();
            branch_back(&self.builder);

            // DEFAULT  (“<Any>” placeholder – should be rare)
            self.builder.position_at_end(bb_deflt);
            let ph = self.make_cstr("ph", b"<Any>\0");
            self.builder
                .build_call(print_str, &[ph.into()], "ph_any")
                .unwrap();
            branch_back(&self.builder);
        }

        // ————————————————————————————————————————————————
        // Print “, ” if idx < len‑1
        // ————————————————————————————————————————————————
        self.builder.position_at_end(bb_body); // (printer left us somewhere inside)
        let is_last = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                idx_val,
                self.builder
                    .build_int_sub(len_val, i64_t.const_int(1, false), "len-1")
                    .unwrap(),
                "is_last",
            )
            .unwrap();
        let bb_comma = ctx.append_basic_block(cur_fn, "comma");
        let bb_no    = ctx.append_basic_block(cur_fn, "no_comma");
        self.builder
            .build_conditional_branch(is_last, bb_no, bb_comma)
            .unwrap();

        self.builder.position_at_end(bb_comma);
        self.builder
            .build_call(print_str, &[comma.into()], "pc")
            .unwrap();
        self.builder.build_unconditional_branch(bb_no).unwrap();

        // increment idx
        self.builder.position_at_end(bb_no);
        let next = self
            .builder
            .build_int_add(idx_val, i64_t.const_int(1, false), "idx+1")
            .unwrap();
        self.builder.build_store(idx_ptr, next).unwrap();
        self.builder.build_unconditional_branch(bb_cond).unwrap();

        // ———————————————————  after
        self.builder.position_at_end(bb_after);
        self.builder.build_call(print_str, &[rbrack.into()], "prb").unwrap();

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

        let _i64_ty = self.llvm_context.i64_type();
        let ptr_ty = self.llvm_context.ptr_type(AddressSpace::default());

        // Cast the opaque pointer to the tuple struct type
        let tup_ptr_ty = self.llvm_context.ptr_type(AddressSpace::default());
        let tup = Self::cast_or_self(&self.builder, tup, tup_ptr_ty, "tup_typed");

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
                    let s_ptr = Self::cast_or_self(&self.builder,
                                                   val.into_pointer_value(),
                                                   ptr_ty,
                                                   "cast_str");
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
