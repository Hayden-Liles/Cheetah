// list_print.rs - Runtime support for list printing

use std::ffi::CString;
use std::os::raw::c_char;
use crate::compiler::runtime::list::{RawList, list_get};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;
use inkwell::execution_engine::ExecutionEngine;

/// Convert a list to a string representation (for printing)
#[no_mangle]
pub extern "C" fn list_to_string(list_ptr: *mut RawList) -> *mut c_char {
    unsafe {
        if list_ptr.is_null() {
            return CString::new("[]").unwrap().into_raw();
        }

        let rl = &*list_ptr;
        let mut s = String::from("[");

        for i in 0..rl.length {
            if i > 0 {
                s.push_str(", ");
            }

            let elem = list_get(list_ptr, i);

            if elem.is_null() {
                s.push_str("None");
            } else {
                // Try to interpret the element based on its value
                let value = elem as usize;

                // If the value is very small, it's likely a small integer or boolean
                if value <= 1 {
                    // Could be a boolean (0 = False, 1 = True)
                    if value == 0 {
                        s.push_str("False");
                    } else {
                        s.push_str("True");
                    }
                } else if value < 1000 {
                    // Likely a small integer
                    s.push_str(&value.to_string());
                } else {
                    // For other values, just print a placeholder
                    s.push_str("<element>");
                }
            }
        }

        s.push(']');
        CString::new(s).unwrap().into_raw()
    }
}

/// Register list printing functions in the LLVM module
pub fn register_list_print_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    module.add_function(
        "list_to_string",
        context.ptr_type(AddressSpace::default()).fn_type(
            &[context.ptr_type(AddressSpace::default()).into()],
            false
        ),
        None,
    );
}

/// Register list printing runtime mappings for the JIT engine
pub fn register_list_print_runtime_functions(
    engine: &ExecutionEngine<'_>,
    module: &Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("list_to_string") {
        engine.add_global_mapping(&f, list_to_string as usize);
    }
    Ok(())
}
