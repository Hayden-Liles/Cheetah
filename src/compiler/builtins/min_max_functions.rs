// min_max_functions.rs - Implementation of the min and max functions

use crate::compiler::context::CompilationContext;
use inkwell::AddressSpace;

impl<'ctx> CompilationContext<'ctx> {
    /// Register the min function
    pub fn register_min_function(&mut self) {
        let context = self.llvm_context;
        let module = &mut self.module;

        // Create min_int function if it doesn't exist
        if module.get_function("min_int").is_none() {
            let min_int_type = context.i64_type().fn_type(
                &[
                    context.i64_type().into(),
                    context.i64_type().into(),
                ],
                false,
            );
            let min_int_fn = module.add_function("min_int", min_int_type, None);
            self.functions.insert("min_int".to_string(), min_int_fn);
        }

        // Create min_float function if it doesn't exist
        if module.get_function("min_float").is_none() {
            let min_float_type = context.f64_type().fn_type(
                &[
                    context.f64_type().into(),
                    context.f64_type().into(),
                ],
                false,
            );
            let min_float_fn = module.add_function("min_float", min_float_type, None);
            self.functions.insert("min_float".to_string(), min_float_fn);
        }

        // Register the min function in our context
        if module.get_function("min").is_none() {
            // The min function takes two arguments of any type
            let ptr_type = context.ptr_type(AddressSpace::default());
            let min_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
            let min_fn = module.add_function("min", min_type, None);
            self.functions.insert("min".to_string(), min_fn);
        }
    }

    /// Register the max function
    pub fn register_max_function(&mut self) {
        let context = self.llvm_context;
        let module = &mut self.module;

        // Create max_int function if it doesn't exist
        if module.get_function("max_int").is_none() {
            let max_int_type = context.i64_type().fn_type(
                &[
                    context.i64_type().into(),
                    context.i64_type().into(),
                ],
                false,
            );
            let max_int_fn = module.add_function("max_int", max_int_type, None);
            self.functions.insert("max_int".to_string(), max_int_fn);
        }

        // Create max_float function if it doesn't exist
        if module.get_function("max_float").is_none() {
            let max_float_type = context.f64_type().fn_type(
                &[
                    context.f64_type().into(),
                    context.f64_type().into(),
                ],
                false,
            );
            let max_float_fn = module.add_function("max_float", max_float_type, None);
            self.functions.insert("max_float".to_string(), max_float_fn);
        }

        // Register the max function in our context
        if module.get_function("max").is_none() {
            // The max function takes two arguments of any type
            let ptr_type = context.ptr_type(AddressSpace::default());
            let max_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
            let max_fn = module.add_function("max", max_type, None);
            self.functions.insert("max".to_string(), max_fn);
        }
    }
}
