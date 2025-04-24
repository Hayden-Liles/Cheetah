// boxed_any_contains.rs - Implementation of the 'in' operator for BoxedAny values

use super::boxed_any::{BoxedAny, type_tags, boxed_any_from_bool, boxed_any_equals};
use super::boxed_dict::boxed_dict_contains;
use std::ffi::{CStr, c_char};

/// Check if a value is contained in a container (dict, list, string)
#[no_mangle]
pub extern "C" fn boxed_any_contains(container: *const BoxedAny, item: *const BoxedAny) -> *mut BoxedAny {
    if container.is_null() || item.is_null() {
        return boxed_any_from_bool(false);
    }

    unsafe {
        match (*container).tag {
            type_tags::DICT => {
                // For dictionaries, check if the key exists
                let dict_ptr = (*container).data.ptr_val as *mut super::boxed_dict::BoxedDict;
                let result = boxed_dict_contains(dict_ptr, item as *mut BoxedAny);
                boxed_any_from_bool(result)
            },
            type_tags::LIST => {
                // For lists, check if the item is in the list
                let list_ptr = (*container).data.ptr_val as *mut super::boxed_list::BoxedList;

                // Manually check if the item is in the list
                let length = (*list_ptr).length;
                let data = (*list_ptr).data;

                for i in 0..length {
                    let list_item = *data.add(i as usize);
                    if !list_item.is_null() && boxed_any_equals(list_item, item) {
                        return boxed_any_from_bool(true);
                    }
                }

                boxed_any_from_bool(false)
            },
            type_tags::STRING => {
                // For strings, check if the item is a substring
                if (*item).tag != type_tags::STRING {
                    return boxed_any_from_bool(false);
                }

                let container_str = (*container).data.ptr_val as *const c_char;
                let item_str = (*item).data.ptr_val as *const c_char;

                if container_str.is_null() || item_str.is_null() {
                    return boxed_any_from_bool(false);
                }

                let container_cstr = CStr::from_ptr(container_str);
                let item_cstr = CStr::from_ptr(item_str);

                let container_bytes = container_cstr.to_bytes();
                let item_bytes = item_cstr.to_bytes();

                // Check if item_bytes is a substring of container_bytes
                let result = if item_bytes.is_empty() {
                    true
                } else if item_bytes.len() > container_bytes.len() {
                    false
                } else {
                    // Simple substring search
                    'outer: for i in 0..=container_bytes.len() - item_bytes.len() {
                        for j in 0..item_bytes.len() {
                            if container_bytes[i + j] != item_bytes[j] {
                                continue 'outer;
                            }
                        }
                        return boxed_any_from_bool(true);
                    }
                    false
                };

                boxed_any_from_bool(result)
            },
            _ => boxed_any_from_bool(false), // Other types don't support 'in' operator
        }
    }
}

/// Register BoxedAny contains functions in the LLVM module
pub fn register_boxed_any_contains_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &mut inkwell::module::Module<'ctx>,
) {
    let boxed_any_ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // Register the boxed_any_contains function
    module.add_function(
        "boxed_any_contains",
        boxed_any_ptr_type.fn_type(&[
            boxed_any_ptr_type.into(),
            boxed_any_ptr_type.into(),
        ], false),
        None,
    );
}

/// Register BoxedAny contains runtime mappings for the JIT engine
pub fn register_boxed_any_contains_runtime_functions(
    engine: &inkwell::execution_engine::ExecutionEngine<'_>,
    module: &inkwell::module::Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("boxed_any_contains") {
        engine.add_global_mapping(&f, boxed_any_contains as usize);
    }

    Ok(())
}
