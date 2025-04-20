// min_max_call.rs - Implementation of the min() and max() built-in functions

use crate::ast::Expr;
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::ExprCompiler;
use crate::compiler::types::Type;
use inkwell::values::BasicValueEnum;

impl<'ctx> CompilationContext<'ctx> {
    /// Compile a call to the min() function
    pub fn compile_min_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if args.len() != 2 {
            return Err(format!(
                "min() takes exactly two arguments ({} given)",
                args.len()
            ));
        }

        let (arg1_val, arg1_type) = self.compile_expr(&args[0])?;
        let (arg2_val, arg2_type) = self.compile_expr(&args[1])?;

        match (&arg1_type, &arg2_type) {
            (Type::Int, Type::Int) => {
                let min_int_fn = match self.module.get_function("min_int") {
                    Some(f) => f,
                    None => return Err("min_int function not found".to_string()),
                };

                let arg1_int = arg1_val.into_int_value();
                let arg2_int = arg2_val.into_int_value();

                let call_site_value = self
                    .builder
                    .build_call(min_int_fn, &[arg1_int.into(), arg2_int.into()], "min_int_result")
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get min_int result".to_string())?;

                Ok((result, Type::Int))
            }
            (Type::Float, Type::Float) => {
                let min_float_fn = match self.module.get_function("min_float") {
                    Some(f) => f,
                    None => return Err("min_float function not found".to_string()),
                };

                let arg1_float = arg1_val.into_float_value();
                let arg2_float = arg2_val.into_float_value();

                let call_site_value = self
                    .builder
                    .build_call(
                        min_float_fn,
                        &[arg1_float.into(), arg2_float.into()],
                        "min_float_result",
                    )
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get min_float result".to_string())?;

                Ok((result, Type::Float))
            }
            (Type::Int, Type::Float) => {
                // Convert int to float and call min_float
                let min_float_fn = match self.module.get_function("min_float") {
                    Some(f) => f,
                    None => return Err("min_float function not found".to_string()),
                };

                let arg1_int = arg1_val.into_int_value();
                let arg1_float = self
                    .builder
                    .build_signed_int_to_float(
                        arg1_int,
                        self.llvm_context.f64_type(),
                        "int_to_float",
                    )
                    .unwrap();
                let arg2_float = arg2_val.into_float_value();

                let call_site_value = self
                    .builder
                    .build_call(
                        min_float_fn,
                        &[arg1_float.into(), arg2_float.into()],
                        "min_float_result",
                    )
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get min_float result".to_string())?;

                Ok((result, Type::Float))
            }
            (Type::Float, Type::Int) => {
                // Convert int to float and call min_float
                let min_float_fn = match self.module.get_function("min_float") {
                    Some(f) => f,
                    None => return Err("min_float function not found".to_string()),
                };

                let arg1_float = arg1_val.into_float_value();
                let arg2_int = arg2_val.into_int_value();
                let arg2_float = self
                    .builder
                    .build_signed_int_to_float(
                        arg2_int,
                        self.llvm_context.f64_type(),
                        "int_to_float",
                    )
                    .unwrap();

                let call_site_value = self
                    .builder
                    .build_call(
                        min_float_fn,
                        &[arg1_float.into(), arg2_float.into()],
                        "min_float_result",
                    )
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get min_float result".to_string())?;

                Ok((result, Type::Float))
            }
            _ => Err(format!(
                "min() not supported for types {:?} and {:?}",
                arg1_type, arg2_type
            )),
        }
    }

    /// Compile a call to the max() function
    pub fn compile_max_call(
        &mut self,
        args: &[Expr],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if args.len() != 2 {
            return Err(format!(
                "max() takes exactly two arguments ({} given)",
                args.len()
            ));
        }

        let (arg1_val, arg1_type) = self.compile_expr(&args[0])?;
        let (arg2_val, arg2_type) = self.compile_expr(&args[1])?;

        match (&arg1_type, &arg2_type) {
            (Type::Int, Type::Int) => {
                let max_int_fn = match self.module.get_function("max_int") {
                    Some(f) => f,
                    None => return Err("max_int function not found".to_string()),
                };

                let arg1_int = arg1_val.into_int_value();
                let arg2_int = arg2_val.into_int_value();

                let call_site_value = self
                    .builder
                    .build_call(max_int_fn, &[arg1_int.into(), arg2_int.into()], "max_int_result")
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get max_int result".to_string())?;

                Ok((result, Type::Int))
            }
            (Type::Float, Type::Float) => {
                let max_float_fn = match self.module.get_function("max_float") {
                    Some(f) => f,
                    None => return Err("max_float function not found".to_string()),
                };

                let arg1_float = arg1_val.into_float_value();
                let arg2_float = arg2_val.into_float_value();

                let call_site_value = self
                    .builder
                    .build_call(
                        max_float_fn,
                        &[arg1_float.into(), arg2_float.into()],
                        "max_float_result",
                    )
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get max_float result".to_string())?;

                Ok((result, Type::Float))
            }
            (Type::Int, Type::Float) => {
                // Convert int to float and call max_float
                let max_float_fn = match self.module.get_function("max_float") {
                    Some(f) => f,
                    None => return Err("max_float function not found".to_string()),
                };

                let arg1_int = arg1_val.into_int_value();
                let arg1_float = self
                    .builder
                    .build_signed_int_to_float(
                        arg1_int,
                        self.llvm_context.f64_type(),
                        "int_to_float",
                    )
                    .unwrap();
                let arg2_float = arg2_val.into_float_value();

                let call_site_value = self
                    .builder
                    .build_call(
                        max_float_fn,
                        &[arg1_float.into(), arg2_float.into()],
                        "max_float_result",
                    )
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get max_float result".to_string())?;

                Ok((result, Type::Float))
            }
            (Type::Float, Type::Int) => {
                // Convert int to float and call max_float
                let max_float_fn = match self.module.get_function("max_float") {
                    Some(f) => f,
                    None => return Err("max_float function not found".to_string()),
                };

                let arg1_float = arg1_val.into_float_value();
                let arg2_int = arg2_val.into_int_value();
                let arg2_float = self
                    .builder
                    .build_signed_int_to_float(
                        arg2_int,
                        self.llvm_context.f64_type(),
                        "int_to_float",
                    )
                    .unwrap();

                let call_site_value = self
                    .builder
                    .build_call(
                        max_float_fn,
                        &[arg1_float.into(), arg2_float.into()],
                        "max_float_result",
                    )
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get max_float result".to_string())?;

                Ok((result, Type::Float))
            }
            _ => Err(format!(
                "max() not supported for types {:?} and {:?}",
                arg1_type, arg2_type
            )),
        }
    }
}
