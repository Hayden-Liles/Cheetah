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

        // Get the print function based on whether we're using BoxedAny values
        let print_fn = self.module.get_function(
            if self.use_boxed_values { "print_boxed_any" } else { "print_string" }
        ).ok_or("print runtime missing")?;

        // If we have no arguments, just print a newline
        if args.is_empty() {
            let nl = self.make_cstr("nl", b"\n\0");

            if self.use_boxed_values {
                // Box the newline string and print it
                let (boxed_nl, _) = self.maybe_box(nl.into(), &Type::String)?;
                self.builder.build_call(print_fn, &[boxed_nl.into()], "print_nl")
                    .map_err(|e| format!("Failed to call print function: {}", e))?;
            } else {
                // Use the regular println_string function
                let println_fn = self.module.get_function("println_string")
                    .ok_or("println_string function not found")?;
                self.builder.build_call(println_fn, &[nl.into()], "print_nl")
                    .map_err(|e| format!("Failed to call println function: {}", e))?;
            }

            return Ok((self.llvm_context.i64_type().const_zero().into(), Type::None));
        }

        // Process each argument
        for (i, arg) in args.iter().enumerate() {
            let (val, ty) = self.compile_expr(arg)?;

            // Box the value if needed
            let (boxed_val, _) = self.maybe_box(val, &ty)?;

            // Call the print function
            self.builder.build_call(print_fn, &[boxed_val.into()], "call_print")
                .map_err(|e| format!("Failed to call print function: {}", e))?;

            // Add space between arguments
            if i + 1 < args.len() {
                let space_str = self.make_cstr("sp", b" \0");
                let (boxed_space, _) = self.maybe_box(space_str.into(), &Type::String)?;
                self.builder.build_call(print_fn, &[boxed_space.into()], "print_space")
                    .map_err(|e| format!("Failed to call print function for space: {}", e))?;
            }
        }

        // Add final newline
        let nl = self.make_cstr("nl", b"\n\0");

        if self.use_boxed_values {
            // Box the newline string and print it
            let (boxed_nl, _) = self.maybe_box(nl.into(), &Type::String)?;
            self.builder.build_call(print_fn, &[boxed_nl.into()], "print_newline")
                .map_err(|e| format!("Failed to call print function for newline: {}", e))?;
        } else {
            // Use the regular println_string function
            let println_fn = self.module.get_function("println_string")
                .ok_or("println_string function not found")?;
            self.builder.build_call(println_fn, &[nl.into()], "print_nl")
                .map_err(|e| format!("Failed to call println function: {}", e))?;
        }

        Ok((self.llvm_context.i64_type().const_zero().into(), Type::None))
    }
}
