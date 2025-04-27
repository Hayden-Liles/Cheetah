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
