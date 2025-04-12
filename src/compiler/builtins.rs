// builtins.rs - Built-in functions for the Cheetah compiler

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

        // Add the function to the module
        let function = module.add_function("len", fn_type, None);

        // Register the function in our context
        self.functions.insert("len".to_string(), function);
    }
}
