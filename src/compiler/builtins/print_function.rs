// print_function.rs - Implementation of the print function

use crate::compiler::context::CompilationContext;
use inkwell::AddressSpace;

impl<'ctx> CompilationContext<'ctx> {
    /// Register the print function
    pub fn register_print_function(&mut self) {
        let context = self.llvm_context;
        let module = &mut self.module;

        // Create print_string function if it doesn't exist
        if module.get_function("print_string").is_none() {
            let print_string_type = context.void_type().fn_type(
                &[context.ptr_type(AddressSpace::default()).into()], // string pointer
                false,
            );
            module.add_function("print_string", print_string_type, None);
        }

        // Create print_int function if it doesn't exist
        if module.get_function("print_int").is_none() {
            let print_int_type = context.void_type().fn_type(
                &[context.i64_type().into()], // integer value
                false,
            );
            module.add_function("print_int", print_int_type, None);
        }

        // Create print_float function if it doesn't exist
        if module.get_function("print_float").is_none() {
            let print_float_type = context.void_type().fn_type(
                &[context.f64_type().into()], // float value
                false,
            );
            module.add_function("print_float", print_float_type, None);
        }

        // Create print_bool function if it doesn't exist
        if module.get_function("print_bool").is_none() {
            let print_bool_type = context.void_type().fn_type(
                &[context.bool_type().into()], // boolean value
                false,
            );
            module.add_function("print_bool", print_bool_type, None);
        }

        // Create println_string function if it doesn't exist
        if module.get_function("println_string").is_none() {
            let println_string_type = context.void_type().fn_type(
                &[context.ptr_type(AddressSpace::default()).into()], // string pointer
                false,
            );
            module.add_function("println_string", println_string_type, None);
        }

        // Register the print function in our context
        if let Some(print_fn) = module.get_function("print_string") {
            self.functions.insert("print".to_string(), print_fn);
        }
    }
}
