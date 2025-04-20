// min_max.rs - Registration and compilation of the min() and max() built-ins

use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::ExprCompiler;
use crate::compiler::types::Type;
use inkwell::AddressSpace;
use inkwell::values::BasicValueEnum;

impl<'ctx> CompilationContext<'ctx> {
    /// Register min, max, and their type‑specific variants
    pub fn register_min_max_functions(&mut self) {
        let ctx = self.llvm_context;
        let m = &mut self.module;

        // min_int, min_float
        if m.get_function("min_int").is_none() {
            let t = ctx.i64_type().fn_type(&[ctx.i64_type().into(), ctx.i64_type().into()], false);
            let f = m.add_function("min_int", t, None);
            self.functions.insert("min_int".into(), f);
        }
        if m.get_function("min_float").is_none() {
            let t = ctx.f64_type().fn_type(&[ctx.f64_type().into(), ctx.f64_type().into()], false);
            let f = m.add_function("min_float", t, None);
            self.functions.insert("min_float".into(), f);
        }

        // max_int, max_float
        if m.get_function("max_int").is_none() {
            let t = ctx.i64_type().fn_type(&[ctx.i64_type().into(), ctx.i64_type().into()], false);
            let f = m.add_function("max_int", t, None);
            self.functions.insert("max_int".into(), f);
        }
        if m.get_function("max_float").is_none() {
            let t = ctx.f64_type().fn_type(&[ctx.f64_type().into(), ctx.f64_type().into()], false);
            let f = m.add_function("max_float", t, None);
            self.functions.insert("max_float".into(), f);
        }

        // min(ptr, ptr)
        if m.get_function("min").is_none() {
            let ptr_t = ctx.ptr_type(AddressSpace::default());
            let t = ptr_t.fn_type(&[ptr_t.into(), ptr_t.into()], false);
            let f = m.add_function("min", t, None);
            self.functions.insert("min".into(), f);
        }

        // max(ptr, ptr)
        if m.get_function("max").is_none() {
            let ptr_t = ctx.ptr_type(AddressSpace::default());
            let t = ptr_t.fn_type(&[ptr_t.into(), ptr_t.into()], false);
            let f = m.add_function("max", t, None);
            self.functions.insert("max".into(), f);
        }
    }

    /// Compile a call to min(a, b)
    pub fn compile_min_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if args.len() != 2 {
            return Err(format!("min() takes exactly two arguments ({} given)", args.len()));
        }
        let (v1, t1) = self.compile_expr(&args[0])?;
        let (v2, t2) = self.compile_expr(&args[1])?;

        match (&t1, &t2) {
            (Type::Int, Type::Int) => self.call_int_fn("min_int", v1, v2).map(|r| (r, Type::Int)),
            (Type::Float, Type::Float) => self.call_float_fn("min_float", v1, v2).map(|r| (r, Type::Float)),
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => {
                // Always promote to float
                let f1 = if let Type::Int = t1 {
                    let i = v1.into_int_value();
                    self.builder.build_signed_int_to_float(i, self.llvm_context.f64_type(), "i2f").unwrap().into()
                } else {
                    v1
                };
                let f2 = if let Type::Int = t2 {
                    let i = v2.into_int_value();
                    self.builder.build_signed_int_to_float(i, self.llvm_context.f64_type(), "i2f").unwrap().into()
                } else {
                    v2
                };
                self.call_float_fn("min_float", f1, f2).map(|r| (r, Type::Float))
            }
            _ => Err(format!("min() not supported for types {:?} and {:?}", t1, t2)),
        }
    }

    /// Compile a call to max(a, b)
    pub fn compile_max_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if args.len() != 2 {
            return Err(format!("max() takes exactly two arguments ({} given)", args.len()));
        }
        let (v1, t1) = self.compile_expr(&args[0])?;
        let (v2, t2) = self.compile_expr(&args[1])?;

        match (&t1, &t2) {
            (Type::Int, Type::Int) => self.call_int_fn("max_int", v1, v2).map(|r| (r, Type::Int)),
            (Type::Float, Type::Float) => self.call_float_fn("max_float", v1, v2).map(|r| (r, Type::Float)),
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => {
                let f1 = if let Type::Int = t1 {
                    let i = v1.into_int_value();
                    self.builder.build_signed_int_to_float(i, self.llvm_context.f64_type(), "i2f").unwrap().into()
                } else {
                    v1
                };
                let f2 = if let Type::Int = t2 {
                    let i = v2.into_int_value();
                    self.builder.build_signed_int_to_float(i, self.llvm_context.f64_type(), "i2f").unwrap().into()
                } else {
                    v2
                };
                self.call_float_fn("max_float", f1, f2).map(|r| (r, Type::Float))
            }
            _ => Err(format!("max() not supported for types {:?} and {:?}", t1, t2)),
        }
    }

    /// Helper to call an integer min/max function
    fn call_int_fn(
        &mut self,
        name: &str,
        a: BasicValueEnum<'ctx>,
        b: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let f = self.module.get_function(name)
            .ok_or_else(|| format!("{} not found", name))?;
        let ai = a.into_int_value();
        let bi = b.into_int_value();
        let call = self.builder.build_call(f, &[ai.into(), bi.into()], "int_cmp").unwrap();
        call.try_as_basic_value().left().ok_or("Failed int call".to_string())
    }

    /// Helper to call a float min/max function
    fn call_float_fn(
        &mut self,
        name: &str,
        a: BasicValueEnum<'ctx>,
        b: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let f = self.module.get_function(name)
            .ok_or_else(|| format!("{} not found", name))?;
        let af = a.into_float_value();
        let bf = b.into_float_value();
        let call = self.builder.build_call(f, &[af.into(), bf.into()], "flt_cmp").unwrap();
        call.try_as_basic_value().left().ok_or("Failed float call".to_string())
    }
}
