// len_function.rs - Implementation of the len function

use crate::compiler::context::CompilationContext;
use inkwell::AddressSpace;

impl<'ctx> CompilationContext<'ctx> {
    /// Register the len function
    pub fn register_len_function(&mut self) {
        let context = self.llvm_context;
        let module = &mut self.module;

        // Create the len function type for strings
        let ptr_type = context.ptr_type(AddressSpace::default());
        let fn_type = context.i64_type().fn_type(&[ptr_type.into()], false);

        // Add the function to the module if it doesn't exist
        if module.get_function("len").is_none() {
            let function = module.add_function("len", fn_type, None);
            // Register the function in our context
            self.functions.insert("len".to_string(), function);
        }

        // Also register the len function for lists
        if module.get_function("list_len").is_none() {
            // Create the list_len function if it doesn't exist
            let list_len_type = context.i64_type().fn_type(&[ptr_type.into()], false);
            let list_len_fn = module.add_function("list_len", list_len_type, None);
            // Register the list_len function
            self.functions.insert("list_len".to_string(), list_len_fn);
        }

        // Register the string_len function
        if module.get_function("string_len").is_none() {
            // Create the string_len function if it doesn't exist
            let string_len_type = context.i64_type().fn_type(&[ptr_type.into()], false);
            let string_len_fn = module.add_function("string_len", string_len_type, None);
            // Register the string_len function
            self.functions.insert("string_len".to_string(), string_len_fn);
        }
    }
}
