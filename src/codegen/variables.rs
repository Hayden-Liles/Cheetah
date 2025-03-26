use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::types::BasicTypeEnum;
use crate::ast::{Expr, ExprContext};
use super::context::{CompilationContext, VariableInfo};
use super::error::CodegenError;
use super::expressions::compile_expr;
use super::types::default_value;

/// Define a new variable with the given name, type, and initial value
pub fn define_variable<'ctx>(
    context: &mut CompilationContext<'ctx>,
    name: &str,
    var_type: BasicTypeEnum<'ctx>,
    initial_value: Option<BasicValueEnum<'ctx>>,
    is_mutable: bool
) -> Result<PointerValue<'ctx>, CodegenError> {
    // Make sure we're in a function
    let function = context.current_function()?;
    
    // Create an entry block builder if not already in a block
    let entry_block = match context.builder.get_insert_block() {
        Some(block) => block,
        None => {
            let entry = context.context.append_basic_block(function, "entry");
            context.builder.position_at_end(entry);
            entry
        }
    };
    
    // Allocate space for the variable
    let alloca = context.builder.build_alloca(var_type, name);
    
    // Store the initial value if provided, otherwise use default
    let value = match initial_value {
        Some(val) => val,
        None => default_value(context.context, var_type),
    };
    
    context.builder.build_store(alloca, value);
    
    // Register the variable in the context
    let var_info = VariableInfo {
        ptr: alloca,
        ty: var_type,
        is_mutable,
    };
    
    context.register_variable(name, var_info);
    
    Ok(alloca)
}

/// Assign a value to an existing variable
pub fn assign_variable<'ctx>(
    context: &mut CompilationContext<'ctx>,
    target: &Expr,
    value: BasicValueEnum<'ctx>
) -> Result<(), CodegenError> {
    match target {
        Expr::Name { id, ctx, .. } => {
            // Make sure this is a Store context
            match ctx {
                ExprContext::Store => {
                    // Look up the variable
                    let var_info = context.lookup_variable(id)?;
                    
                    // Make sure it's mutable if not just defined
                    if !var_info.is_mutable {
                        return Err(CodegenError::type_error(
                            &format!("Cannot assign to immutable variable '{}'", id)
                        ));
                    }
                    
                    // Store the value
                    context.builder.build_store(var_info.ptr, value);
                    Ok(())
                },
                _ => {
                    Err(CodegenError::type_error(
                        &format!("Cannot assign to '{}' in load context", id)
                    ))
                }
            }
        },
        
        Expr::Subscript { value: val_expr, slice, .. } => {
            // Get the array/container
            let container = compile_expr(context, val_expr)?;
            let index = compile_expr(context, slice)?;
            
            // This depends on container type - example for arrays:
            // TODO: Implement subscript assignment for different container types
            Err(CodegenError::unsupported_feature("Subscript assignment"))
        },
        
        Expr::Attribute { value: val_expr, attr, .. } => {
            // Get the object
            let object = compile_expr(context, val_expr)?;
            
            // This depends on object type - example for structs:
            // TODO: Implement attribute assignment
            Err(CodegenError::unsupported_feature("Attribute assignment"))
        },
        
        _ => {
            Err(CodegenError::type_error(
                &format!("Invalid assignment target: {:?}", target)
            ))
        }
    }
}