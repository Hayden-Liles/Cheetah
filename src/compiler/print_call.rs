use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::ExprCompiler;
use crate::compiler::types::Type;
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::AddressSpace;

impl<'ctx> CompilationContext<'ctx> {
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

    pub fn compile_print_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Lookup the runtime print functions
        let print_str_fn = self.module.get_function("print_string")
            .ok_or("print_string function not found".to_string())?;
        let print_int_fn = self.module.get_function("print_int")
            .ok_or("print_int function not found".to_string())?;
        let print_float_fn = self.module.get_function("print_float")
            .ok_or("print_float function not found".to_string())?;
        let print_bool_fn = self.module.get_function("print_bool")
            .ok_or("print_bool function not found".to_string())?;
        let println_fn = self.module.get_function("println_string")
            .ok_or("println_string function not found".to_string())?;

        // Emit each argument without newline, separated by spaces
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
                    let _ = self.builder.build_call(print_float_fn, &[val.into()], "print_float");
                }
                Type::Bool => {
                    let _ = self.builder.build_call(print_bool_fn, &[val.into()], "print_bool");
                }
                other => {
                    let placeholder = format!("<{:?}>\0", other);
                    let ptr = self.make_cstr("placeholder", placeholder.as_bytes());
                    let _ = self.builder.build_call(print_str_fn, &[ptr.into()], "print_placeholder");
                }
            }

            // Space between arguments
            if i + 1 < args.len() {
                let ptr = self.make_cstr("space", b" \0");
                let _ = self.builder.build_call(print_str_fn, &[ptr.into()], "print_space");
            }
        }

        // Single newline at the end
        let nl_ptr = self.make_cstr("newline", b"\n\0");
        let _ = self.builder.build_call(println_fn, &[nl_ptr.into()], "print_end");

        Ok((self.llvm_context.i64_type().const_zero().into(), Type::None))
    }
}
