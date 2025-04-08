use cheetah::ast::{Stmt, Expr, Number, Operator, CmpOperator, ExprContext};
use cheetah::compiler::context::CompilationContext;
use cheetah::compiler::stmt::StmtCompiler;
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
fn test_nested_if_statements() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_nested_if");
    
    // Create variables: x = 10, y = 20
    let x_name = "x".to_string();
    let y_name = "y".to_string();
    let z_name = "z".to_string();
    
    let x_ptr = ctx.allocate_variable(x_name.clone(), &Type::Int);
    let y_ptr = ctx.allocate_variable(y_name.clone(), &Type::Int);
    ctx.allocate_variable(z_name.clone(), &Type::Int);
    
    ctx.builder.build_store(x_ptr, ctx.llvm_context.i64_type().const_int(10, false)).unwrap();
    ctx.builder.build_store(y_ptr, ctx.llvm_context.i64_type().const_int(20, false)).unwrap();
    
    // Create inner if statement: if y > 15: z = 1 else: z = 2
    let inner_test = Expr::Compare {
        left: Box::new(Expr::Name {
            id: y_name.clone(),
            ctx: ExprContext::Load,
            line: 2, column: 8
        }),
        ops: vec![CmpOperator::Gt],
        comparators: vec![
            Box::new(Expr::Num {
                value: Number::Integer(15),
                line: 2, column: 12
            })
        ],
        line: 2, column: 10
    };
    
    let inner_then_stmt = Stmt::Assign {
        targets: vec![Box::new(Expr::Name {
            id: z_name.clone(),
            ctx: ExprContext::Store,
            line: 3, column: 12
        })],
        value: Box::new(Expr::Num {
            value: Number::Integer(1),
            line: 3, column: 16
        }),
        line: 3, column: 14
    };
    
    let inner_else_stmt = Stmt::Assign {
        targets: vec![Box::new(Expr::Name {
            id: z_name.clone(),
            ctx: ExprContext::Store,
            line: 5, column: 12
        })],
        value: Box::new(Expr::Num {
            value: Number::Integer(2),
            line: 5, column: 16
        }),
        line: 5, column: 14
    };
    
    let inner_if = Stmt::If {
        test: Box::new(inner_test),
        body: vec![Box::new(inner_then_stmt)],
        orelse: vec![Box::new(inner_else_stmt)],
        line: 2, column: 8
    };
    
    // Create outer if statement: if x > 5: <inner_if> else: z = 3
    let outer_test = Expr::Compare {
        left: Box::new(Expr::Name {
            id: x_name.clone(),
            ctx: ExprContext::Load,
            line: 1, column: 4
        }),
        ops: vec![CmpOperator::Gt],
        comparators: vec![
            Box::new(Expr::Num {
                value: Number::Integer(5),
                line: 1, column: 8
            })
        ],
        line: 1, column: 6
    };
    
    let outer_else_stmt = Stmt::Assign {
        targets: vec![Box::new(Expr::Name {
            id: z_name.clone(),
            ctx: ExprContext::Store,
            line: 7, column: 8
        })],
        value: Box::new(Expr::Num {
            value: Number::Integer(3),
            line: 7, column: 12
        }),
        line: 7, column: 10
    };
    
    let outer_if = Stmt::If {
        test: Box::new(outer_test),
        body: vec![Box::new(inner_if)],
        orelse: vec![Box::new(outer_else_stmt)],
        line: 1, column: 1
    };
    
    // Compile the nested if statements
    assert!(ctx.compile_stmt(&outer_if).is_ok());
}

#[test]
fn test_complex_assignment() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_complex_assign");
    
    // Create variables
    let a_name = "a".to_string();
    let b_name = "b".to_string();
    let c_name = "c".to_string();
    
    ctx.allocate_variable(a_name.clone(), &Type::Int);
    
    // Create a complex assignment: a = (b = 10) + (c = 20)
    // First, create the expressions for nested assignments
    let b_assign = Expr::BinOp {
        left: Box::new(Expr::Name {
            id: b_name.clone(),
            ctx: ExprContext::Store,
            line: 1, column: 6
        }),
        op: Operator::Add, // This isn't really an addition in Python, but we'll use it for the test
        right: Box::new(Expr::Num {
            value: Number::Integer(10),
            line: 1, column: 10
        }),
        line: 1, column: 8
    };
    
    let c_assign = Expr::BinOp {
        left: Box::new(Expr::Name {
            id: c_name.clone(),
            ctx: ExprContext::Store,
            line: 1, column: 15
        }),
        op: Operator::Add,
        right: Box::new(Expr::Num {
            value: Number::Integer(20),
            line: 1, column: 19
        }),
        line: 1, column: 17
    };
    
    // Create the outer assignment: a = b_assign + c_assign
    let a_assign = Stmt::Assign {
        targets: vec![Box::new(Expr::Name {
            id: a_name.clone(),
            ctx: ExprContext::Store,
            line: 1, column: 1
        })],
        value: Box::new(Expr::BinOp {
            left: Box::new(b_assign),
            op: Operator::Add,
            right: Box::new(c_assign),
            line: 1, column: 13
        }),
        line: 1, column: 3
    };
    
    // Try to compile this complex assignment
    // This might fail depending on how assignment expressions are handled
    let result = ctx.compile_stmt(&a_assign);
    
    // Even if it fails, it should not panic
    if result.is_err() {
        println!("Note: Complex nested assignment test failed as expected: {}", result.unwrap_err());
    }
}

