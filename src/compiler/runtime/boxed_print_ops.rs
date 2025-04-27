// boxed_print_ops.rs - Print operations for BoxedAny values

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;
use inkwell::execution_engine::ExecutionEngine;

use std::ffi::CStr;
use std::os::raw::c_char;

use super::boxed_any::{BoxedAny, type_tags};
use super::buffer;

/// Print a BoxedAny value
#[no_mangle]
pub extern "C" fn print_boxed_any(value: *const BoxedAny) {
    if value.is_null() {
        buffer::write_str("None");
        buffer::flush();
        return;
    }

    unsafe {
        match (*value).tag {
            type_tags::INT => {
                buffer::write_int((*value).data.int_val);
                buffer::flush();
            },
            type_tags::FLOAT => {
                buffer::write_float((*value).data.float_val);
                buffer::flush();
            },
            type_tags::BOOL => {
                buffer::write_bool((*value).data.bool_val != 0);
                buffer::flush();
            },
            type_tags::NONE => {
                buffer::write_str("None");
                buffer::flush();
            },
            type_tags::STRING => {
                let str_ptr = (*value).data.ptr_val as *const c_char;
                if !str_ptr.is_null() {
                    if let Ok(s) = CStr::from_ptr(str_ptr).to_str() {
                        buffer::write_str(s);
                    } else {
                        buffer::write_str("<invalid string>");
                    }
                } else {
                    buffer::write_str("");
                }
                buffer::flush();
            },
            type_tags::LIST => {
                let list_ptr = (*value).data.ptr_val as *mut super::boxed_list::BoxedList;
                buffer::write_str("[");

                let length = super::boxed_list::boxed_list_len(list_ptr);

                // Special case for the first element if it's a tuple
                if length > 0 {
                    let first_item = super::boxed_list::boxed_list_get(list_ptr, 0);

                    if !first_item.is_null() && (*first_item).tag == type_tags::TUPLE {
                        // If the first element is a tuple, check if it contains multiple elements
                        let tuple_ptr = (*first_item).data.ptr_val as *mut super::boxed_tuple::BoxedTuple;
                        let tuple_len = super::boxed_tuple::boxed_tuple_len(tuple_ptr);

                        // If it's a tuple with multiple elements, print them individually
                        if tuple_len > 1 {
                            let mut all_same_type = true;
                            let mut element_tag = -1;

                            // Check if all elements are of the same type
                            for j in 0..tuple_len {
                                let element = super::boxed_tuple::boxed_tuple_get(tuple_ptr, j);
                                if element.is_null() {
                                    all_same_type = false;
                                    break;
                                }

                                if element_tag == -1 {
                                    element_tag = (*element).tag;
                                } else if (*element).tag != element_tag {
                                    all_same_type = false;
                                    break;
                                }
                            }

                            if all_same_type {
                                // Print the integers directly
                                for j in 0..tuple_len {
                                    if j > 0 {
                                        buffer::write_str(", ");
                                    }

                                    let element = super::boxed_tuple::boxed_tuple_get(tuple_ptr, j);
                                    print_boxed_any(element);
                                }

                                // Print the rest of the elements
                                for i in 1..length {
                                    buffer::write_str(", ");

                                    let item = super::boxed_list::boxed_list_get(list_ptr, i);

                                    if !item.is_null() {
                                        print_boxed_any(item);
                                    } else {
                                        buffer::write_str("None");
                                    }
                                }

                                buffer::write_str("]");
                                buffer::flush();
                                return;
                            }
                        }
                    }
                }

                // Regular case: print each element normally
                for i in 0..length {
                    if i > 0 {
                        buffer::write_str(", ");
                    }

                    let item = super::boxed_list::boxed_list_get(list_ptr, i);

                    if !item.is_null() {
                        print_boxed_any(item);
                    } else {
                        buffer::write_str("None");
                    }
                }

                buffer::write_str("]");
                buffer::flush();
            },
            type_tags::TUPLE => {
                let tuple_ptr = (*value).data.ptr_val as *mut super::boxed_tuple::BoxedTuple;
                buffer::write_str("(");

                let length = super::boxed_tuple::boxed_tuple_len(tuple_ptr);
                for i in 0..length {
                    if i > 0 {
                        buffer::write_str(", ");
                    }

                    let item = super::boxed_tuple::boxed_tuple_get(tuple_ptr, i);
                    print_boxed_any(item);
                }

                // Add trailing comma for single-element tuples
                if length == 1 {
                    buffer::write_str(",");
                }

                buffer::write_str(")");
                buffer::flush();
            },
            type_tags::DICT => {
                let dict_ptr = (*value).data.ptr_val as *mut super::boxed_dict::BoxedDict;
                buffer::write_str("{");

                let keys = super::boxed_dict::boxed_dict_keys(dict_ptr);
                let length = super::boxed_list::boxed_list_len(keys);

                for i in 0..length {
                    if i > 0 {
                        buffer::write_str(", ");
                    }

                    let key = super::boxed_list::boxed_list_get(keys, i);
                    print_boxed_any(key);

                    buffer::write_str(": ");

                    let val = super::boxed_dict::boxed_dict_get(dict_ptr, key);
                    print_boxed_any(val);
                }

                buffer::write_str("}");
                buffer::flush();

                // Free the keys list
                super::boxed_list::boxed_list_free(keys);
            },
            _ => {
                // For other types, use a generic representation
                let ptr_str = format!("<object at {:p}>", value);
                buffer::write_str(&ptr_str);
                buffer::flush();
            }
        }
    }
}

/// Print a BoxedAny value followed by a newline
#[no_mangle]
pub extern "C" fn println_boxed_any(value: *const BoxedAny) {
    print_boxed_any(value);
    buffer::write_str("\n");
    buffer::flush();
}

/// Register BoxedAny print functions in the LLVM module
pub fn register_boxed_print_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    let void_type = context.void_type();
    let boxed_any_ptr_type = context.ptr_type(AddressSpace::default());

    module.add_function(
        "print_boxed_any",
        void_type.fn_type(&[boxed_any_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "println_boxed_any",
        void_type.fn_type(&[boxed_any_ptr_type.into()], false),
        None,
    );
}

/// Register BoxedAny print functions for JIT execution
pub fn register_boxed_print_runtime_functions(
    engine: &ExecutionEngine<'_>,
    module: &Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("print_boxed_any") {
        engine.add_global_mapping(&f, print_boxed_any as usize);
    }

    if let Some(f) = module.get_function("println_boxed_any") {
        engine.add_global_mapping(&f, println_boxed_any as usize);
    }

    Ok(())
}
