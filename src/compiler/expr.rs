use crate::ast::{Expr, Number, NameConstant};
use crate::compiler::context::CompilationContext;
use crate::compiler::types::Type;
use inkwell::values::BasicValueEnum;

/// Extension trait for handling expression code generation
pub trait ExprCompiler<'ctx> {
    /// Compile an expression and return the resulting LLVM value with its type
    fn compile_expr(&self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String>;
    
    /// Compile a numeric literal
    fn compile_number(&self, num: &Number) -> Result<(BasicValueEnum<'ctx>, Type), String>;
    
    /// Compile a name constant (True, False, None)
    fn compile_name_constant(&self, constant: &NameConstant) -> Result<(BasicValueEnum<'ctx>, Type), String>;
}

impl<'ctx> ExprCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_expr(&self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        match expr {
            Expr::Num { value, .. } => self.compile_number(value),
            Expr::NameConstant { value, .. } => self.compile_name_constant(value),
            Expr::Name { id, .. } => {
                // Look up variable type and value
                if let Some(_ty) = self.lookup_variable_type(id) {
                    // This would require storing variable values in the context
                    // Implementation depends on how you store variables during compilation
                    Err(format!("Variable lookup not yet implemented for: {}", id))
                } else {
                    Err(format!("Undefined variable: {}", id))
                }
            },
            // Handle other expression types
            _ => Err(format!("Unsupported expression type: {:?}", expr)),
        }
    }
    
    fn compile_number(&self, num: &Number) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        match num {
            Number::Integer(value) => {
                let int_type = self.llvm_context.i64_type();
                let int_value = int_type.const_int(*value as u64, true);
                Ok((int_value.into(), Type::Int))
            },
            Number::Float(value) => {
                let float_type = self.llvm_context.f64_type();
                let float_value = float_type.const_float(*value);
                Ok((float_value.into(), Type::Float))
            },
            Number::Complex { real, imag } => {
                // For complex numbers, you might create a struct with real and imaginary parts
                let float_type = self.llvm_context.f64_type();
                let struct_type = self.llvm_context.struct_type(&[
                    float_type.into(),
                    float_type.into(),
                ], false);
                
                let real_value = float_type.const_float(*real);
                let imag_value = float_type.const_float(*imag);
                
                let complex_value = struct_type.const_named_struct(&[
                    real_value.into(),
                    imag_value.into(),
                ]);
                
                Ok((complex_value.into(), Type::Float)) // Simplified for now
            },
        }
    }
    
    fn compile_name_constant(&self, constant: &NameConstant) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        match constant {
            NameConstant::True => {
                let bool_type = self.llvm_context.bool_type();
                let bool_value = bool_type.const_int(1, false);
                Ok((bool_value.into(), Type::Bool))
            },
            NameConstant::False => {
                let bool_type = self.llvm_context.bool_type();
                let bool_value = bool_type.const_int(0, false);
                Ok((bool_value.into(), Type::Bool))
            },
            NameConstant::None => {
                let ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let null_value = ptr_type.const_null();
                Ok((null_value.into(), Type::None))
            },
        }
    }
}