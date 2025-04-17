// range_function.rs - Implementation of the range function

use crate::compiler::context::CompilationContext;
use crate::compiler::Type;

impl<'ctx> CompilationContext<'ctx> {
    /// Register the range function
    pub fn register_range_function(&mut self) {
        // Check if the range function is already registered
        if self.functions.contains_key("range") {
            // Range function is already registered, no need to do anything
            return;
        }

        let context = self.llvm_context;
        let module = &mut self.module;

        // Create range_1 function (range with stop only)
        if module.get_function("range_1").is_none() {
            let range_1_type = context.i64_type().fn_type(&[context.i64_type().into()], false);
            let range_1_fn = module.add_function("range_1", range_1_type, None);
            // Register the function in our context
            self.functions.insert("range_1".to_string(), range_1_fn);
        }

        // Create range_2 function (range with start and stop)
        if module.get_function("range_2").is_none() {
            let range_2_type = context.i64_type().fn_type(
                &[
                    context.i64_type().into(), // start
                    context.i64_type().into(), // stop
                ],
                false,
            );
            let range_2_fn = module.add_function("range_2", range_2_type, None);
            // Register the function in our context
            self.functions.insert("range_2".to_string(), range_2_fn);
        }

        // Create range_3 function (range with start, stop, and step)
        if module.get_function("range_3").is_none() {
            let range_3_type = context.i64_type().fn_type(
                &[
                    context.i64_type().into(), // start
                    context.i64_type().into(), // stop
                    context.i64_type().into(), // step
                ],
                false,
            );
            let range_3_fn = module.add_function("range_3", range_3_type, None);
            // Register the function in our context
            self.functions.insert("range_3".to_string(), range_3_fn);
        }

        // Register the default range function (with one argument)
        if let Some(range_fn) = module.get_function("range_1") {
            self.functions.insert("range".to_string(), range_fn);

            // Register the range function in the type environment
            self.register_variable("range".to_string(), Type::function(
                vec![Type::Int],
                Type::List(Box::new(Type::Int))
            ));

            // Add the range function to the global scope
            if let Some(global_scope) = self.scope_stack.global_scope_mut() {
                // We need to create a pointer for the function
                // This is just a placeholder since we don't actually need to access it
                let fn_ptr = range_fn.as_global_value().as_pointer_value();

                // Add the range function to the global scope
                global_scope.add_variable(
                    "range".to_string(),
                    fn_ptr,
                    Type::function(vec![Type::Int], Type::List(Box::new(Type::Int)))
                );
            }
        }
    }
}
