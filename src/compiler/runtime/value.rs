// value.rs - Tagged value type for runtime type information

use std::ffi::c_void;
use super::list::RawList;
use super::dict::Dict;

/// Tag for runtime type information
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueTag {
    None,
    Int,
    Float,
    Bool,
    Str,
    List,
    Dict,
    Tuple,
}

/// Tagged value with runtime type information
#[repr(C)]
pub struct Value {
    pub tag: ValueTag,
    pub data: *mut c_void,
}

impl Value {
    /// Create a new Value with the given tag and data
    pub fn new(tag: ValueTag, data: *mut c_void) -> Self {
        Self { tag, data }
    }

    /// Create a new None value
    pub fn none() -> Self {
        Self {
            tag: ValueTag::None,
            data: std::ptr::null_mut(),
        }
    }

    /// Create a new integer value
    pub unsafe fn int(value: i64) -> Self {
        let data = Box::into_raw(Box::new(value)) as *mut c_void;
        Self {
            tag: ValueTag::Int,
            data,
        }
    }

    /// Create a new float value
    pub unsafe fn float(value: f64) -> Self {
        let data = Box::into_raw(Box::new(value)) as *mut c_void;
        Self {
            tag: ValueTag::Float,
            data,
        }
    }

    /// Create a new boolean value
    pub unsafe fn bool(value: bool) -> Self {
        let data = Box::into_raw(Box::new(value)) as *mut c_void;
        Self {
            tag: ValueTag::Bool,
            data,
        }
    }

    /// Create a new string value (takes ownership of the string pointer)
    pub unsafe fn str(value: *mut c_void) -> Self {
        Self {
            tag: ValueTag::Str,
            data: value,
        }
    }

    /// Create a new list value (takes ownership of the list pointer)
    pub unsafe fn list(value: *mut RawList) -> Self {
        Self {
            tag: ValueTag::List,
            data: value as *mut c_void,
        }
    }

    /// Create a new dictionary value (takes ownership of the dict pointer)
    pub unsafe fn dict(value: *mut Dict) -> Self {
        Self {
            tag: ValueTag::Dict,
            data: value as *mut c_void,
        }
    }

    /// Create a new tuple value (takes ownership of the tuple pointer)
    pub unsafe fn tuple(value: *mut c_void) -> Self {
        Self {
            tag: ValueTag::Tuple,
            data: value,
        }
    }
}

/// Allocate a new Value on the heap and return a pointer to it
#[no_mangle]
pub unsafe extern "C" fn value_alloc(tag: ValueTag, data: *mut c_void) -> *mut Value {
    Box::into_raw(Box::new(Value::new(tag, data)))
}

/// Free a Value
#[no_mangle]
pub unsafe extern "C" fn value_free(value: *mut Value) {
    if value.is_null() {
        return;
    }
    
    // Free the data based on its tag
    match (*value).tag {
        ValueTag::None => {},
        ValueTag::Int => {
            let _ = Box::from_raw((*value).data as *mut i64);
        },
        ValueTag::Float => {
            let _ = Box::from_raw((*value).data as *mut f64);
        },
        ValueTag::Bool => {
            let _ = Box::from_raw((*value).data as *mut bool);
        },
        ValueTag::Str => {
            // String data is managed elsewhere
        },
        ValueTag::List => {
            // List data is managed elsewhere
        },
        ValueTag::Dict => {
            // Dict data is managed elsewhere
        },
        ValueTag::Tuple => {
            // Tuple data is managed elsewhere
        },
    }
    
    // Free the Value itself
    let _ = Box::from_raw(value);
}

/// Register value functions in the LLVM module
pub fn register_value_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &mut inkwell::module::Module<'ctx>,
) {
    use inkwell::AddressSpace;
    
    // Define the ValueTag enum type
    let value_tag_type = context.i32_type();
    
    // Define the Value struct type
    let value_struct_type = context.struct_type(
        &[
            value_tag_type.into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    
    // Register value_alloc function
    let value_alloc_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            value_tag_type.into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("value_alloc", value_alloc_type, None);
    
    // Register value_free function
    let value_free_type = context.void_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()],
        false,
    );
    module.add_function("value_free", value_free_type, None);
}

/// Register value runtime functions for JIT execution
pub fn register_value_runtime_functions(
    engine: &inkwell::execution_engine::ExecutionEngine<'_>,
    module: &inkwell::module::Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("value_alloc") {
        engine.add_global_mapping(&f, value_alloc as usize);
    }
    if let Some(f) = module.get_function("value_free") {
        engine.add_global_mapping(&f, value_free as usize);
    }
    Ok(())
}
