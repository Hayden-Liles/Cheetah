// string_ops_register.rs - Register string operations in the LLVM module

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

/// Register string operation functions in the module
pub fn register_string_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Create string_get_char function (gets a character from a string at a given index)
    let string_get_char_type = context.i64_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // string pointer
            context.i64_type().into(),                        // index
        ],
        false,
    );
    module.add_function("string_get_char", string_get_char_type, None);

    // Create char_to_string function (converts a character code to a string)
    let char_to_string_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.i64_type().into(),                        // character code
        ],
        false,
    );
    module.add_function("char_to_string", char_to_string_type, None);

    // Create string_slice function (gets a slice from a string)
    let string_slice_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // string pointer
            context.i64_type().into(),                        // start index
            context.i64_type().into(),                        // stop index
            context.i64_type().into(),                        // step
        ],
        false,
    );
    module.add_function("string_slice", string_slice_type, None);

    // Create string_len function (gets the length of a string)
    let string_len_type = context.i64_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // string pointer
        ],
        false,
    );
    module.add_function("string_len", string_len_type, None);

    // Create string_concat function (concatenates two strings)
    let string_concat_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // first string pointer
            context.ptr_type(AddressSpace::default()).into(), // second string pointer
        ],
        false,
    );
    module.add_function("string_concat", string_concat_type, None);

    // Create free_string function (frees memory allocated by string functions)
    let free_string_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // string pointer
        ],
        false,
    );
    module.add_function("free_string", free_string_type, None);
}
