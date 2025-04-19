// len_function.rs - Implementation of the len function

use crate::compiler::context::CompilationContext;
use inkwell::AddressSpace;

impl<'ctx> CompilationContext<'ctx> {
    /// Register the len function
    pub fn register_len_function(&mut self) {
        let context = self.llvm_context;
        let module = &mut self.module;

        let ptr_type = context.ptr_type(AddressSpace::default());
        let fn_type = context.i64_type().fn_type(&[ptr_type.into()], false);

        if module.get_function("len").is_none() {
            let function = module.add_function("len", fn_type, None);
            self.functions.insert("len".to_string(), function);
        }

        if module.get_function("list_len").is_none() {
            let list_len_type = context.i64_type().fn_type(&[ptr_type.into()], false);
            let list_len_fn = module.add_function("list_len", list_len_type, None);
            self.functions.insert("list_len".to_string(), list_len_fn);
        }

        if module.get_function("string_len").is_none() {
            let string_len_type = context.i64_type().fn_type(&[ptr_type.into()], false);
            let string_len_fn = module.add_function("string_len", string_len_type, None);
            self.functions
                .insert("string_len".to_string(), string_len_fn);
        }
    }
}
