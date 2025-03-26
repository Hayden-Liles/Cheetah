use inkwell::context::Context;
use inkwell::types::{BasicTypeEnum, BasicType};
use crate::ast::Number;

use super::context::CompilationContext;
use super::error::CodegenError;

/// Maps Cheetah language types to LLVM types
pub fn map_type<'ctx>(context: &'ctx Context, type_name: &str) -> Result<BasicTypeEnum<'ctx>, CodegenError> {
    match type_name {
        "int" => Ok(context.i64_type().as_basic_type_enum()),
        "float" => Ok(context.f64_type().as_basic_type_enum()),
        "bool" => Ok(context.bool_type().as_basic_type_enum()),
        "str" => {
            // Strings are represented as i8* (char*)
            Ok(context.i8_type().ptr_type(Default::default()).as_basic_type_enum())
        }
        "None" => Ok(context.struct_type(&[], false).as_basic_type_enum()),
        _ => Err(CodegenError::type_error(&format!("Unknown type: {}", type_name))),
    }
}

/// Get the default value for a type
pub fn default_value<'ctx>(
    context: &'ctx Context,
    ty: BasicTypeEnum<'ctx>,
) -> BasicValueEnum<'ctx> {
    match ty {
        BasicTypeEnum::IntType(int_ty) => int_ty.const_zero().into(),
        BasicTypeEnum::FloatType(float_ty) => float_ty.const_zero().into(),
        BasicTypeEnum::PointerType(ptr_ty) => ptr_ty.const_null().into(),
        BasicTypeEnum::StructType(struct_ty) => struct_ty.const_zero().into(),
        BasicTypeEnum::ArrayType(array_ty) => array_ty.const_zero().into(),
        _ => panic!("Unsupported type for default value"),
    }
}

/// Convert AST number to LLVM value
pub fn number_to_llvm_value<'ctx>(
    context: &'ctx Context,
    number: &Number
) -> BasicValueEnum<'ctx> {
    match number {
        Number::Integer(i) => context.i64_type().const_int(*i as u64, true).into(),
        Number::Float(f) => context.f64_type().const_float(*f).into(),
        Number::Complex { real, imag } => {
            // For complex numbers, create a struct with real and imaginary parts
            let complex_type = context.struct_type(
                &[context.f64_type().into(), context.f64_type().into()], 
                false
            );
            
            let real_val = context.f64_type().const_float(*real);
            let imag_val = context.f64_type().const_float(*imag);
            
            complex_type.const_named_struct(&[real_val.into(), imag_val.into()]).into()
        }
    }
}