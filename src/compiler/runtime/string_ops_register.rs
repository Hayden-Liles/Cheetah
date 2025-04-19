// string_ops_register.rs - Register string operations in the LLVM module

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

/// Register string operation functions in the module
pub fn register_string_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    let string_get_char_type = context.i64_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ],
        false,
    );
    module.add_function("string_get_char", string_get_char_type, None);

    let char_to_string_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.i64_type().into()], false);
    module.add_function("char_to_string", char_to_string_type, None);

    let string_slice_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
            context.i64_type().into(),
            context.i64_type().into(),
        ],
        false,
    );
    module.add_function("string_slice", string_slice_type, None);

    let string_len_type = context
        .i64_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("string_len", string_len_type, None);

    let string_concat_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("string_concat", string_concat_type, None);

    let free_string_type = context
        .void_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("free_string", free_string_type, None);
}
