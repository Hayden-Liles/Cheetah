use cheetah::ast::{Expr, Number, NameConstant, Operator, UnaryOperator, CmpOperator, ExprContext};
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
fn test_nested_expressions() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_nested_expr");
    
    // Create a nested expression: (10 + 20) * (5 - 2)
    let left = Expr::BinOp {
        left: Box::new(Expr::Num {
            value: Number::Integer(10),
            line: 1, column: 1
        }),
        op: Operator::Add,
        right: Box::new(Expr::Num {
            value: Number::Integer(20),
            line: 1, column: 5
        }),
        line: 1, column: 3
    };
    
    let right = Expr::BinOp {
        left: Box::new(Expr::Num {
            value: Number::Integer(5),
            line: 1, column: 11
        }),
        op: Operator::Sub,
        right: Box::new(Expr::Num {
            value: Number::Integer(2),
            line: 1, column: 15
        }),
        line: 1, column: 13
    };
    
    let expr = Expr::BinOp {
        left: Box::new(left),
        op: Operator::Mult,
        right: Box::new(right),
        line: 1, column: 8
    };
    
    // Compile the nested expression
    let (val, ty) = ctx.compile_expr(&expr).unwrap();
    
    // The result should be an integer
    assert!(matches!(ty, Type::Int));
    assert!(val.is_int_value());
    
    // If the compiler does constant folding correctly, the result should be (10+20)*(5-2) = 30*3 = 90
    if let Some(const_val) = val.into_int_value().get_zero_extended_constant() {
        assert_eq!(const_val, 90);
    }
}

#[test]
fn test_comparison_chains() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_comparison_chains");
    
    // Create variables to use in the comparison
    let var_a = "a".to_string();
    let var_b = "b".to_string();
    let var_c = "c".to_string();
    
    // Allocate and initialize variables: a = 10, b = 20, c = 30
    let a_ptr = ctx.allocate_variable(var_a.clone(), &Type::Int);
    let b_ptr = ctx.allocate_variable(var_b.clone(), &Type::Int);
    let c_ptr = ctx.allocate_variable(var_c.clone(), &Type::Int);
    
    ctx.builder.build_store(a_ptr, ctx.llvm_context.i64_type().const_int(10, false)).unwrap();
    ctx.builder.build_store(b_ptr, ctx.llvm_context.i64_type().const_int(20, false)).unwrap();
    ctx.builder.build_store(c_ptr, ctx.llvm_context.i64_type().const_int(30, false)).unwrap();
    
    // Create a comparison chain: a < b < c
    let expr = Expr::Compare {
        left: Box::new(Expr::Name {
            id: var_a.clone(),
            ctx: ExprContext::Load,
            line: 1, column: 1
        }),
        ops: vec![CmpOperator::Lt, CmpOperator::Lt],
        comparators: vec![
            Box::new(Expr::Name {
                id: var_b.clone(),
                ctx: ExprContext::Load,
                line: 1, column: 5
            }),
            Box::new(Expr::Name {
                id: var_c.clone(),
                ctx: ExprContext::Load,
                line: 1, column: 9
            })
        ],
        line: 1, column: 3
    };
    
    // Compile the comparison chain
    let (val, ty) = ctx.compile_expr(&expr).unwrap();
    
    // The result should be a boolean
    assert!(matches!(ty, Type::Bool));
    assert!(val.is_int_value());
}

#[test]
fn test_mixed_type_operations() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_mixed_types");
    
    // Create mixed-type expressions: 10 + 3.14
    let expr = Expr::BinOp {
        left: Box::new(Expr::Num {
            value: Number::Integer(10),
            line: 1, column: 1
        }),
        op: Operator::Add,
        right: Box::new(Expr::Num {
            value: Number::Float(3.14),
            line: 1, column: 5
        }),
        line: 1, column: 3
    };
    
    // Compile the expression
    let (val, ty) = ctx.compile_expr(&expr).unwrap();
    
    // The result should be a float (due to type coercion)
    assert!(matches!(ty, Type::Float));
    assert!(val.is_float_value());
}

#[test]
fn test_boolean_operations() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_boolean_ops");
    
    // Test boolean "not" operator
    let not_expr = Expr::UnaryOp {
        op: UnaryOperator::Not,
        operand: Box::new(Expr::NameConstant {
            value: NameConstant::True,
            line: 1, column: 5
        }),
        line: 1, column: 1
    };
    
    let (not_val, not_type) = ctx.compile_expr(&not_expr).unwrap();
    assert!(matches!(not_type, Type::Bool));
    assert!(not_val.is_int_value());
    
    // Test boolean operation on non-boolean type (should convert to bool)
    let non_bool_not = Expr::UnaryOp {
        op: UnaryOperator::Not,
        operand: Box::new(Expr::Num {
            value: Number::Integer(42),
            line: 1, column: 5
        }),
        line: 1, column: 1
    };
    
    let (val, ty) = ctx.compile_expr(&non_bool_not).unwrap();
    assert!(matches!(ty, Type::Bool));
    assert!(val.is_int_value());
}

#[test]
fn test_variable_updates() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_var_updates");
    
    // Create a variable x = 10
    let var_name = "x".to_string();
    let var_ptr = ctx.allocate_variable(var_name.clone(), &Type::Int);
    ctx.builder.build_store(var_ptr, ctx.llvm_context.i64_type().const_int(10, false)).unwrap();
    
    // Create reference to the variable
    let var_expr = Expr::Name {
        id: var_name.clone(),
        ctx: ExprContext::Load,
        line: 1, column: 1
    };
    
    // Compile the variable reference
    let (val, ty) = ctx.compile_expr(&var_expr).unwrap();
    assert!(matches!(ty, Type::Int));
    assert!(val.is_int_value());
    
    // The value should be 10
    if let Some(const_val) = val.into_int_value().get_zero_extended_constant() {
        assert_eq!(const_val, 10);
    }
}