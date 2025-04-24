use cheetah::ast::{Expr, Number, NameConstant, Operator, UnaryOperator, ExprContext};
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
fn test_number_compilation() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_numbers");

    // Test integer literal
    let integer_expr = Expr::Num {
        value: Number::Integer(42),
        line: 1, column: 1
    };
    let (int_val, int_type) = ctx.compile_expr(&integer_expr).unwrap();
    assert!(matches!(int_type, Type::Any));
    assert!(int_val.is_pointer_value());
    // BoxedAny values are pointers, so we can't check the actual value

    // Test float literal
    let float_expr = Expr::Num {
        value: Number::Float(3.14),
        line: 1, column: 1
    };
    let (float_val, float_type) = ctx.compile_expr(&float_expr).unwrap();
    assert!(matches!(float_type, Type::Any));
    assert!(float_val.is_pointer_value());
}

#[test]
fn test_constant_compilation() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_constants");

    // Test True constant
    let true_expr = Expr::NameConstant {
        value: NameConstant::True,
        line: 1, column: 1
    };
    let (true_val, true_type) = ctx.compile_expr(&true_expr).unwrap();
    assert!(matches!(true_type, Type::Any));
    assert!(true_val.is_pointer_value());
    // BoxedAny values are pointers, so we can't check the actual value

    // Test False constant
    let false_expr = Expr::NameConstant {
        value: NameConstant::False,
        line: 1, column: 1
    };
    let (false_val, false_type) = ctx.compile_expr(&false_expr).unwrap();
    assert!(matches!(false_type, Type::Any));
    assert!(false_val.is_pointer_value());
    // BoxedAny values are pointers, so we can't check the actual value

    // Test None constant
    let none_expr = Expr::NameConstant {
        value: NameConstant::None,
        line: 1, column: 1
    };
    let (none_val, none_type) = ctx.compile_expr(&none_expr).unwrap();
    assert!(matches!(none_type, Type::Any));
    assert!(none_val.is_pointer_value());
    // BoxedAny None is a pointer to a BoxedAny struct, not a null pointer
}

#[test]
fn test_unary_operations() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_unary_ops");

    // Test logical not
    let true_expr = Expr::NameConstant {
        value: NameConstant::True,
        line: 1, column: 1
    };
    let not_expr = Expr::UnaryOp {
        op: UnaryOperator::Not,
        operand: Box::new(true_expr),
        line: 1, column: 1
    };
    let (not_val, not_type) = ctx.compile_expr(&not_expr).unwrap();
    assert!(matches!(not_type, Type::Any));
    assert!(not_val.is_pointer_value());

    // Test numeric negation
    let int_expr = Expr::Num {
        value: Number::Integer(42),
        line: 1, column: 1
    };
    let neg_expr = Expr::UnaryOp {
        op: UnaryOperator::USub,
        operand: Box::new(int_expr),
        line: 1, column: 1
    };
    let (neg_val, neg_type) = ctx.compile_expr(&neg_expr).unwrap();
    assert!(matches!(neg_type, Type::Any));
    assert!(neg_val.is_pointer_value());
}

#[test]
fn test_binary_operations() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_binary_ops");

    // Create two integer literals
    let int_expr1 = Expr::Num {
        value: Number::Integer(40),
        line: 1, column: 1
    };
    let int_expr2 = Expr::Num {
        value: Number::Integer(2),
        line: 1, column: 5
    };

    // Test addition
    let add_expr = Expr::BinOp {
        left: Box::new(int_expr1.clone()),
        op: Operator::Add,
        right: Box::new(int_expr2.clone()),
        line: 1, column: 3
    };

    let (add_val, add_type) = ctx.compile_expr(&add_expr).unwrap();
    assert!(matches!(add_type, Type::Any));
    assert!(add_val.is_pointer_value());

    // Test multiplication
    let mul_expr = Expr::BinOp {
        left: Box::new(int_expr1.clone()),
        op: Operator::Mult,
        right: Box::new(int_expr2.clone()),
        line: 1, column: 3
    };

    let (mul_val, mul_type) = ctx.compile_expr(&mul_expr).unwrap();
    assert!(matches!(mul_type, Type::Any));
    assert!(mul_val.is_pointer_value());

    // Test mixed types (int + float)
    let float_expr = Expr::Num {
        value: Number::Float(3.5),
        line: 1, column: 5
    };

    let mixed_expr = Expr::BinOp {
        left: Box::new(int_expr1.clone()),
        op: Operator::Add,
        right: Box::new(float_expr),
        line: 1, column: 3
    };

    let (mixed_val, mixed_type) = ctx.compile_expr(&mixed_expr).unwrap();
    assert!(matches!(mixed_type, Type::Any));
    assert!(mixed_val.is_pointer_value());
}

#[test]
fn test_variable_references() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_variables");

    // Create an integer literal
    let int_expr = Expr::Num {
        value: Number::Integer(42),
        line: 1, column: 1
    };

    // Compile the integer literal
    let (int_val, int_type) = ctx.compile_expr(&int_expr).unwrap();

    // Allocate storage for a variable
    let var_name = "test_var".to_string();
    let var_ptr = ctx.allocate_variable(var_name.clone(), &int_type);

    // Store the integer value in the variable
    ctx.builder.build_store(var_ptr, int_val).unwrap();

    // Create a variable reference expression
    let var_expr = Expr::Name {
        id: var_name.clone(),
        ctx: ExprContext::Load,
        line: 2, column: 1
    };

    // Compile the variable reference
    let (var_val, var_type) = ctx.compile_expr(&var_expr).unwrap();
    assert!(matches!(var_type, Type::Any));
    assert!(var_val.is_pointer_value());
}