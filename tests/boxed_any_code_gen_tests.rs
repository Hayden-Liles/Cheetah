#[cfg(test)]
mod boxed_any_code_gen_tests {
    use cheetah::ast::{Expr, Number, NameConstant};
    use cheetah::compiler::context::CompilationContext;
    use cheetah::compiler::expr::ExprCompiler;
    use cheetah::compiler::types::Type;
    use inkwell::context::Context;

    fn setup_context<'ctx>(context: &'ctx Context) -> CompilationContext<'ctx> {
        CompilationContext::new(context, "test_module")
    }

    fn create_function<'ctx>(ctx: &mut CompilationContext<'ctx>, name: &str) {
        let void_type = ctx.llvm_context.void_type();
        let fn_type = void_type.fn_type(&[], false);
        let function = ctx.module.add_function(name, fn_type, None);
        let basic_block = ctx.llvm_context.append_basic_block(function, "entry");
        ctx.builder.position_at_end(basic_block);
    }

    #[test]
    fn test_boxed_any_integer() {
        let context = Context::create();
        let mut ctx = setup_context(&context);
        create_function(&mut ctx, "test_boxed_any_integer");
        
        // Register the BoxedAny functions
        cheetah::compiler::runtime::boxed_any::register_boxed_any_functions(&context, &mut ctx.module);
        
        // Create an integer expression
        let integer_expr = Expr::Num {
            value: Number::Integer(42),
            line: 1, column: 1
        };
        
        // Compile the expression
        let (int_val, int_type) = ctx.compile_expr(&integer_expr).unwrap();
        
        // Check the type
        assert!(matches!(int_type, Type::Int));
        
        // Check that the value is a pointer (BoxedAny)
        assert!(int_val.is_pointer_value());
    }

    #[test]
    fn test_boxed_any_float() {
        let context = Context::create();
        let mut ctx = setup_context(&context);
        create_function(&mut ctx, "test_boxed_any_float");
        
        // Register the BoxedAny functions
        cheetah::compiler::runtime::boxed_any::register_boxed_any_functions(&context, &mut ctx.module);
        
        // Create a float expression
        let float_expr = Expr::Num {
            value: Number::Float(3.14),
            line: 1, column: 1
        };
        
        // Compile the expression
        let (float_val, float_type) = ctx.compile_expr(&float_expr).unwrap();
        
        // Check the type
        assert!(matches!(float_type, Type::Float));
        
        // Check that the value is a pointer (BoxedAny)
        assert!(float_val.is_pointer_value());
    }

    #[test]
    fn test_boxed_any_bool() {
        let context = Context::create();
        let mut ctx = setup_context(&context);
        create_function(&mut ctx, "test_boxed_any_bool");
        
        // Register the BoxedAny functions
        cheetah::compiler::runtime::boxed_any::register_boxed_any_functions(&context, &mut ctx.module);
        
        // Create a boolean expression (True)
        let true_expr = Expr::NameConstant {
            value: NameConstant::True,
            line: 1, column: 1
        };
        
        // Compile the expression
        let (bool_val, bool_type) = ctx.compile_expr(&true_expr).unwrap();
        
        // Check the type
        assert!(matches!(bool_type, Type::Bool));
        
        // Check that the value is a pointer (BoxedAny)
        assert!(bool_val.is_pointer_value());
    }

    #[test]
    fn test_boxed_any_none() {
        let context = Context::create();
        let mut ctx = setup_context(&context);
        create_function(&mut ctx, "test_boxed_any_none");
        
        // Register the BoxedAny functions
        cheetah::compiler::runtime::boxed_any::register_boxed_any_functions(&context, &mut ctx.module);
        
        // Create a None expression
        let none_expr = Expr::NameConstant {
            value: NameConstant::None,
            line: 1, column: 1
        };
        
        // Compile the expression
        let (none_val, none_type) = ctx.compile_expr(&none_expr).unwrap();
        
        // Check the type
        assert!(matches!(none_type, Type::None));
        
        // Check that the value is a pointer (BoxedAny)
        assert!(none_val.is_pointer_value());
    }
}
