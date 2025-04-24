use cheetah::ast::{Stmt, Expr, Number, ExprContext};
use cheetah::compiler::context::CompilationContext;
use cheetah::compiler::stmt::StmtCompiler;
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
fn test_expression_statement() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_expr_stmt");

    // Create a simple expression statement (just an integer literal)
    let int_expr = Expr::Num {
        value: Number::Integer(42),
        line: 1, column: 1
    };

    let expr_stmt = Stmt::Expr {
        value: Box::new(int_expr),
        line: 1, column: 1
    };

    // This should compile without error
    assert!(ctx.compile_stmt(&expr_stmt).is_ok());
}

#[test]
fn test_assignment_statement() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_assign_stmt");

    // Create an integer literal
    let int_expr = Expr::Num {
        value: Number::Integer(42),
        line: 1, column: 5
    };

    // First, allocate storage for the variable
    let var_name = "x".to_string();
    ctx.allocate_variable(var_name.clone(), &Type::Any);

    // Create a target name
    let target_expr = Expr::Name {
        id: var_name,
        ctx: ExprContext::Store,
        line: 1, column: 1
    };

    // Create an assignment statement: x = 42
    let assign_stmt = Stmt::Assign {
        targets: vec![Box::new(target_expr)],
        value: Box::new(int_expr),
        line: 1, column: 3
    };

    // This should compile without error
    assert!(ctx.compile_stmt(&assign_stmt).is_ok());

    // Verify the variable was created with the correct type
    assert!(ctx.type_env.contains_key("x"));
    assert!(matches!(ctx.type_env.get("x").unwrap(), Type::Any));
}

#[test]
fn test_variable_declaration() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_var_decl");

    // Create an integer value
    let int_expr = Expr::Num {
        value: Number::Integer(42),
        line: 1, column: 5
    };

    // Compile the integer expression
    let (int_val, int_type) = ctx.compile_expr(&int_expr).unwrap();

    // Declare a variable with the integer value
    let result = ctx.declare_variable(
        "test_var".to_string(),
        int_val,
        &int_type
    );

    assert!(result.is_ok());

    // Verify the variable was created
    assert!(ctx.type_env.contains_key("test_var"));
    assert!(ctx.variables.contains_key("test_var"));
}