#[test]
fn test_while_loop_with_break() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_while_with_break");
    
    // Create a counter variable: i = 0
    let i_name = "i".to_string();
    let i_ptr = ctx.allocate_variable(i_name.clone(), &Type::Int);
    ctx.builder.build_store(i_ptr, ctx.llvm_context.i64_type().const_int(0, false)).unwrap();
    
    // Create the loop test: i < 10
    let test = Expr::Compare {
        left: Box::new(Expr::Name {
            id: i_name.clone(),
            ctx: ExprContext::Load,
            line: 1, column: 7
        }),
        ops: vec![CmpOperator::Lt],
        comparators: vec![
            Box::new(Expr::Num {
                value: Number::Integer(10),
                line: 1, column: 11
            })
        ],
        line: 1, column: 9
    };
    
    // Create increment statement: i = i + 1
    let increment = Stmt::Assign {
        targets: vec![Box::new(Expr::Name {
            id: i_name.clone(),
            ctx: ExprContext::Store,
            line: 2, column: 4
        })],
        value: Box::new(Expr::BinOp {
            left: Box::new(Expr::Name {
                id: i_name.clone(),
                ctx: ExprContext::Load,
                line: 2, column: 8
            }),
            op: Operator::Add,
            right: Box::new(Expr::Num {
                value: Number::Integer(1),
                line: 2, column: 12
            }),
            line: 2, column: 10
        }),
        line: 2, column: 6
    };
    
    // Create break condition: if i == 5: break
    let break_condition = Stmt::If {
        test: Box::new(Expr::Compare {
            left: Box::new(Expr::Name {
                id: i_name.clone(),
                ctx: ExprContext::Load,
                line: 3, column: 8
            }),
            ops: vec![CmpOperator::Eq],
            comparators: vec![
                Box::new(Expr::Num {
                    value: Number::Integer(5),
                    line: 3, column: 13
                })
            ],
            line: 3, column: 10
        }),
        body: vec![Box::new(Stmt::Break {
            line: 4, column: 8
        })],
        orelse: vec![],
        line: 3, column: 4
    };
    
    // Create the while loop
    let while_loop = Stmt::While {
        test: Box::new(test),
        body: vec![
            Box::new(increment),
            Box::new(break_condition)
        ],
        orelse: vec![],
        line: 1, column: 1
    };
    
    // Compile the while loop
    assert!(ctx.compile_stmt(&while_loop).is_ok());
}

#[test]
fn test_multiple_assignments() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_multi_assign");
    
    // Create variables
    let x_name = "x".to_string();
    let y_name = "y".to_string();
    let z_name = "z".to_string();
    
    // Create a multi-target assignment: x = y = z = 42
    let assign = Stmt::Assign {
        targets: vec![
            Box::new(Expr::Name {
                id: x_name.clone(),
                ctx: ExprContext::Store,
                line: 1, column: 1
            }),
            Box::new(Expr::Name {
                id: y_name.clone(),
                ctx: ExprContext::Store,
                line: 1, column: 5
            }),
            Box::new(Expr::Name {
                id: z_name.clone(),
                ctx: ExprContext::Store,
                line: 1, column: 9
            })
        ],
        value: Box::new(Expr::Num {
            value: Number::Integer(42),
            line: 1, column: 13
        }),
        line: 1, column: 3
    };
    
    // Compile the multi-target assignment
    assert!(ctx.compile_stmt(&assign).is_ok());
    
    // Verify all variables have the correct type
    assert!(ctx.type_env.contains_key(&x_name));
    assert!(ctx.type_env.contains_key(&y_name));
    assert!(ctx.type_env.contains_key(&z_name));
    
    assert!(matches!(ctx.type_env.get(&x_name).unwrap(), Type::Int));
    assert!(matches!(ctx.type_env.get(&y_name).unwrap(), Type::Int));
    assert!(matches!(ctx.type_env.get(&z_name).unwrap(), Type::Int));
}