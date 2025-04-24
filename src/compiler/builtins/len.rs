// len.rs - Registration and compilation of the len() built-in

use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::ExprCompiler;
use crate::compiler::types::Type;
use inkwell::AddressSpace;
use inkwell::values::BasicValueEnum;

impl<'ctx> CompilationContext<'ctx> {
    /// Register the len, string_len, list_len, and dict_len functions
    pub fn register_len_function(&mut self) {
        let context = self.llvm_context;
        let module = &mut self.module;
        let ptr_type = context.ptr_type(AddressSpace::default());

        // len()
        if module.get_function("len").is_none() {
            let fn_type = context.i64_type().fn_type(&[ptr_type.into()], false);
            let function = module.add_function("len", fn_type, None);
            self.functions.insert("len".to_string(), function);
        }

        // list_len()
        if module.get_function("list_len").is_none() {
            let list_len_type = context.i64_type().fn_type(&[ptr_type.into()], false);
            let list_len_fn = module.add_function("list_len", list_len_type, None);
            self.functions.insert("list_len".to_string(), list_len_fn);
        }

        // string_len()
        if module.get_function("string_len").is_none() {
            let string_len_type = context.i64_type().fn_type(&[ptr_type.into()], false);
            let string_len_fn = module.add_function("string_len", string_len_type, None);
            self.functions.insert("string_len".to_string(), string_len_fn);
        }

        // dict_len()
        if module.get_function("dict_len").is_none() {
            let dict_len_type = context.i64_type().fn_type(&[ptr_type.into()], false);
            let dict_len_fn = module.add_function("dict_len", dict_len_type, None);
            self.functions.insert("dict_len".to_string(), dict_len_fn);
        }
    }

    /// Compile a call to the len() function
    pub fn compile_len_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if args.len() != 1 {
            return Err(format!(
                "len() takes exactly one argument ({} given)",
                args.len()
            ));
        }

        let (arg_val, arg_type) = self.compile_expr(&args[0])?;

        // For BoxedAny values, we need to use the appropriate boxed_*_len function
        if arg_type == Type::Any {
            // For BoxedAny, we need to check the type tag and call the appropriate function
            if !arg_val.is_pointer_value() {
                return Err("Expected pointer value for BoxedAny".to_string());
            }

            // Try to get the length using the appropriate function
            if let Ok(v) = self.try_get_boxed_any_length(arg_val) {
                return Ok((v, Type::Int));
            }

            return Err("Cannot determine length of Any type".to_string());
        }

        // For non-BoxedAny values, use the appropriate len function
        let (fn_name, ptr_val) = match arg_type {
            Type::String => ("string_len", arg_val),
            Type::List(_) => ("list_len", arg_val),
            Type::Dict(_, _) => ("dict_len", arg_val),
            _ => return Err(format!("Object of type '{:?}' has no len()", arg_type)),
        };

        let fn_val = self.module.get_function(fn_name)
            .ok_or_else(|| format!("{} function not found", fn_name))?;

        // Ensure pointer
        let arg_ptr = if ptr_val.is_pointer_value() {
            ptr_val.into_pointer_value()
        } else {
            let tmp = self.builder.build_alloca(ptr_val.get_type(), "arg").unwrap();
            self.builder.build_store(tmp, ptr_val).unwrap();
            tmp
        };

        let call_site = self.builder
            .build_call(fn_val, &[arg_ptr.into()], "len_result")
            .unwrap();
        let result = call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to get length result".to_string())?;

        Ok((result, Type::Int))
    }

    /// Try to get the length of a BoxedAny value
    fn try_get_boxed_any_length(
        &self,
        value: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // For BoxedAny values, we can directly call the boxed_any_len function
        let boxed_any_len_fn = self.module.get_function("boxed_any_len")
            .ok_or("boxed_any_len function not found".to_string())?;

        let ptr = value.into_pointer_value();
        let call = self.builder.build_call(boxed_any_len_fn, &[ptr.into()], "boxed_any_len_result").unwrap();
        let result = call.try_as_basic_value().left().ok_or("Failed to get BoxedAny length".to_string())?;

        Ok(result)
    }

    // These methods are no longer used since we're using boxed_any_len instead
    // They are kept here for reference in case we need to reimplement them in the future
}
