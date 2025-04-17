// print_ops.rs - Runtime support for print function

use std::ffi::CStr;
use std::os::raw::c_char;
use std::io::{self, Write};

// Thread-local storage for print function state
thread_local! {
    // Store the last printed value and whether we need a space before the next value
    static PRINT_STATE: std::cell::RefCell<PrintState> = std::cell::RefCell::new(PrintState::new());
}

// State for managing print formatting
struct PrintState {
    needs_space: bool,
    in_line: bool,
}

impl PrintState {
    fn new() -> Self {
        PrintState {
            needs_space: false,
            in_line: false,
        }
    }

    fn reset(&mut self) {
        self.needs_space = false;
        self.in_line = false;
    }
}

/// Print a string to stdout (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn print_string(value: *const c_char) {
    unsafe {
        if !value.is_null() {
            let c_str = CStr::from_ptr(value);
            if let Ok(str_slice) = c_str.to_str() {
                PRINT_STATE.with(|state| {
                    let mut state = state.borrow_mut();

                    // If we need a space and we're not at the start of a line, print one
                    if state.needs_space && state.in_line {
                        print!(" ");
                    }

                    // Print the string
                    print!("{}", str_slice);

                    // Update the state
                    state.in_line = true;
                    state.needs_space = true;
                });

                // Ensure the output is displayed immediately
                io::stdout().flush().ok();
            }
        }
    }
}

/// Print a string with a newline to stdout (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn println_string(value: *const c_char) {
    unsafe {
        if !value.is_null() {
            let c_str = CStr::from_ptr(value);
            if let Ok(str_slice) = c_str.to_str() {
                PRINT_STATE.with(|state| {
                    let mut state = state.borrow_mut();

                    // If we need a space and we're not at the start of a line, print one
                    if state.needs_space && state.in_line {
                        print!(" ");
                    }

                    // Print the string and a newline
                    println!("{}", str_slice);

                    // Reset the state for the next line
                    state.reset();
                });

                // Ensure the output is displayed immediately
                io::stdout().flush().ok();
            }
        } else {
            // Just print a newline for null strings
            println!();
            PRINT_STATE.with(|state| {
                state.borrow_mut().reset();
            });
            io::stdout().flush().ok();
        }
    }
}

/// Print an integer to stdout (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn print_int(value: i64) {
    PRINT_STATE.with(|state| {
        let mut state = state.borrow_mut();

        // If we need a space and we're not at the start of a line, print one
        if state.needs_space && state.in_line {
            print!(" ");
        }

        // Print the integer
        print!("{}", value);

        // Update the state
        state.in_line = true;
        state.needs_space = true;
    });

    // Ensure the output is displayed immediately
    io::stdout().flush().ok();
}

/// Print a float to stdout (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn print_float(value: f64) {
    PRINT_STATE.with(|state| {
        let mut state = state.borrow_mut();

        // If we need a space and we're not at the start of a line, print one
        if state.needs_space && state.in_line {
            print!(" ");
        }

        // Print the float
        print!("{}", value);

        // Update the state
        state.in_line = true;
        state.needs_space = true;
    });

    // Ensure the output is displayed immediately
    io::stdout().flush().ok();
}

/// Print a boolean to stdout (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn print_bool(value: bool) {
    PRINT_STATE.with(|state| {
        let mut state = state.borrow_mut();

        // If we need a space and we're not at the start of a line, print one
        if state.needs_space && state.in_line {
            print!(" ");
        }

        // Print the boolean as "True" or "False"
        if value {
            print!("True");
        } else {
            print!("False");
        }

        // Update the state
        state.in_line = true;
        state.needs_space = true;
    });

    // Ensure the output is displayed immediately
    io::stdout().flush().ok();
}

/// Print a newline (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn print_newline() {
    println!();

    PRINT_STATE.with(|state| {
        state.borrow_mut().reset();
    });

    // Ensure the output is displayed immediately
    io::stdout().flush().ok();
}

/// Register print operation functions in the module
pub fn register_print_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &mut inkwell::module::Module<'ctx>,
) {
    use inkwell::AddressSpace;

    // Create print_string function
    let print_string_type = context.void_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // string pointer
        false,
    );
    module.add_function("print_string", print_string_type, None);

    // Create println_string function
    let println_string_type = context.void_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // string pointer
        false,
    );
    module.add_function("println_string", println_string_type, None);

    // Create print_int function
    let print_int_type = context.void_type().fn_type(
        &[context.i64_type().into()], // integer value
        false,
    );
    module.add_function("print_int", print_int_type, None);

    // Create print_float function
    let print_float_type = context.void_type().fn_type(
        &[context.f64_type().into()], // float value
        false,
    );
    module.add_function("print_float", print_float_type, None);

    // Create print_bool function
    let print_bool_type = context.void_type().fn_type(
        &[context.bool_type().into()], // boolean value
        false,
    );
    module.add_function("print_bool", print_bool_type, None);

    // Create print_newline function
    let print_newline_type = context.void_type().fn_type(&[], false);
    module.add_function("print_newline", print_newline_type, None);
}