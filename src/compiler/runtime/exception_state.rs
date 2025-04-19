// exception_state.rs - Global exception state management

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::PointerValue;
use inkwell::AddressSpace;

/// Register exception state functions in the module
pub fn register_exception_state_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    let exception_ptr_type = context.ptr_type(AddressSpace::default());
    let global_exception = module.add_global(exception_ptr_type, None, "__current_exception");
    global_exception.set_initializer(&exception_ptr_type.const_null());

    let get_exception_type = exception_ptr_type.fn_type(&[], false);
    let get_exception_fn = module.add_function("get_current_exception", get_exception_type, None);

    let set_exception_type = context
        .void_type()
        .fn_type(&[exception_ptr_type.into()], false);
    let set_exception_fn = module.add_function("set_current_exception", set_exception_type, None);

    let clear_exception_type = context.void_type().fn_type(&[], false);
    let clear_exception_fn =
        module.add_function("clear_current_exception", clear_exception_type, None);

    let entry = context.append_basic_block(get_exception_fn, "entry");
    let builder = context.create_builder();
    builder.position_at_end(entry);

    let exception = builder
        .build_load(
            exception_ptr_type,
            global_exception.as_pointer_value(),
            "current_exception",
        )
        .unwrap();
    builder.build_return(Some(&exception)).unwrap();

    let entry = context.append_basic_block(set_exception_fn, "entry");
    builder.position_at_end(entry);

    let exception = set_exception_fn
        .get_nth_param(0)
        .unwrap()
        .into_pointer_value();
    builder
        .build_store(global_exception.as_pointer_value(), exception)
        .unwrap();
    builder.build_return(None).unwrap();

    let entry = context.append_basic_block(clear_exception_fn, "entry");
    builder.position_at_end(entry);

    builder
        .build_store(
            global_exception.as_pointer_value(),
            exception_ptr_type.const_null(),
        )
        .unwrap();
    builder.build_return(None).unwrap();
}

/// Get the global exception variable
pub fn get_exception_global<'ctx>(module: &Module<'ctx>) -> Option<PointerValue<'ctx>> {
    module
        .get_global("__current_exception")
        .map(|g| g.as_pointer_value())
}
