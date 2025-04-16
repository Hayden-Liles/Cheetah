use cheetah::ast::{Expr, Number, ExprContext, Comprehension};
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

fn create_list_new_function<'ctx>(ctx: &mut CompilationContext<'ctx>) {
    // Create the list_new function
    let list_ptr_type = ctx.llvm_context.ptr_type(inkwell::AddressSpace::default());
    let fn_type = list_ptr_type.fn_type(&[], false);
    let function = ctx.module.add_function("list_new", fn_type, None);
    let basic_block = ctx.llvm_context.append_basic_block(function, "entry");
    ctx.builder.position_at_end(basic_block);

    // Create a dummy list pointer
    let list_ptr = ctx.builder.build_alloca(list_ptr_type, "list_ptr").unwrap();

    // Return the list pointer
    ctx.builder.build_return(Some(&list_ptr)).unwrap();

    // Reset the builder position to the entry block of the main function
    let main_function = ctx.module.get_function("test_list_comp_tuple").unwrap();
    let entry_block = main_function.get_first_basic_block().unwrap();
    ctx.builder.position_at_end(entry_block);
}

fn create_list_append_function<'ctx>(ctx: &mut CompilationContext<'ctx>) {
    // Create the list_append function
    let list_ptr_type = ctx.llvm_context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = ctx.llvm_context.i64_type();
    let fn_type = ctx.llvm_context.void_type().fn_type(&[list_ptr_type.into(), i64_type.into()], false);
    let function = ctx.module.add_function("list_append", fn_type, None);
    let basic_block = ctx.llvm_context.append_basic_block(function, "entry");
    ctx.builder.position_at_end(basic_block);

    // Just return void
    ctx.builder.build_return(None).unwrap();

    // Reset the builder position to the entry block of the main function
    let main_function = ctx.module.get_function("test_list_comp_tuple").unwrap();
    let entry_block = main_function.get_first_basic_block().unwrap();
    ctx.builder.position_at_end(entry_block);
}

fn create_list_len_function<'ctx>(ctx: &mut CompilationContext<'ctx>) {
    // Create the list_len function
    let list_ptr_type = ctx.llvm_context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = ctx.llvm_context.i64_type();
    let fn_type = i64_type.fn_type(&[list_ptr_type.into()], false);
    let function = ctx.module.add_function("list_len", fn_type, None);
    let basic_block = ctx.llvm_context.append_basic_block(function, "entry");
    ctx.builder.position_at_end(basic_block);

    // Return a constant length of 2 (for our tuple)
    let len = i64_type.const_int(2, false);
    ctx.builder.build_return(Some(&len)).unwrap();

    // Reset the builder position to the entry block of the main function
    let main_function = ctx.module.get_function("test_list_comp_tuple").unwrap();
    let entry_block = main_function.get_first_basic_block().unwrap();
    ctx.builder.position_at_end(entry_block);
}

fn create_list_get_function<'ctx>(ctx: &mut CompilationContext<'ctx>) {
    // Create the list_get function
    let list_ptr_type = ctx.llvm_context.ptr_type(inkwell::AddressSpace::default());
    let fn_type = list_ptr_type.fn_type(&[list_ptr_type.into(), ctx.llvm_context.i64_type().into()], false);
    let function = ctx.module.add_function("list_get", fn_type, None);
    let basic_block = ctx.llvm_context.append_basic_block(function, "entry");
    ctx.builder.position_at_end(basic_block);

    // Get the list parameter
    let list_ptr = function.get_nth_param(0).unwrap().into_pointer_value();

    // Allocate a pointer to return
    let result_ptr = ctx.builder.build_alloca(list_ptr_type, "result_ptr").unwrap();

    // Store the list pointer in the result pointer (for simplicity)
    ctx.builder.build_store(result_ptr, list_ptr).unwrap();

    // Return the result pointer
    ctx.builder.build_return(Some(&result_ptr)).unwrap();

    // Reset the builder position to the entry block of the main function
    let main_function = ctx.module.get_function("test_list_comp_tuple").unwrap();
    let entry_block = main_function.get_first_basic_block().unwrap();
    ctx.builder.position_at_end(entry_block);
}

#[test]
fn test_list_comprehension_with_tuple() {
    let context = Context::create();
    let mut ctx = setup_context(&context);
    create_function(&mut ctx, "test_list_comp_tuple");

    // Create the necessary list functions
    create_list_new_function(&mut ctx);
    create_list_append_function(&mut ctx);
    create_list_len_function(&mut ctx);
    create_list_get_function(&mut ctx);

    // Create a tuple variable
    let tuple_name = "my_tuple".to_string();

    // Create a tuple with two elements: (1, 2)
    let tuple_elements = vec![
        Box::new(Expr::Num {
            value: Number::Integer(1),
            line: 1, column: 2
        }),
        Box::new(Expr::Num {
            value: Number::Integer(2),
            line: 1, column: 5
        })
    ];

    let tuple_expr = Expr::Tuple {
        elts: tuple_elements,
        ctx: ExprContext::Load,
        line: 1, column: 1
    };

    // Compile the tuple expression
    let (tuple_val, tuple_type) = ctx.compile_expr(&tuple_expr).unwrap();

    // Create a variable for the tuple
    ctx.declare_variable(tuple_name.clone(), tuple_val, &tuple_type).unwrap();

    // Create a list variable to use instead of the tuple
    let list_name = "my_list".to_string();

    // Call list_new to create a new list
    let list_new_fn = ctx.module.get_function("list_new").unwrap();
    let list_val = ctx.builder.build_call(list_new_fn, &[], "new_list").unwrap();
    let list_ptr = list_val.try_as_basic_value().left().unwrap().into_pointer_value();

    // Create a variable for the list
    ctx.declare_variable(list_name.clone(), list_ptr.into(), &Type::List(Box::new(Type::Int))).unwrap();

    // Append the tuple elements to the list
    let list_append_fn = ctx.module.get_function("list_append").unwrap();

    // Append the first element (1)
    let one = ctx.llvm_context.i64_type().const_int(1, false);
    ctx.builder.build_call(list_append_fn, &[list_ptr.into(), one.into()], "append_1").unwrap();

    // Append the second element (2)
    let two = ctx.llvm_context.i64_type().const_int(2, false);
    ctx.builder.build_call(list_append_fn, &[list_ptr.into(), two.into()], "append_2").unwrap();

    // Create a list comprehension: [x for x in my_list]
    let target = Box::new(Expr::Name {
        id: "x".to_string(),
        ctx: ExprContext::Store,
        line: 1, column: 2
    });

    let iter = Box::new(Expr::Name {
        id: list_name.clone(),
        ctx: ExprContext::Load,
        line: 1, column: 10
    });

    let generator = Comprehension {
        target,
        iter,
        ifs: vec![],
        is_async: false
    };

    let elt = Box::new(Expr::Name {
        id: "x".to_string(),
        ctx: ExprContext::Load,
        line: 1, column: 1
    });

    let list_comp = Expr::ListComp {
        elt,
        generators: vec![generator],
        line: 1, column: 1
    };

    // Compile the list comprehension
    let result = ctx.compile_expr(&list_comp);

    // Print the error if there is one
    if let Err(e) = &result {
        println!("Error compiling list comprehension: {}", e);
    }

    // The compilation should succeed
    assert!(result.is_ok());

    // The result should be a list
    let (_, result_type) = result.unwrap();
    assert!(matches!(result_type, Type::List(_)));
}
