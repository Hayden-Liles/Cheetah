use inkwell::types::BasicType;
use super::context::CompilationContext;
use super::error::CodegenError;
use super::functions::FunctionInfo;

/// Register built-in functions in the compilation context
pub fn register_builtins<'ctx>(
    context: &mut CompilationContext<'ctx>
) -> Result<(), CodegenError> {
    // Register print function
    register_print(context)?;
    
    // Register other built-ins
    register_input(context)?;
    register_len(context)?;
    register_range(context)?;
    
    Ok(())
}

/// Register the print function
fn register_print<'ctx>(
    context: &mut CompilationContext<'ctx>
) -> Result<(), CodegenError> {
    // We'll implement print by calling printf from libc
    let printf_type = context.context.i32_type().fn_type(
        &[context.context.i8_type().ptr_type(Default::default()).into()],
        true
    );
    
    let printf_fn = context.module.add_function("printf", printf_type, None);
    
    // Create a wrapper for printf that takes an i8* (string pointer)
    let print_type = context.context.void_type().fn_type(
        &[context.context.i8_type().ptr_type(Default::default()).into()],
        false
    );
    
    let print_fn = context.module.add_function("print", print_type, None);
    
    // Implement the print function body
    let entry_block = context.context.append_basic_block(print_fn, "entry");
    
    let builder = context.context.create_builder();
    builder.position_at_end(entry_block);
    
    // Call printf with the string
    let param = print_fn.get_nth_param(0).unwrap().into_pointer_value();
    builder.build_call(printf_fn, &[param.into()], "");
    
    // Print a newline
    let newline = builder.build_global_string_ptr("\n", "newline");
    builder.build_call(printf_fn, &[newline.as_pointer_value().into()], "");
    
    // Return void
    builder.build_return(None);
    
    // Register in context
    let func_info = FunctionInfo {
        function: print_fn,
        return_type: None,
    };
    
    context.register_function("print", func_info);
    
    Ok(())
}

/// Register the input function
fn register_input<'ctx>(
    context: &mut CompilationContext<'ctx>
) -> Result<(), CodegenError> {
    // Simplified version - in reality you'd call fgets or similar
    // This just returns a static string for demonstration
    
    let input_type = context.context.i8_type()
        .ptr_type(Default::default())
        .fn_type(&[], false);
    
    let input_fn = context.module.add_function("input", input_type, None);
    
    let entry_block = context.context.append_basic_block(input_fn, "entry");
    
    let builder = context.context.create_builder();
    builder.position_at_end(entry_block);
    
    // Create a static string (this is just a placeholder)
    let static_str = builder.build_global_string_ptr("Input String", "static_input");
    
    // Return the string pointer
    builder.build_return(Some(&static_str.as_pointer_value()));
    
    // Register in context
    let func_info = FunctionInfo {
        function: input_fn,
        return_type: Some(context.context.i8_type().ptr_type(Default::default()).into()),
    };
    
    context.register_function("input", func_info);
    
    Ok(())
}

/// Register the len function
fn register_len<'ctx>(
    context: &mut CompilationContext<'ctx>
) -> Result<(), CodegenError> {
    // Simple len function for strings only
    
    let len_type = context.context.i64_type()
        .fn_type(&[context.context.i8_type().ptr_type(Default::default()).into()], false);
    
    let len_fn = context.module.add_function("len", len_type, None);
    
    let entry_block = context.context.append_basic_block(len_fn, "entry");
    
    let builder = context.context.create_builder();
    builder.position_at_end(entry_block);
    
    // Get the string parameter
    let string_ptr = len_fn.get_nth_param(0).unwrap().into_pointer_value();
    
    // Call strlen from libc
    let strlen_type = context.context.i64_type()
        .fn_type(&[context.context.i8_type().ptr_type(Default::default()).into()], false);
    
    let strlen_fn = context.module.add_function("strlen", strlen_type, None);
    
    let result = builder.build_call(strlen_fn, &[string_ptr.into()], "strlen_result");
    
    // Return the length
    if let Some(ret_val) = result.try_as_basic_value().left() {
        builder.build_return(Some(&ret_val));
    } else {
        return Err(CodegenError::internal_error("Failed to get strlen result"));
    }
    
    // Register in context
    let func_info = FunctionInfo {
        function: len_fn,
        return_type: Some(context.context.i64_type().into()),
    };
    
    context.register_function("len", func_info);
    
    Ok(())
}

/// Register the range function
fn register_range<'ctx>(
    context: &mut CompilationContext<'ctx>
) -> Result<(), CodegenError> {
    // This is a simplified version that doesn't actually create a range object
    // Just to demonstrate - in a real implementation you'd create a proper range type
    
    let range_type = context.context.void_type().fn_type(
        &[context.context.i64_type().into()],
        false
    );
    
    let range_fn = context.module.add_function("range", range_type, None);
    
    let entry_block = context.context.append_basic_block(range_fn, "entry");
    
    let builder = context.context.create_builder();
    builder.position_at_end(entry_block);
    
    // Simply return void - this is just a placeholder
    builder.build_return(None);
    
    // Register in context
    let func_info = FunctionInfo {
        function: range_fn,
        return_type: None,
    };
    
    context.register_function("range", func_info);
    
    Ok(())
}