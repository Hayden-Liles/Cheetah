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
        println!("DEBUG: print_list called with element type: {:?}", elem_ty);
        use inkwell::{AddressSpace, IntPredicate};
        use crate::compiler::runtime::list::{get_list_struct_type, TypeTag};

        let ctx            = self.llvm_context;
        let i64_t          = ctx.i64_type();
        let i8_t           = ctx.i8_type();
        let void_ptr_t     = ctx.ptr_type(AddressSpace::default());          // i8*
        let void_ptr_ptr_t = void_ptr_t.ptr_type(AddressSpace::default());   // i8**

        // RawList layout (length, capacity, data, tags)
        let rawlist_ty = get_list_struct_type(ctx);
        println!("DEBUG: Got RawList struct type");

        // ——————————————————————————————————————————————————————————
        // Runtime helpers we may call
        // ——————————————————————————————————————————————————————————
        let print_str   = self.module.get_function("print_string").ok_or("print_string not found")?;
        let _print_int  = self.module.get_function("print_int").ok_or("print_int not found")?;
        let _print_flt  = self.module.get_function("print_float").ok_or("print_float not found")?;
        let _print_bool = self.module.get_function("print_bool").ok_or("print_bool not found")?;
        println!("DEBUG: Got print functions");

        // list_get_tag(list, idx) → u8
        let list_get_tag = self.module.get_function("list_get_tag").unwrap_or_else(|| {
            println!("DEBUG: Creating list_get_tag function");
            let fn_ty = i8_t.fn_type(&[void_ptr_t.into(), i64_t.into()], false);
            self.module.add_function("list_get_tag", fn_ty, None)
        });
        println!("DEBUG: Got list_get_tag function");

        // ——————————————————————————————————————————————————————————
        // Handy literals
        // ——————————————————————————————————————————————————————————
        let lbrack   = self.make_cstr("lb", b"[\0");
        let rbrack   = self.make_cstr("rb", b"]\0");
        let comma    = self.make_cstr("cm", b", \0");
        let quote    = self.make_cstr("qt", b"'\0");
        let none_lit = self.make_cstr("none", b"None\0");
        println!("DEBUG: Created string literals");

        // print “[”
        self.builder.build_call(print_str, &[lbrack.into()], "pr_lb").unwrap();
        println!("DEBUG: Printed opening bracket");

        // len = list.length
        let len_val = {
            let len_ptr = self
                .builder
                .build_struct_gep(rawlist_ty, list_ptr, 0, "len_ptr").unwrap();
            let len = self.builder
                .build_load(i64_t, len_ptr, "len").unwrap()
                .into_int_value();
            if let Some(len_const) = len.get_zero_extended_constant() {
                println!("DEBUG: List length: {}", len_const);
            } else {
                println!("DEBUG: List length is not a constant");
            }
            len
        };

        // i = 0
        let idx_ptr = self.builder.build_alloca(i64_t, "idx").unwrap();
        self.builder.build_store(idx_ptr, i64_t.const_zero()).unwrap();
        println!("DEBUG: Initialized index to 0");

        let cur_fn   = self.current_fn();
        let bb_cond  = ctx.append_basic_block(cur_fn, "list.cond");
        let bb_body  = ctx.append_basic_block(cur_fn, "list.body");
        let bb_after = ctx.append_basic_block(cur_fn, "list.after");
        println!("DEBUG: Created basic blocks for loop");
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
        println!("DEBUG: Built loop condition");
        self.builder.build_conditional_branch(cmp, bb_body, bb_after).unwrap();

        // ———————————————————  body
        self.builder.position_at_end(bb_body);
        println!("DEBUG: Entering loop body");

        // data_ptr = (*list).data      — **load as i8 ** not i8*
        let data_ptr_ptr = self
            .builder
            .build_struct_gep(rawlist_ty, list_ptr, 2, "data_ptr_ptr").unwrap();
        let data_ptr = self
            .builder
            .build_load(void_ptr_ptr_t, data_ptr_ptr, "data_ptr").unwrap()
            .into_pointer_value();
        println!("DEBUG: Got data pointer from list");

        // elem_ptr = data_ptr[idx]
        let elem_addr = unsafe {
            self.builder
                .build_in_bounds_gep(void_ptr_t, data_ptr, &[idx_val], "elem_addr").unwrap()
        };
        let elem_ptr = self.builder.build_load(void_ptr_t, elem_addr, "elem_ptr").unwrap();
        if let Some(idx_const) = idx_val.get_zero_extended_constant() {
            println!("DEBUG: Got element pointer at index {}", idx_const);
        } else {
            println!("DEBUG: Got element pointer at dynamic index");
        }


        // ----------------------------------------------------------
        // STATIC path  (homogeneous list, elem_ty != Any)
        // ----------------------------------------------------------
        if elem_ty != &Type::Any {
            println!("DEBUG: Using STATIC path for homogeneous list with element type: {:?}", elem_ty);
            self.print_value_by_type(elem_ptr, elem_ty, quote, none_lit)?;

            // Handle comma, increment index for static path
            // Print ", " if idx < len-1
            let is_last = self.builder
                .build_int_compare(
                    IntPredicate::EQ,
                    idx_val,
                    self.builder
                        .build_int_sub(len_val, i64_t.const_int(1, false), "len-1")
                        .unwrap(),
                    "is_last_static",
                )
                .unwrap();

            let bb_comma_static = ctx.append_basic_block(cur_fn, "comma_static");
            let bb_no_static = ctx.append_basic_block(cur_fn, "no_comma_static");
            self.builder
                .build_conditional_branch(is_last, bb_no_static, bb_comma_static)
                .unwrap();

            // Print comma if not the last element
            self.builder.position_at_end(bb_comma_static);
            self.builder
                .build_call(print_str, &[comma.into()], "pc_static")
                .unwrap();
            self.builder.build_unconditional_branch(bb_no_static).unwrap();

            // Increment index and continue
            self.builder.position_at_end(bb_no_static);
            // idx = idx + 1
            let next = self.builder
                .build_int_add(idx_val, i64_t.const_int(1, false), "inc_static")
                .unwrap();
            self.builder.build_store(idx_ptr, next).unwrap();
            self.builder.build_unconditional_branch(bb_cond).unwrap();
        }
        // ----------------------------------------------------------
        // DYNAMIC path  (elem_ty == Any) – dispatch by TypeTag
        // ----------------------------------------------------------
        else {
            println!("DEBUG: Using DYNAMIC path for heterogeneous list (Any type)");
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

            if let Some(tag_const) = tag_val.get_zero_extended_constant() {
                println!("DEBUG: Element tag value: {}", tag_const);
            } else {
                println!("DEBUG: Element tag value is not a constant");
            }

            // prepare blocks for the switch
            let bb_int    = ctx.append_basic_block(cur_fn, "tag.int");
            let bb_flt    = ctx.append_basic_block(cur_fn, "tag.flt");
            let bb_bool   = ctx.append_basic_block(cur_fn, "tag.bool");
            let bb_str    = ctx.append_basic_block(cur_fn, "tag.str");
            let bb_list   = ctx.append_basic_block(cur_fn, "tag.list");
            let bb_tuple  = ctx.append_basic_block(cur_fn, "tag.tuple");
            let bb_none   = ctx.append_basic_block(cur_fn, "tag.none");
            let bb_deflt  = ctx.append_basic_block(cur_fn, "tag.deflt");
            println!("DEBUG: Created basic blocks for tag switch");

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
            println!("DEBUG: Built switch statement for tag dispatch");

            // Create a common tail block for all tag cases
            let bb_tag_tail = ctx.append_basic_block(cur_fn, "tag.tail");

            // INT
            self.builder.position_at_end(bb_int);
            println!("DEBUG: Handling INT element");
            self.print_value_by_type(elem_ptr, &Type::Int, quote, none_lit)?;
            self.builder.build_unconditional_branch(bb_tag_tail).unwrap();

            // FLOAT
            self.builder.position_at_end(bb_flt);
            println!("DEBUG: Handling FLOAT element");
            self.print_value_by_type(elem_ptr, &Type::Float, quote, none_lit)?;
            self.builder.build_unconditional_branch(bb_tag_tail).unwrap();

            // BOOL
            self.builder.position_at_end(bb_bool);
            println!("DEBUG: Handling BOOL element");
            self.print_value_by_type(elem_ptr, &Type::Bool, quote, none_lit)?;
            self.builder.build_unconditional_branch(bb_tag_tail).unwrap();

            // STRING
            self.builder.position_at_end(bb_str);
            println!("DEBUG: Handling STRING element");
            self.print_value_by_type(elem_ptr, &Type::String, quote, none_lit)?;
            self.builder.build_unconditional_branch(bb_tag_tail).unwrap();

            // LIST   (recurse)
            self.builder.position_at_end(bb_list);
            println!("DEBUG: Handling nested LIST element");

            // Check if the element pointer is the same as the current list pointer
            // to prevent infinite recursion
            let is_same_list = self.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                self.builder.build_ptr_to_int(elem_ptr.into_pointer_value(), ctx.i64_type(), "elem_ptr_int").unwrap(),
                self.builder.build_ptr_to_int(list_ptr, ctx.i64_type(), "list_ptr_int").unwrap(),
                "is_same_list"
            ).unwrap();

            let bb_safe_recurse = ctx.append_basic_block(cur_fn, "safe_recurse");
            let bb_skip_recurse = ctx.append_basic_block(cur_fn, "skip_recurse");
            self.builder.build_conditional_branch(is_same_list, bb_skip_recurse, bb_safe_recurse).unwrap();

            // Safe to recurse - different list
            self.builder.position_at_end(bb_safe_recurse);
            let list_ptr_cast = Self::cast_or_self(
                &self.builder,
                elem_ptr.into_pointer_value(),
                list_ptr.get_type(),
                "cast_list",
            );
            self.print_list(list_ptr_cast, &Type::Any)?;
            self.builder.build_unconditional_branch(bb_skip_recurse).unwrap();

            // Skip recursion - same list (would cause infinite recursion)
            self.builder.position_at_end(bb_skip_recurse);
            let placeholder = self.make_cstr("cycle", b"<cycle>\0");
            self.builder.build_call(print_str, &[placeholder.into()], "print_cycle").unwrap();
            self.builder.build_unconditional_branch(bb_tag_tail).unwrap();

            // TUPLE  (recurse)
            self.builder.position_at_end(bb_tuple);
            println!("DEBUG: Handling TUPLE element");

            // Check if the element pointer is the same as the current list pointer
            // to prevent infinite recursion (could happen with nested structures)
            let is_same_ptr = self.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                self.builder.build_ptr_to_int(elem_ptr.into_pointer_value(), ctx.i64_type(), "elem_ptr_int_tup").unwrap(),
                self.builder.build_ptr_to_int(list_ptr, ctx.i64_type(), "list_ptr_int_tup").unwrap(),
                "is_same_ptr_tup"
            ).unwrap();

            let bb_safe_recurse_tup = ctx.append_basic_block(cur_fn, "safe_recurse_tup");
            let bb_skip_recurse_tup = ctx.append_basic_block(cur_fn, "skip_recurse_tup");
            self.builder.build_conditional_branch(is_same_ptr, bb_skip_recurse_tup, bb_safe_recurse_tup).unwrap();

            // Safe to recurse - different pointer
            self.builder.position_at_end(bb_safe_recurse_tup);
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
            self.builder.build_unconditional_branch(bb_skip_recurse_tup).unwrap();

            // Skip recursion - same pointer (would cause infinite recursion)
            self.builder.position_at_end(bb_skip_recurse_tup);
            let placeholder = self.make_cstr("cycle_tup", b"<cycle>\0");
            self.builder.build_call(print_str, &[placeholder.into()], "print_cycle_tup").unwrap();
            self.builder.build_unconditional_branch(bb_tag_tail).unwrap();

            // NONE
            self.builder.position_at_end(bb_none);
            println!("DEBUG: Handling NONE element");
            self.builder
                .build_call(print_str, &[none_lit.into()], "pnone")
                .unwrap();
            self.builder.build_unconditional_branch(bb_tag_tail).unwrap();

            // DEFAULT  (“<Any>” placeholder – should be rare)
            self.builder.position_at_end(bb_deflt);
            println!("DEBUG: Handling DEFAULT case (unknown tag)");
            let ph = self.make_cstr("ph", b"<Any>\0");
            self.builder
                .build_call(print_str, &[ph.into()], "ph_any")
                .unwrap();
            self.builder.build_unconditional_branch(bb_tag_tail).unwrap();

            // Common tail for all tag cases - handle comma, increment index, and branch back to condition
            self.builder.position_at_end(bb_tag_tail);

            // Print ", " if idx < len-1
            let is_last = self.builder
                .build_int_compare(
                    IntPredicate::EQ,
                    idx_val,
                    self.builder
                        .build_int_sub(len_val, i64_t.const_int(1, false), "len-1")
                        .unwrap(),
                    "is_last",
                )
                .unwrap();

            let bb_comma_dyn = ctx.append_basic_block(cur_fn, "comma_dyn");
            let bb_no_dyn = ctx.append_basic_block(cur_fn, "no_comma_dyn");
            self.builder
                .build_conditional_branch(is_last, bb_no_dyn, bb_comma_dyn)
                .unwrap();

            // Print comma if not the last element
            self.builder.position_at_end(bb_comma_dyn);
            self.builder
                .build_call(print_str, &[comma.into()], "pc_dyn")
                .unwrap();
            self.builder.build_unconditional_branch(bb_no_dyn).unwrap();

            // Increment index and continue
            self.builder.position_at_end(bb_no_dyn);
            // idx = idx + 1
            let next = self.builder
                .build_int_add(idx_val, i64_t.const_int(1, false), "inc")
                .unwrap();
            self.builder.build_store(idx_ptr, next).unwrap();
            self.builder.build_unconditional_branch(bb_cond).unwrap();
        }

        // We've moved the comma handling logic into both the static and dynamic paths
        // so we no longer need this section

        // ————————————————————————————————————————————————
        // Skip the old comma handling code since we've moved it into the static and dynamic paths
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

        println!("DEBUG: Added comma if needed");
        self.builder.position_at_end(bb_comma);
        self.builder
            .build_call(print_str, &[comma.into()], "pc")
            .unwrap();
        self.builder.build_unconditional_branch(bb_no).unwrap();

        // We no longer need to increment the index here since we do it in both static and dynamic paths
        self.builder.position_at_end(bb_no);
        println!("DEBUG: Skipping duplicate index increment");
        // Branch directly to the condition block
        self.builder.build_unconditional_branch(bb_cond).unwrap();

        println!("DEBUG: Printed closing bracket");
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
