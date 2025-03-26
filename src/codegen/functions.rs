use inkwell::values::{FunctionValue, BasicValueEnum};
use inkwell::types::{BasicTypeEnum, BasicType};
use std::collections::HashMap;
use crate::ast::{Stmt, Parameter, Expr};
use super::context::{CompilationContext, FunctionInfo};
use super::error::CodegenError;
use super::statements::compile_stmt;
use super::expressions::compile_expr;
use super::types::map_type;

/// Declare a new function with the given name, parameters, and return type
pub fn declare_function<'ctx>(
    context: &mut CompilationContext<'ctx>,
    name: &str,
    params: &[Parameter],
    return_type: Option<BasicTypeEnum<'ctx>>,
    is_variadic: bool
) -> Result<FunctionValue<'ctx>, CodegenError> {
    // Build parameter types
    let mut param_types = Vec::with_capacity(params.len());
    
    for param in params {
        let param_type = match &param.typ {
            Some(typ) => {
                // Get type from annotation
                // This would need to be expanded for actual type inference
                match typ.as_ref() {
                    Expr::Name { id, .. } => map_type(context.context, id)?,
                    _ => return Err(CodegenError::type_error(
                        "Complex type annotations not supported yet"
                    )),
                }
            },
            None => {
                // Default to i64 for now
                context.context.i64_type().into()
            }
        };
        
        param_types.push(param_type);
    }
    
    // Build function type
    let return_type = match return_type {
        Some(ty) => ty,
        None => context.context.void_type().into(),
    };
    
    let fn_type = match return_type {
        BasicTypeEnum::IntType(int_type) => {
            int_type.fn_type(&param_types, is_variadic)
        },
        BasicTypeEnum::FloatType(float_type) => {
            float_type.fn_type(&param_types, is_variadic)
        },
        BasicTypeEnum::PointerType(ptr_type) => {
            ptr_type.fn_type(&param_types, is_variadic)
        },
        BasicTypeEnum::StructType(struct_type) => {
            struct_type.fn_type(&param_types, is_variadic)
        },
        BasicTypeEnum::ArrayType(array_type) => {
            array_type.fn_type(&param_types, is_variadic)
        },
        BasicTypeEnum::VectorType(vector_type) => {
            vector_type.fn_type(&param_types, is_variadic)
        },
    };
    
    // Create the function
    let function = context.module.add_function(name, fn_type, None);
    
    // Set parameter names
    for (i, param) in params.iter().enumerate() {
        if let Some(param_value) = function.get_nth_param(i as u32) {
            param_value.set_name(&param.name);
        }
    }
    
    // Register the function
    let func_info = FunctionInfo {
        function,
        return_type: Some(return_type),
    };
    
    context.register_function(name, func_info);
    
    Ok(function)
}

/// Define a function body
pub fn define_function<'ctx>(
    context: &mut CompilationContext<'ctx>,
    function: FunctionValue<'ctx>,
    params: &[Parameter],
    body: &[Box<Stmt>]
) -> Result<(), CodegenError> {
    // Create entry block
    let entry_block = context.context.append_basic_block(function, "entry");
    context.builder.position_at_end(entry_block);
    
    // Set current function
    let old_function = context.current_function;
    context.current_function = Some(function);
    
    // Create parameter variables
    let mut param_vars = HashMap::new();
    
    for (i, param) in params.iter().enumerate() {
        if let Some(param_value) = function.get_nth_param(i as u32) {
            // Allocate storage for the parameter
            let param_ptr = context.builder.build_alloca(param_value.get_type(), &param.name);
            
            // Store the parameter value
            context.builder.build_store(param_ptr, param_value);
            
            // Add to local variables
            param_vars.insert(param.name.clone(), param_ptr);
        }
    }
    
    // Compile the function body
    for stmt in body {
        compile_stmt(context, stmt)?;
    }
    
    // Add implicit return for void functions
    if function.get_type().get_return_type().is_none() {
        context.builder.build_return(None);
    }
    
    // Restore previous function context
    context.current_function = old_function;
    
    Ok(())
}

/// Compile a function call
pub fn compile_call<'ctx>(
    context: &mut CompilationContext<'ctx>,
    func_expr: &Expr,
    args: &[Box<Expr>],
    keywords: &[(Option<String>, Box<Expr>)]
) -> Result<BasicValueEnum<'ctx>, CodegenError> {
    // Get the function to call
    let (fn_value, is_method_call) = match func_expr {
        Expr::Name { id, .. } => {
            // Simple function call
            let fn_info = context.lookup_function(id)?;
            (fn_info.function, false)
        },
        
        Expr::Attribute { value, attr, .. } => {
            // Method call
            // This is more complex and depends on your object model
            // For now, just return unsupported
            return Err(CodegenError::unsupported_feature("Method calls"));
        },
        
        _ => {
            return Err(CodegenError::expression_error(
                "Invalid function call target"
            ));
        }
    };
    
    // Compile arguments
    let mut call_args = Vec::with_capacity(args.len());
    
    for arg in args {
        let arg_value = compile_expr(context, arg)?;
        call_args.push(arg_value);
    }
    
    // Handle keyword arguments
    // This would require a more complex implementation for Python-style keyword args
    if !keywords.is_empty() {
        return Err(CodegenError::unsupported_feature("Keyword arguments"));
    }
    
    // Build the call
    let call_site_value = context.builder.build_call(fn_value, &call_args, "call");
    
    // Return the result if any
    if let Some(ret_val) = call_site_value.try_as_basic_value().left() {
        Ok(ret_val)
    } else {
        // Void function
        Err(CodegenError::type_error(
            "Cannot use void function in an expression"
        ))
    }
}