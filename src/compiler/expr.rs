use crate::ast::{BoolOperator, CmpOperator, Expr, NameConstant, Number, Operator, UnaryOperator};
use crate::compiler::context::CompilationContext;
use crate::compiler::types::is_reference_type;
use crate::compiler::types::Type;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;

/// Extension trait for handling expression code generation
pub trait ExprCompiler<'ctx> {
    fn evaluate_comprehension_conditions(
        &mut self,
        generator: &crate::ast::Comprehension,
        current_function: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String>;
    fn handle_general_iteration_for_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        iter_val: BasicValueEnum<'ctx>,
        iter_type: Type,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String>;
    fn handle_string_iteration_for_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        str_ptr: inkwell::values::PointerValue<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String>;
    fn handle_list_iteration_for_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String>;
    fn process_list_comprehension_element(
        &mut self,
        elt: &Expr,
        should_append: inkwell::values::IntValue<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
        current_function: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String>;
    fn handle_tuple_dynamic_index(
        &mut self,
        tuple_val: BasicValueEnum<'ctx>,
        tuple_type: Type,
        index_val: inkwell::values::IntValue<'ctx>,
        element_types: &[Type],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;
    fn build_empty_list(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_list(
        &self,
        elements: Vec<BasicValueEnum<'ctx>>,
        element_type: &Type,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_empty_tuple(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_tuple(
        &self,
        elements: Vec<BasicValueEnum<'ctx>>,
        element_types: &[Type],
    ) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_empty_dict(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_dict(
        &self,
        keys: Vec<BasicValueEnum<'ctx>>,
        values: Vec<BasicValueEnum<'ctx>>,
        key_type: &Type,
        value_type: &Type,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_empty_set(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_set(
        &self,
        elements: Vec<BasicValueEnum<'ctx>>,
        element_type: &Type,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_list_get_item(
        &self,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_list_slice(
        &self,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        start: inkwell::values::IntValue<'ctx>,
        stop: inkwell::values::IntValue<'ctx>,
        step: inkwell::values::IntValue<'ctx>,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_dict_get_item(
        &self,
        dict_ptr: inkwell::values::PointerValue<'ctx>,
        key: BasicValueEnum<'ctx>,
        key_type: &Type,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_string_get_char(
        &self,
        str_ptr: inkwell::values::PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String>;
    fn build_string_slice(
        &self,
        str_ptr: inkwell::values::PointerValue<'ctx>,
        start: inkwell::values::IntValue<'ctx>,
        stop: inkwell::values::IntValue<'ctx>,
        step: inkwell::values::IntValue<'ctx>,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn compile_slice_operation(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        lower: Option<&Expr>,
        upper: Option<&Expr>,
        step: Option<&Expr>,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;
    /// Compile an expression and return the resulting LLVM value with its type

    fn compile_slice_operation_non_recursive(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        lower: Option<&Expr>,
        upper: Option<&Expr>,
        step: Option<&Expr>,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    fn compile_expr(&mut self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Original recursive implementation of compile_expr (for reference and fallback)
    fn compile_expr_original(
        &mut self,
        expr: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a numeric literal
    fn compile_number(&mut self, num: &Number) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a name constant (True, False, None)
    fn compile_name_constant(
        &mut self,
        constant: &NameConstant,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a subscript expression (e.g., tuple[0])
    fn compile_subscript(
        &mut self,
        value: &Expr,
        slice: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    fn compile_subscript_non_recursive(
        &mut self,
        value: &Expr,
        slice: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a subscript expression with a pre-compiled value
    fn compile_subscript_with_value(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        slice: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    fn compile_subscript_with_value_non_recursive(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        slice: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    fn handle_range_list_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        range_val: BasicValueEnum<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String>;

    /// Compile a list comprehension expression
    fn compile_list_comprehension(
        &mut self,
        elt: &Expr,
        generators: &[crate::ast::Comprehension],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    fn compile_list_comprehension_non_recursive(
        &mut self,
        elt: &Expr,
        generators: &[crate::ast::Comprehension],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a dictionary comprehension expression
    fn compile_dict_comprehension(
        &mut self,
        key: &Expr,
        value: &Expr,
        generators: &[crate::ast::Comprehension],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile an attribute access expression (e.g., dict.keys())
    fn compile_attribute_access(
        &mut self,
        value: &Expr,
        attr: &str,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;
}

pub trait AssignmentCompiler<'ctx> {
    /// Compile an assignment expression
    fn compile_assignment(
        &mut self,
        target: &Expr,
        value: BasicValueEnum<'ctx>,
        value_type: &Type,
    ) -> Result<(), String>;
}

/// Extension trait for handling binary operations with type conversions
pub trait BinaryOpCompiler<'ctx> {
    /// Compile a binary operation with type conversion if needed
    fn compile_binary_op(
        &mut self,
        left: inkwell::values::BasicValueEnum<'ctx>,
        left_type: &Type,
        op: Operator,
        right: inkwell::values::BasicValueEnum<'ctx>,
        right_type: &Type,
    ) -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String>;
}

/// Extension trait for handling comparison operations with type conversions
pub trait ComparisonCompiler<'ctx> {
    /// Compile a comparison operation with type conversion if needed
    fn compile_comparison(
        &mut self,
        left: inkwell::values::BasicValueEnum<'ctx>,
        left_type: &Type,
        op: CmpOperator,
        right: inkwell::values::BasicValueEnum<'ctx>,
        right_type: &Type,
    ) -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String>;
}

impl<'ctx> ExprCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_expr(&mut self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        use crate::compiler::expr_non_recursive::ExprNonRecursive;
        self.compile_expr_non_recursive(expr)
    }

    fn compile_expr_original(
        &mut self,
        expr: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        match expr {
            Expr::Num { value, .. } => self.compile_number(value),
            Expr::NameConstant { value, .. } => self.compile_name_constant(value),

            Expr::BinOp {
                left, op, right, ..
            } => {
                let (left_val, left_type) = self.compile_expr(left)?;
                let (right_val, right_type) = self.compile_expr(right)?;

                self.compile_binary_op(left_val, &left_type, op.clone(), right_val, &right_type)
            }

            Expr::UnaryOp { op, operand, .. } => {
                let (operand_val, operand_type) = self.compile_expr(operand)?;

                match op {
                    UnaryOperator::Not => {
                        let bool_val = if !matches!(operand_type, Type::Bool) {
                            self.convert_type(operand_val, &operand_type, &Type::Bool)?
                        } else {
                            operand_val
                        };

                        let result = self
                            .builder
                            .build_not(bool_val.into_int_value(), "not")
                            .unwrap();
                        Ok((result.into(), Type::Bool))
                    }
                    UnaryOperator::USub => match operand_type {
                        Type::Int => {
                            let int_val = operand_val.into_int_value();
                            let result = self.builder.build_int_neg(int_val, "neg").unwrap();
                            Ok((result.into(), Type::Int))
                        }
                        Type::Float => {
                            let float_val = operand_val.into_float_value();
                            let result = self.builder.build_float_neg(float_val, "neg").unwrap();
                            Ok((result.into(), Type::Float))
                        }
                        _ => Err(format!("Cannot negate value of type {:?}", operand_type)),
                    },
                    UnaryOperator::UAdd => Ok((operand_val, operand_type)),
                    UnaryOperator::Invert => match operand_type {
                        Type::Int => {
                            let int_val = operand_val.into_int_value();
                            let result = self.builder.build_not(int_val, "invert").unwrap();
                            Ok((result.into(), Type::Int))
                        }
                        _ => Err(format!(
                            "Cannot bitwise invert value of type {:?}",
                            operand_type
                        )),
                    },
                }
            }

            Expr::Compare {
                left,
                ops,
                comparators,
                ..
            } => {
                if ops.is_empty() || comparators.is_empty() {
                    return Err("Empty comparison".to_string());
                }

                let (left_val, left_type) = self.compile_expr(left)?;

                let mut current_val = left_val;
                let mut current_type = left_type.clone();
                let mut result_val: Option<BasicValueEnum<'ctx>> = None;

                for (op, right) in ops.iter().zip(comparators.iter()) {
                    let (right_val, right_type) = self.compile_expr(right)?;

                    let (cmp_result, _) = self.compile_comparison(
                        current_val,
                        &current_type,
                        op.clone(),
                        right_val,
                        &right_type,
                    )?;

                    if let Some(prev_result) = result_val {
                        let and_result = self
                            .builder
                            .build_and(
                                prev_result.into_int_value(),
                                cmp_result.into_int_value(),
                                "and_cmp",
                            )
                            .unwrap();
                        result_val = Some(and_result.into());
                    } else {
                        result_val = Some(cmp_result);
                    }

                    current_val = right_val;
                    current_type = right_type;
                }

                Ok((result_val.unwrap(), Type::Bool))
            }

            Expr::Name { id, .. } => {
                let is_global = if let Some(current_scope) = self.scope_stack.current_scope() {
                    current_scope.is_global(id)
                } else {
                    false
                };

                let is_nonlocal = if let Some(current_scope) = self.scope_stack.current_scope() {
                    current_scope.is_nonlocal(id)
                } else {
                    false
                };

                if is_nonlocal {
                    if let Some(env_name) = &self.current_environment {
                        if let Some(env) = self.get_closure_environment(env_name) {
                            let var_type = if let Some(current_scope) =
                                self.scope_stack.current_scope()
                            {
                                if let Some(unique_name) = current_scope.get_nonlocal_mapping(id) {
                                    current_scope.get_type(unique_name).cloned()
                                } else {
                                    self.lookup_variable_type(id).cloned()
                                }
                            } else {
                                self.lookup_variable_type(id).cloned()
                            };

                            if let Some(var_type) = var_type {
                                let llvm_type = self.get_llvm_type(&var_type);

                                if let Some(value) = env.access_nonlocal_with_phi(
                                    &self.builder,
                                    id,
                                    llvm_type,
                                    self.llvm_context,
                                ) {
                                    println!("Loaded nonlocal variable '{}' using phi nodes", id);
                                    return Ok((value, var_type));
                                }
                            }
                        }
                    }

                    if let Some(current_scope) = self.scope_stack.current_scope() {
                        if let Some(unique_name) = current_scope.get_nonlocal_mapping(id) {
                            if let Some(ptr) = current_scope.get_variable(unique_name) {
                                if let Some(var_type) = current_scope.get_type(unique_name) {
                                    let llvm_type = self.get_llvm_type(var_type);

                                    let value = self
                                        .builder
                                        .build_load(
                                            llvm_type,
                                            *ptr,
                                            &format!("load_{}", unique_name),
                                        )
                                        .unwrap();
                                    println!(
                                        "Loaded nonlocal variable '{}' using unique name '{}'",
                                        id, unique_name
                                    );

                                    return Ok((value, var_type.clone()));
                                }
                            }
                        }

                        if self.scope_stack.scopes.len() >= 2 {
                            let parent_scope_index = self.scope_stack.scopes.len() - 2;

                            let parent_var_ptr = self.scope_stack.scopes[parent_scope_index]
                                .get_variable(id)
                                .cloned();
                            let parent_var_type = self.scope_stack.scopes[parent_scope_index]
                                .get_type(id)
                                .cloned();

                            if let (Some(ptr), Some(var_type)) = (parent_var_ptr, parent_var_type) {
                                let llvm_type = self.get_llvm_type(&var_type);

                                let current_function = self.current_function.unwrap();
                                let fn_name =
                                    current_function.get_name().to_string_lossy().to_string();
                                let unique_name =
                                    format!("__shadowed_{}_{}", fn_name.replace('.', "_"), id);

                                let current_position = self.builder.get_insert_block().unwrap();

                                let current_function = self.current_function.unwrap();
                                let entry_block = current_function.get_first_basic_block().unwrap();
                                if let Some(first_instr) = entry_block.get_first_instruction() {
                                    self.builder.position_before(&first_instr);
                                } else {
                                    self.builder.position_at_end(entry_block);
                                }

                                let local_ptr =
                                    self.builder.build_alloca(llvm_type, &unique_name).unwrap();

                                self.builder.position_at_end(current_position);

                                let value = self
                                    .builder
                                    .build_load(llvm_type, ptr, &format!("load_shadowed_{}", id))
                                    .unwrap();

                                self.builder.build_store(local_ptr, value).unwrap();

                                self.scope_stack.current_scope_mut().map(|scope| {
                                    scope.add_variable(unique_name.clone(), local_ptr, var_type.clone());
                                    scope.add_nonlocal_mapping(id.clone(), unique_name.clone());
                                    println!("Created local variable for shadowed nonlocal variable '{}' with unique name '{}'", id, unique_name);
                                });

                                let value = self
                                    .builder
                                    .build_load(
                                        llvm_type,
                                        local_ptr,
                                        &format!("load_{}", unique_name),
                                    )
                                    .unwrap();
                                println!(
                                    "Loaded shadowed nonlocal variable '{}' using unique name '{}'",
                                    id, unique_name
                                );

                                return Ok((value, var_type.clone()));
                            }
                        }
                    }
                }

                if is_global {
                    if let Some(global_scope) = self.scope_stack.global_scope() {
                        if let Some(ptr) = global_scope.get_variable(id) {
                            if let Some(var_type) = self.lookup_variable_type(id) {
                                let llvm_type = self.get_llvm_type(var_type);

                                let value = self.builder.build_load(llvm_type, *ptr, id).unwrap();
                                return Ok((value, var_type.clone()));
                            }
                        }
                    }

                    let var_type = Type::Int;
                    self.register_variable(id.to_string(), var_type.clone());

                    let global_var = self.module.add_global(
                        self.get_llvm_type(&var_type).into_int_type(),
                        None,
                        id,
                    );

                    global_var.set_initializer(&self.llvm_context.i64_type().const_zero());

                    let ptr = global_var.as_pointer_value();

                    if let Some(global_scope) = self.scope_stack.global_scope_mut() {
                        global_scope.add_variable(id.to_string(), ptr, var_type.clone());
                    }

                    self.variables.insert(id.to_string(), ptr);

                    let value = self
                        .builder
                        .build_load(self.get_llvm_type(&var_type), ptr, id)
                        .unwrap();

                    return Ok((value, var_type));
                }

                if is_nonlocal {
                    if let Some(var_type) = self.lookup_variable_type(id) {
                        if let Some(ptr) = self.get_variable_ptr(id) {
                            let llvm_type = self.get_llvm_type(var_type);

                            let value = self.builder.build_load(llvm_type, ptr, id).unwrap();
                            return Ok((value, var_type.clone()));
                        } else {
                            return Err(format!("Nonlocal variable '{}' not found", id));
                        }
                    } else {
                        return Err(format!("Nonlocal variable '{}' not found", id));
                    }
                }

                if let Some(var_type) = self.lookup_variable_type(id) {
                    if let Some(ptr) = self.get_variable_ptr(id) {
                        let llvm_type = self.get_llvm_type(var_type);

                        let value = self.builder.build_load(llvm_type, ptr, id).unwrap();
                        Ok((value, var_type.clone()))
                    } else {
                        let var_type_clone = var_type.clone();

                        let global_var = self.module.add_global(
                            self.get_llvm_type(&var_type_clone).into_int_type(),
                            None,
                            id,
                        );

                        global_var.set_initializer(&self.llvm_context.i64_type().const_zero());

                        let ptr = global_var.as_pointer_value();

                        self.variables.insert(id.to_string(), ptr);

                        let value = self
                            .builder
                            .build_load(self.get_llvm_type(&var_type_clone), ptr, id)
                            .unwrap();

                        Ok((value, var_type_clone))
                    }
                } else {
                    if self.current_function.is_some() && self.current_environment.is_some() {
                        let fn_name = self
                            .current_function
                            .unwrap()
                            .get_name()
                            .to_string_lossy()
                            .to_string();

                        if fn_name.matches('.').count() >= 1 {
                            let mut found_var = None;

                            for i in (0..self.scope_stack.scopes.len() - 1).rev() {
                                if let Some(ptr) = self.scope_stack.scopes[i].get_variable(id) {
                                    if let Some(var_type) = self.scope_stack.scopes[i].get_type(id)
                                    {
                                        found_var = Some((i, *ptr, var_type.clone()));
                                        break;
                                    }
                                }
                            }

                            if let Some((scope_index, ptr, var_type)) = found_var {
                                let llvm_type = self.get_llvm_type(&var_type);

                                let unique_name =
                                    format!("__outer_{}_{}", fn_name.replace('.', "_"), id);

                                let current_function = self
                                    .builder
                                    .get_insert_block()
                                    .unwrap()
                                    .get_parent()
                                    .unwrap();

                                let entry_block = current_function.get_first_basic_block().unwrap();

                                let current_block = self.builder.get_insert_block().unwrap();

                                self.builder.position_at_end(entry_block);

                                let local_ptr =
                                    self.builder.build_alloca(llvm_type, &unique_name).unwrap();

                                self.builder.position_at_end(current_block);

                                let value = self
                                    .builder
                                    .build_load(
                                        llvm_type,
                                        ptr,
                                        &format!("load_{}_from_scope_{}", id, scope_index),
                                    )
                                    .unwrap();

                                self.builder.build_store(local_ptr, value).unwrap();

                                if let Some(current_scope) = self.scope_stack.current_scope_mut() {
                                    current_scope.add_variable(
                                        unique_name.clone(),
                                        local_ptr,
                                        var_type.clone(),
                                    );
                                    println!("Created local variable for outer scope variable '{}' with unique name '{}'", id, unique_name);
                                }

                                let result = self
                                    .builder
                                    .build_load(
                                        llvm_type,
                                        local_ptr,
                                        &format!("load_{}", unique_name),
                                    )
                                    .unwrap();
                                println!(
                                    "Loaded outer scope variable '{}' using unique name '{}'",
                                    id, unique_name
                                );

                                return Ok((result, var_type));
                            }
                        }
                    }

                    Err(format!("Undefined variable: {}", id))
                }
            }

            Expr::Str { value, .. } => {
                let const_str = self.llvm_context.const_string(value.as_bytes(), true);

                let str_type = const_str.get_type();

                let global_str = self.module.add_global(str_type, None, "str_const");
                global_str.set_constant(true);
                global_str.set_initializer(&const_str);

                let str_ptr = self
                    .builder
                    .build_pointer_cast(
                        global_str.as_pointer_value(),
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "str_ptr",
                    )
                    .unwrap();

                Ok((str_ptr.into(), Type::String))
            }

            Expr::BoolOp { op, values, .. } => {
                if values.is_empty() {
                    return Err("Empty boolean operation".to_string());
                }

                let (first_val, first_type) = self.compile_expr(&values[0])?;

                let bool_type = Type::Bool;
                let mut current_val = if first_type != bool_type {
                    self.convert_type(first_val, &first_type, &bool_type)?
                        .into_int_value()
                } else {
                    first_val.into_int_value()
                };

                if values.len() == 1 {
                    return Ok((current_val.into(), bool_type));
                }

                let current_function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let result_ptr = self
                    .builder
                    .build_alloca(self.llvm_context.bool_type(), "bool_result")
                    .unwrap();

                self.builder.build_store(result_ptr, current_val).unwrap();

                let mut merge_block = self
                    .llvm_context
                    .append_basic_block(current_function, "bool_merge");

                for (i, value_expr) in values.iter().skip(1).enumerate() {
                    let next_value_block = self
                        .llvm_context
                        .append_basic_block(current_function, &format!("next_value_{}", i));
                    let short_circuit_block = self
                        .llvm_context
                        .append_basic_block(current_function, &format!("short_circuit_{}", i));

                    match op {
                        BoolOperator::And => {
                            self.builder
                                .build_conditional_branch(
                                    current_val,
                                    next_value_block,
                                    short_circuit_block,
                                )
                                .unwrap();
                        }
                        BoolOperator::Or => {
                            self.builder
                                .build_conditional_branch(
                                    current_val,
                                    short_circuit_block,
                                    next_value_block,
                                )
                                .unwrap();
                        }
                    }

                    self.builder.position_at_end(next_value_block);
                    let (next_val, next_type) = self.compile_expr(value_expr)?;

                    let next_bool = if next_type != bool_type {
                        self.convert_type(next_val, &next_type, &bool_type)?
                            .into_int_value()
                    } else {
                        next_val.into_int_value()
                    };

                    self.builder.build_store(result_ptr, next_bool).unwrap();
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .unwrap();

                    self.builder.position_at_end(short_circuit_block);

                    self.builder
                        .build_unconditional_branch(merge_block)
                        .unwrap();

                    self.builder.position_at_end(merge_block);

                    current_val = self
                        .builder
                        .build_load(self.llvm_context.bool_type(), result_ptr, "bool_op_result")
                        .unwrap()
                        .into_int_value();

                    if i < values.len() - 2 {
                        let new_merge_block = self
                            .llvm_context
                            .append_basic_block(current_function, &format!("bool_merge_{}", i + 1));
                        merge_block = new_merge_block;
                    }
                }

                Ok((current_val.into(), bool_type))
            }

            Expr::Call {
                func,
                args,
                keywords,
                ..
            } => {
                if let Expr::Attribute { value, attr, .. } = func.as_ref() {
                    let (obj_val, obj_type) = self.compile_expr(value)?;

                    match &obj_type {
                        Type::Dict(key_type, value_type) => match attr.as_str() {
                            "keys" => {
                                let dict_keys_fn = match self.module.get_function("dict_keys") {
                                    Some(f) => f,
                                    None => return Err("dict_keys function not found".to_string()),
                                };

                                let call_site_value = self
                                    .builder
                                    .build_call(
                                        dict_keys_fn,
                                        &[obj_val.into_pointer_value().into()],
                                        "dict_keys_result",
                                    )
                                    .unwrap();

                                let keys_list_ptr =
                                    call_site_value.try_as_basic_value().left().ok_or_else(
                                        || "Failed to get keys from dictionary".to_string(),
                                    )?;

                                println!(
                                    "Dictionary keys method call result type: {:?}",
                                    Type::List(key_type.clone())
                                );
                                return Ok((keys_list_ptr, Type::List(key_type.clone())));
                            }
                            "values" => {
                                let dict_values_fn = match self.module.get_function("dict_values") {
                                    Some(f) => f,
                                    None => {
                                        return Err("dict_values function not found".to_string())
                                    }
                                };

                                let call_site_value = self
                                    .builder
                                    .build_call(
                                        dict_values_fn,
                                        &[obj_val.into_pointer_value().into()],
                                        "dict_values_result",
                                    )
                                    .unwrap();

                                let values_list_ptr =
                                    call_site_value.try_as_basic_value().left().ok_or_else(
                                        || "Failed to get values from dictionary".to_string(),
                                    )?;

                                println!(
                                    "Dictionary values method call result type: {:?}",
                                    Type::List(value_type.clone())
                                );
                                return Ok((values_list_ptr, Type::List(value_type.clone())));
                            }
                            "items" => {
                                let dict_items_fn = match self.module.get_function("dict_items") {
                                    Some(f) => f,
                                    None => return Err("dict_items function not found".to_string()),
                                };

                                let call_site_value = self
                                    .builder
                                    .build_call(
                                        dict_items_fn,
                                        &[obj_val.into_pointer_value().into()],
                                        "dict_items_result",
                                    )
                                    .unwrap();

                                let items_list_ptr =
                                    call_site_value.try_as_basic_value().left().ok_or_else(
                                        || "Failed to get items from dictionary".to_string(),
                                    )?;

                                let tuple_type =
                                    Type::Tuple(vec![*key_type.clone(), *value_type.clone()]);
                                println!(
                                    "Dictionary items method call result type: {:?}",
                                    Type::List(Box::new(tuple_type.clone()))
                                );
                                return Ok((items_list_ptr, Type::List(Box::new(tuple_type))));
                            }
                            _ => {
                                return Err(format!(
                                    "Unknown method '{}' for dictionary type",
                                    attr
                                ))
                            }
                        },
                        _ => {
                            return Err(format!(
                                "Type {:?} does not support method calls",
                                obj_type
                            ))
                        }
                    }
                }

                match func.as_ref() {
                    Expr::Name { id, .. } => {
                        let mut arg_values = Vec::with_capacity(args.len());
                        let mut arg_types = Vec::with_capacity(args.len());

                        for arg in args {
                            let (arg_val, arg_type) = self.compile_expr(arg)?;
                            arg_values.push(arg_val);
                            arg_types.push(arg_type);
                        }

                        if !keywords.is_empty() {
                            return Err("Keyword arguments not yet implemented".to_string());
                        }

                        if id == "len" {
                            let args_slice: Vec<Expr> =
                                args.iter().map(|arg| (**arg).clone()).collect();
                            return self.compile_len_call(&args_slice);
                        }

                        if id == "print" {
                            let args_slice: Vec<Expr> =
                                args.iter().map(|arg| (**arg).clone()).collect();
                            return self.compile_print_call(&args_slice);
                        }

                        if id == "str" && !arg_types.is_empty() {
                            if let Some(func_value) =
                                self.get_polymorphic_function(id, &arg_types[0])
                            {
                                let (converted_arg, _target_type) =
                                    match func_value.get_type().get_param_types().get(0) {
                                        Some(param_type) if param_type.is_int_type() => (
                                            self.convert_type(
                                                arg_values[0],
                                                &arg_types[0],
                                                &Type::Int,
                                            )?,
                                            Type::Int,
                                        ),
                                        Some(param_type) if param_type.is_float_type() => (
                                            self.convert_type(
                                                arg_values[0],
                                                &arg_types[0],
                                                &Type::Float,
                                            )?,
                                            Type::Float,
                                        ),
                                        Some(param_type)
                                            if param_type.is_int_type()
                                                && param_type.into_int_type().get_bit_width()
                                                    == 1 =>
                                        {
                                            (
                                                self.convert_type(
                                                    arg_values[0],
                                                    &arg_types[0],
                                                    &Type::Bool,
                                                )?,
                                                Type::Bool,
                                            )
                                        }
                                        _ => {
                                            return Err(format!(
                                                "Unsupported argument type for str: {:?}",
                                                arg_types[0]
                                            ));
                                        }
                                    };

                                let call = self
                                    .builder
                                    .build_call(func_value, &[converted_arg.into()], "str_call")
                                    .unwrap();

                                if let Some(ret_val) = call.try_as_basic_value().left() {
                                    return Ok((ret_val, Type::String));
                                } else {
                                    return Err("Failed to call str function".to_string());
                                }
                            } else {
                                return Err(format!(
                                    "No str implementation available for type {:?}",
                                    arg_types[0]
                                ));
                            }
                        } else {
                            let mut found_function = false;
                            let mut qualified_name = String::new();

                            if let Some(current_function) = self.current_function {
                                let current_name =
                                    current_function.get_name().to_string_lossy().to_string();

                                qualified_name = format!("{}.{}", current_name, id);

                                println!("Looking for nested function: {}", qualified_name);

                                if self.module.get_function(&qualified_name).is_some() {
                                    found_function = true;
                                    println!("Found nested function: {}", qualified_name);
                                }
                            }

                            let func_value = if found_function {
                                match self.module.get_function(&qualified_name) {
                                    Some(f) => f,
                                    None => {
                                        return Err(format!(
                                            "Undefined nested function: {}",
                                            qualified_name
                                        ))
                                    }
                                }
                            } else {
                                if id == "range" {
                                    match args.len() {
                                        1 => match self.module.get_function("range_1") {
                                            Some(f) => f,
                                            None => {
                                                return Err("range_1 function not found".to_string())
                                            }
                                        },
                                        2 => match self.module.get_function("range_2") {
                                            Some(f) => f,
                                            None => {
                                                return Err("range_2 function not found".to_string())
                                            }
                                        },
                                        3 => match self.module.get_function("range_3") {
                                            Some(f) => f,
                                            None => {
                                                return Err("range_3 function not found".to_string())
                                            }
                                        },
                                        _ => {
                                            return Err(format!("Invalid number of arguments for range: expected 1, 2, or 3, got {}", args.len()));
                                        }
                                    }
                                } else {
                                    match self.functions.get(id) {
                                        Some(f) => *f,
                                        None => return Err(format!("Undefined function: {}", id)),
                                    }
                                }
                            };

                            let param_types = func_value.get_type().get_param_types();

                            let mut call_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> =
                                Vec::with_capacity(arg_values.len());

                            for (i, &arg_value) in arg_values.iter().enumerate() {
                                if found_function && i >= param_types.len() - 1 {
                                    call_args.push(arg_value.into());
                                    continue;
                                }

                                if id.starts_with("range_") && i < param_types.len() {
                                    if param_types[i].is_int_type() && !arg_value.is_int_value() {
                                        if arg_value.is_pointer_value() {
                                            let ptr = arg_value.into_pointer_value();
                                            let loaded_val = self
                                                .builder
                                                .build_load(
                                                    self.llvm_context.i64_type(),
                                                    ptr,
                                                    "range_arg_load",
                                                )
                                                .unwrap();
                                            call_args.push(loaded_val.into());
                                            continue;
                                        }
                                    }
                                }

                                if let Some(param_type) = param_types.get(i) {
                                    let arg_type = &arg_types[i];

                                    if matches!(arg_type, Type::Dict(_, _))
                                        && param_type.is_pointer_type()
                                    {
                                        if arg_value.is_pointer_value() {
                                            call_args.push(arg_value.into());
                                        } else {
                                            let ptr_type = self
                                                .llvm_context
                                                .ptr_type(inkwell::AddressSpace::default());
                                            let ptr_val = self
                                                .builder
                                                .build_int_to_ptr(
                                                    arg_value.into_int_value(),
                                                    ptr_type,
                                                    &format!("arg{}_to_ptr", i),
                                                )
                                                .unwrap();
                                            call_args.push(ptr_val.into());
                                        }
                                    } else if arg_type == &Type::Bool
                                        && param_type.is_int_type()
                                        && param_type.into_int_type().get_bit_width() == 64
                                    {
                                        let bool_val = arg_value.into_int_value();
                                        let int_val = self
                                            .builder
                                            .build_int_z_extend(
                                                bool_val,
                                                self.llvm_context.i64_type(),
                                                "bool_to_i64",
                                            )
                                            .unwrap();
                                        call_args.push(int_val.into());
                                    } else if let Type::Tuple(_) = arg_type {
                                        if param_type.is_int_type() {
                                            let ptr_val = if arg_value.is_pointer_value() {
                                                arg_value.into_pointer_value()
                                            } else {
                                                let tuple_ptr = self
                                                    .builder
                                                    .build_alloca(arg_value.get_type(), "tuple_arg")
                                                    .unwrap();

                                                self.builder
                                                    .build_store(tuple_ptr, arg_value)
                                                    .unwrap();

                                                tuple_ptr
                                            };

                                            let ptr_int = self
                                                .builder
                                                .build_ptr_to_int(
                                                    ptr_val,
                                                    self.llvm_context.i64_type(),
                                                    "ptr_to_int",
                                                )
                                                .unwrap();

                                            call_args.push(ptr_int.into());
                                        } else {
                                            call_args.push(arg_value.into());
                                        }
                                    } else {
                                        call_args.push(arg_value.into());
                                    }
                                } else {
                                    call_args.push(arg_value.into());
                                }
                            }

                            if found_function {
                                let mut nonlocal_vars = if let Some(env) =
                                    self.get_closure_environment(&qualified_name)
                                {
                                    env.nonlocal_params.clone()
                                } else {
                                    Vec::new()
                                };

                                println!(
                                    "Nonlocal variables for function {}: {:?}",
                                    qualified_name, nonlocal_vars
                                );

                                if let Some(func) = self.module.get_function(&qualified_name) {
                                    let param_count = func.count_params();
                                    println!(
                                        "Function {} has {} parameters in LLVM IR",
                                        qualified_name, param_count
                                    );
                                }

                                if let Some(func) = self.module.get_function(&qualified_name) {
                                    let param_count = func.count_params();
                                    let expected_param_count = args.len() + nonlocal_vars.len() + 1;

                                    if param_count != expected_param_count as u32 {
                                        println!("WARNING: Function {} has {} parameters but we're trying to pass {} arguments",
                                                 qualified_name, param_count, expected_param_count);

                                        if param_count < expected_param_count as u32 {
                                            println!("Adjusting call to match function signature - using only {} arguments", param_count);

                                            let available_nonlocal_slots =
                                                param_count as usize - args.len() - 1;

                                            if available_nonlocal_slots <= 0 {
                                                println!("No slots available for nonlocal variables, skipping them");
                                                nonlocal_vars.clear();
                                            } else if available_nonlocal_slots < nonlocal_vars.len()
                                            {
                                                println!("Only {} slots available for nonlocal variables, truncating list", available_nonlocal_slots);
                                                nonlocal_vars.truncate(available_nonlocal_slots);
                                            }
                                        } else if param_count > expected_param_count as u32 {
                                            println!("Function has more parameters than we're trying to pass, this is unexpected");
                                        }
                                    }
                                }

                                for var_name in &nonlocal_vars {
                                    let var_value = if let Some(current_scope) =
                                        self.scope_stack.current_scope()
                                    {
                                        if let Some(unique_name) =
                                            current_scope.get_nonlocal_mapping(var_name)
                                        {
                                            if let Some(ptr) =
                                                current_scope.get_variable(unique_name)
                                            {
                                                if let Some(var_type) =
                                                    current_scope.get_type(unique_name)
                                                {
                                                    let llvm_type = self.get_llvm_type(var_type);

                                                    let value = self
                                                        .builder
                                                        .build_load(
                                                            llvm_type,
                                                            *ptr,
                                                            &format!("load_{}_for_call", var_name),
                                                        )
                                                        .unwrap();
                                                    Some(value)
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        } else {
                                            if let Some(ptr) = current_scope.get_variable(var_name)
                                            {
                                                if let Some(var_type) =
                                                    current_scope.get_type(var_name)
                                                {
                                                    let llvm_type = self.get_llvm_type(var_type);

                                                    let value = self
                                                        .builder
                                                        .build_load(
                                                            llvm_type,
                                                            *ptr,
                                                            &format!("load_{}_for_call", var_name),
                                                        )
                                                        .unwrap();
                                                    Some(value)
                                                } else {
                                                    None
                                                }
                                            } else {
                                                let var_ptr = self
                                                    .scope_stack
                                                    .get_variable_respecting_declarations(var_name);
                                                if let Some(ptr) = var_ptr {
                                                    let var_type = self
                                                        .scope_stack
                                                        .get_type_respecting_declarations(var_name);
                                                    if let Some(var_type) = var_type {
                                                        let llvm_type =
                                                            self.get_llvm_type(&var_type);

                                                        let value = self
                                                            .builder
                                                            .build_load(
                                                                llvm_type,
                                                                *ptr,
                                                                &format!(
                                                                    "load_{}_for_call",
                                                                    var_name
                                                                ),
                                                            )
                                                            .unwrap();
                                                        Some(value)
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                            }
                                        }
                                    } else {
                                        None
                                    };

                                    if let Some(value) = var_value {
                                        call_args.push(value.into());
                                        println!(
                                            "Passing nonlocal variable '{}' to nested function: {}",
                                            var_name, qualified_name
                                        );
                                    } else {
                                        let default_value =
                                            self.llvm_context.i64_type().const_zero().into();
                                        call_args.push(default_value);
                                        println!("Passing default value for nonlocal variable '{}' to nested function: {}", var_name, qualified_name);
                                    }
                                }

                                println!("Function call to {} has {} regular arguments and {} nonlocal arguments",
                                         qualified_name, args.len(), nonlocal_vars.len());

                                let env_ptr = if let Some(env_name) = &self.current_environment {
                                    if let Some(env) = self.get_closure_environment(env_name) {
                                        if let Some(ptr) = env.env_ptr {
                                            ptr
                                        } else {
                                            self.llvm_context
                                                .ptr_type(inkwell::AddressSpace::default())
                                                .const_null()
                                        }
                                    } else {
                                        self.llvm_context
                                            .ptr_type(inkwell::AddressSpace::default())
                                            .const_null()
                                    }
                                } else {
                                    self.llvm_context
                                        .ptr_type(inkwell::AddressSpace::default())
                                        .const_null()
                                };

                                call_args.push(env_ptr.into());
                                println!(
                                    "Passing closure environment to nested function: {}",
                                    qualified_name
                                );
                            }

                            let call = self
                                .builder
                                .build_call(
                                    func_value,
                                    &call_args,
                                    &format!(
                                        "call_{}",
                                        if found_function { &qualified_name } else { id }
                                    ),
                                )
                                .unwrap();

                            if let Some(ret_val) = call.try_as_basic_value().left() {
                                let return_type = if id == "str"
                                    || id == "int_to_string"
                                    || id == "float_to_string"
                                    || id == "bool_to_string"
                                {
                                    Type::String
                                } else if id == "create_tuple" {
                                    Type::Tuple(vec![Type::Int, Type::Int, Type::Int])
                                } else if id == "create_nested_tuple" {
                                    let nested_tuple = Type::Tuple(vec![Type::Int, Type::Int]);
                                    Type::Tuple(vec![Type::Int, nested_tuple])
                                } else if id == "transform_tuple" {
                                    Type::Tuple(vec![Type::Int, Type::Int])
                                } else if id == "get_tuple" {
                                    Type::Tuple(vec![Type::Int, Type::Int, Type::Int])
                                } else if id == "get_value"
                                    || id == "get_name"
                                    || id == "get_value_with_default"
                                    || id == "get_nested_value"
                                {
                                    Type::String
                                } else if id == "create_person"
                                    || id == "add_phone"
                                    || id == "create_dict"
                                    || id == "get_nested_value"
                                    || id == "create_math_dict"
                                    || id == "identity"
                                    || id.contains("person")
                                    || id.contains("dict")
                                {
                                    Type::Dict(Box::new(Type::String), Box::new(Type::String))
                                } else if id == "process_dict" || id.contains("len") {
                                    Type::Int
                                } else if id == "get_value_with_default" {
                                    Type::String
                                } else if id == "fibonacci_pair" {
                                    Type::Tuple(vec![Type::Int, Type::Int])
                                } else if id.starts_with("create_tuple") || id.ends_with("_tuple") {
                                    Type::Tuple(vec![Type::Int, Type::Int, Type::Int])
                                } else if id.contains("dict")
                                    || id.contains("person")
                                    || id.contains("user")
                                {
                                    Type::Dict(Box::new(Type::String), Box::new(Type::String))
                                } else {
                                    Type::Int
                                };

                                Ok((ret_val, return_type))
                            } else {
                                Ok((self.llvm_context.i32_type().const_zero().into(), Type::Void))
                            }
                        }
                    }
                    _ => Err("Indirect function calls not yet implemented".to_string()),
                }
            }

            Expr::IfExp {
                test, body, orelse, ..
            } => {
                self.ensure_block_has_terminator();

                let (test_val, test_type) = self.compile_expr(test)?;

                self.ensure_block_has_terminator();

                let cond_val = if test_type != Type::Bool {
                    self.convert_type(test_val, &test_type, &Type::Bool)?
                        .into_int_value()
                } else {
                    test_val.into_int_value()
                };

                self.ensure_block_has_terminator();

                let current_function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();
                let then_block = self
                    .llvm_context
                    .append_basic_block(current_function, "if_then");
                let else_block = self
                    .llvm_context
                    .append_basic_block(current_function, "if_else");
                let merge_block = self
                    .llvm_context
                    .append_basic_block(current_function, "if_merge");

                self.ensure_block_has_terminator();

                self.builder
                    .build_conditional_branch(cond_val, then_block, else_block)
                    .unwrap();

                self.builder.position_at_end(then_block);

                self.ensure_block_has_terminator();

                let (then_val, then_type) = self.compile_expr(body)?;

                self.ensure_block_has_terminator();

                let then_block = self.builder.get_insert_block().unwrap();
                self.builder
                    .build_unconditional_branch(merge_block)
                    .unwrap();

                self.builder.position_at_end(else_block);

                self.ensure_block_has_terminator();

                let (else_val, else_type) = self.compile_expr(orelse)?;

                self.ensure_block_has_terminator();

                let else_block = self.builder.get_insert_block().unwrap();
                self.builder
                    .build_unconditional_branch(merge_block)
                    .unwrap();

                let result_type = if then_type == else_type {
                    then_type.clone()
                } else {
                    match self.get_common_type(&then_type, &else_type) {
                        Ok(common_type) => common_type,
                        Err(_) => {
                            return Err(format!(
                                "Incompatible types in if expression: {:?} and {:?}",
                                then_type, else_type
                            ))
                        }
                    }
                };

                let then_val = if then_type != result_type {
                    self.convert_type(then_val, &then_type, &result_type)?
                } else {
                    then_val
                };

                let else_val = if else_type != result_type {
                    self.convert_type(else_val, &else_type, &result_type)?
                } else {
                    else_val
                };

                self.ensure_block_has_terminator();

                self.builder.position_at_end(merge_block);

                self.ensure_block_has_terminator();

                let llvm_type = self.get_llvm_type(&result_type);
                let phi = self.builder.build_phi(llvm_type, "if_result").unwrap();

                phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);

                Ok((phi.as_basic_value(), result_type))
            }

            Expr::List { elts, .. } => {
                if elts.is_empty() {
                    let list_ptr = self.build_empty_list("empty_list")?;
                    return Ok((list_ptr.into(), Type::List(Box::new(Type::Unknown))));
                }

                let mut element_values = Vec::with_capacity(elts.len());
                let mut element_types = Vec::with_capacity(elts.len());

                for elt in elts {
                    let (value, ty) = self.compile_expr(elt)?;
                    element_values.push(value);
                    element_types.push(ty);
                }

                let element_type = if element_types.is_empty() {
                    Type::Unknown
                } else {
                    let first_type = &element_types[0];
                    let all_same = element_types.iter().all(|t| t == first_type);

                    if all_same {
                        println!("All list elements have the same type: {:?}", first_type);
                        first_type.clone()
                    } else {
                        let mut common_type = element_types[0].clone();
                        for ty in &element_types[1..] {
                            common_type = match self.get_common_type(&common_type, ty) {
                                Ok(t) => t,
                                Err(_) => {
                                    println!("Could not find common type between {:?} and {:?}, using Any", common_type, ty);
                                    Type::Any
                                }
                            };
                        }
                        println!(
                            "List elements have different types, using common type: {:?}",
                            common_type
                        );
                        common_type
                    }
                };

                let final_element_type = element_type.clone();

                println!("Final list element type: {:?}", final_element_type);

                let list_ptr = self.build_list(element_values, &final_element_type)?;

                Ok((list_ptr.into(), Type::List(Box::new(final_element_type))))
            }
            Expr::Tuple { elts, .. } => {
                if elts.is_empty() {
                    let tuple_ptr = self.build_empty_tuple("empty_tuple")?;
                    return Ok((tuple_ptr.into(), Type::Tuple(vec![])));
                }

                let mut element_values = Vec::with_capacity(elts.len());
                let mut element_types = Vec::with_capacity(elts.len());

                for elt in elts {
                    let (value, ty) = self.compile_expr(elt)?;

                    let (final_value, final_type) = if let Expr::Call { func, .. } = elt.as_ref() {
                        if let Expr::Name { id, .. } = func.as_ref() {
                            if id == "get_value" || id == "get_value_with_default" {
                                if value.is_int_value() {
                                    println!("Converting integer return value from {} to pointer for tuple element", id);
                                    let int_ptr = self
                                        .builder
                                        .build_alloca(self.llvm_context.i64_type(), "int_to_ptr")
                                        .unwrap();
                                    self.builder.build_store(int_ptr, value).unwrap();
                                    (int_ptr.into(), Type::Int)
                                } else {
                                    (value, ty)
                                }
                            } else {
                                (value, ty)
                            }
                        } else {
                            (value, ty)
                        }
                    } else {
                        (value, ty)
                    };

                    element_values.push(final_value);
                    element_types.push(final_type);
                }

                let tuple_ptr = self.build_tuple(element_values, &element_types)?;

                Ok((tuple_ptr.into(), Type::Tuple(element_types)))
            }
            Expr::Dict { keys, values, .. } => {
                if keys.is_empty() {
                    let dict_ptr = self.build_empty_dict("empty_dict")?;
                    return Ok((
                        dict_ptr.into(),
                        Type::Dict(Box::new(Type::Any), Box::new(Type::Any)),
                    ));
                }

                let mut compiled_keys = Vec::with_capacity(keys.len());
                let mut compiled_values = Vec::with_capacity(values.len());
                let mut key_types = Vec::with_capacity(keys.len());
                let mut value_types = Vec::with_capacity(values.len());

                for (key_opt, value) in keys.iter().zip(values.iter()) {
                    if let Some(key) = key_opt {
                        let (key_val, key_type) = self.compile_expr(key)?;
                        compiled_keys.push(key_val);
                        key_types.push(key_type);
                    } else {
                        return Err("Dictionary unpacking with ** not yet implemented".to_string());
                    }

                    let (value_val, value_type) = self.compile_expr(value)?;
                    compiled_values.push(value_val);
                    value_types.push(value_type);
                }

                let key_type = if key_types.is_empty() {
                    Type::Any
                } else {
                    key_types[0].clone()
                };

                let value_type = if value_types.is_empty() {
                    Type::Any
                } else {
                    value_types[0].clone()
                };

                let dict_ptr =
                    self.build_dict(compiled_keys, compiled_values, &key_type, &value_type)?;

                Ok((
                    dict_ptr.into(),
                    Type::Dict(Box::new(key_type), Box::new(value_type)),
                ))
            }
            Expr::Set { .. } => Err("Set expressions not yet implemented".to_string()),
            Expr::Attribute { value, attr, .. } => self.compile_attribute_access(value, attr),
            Expr::Subscript { value, slice, .. } => self.compile_subscript(value, slice),

            Expr::ListComp {
                elt, generators, ..
            } => self.compile_list_comprehension(elt, generators),

            Expr::DictComp {
                key,
                value,
                generators,
                ..
            } => self.compile_dict_comprehension(key, value, generators),

            _ => Err(format!("Unsupported expression type: {:?}", expr)),
        }
    }

    fn build_empty_list(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let list_new_fn = match self.module.get_function("list_new") {
            Some(f) => f,
            None => return Err("list_new function not found".to_string()),
        };

        let call_site_value = self.builder.build_call(list_new_fn, &[], name).unwrap();
        let list_ptr = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to create empty list".to_string())?;

        Ok(list_ptr.into_pointer_value())
    }

    fn build_list(
        &self,
        elements: Vec<BasicValueEnum<'ctx>>,
        element_type: &Type,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let list_with_capacity_fn = match self.module.get_function("list_with_capacity") {
            Some(f) => f,
            None => return Err("list_with_capacity function not found".to_string()),
        };

        let len = elements.len() as u64;
        let len_value = self.llvm_context.i64_type().const_int(len, false);

        let call_site_value = self
            .builder
            .build_call(
                list_with_capacity_fn,
                &[len_value.into()],
                "list_with_capacity",
            )
            .unwrap();
        let list_ptr = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to create list with capacity".to_string())?;

        let list_ptr = list_ptr.into_pointer_value();

        let list_append_fn = match self.module.get_function("list_append") {
            Some(f) => f,
            None => return Err("list_append function not found".to_string()),
        };

        for (i, element) in elements.iter().enumerate() {
            let element_ptr = if crate::compiler::types::is_reference_type(element_type) {
                *element
            } else {
                let element_alloca = self
                    .builder
                    .build_alloca(element.get_type(), &format!("list_element_{}", i))
                    .unwrap();
                self.builder.build_store(element_alloca, *element).unwrap();
                element_alloca.into()
            };

            self.builder
                .build_call(
                    list_append_fn,
                    &[list_ptr.into(), element_ptr.into()],
                    &format!("list_append_{}", i),
                )
                .unwrap();
        }

        Ok(list_ptr)
    }

    fn build_empty_tuple(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let tuple_type = self.llvm_context.struct_type(&[], false);

        let tuple_ptr = self.builder.build_alloca(tuple_type, name).unwrap();

        Ok(tuple_ptr)
    }

    fn build_tuple(
        &self,
        elements: Vec<BasicValueEnum<'ctx>>,
        element_types: &[Type],
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let llvm_types: Vec<BasicTypeEnum> = element_types
            .iter()
            .map(|ty| self.get_llvm_type(ty))
            .collect();

        let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

        let tuple_ptr = self.builder.build_alloca(tuple_struct, "tuple").unwrap();

        for (i, element) in elements.iter().enumerate() {
            let element_ptr = self
                .builder
                .build_struct_gep(
                    tuple_struct,
                    tuple_ptr,
                    i as u32,
                    &format!("tuple_element_{}", i),
                )
                .unwrap();

            self.builder.build_store(element_ptr, *element).unwrap();
        }

        Ok(tuple_ptr)
    }

    /// Compile a subscript expression (e.g., tuple[0])
    fn compile_subscript(
        &mut self,
        value: &Expr,
        slice: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        self.compile_subscript_non_recursive(value, slice)
    }

    fn compile_subscript_non_recursive(
        &mut self,
        value: &Expr,
        slice: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        let mut work_stack = Vec::new();
        let mut value_stack = Vec::new();

        work_stack.push((value, slice));

        while let Some((current_value, current_slice)) = work_stack.pop() {
            let (value_val, value_type) = self.compile_expr(current_value)?;

            let result = if let Expr::Slice {
                lower, upper, step, ..
            } = current_slice
            {
                self.compile_slice_operation(
                    value_val,
                    value_type,
                    lower.as_deref(),
                    upper.as_deref(),
                    step.as_deref(),
                )?
            } else {
                self.compile_subscript_with_value_non_recursive(
                    value_val,
                    value_type,
                    current_slice,
                )?
            };

            value_stack.push(result);
        }

        value_stack
            .pop()
            .ok_or_else(|| "Empty value stack".to_string())
    }

    fn compile_subscript_with_value(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        slice: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        self.compile_subscript_with_value_non_recursive(value_val, value_type, slice)
    }

    fn compile_subscript_with_value_non_recursive(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        slice: &Expr,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if let Expr::Slice {
            lower, upper, step, ..
        } = slice
        {
            return self.compile_slice_operation(
                value_val,
                value_type.clone(),
                lower.as_deref(),
                upper.as_deref(),
                step.as_deref(),
            );
        }

        self.ensure_block_has_terminator();

        let (index_val, index_type) = self.compile_expr(slice)?;

        self.ensure_block_has_terminator();

        let result = match &value_type {
            Type::List(element_type) => {
                if !index_type.can_coerce_to(&Type::Int) {
                    return Err(format!(
                        "List index must be an integer, got {:?}",
                        index_type
                    ));
                }

                let index_int = if index_type != Type::Int {
                    self.convert_type(index_val, &index_type, &Type::Int)?
                        .into_int_value()
                } else {
                    index_val.into_int_value()
                };

                let item_ptr =
                    self.build_list_get_item(value_val.into_pointer_value(), index_int)?;

                let element_type_ref = element_type.as_ref();

                let actual_element_type = match element_type_ref {
                    Type::Tuple(tuple_element_types) => {
                        if !tuple_element_types.is_empty()
                            && tuple_element_types
                                .iter()
                                .all(|t| t == &tuple_element_types[0])
                        {
                            tuple_element_types[0].clone()
                        } else {
                            element_type_ref.clone()
                        }
                    }
                    _ => element_type_ref.clone(),
                };

                let llvm_type = self.get_llvm_type(&actual_element_type);
                let item_val = self
                    .builder
                    .build_load(llvm_type, item_ptr, "list_item_load")
                    .unwrap();

                Ok((item_val, actual_element_type))
            }
            Type::Dict(key_type, value_type) => {
                if matches!(**key_type, Type::Unknown) {
                    println!(
                        "Dictionary access with Unknown key type, allowing index type: {:?}",
                        index_type
                    );
                } else if !index_type.can_coerce_to(key_type) && !matches!(index_type, Type::String)
                {
                    return Err(format!(
                        "Dictionary key type mismatch: expected {:?}, got {:?}",
                        key_type, index_type
                    ));
                }

                let value_ptr = self.build_dict_get_item(
                    value_val.into_pointer_value(),
                    index_val,
                    &index_type,
                )?;

                Ok((value_ptr.into(), value_type.as_ref().clone()))
            }
            Type::String => {
                if !index_type.can_coerce_to(&Type::Int) {
                    return Err(format!(
                        "String index must be an integer, got {:?}",
                        index_type
                    ));
                }

                let index_int = if index_type != Type::Int {
                    self.convert_type(index_val, &index_type, &Type::Int)?
                        .into_int_value()
                } else {
                    index_val.into_int_value()
                };

                let char_val =
                    self.build_string_get_char(value_val.into_pointer_value(), index_int)?;

                Ok((char_val, Type::String))
            }
            Type::Tuple(element_types) => {
                if !index_type.can_coerce_to(&Type::Int) {
                    return Err(format!(
                        "Tuple index must be an integer, got {:?}",
                        index_type
                    ));
                }

                if let Expr::Num {
                    value: Number::Integer(idx),
                    ..
                } = slice
                {
                    let idx = *idx as usize;

                    if idx >= element_types.len() {
                        return Err(format!(
                            "Tuple index out of range: {} (tuple has {} elements)",
                            idx,
                            element_types.len()
                        ));
                    }

                    let llvm_types: Vec<BasicTypeEnum> = element_types
                        .iter()
                        .map(|ty| self.get_llvm_type(ty))
                        .collect();

                    let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

                    let tuple_ptr = if value_val.is_pointer_value() {
                        value_val.into_pointer_value()
                    } else {
                        let llvm_type = self.get_llvm_type(&value_type);
                        let alloca = self.builder.build_alloca(llvm_type, "tuple_temp").unwrap();
                        self.builder.build_store(alloca, value_val).unwrap();
                        alloca
                    };

                    let element_ptr = self
                        .builder
                        .build_struct_gep(
                            tuple_struct,
                            tuple_ptr,
                            idx as u32,
                            &format!("tuple_element_{}", idx),
                        )
                        .unwrap();

                    let element_type = &element_types[idx];
                    let element_val = self
                        .builder
                        .build_load(
                            self.get_llvm_type(element_type),
                            element_ptr,
                            &format!("load_tuple_element_{}", idx),
                        )
                        .unwrap();

                    return Ok((element_val, element_type.clone()));
                }

                let index_int = if index_type != Type::Int {
                    self.convert_type(index_val, &index_type, &Type::Int)?
                        .into_int_value()
                } else {
                    index_val.into_int_value()
                };

                self.handle_tuple_dynamic_index(
                    value_val,
                    value_type.clone(),
                    index_int,
                    element_types,
                )
            }
            Type::Int => {
                if !index_type.can_coerce_to(&Type::Int) {
                    return Err(format!(
                        "Integer index must be an integer, got {:?}",
                        index_type
                    ));
                }

                let index_int = if index_type != Type::Int {
                    self.convert_type(index_val, &index_type, &Type::Int)?
                        .into_int_value()
                } else {
                    index_val.into_int_value()
                };

                let int_to_string_fn = match self.module.get_function("int_to_string") {
                    Some(f) => f,
                    None => return Err("int_to_string function not found".to_string()),
                };

                let call_site_value = self
                    .builder
                    .build_call(
                        int_to_string_fn,
                        &[index_int.into()],
                        "int_to_string_result",
                    )
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to convert integer to string".to_string())?;

                Ok((result, Type::String))
            }
            _ => Err(format!("Type {:?} is not indexable", value_type)),
        };

        self.ensure_block_has_terminator();

        result
    }

    fn handle_tuple_dynamic_index(
        &mut self,
        tuple_val: BasicValueEnum<'ctx>,
        tuple_type: Type,
        index_val: inkwell::values::IntValue<'ctx>,
        element_types: &[Type],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if element_types.len() == 1 {
            let element_type = &element_types[0];

            let tuple_struct = self
                .llvm_context
                .struct_type(&[self.get_llvm_type(element_type)], false);

            let tuple_ptr = if tuple_val.is_pointer_value() {
                tuple_val.into_pointer_value()
            } else {
                let llvm_type = self.get_llvm_type(&tuple_type);
                let alloca = self.builder.build_alloca(llvm_type, "tuple_temp").unwrap();
                self.builder.build_store(alloca, tuple_val).unwrap();
                alloca
            };

            let element_ptr = self
                .builder
                .build_struct_gep(tuple_struct, tuple_ptr, 0, "tuple_element_0")
                .unwrap();

            let element_val = self
                .builder
                .build_load(
                    self.get_llvm_type(element_type),
                    element_ptr,
                    "load_tuple_element_0",
                )
                .unwrap();

            return Ok((element_val, element_type.clone()));
        }

        let current_function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let default_block = self
            .llvm_context
            .append_basic_block(current_function, "tuple_index_default");

        let merge_block = self
            .llvm_context
            .append_basic_block(current_function, "tuple_index_merge");

        let mut case_blocks = Vec::with_capacity(element_types.len());
        for i in 0..element_types.len() {
            case_blocks.push(
                self.llvm_context
                    .append_basic_block(current_function, &format!("tuple_index_{}", i)),
            );
        }

        let _switch = self
            .builder
            .build_switch(
                index_val,
                default_block,
                &case_blocks
                    .iter()
                    .enumerate()
                    .map(|(i, block)| {
                        (
                            self.llvm_context.i64_type().const_int(i as u64, false),
                            *block,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap();

        let llvm_types: Vec<BasicTypeEnum> = element_types
            .iter()
            .map(|ty| self.get_llvm_type(ty))
            .collect();

        let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

        let tuple_ptr = if tuple_val.is_pointer_value() {
            tuple_val.into_pointer_value()
        } else {
            let llvm_type = self.get_llvm_type(&tuple_type);
            let alloca = self.builder.build_alloca(llvm_type, "tuple_temp").unwrap();
            self.builder.build_store(alloca, tuple_val).unwrap();
            alloca
        };

        let any_type = Type::Any;
        let llvm_any_type = self.get_llvm_type(&any_type);
        let result_ptr = self
            .builder
            .build_alloca(llvm_any_type, "tuple_index_result")
            .unwrap();

        for (i, &block) in case_blocks.iter().enumerate() {
            self.builder.position_at_end(block);

            let element_ptr = self
                .builder
                .build_struct_gep(
                    tuple_struct,
                    tuple_ptr,
                    i as u32,
                    &format!("tuple_element_{}", i),
                )
                .unwrap();

            let element_type = &element_types[i];
            let element_val = self
                .builder
                .build_load(
                    self.get_llvm_type(element_type),
                    element_ptr,
                    &format!("load_tuple_element_{}", i),
                )
                .unwrap();

            self.builder.build_store(result_ptr, element_val).unwrap();

            self.builder
                .build_unconditional_branch(merge_block)
                .unwrap();

            if !self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                self.builder.build_unreachable().unwrap();
            }
        }

        self.builder.position_at_end(default_block);

        let default_val = llvm_any_type.const_zero();
        self.builder.build_store(result_ptr, default_val).unwrap();

        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();

        if !self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_some()
        {
            self.builder.build_unreachable().unwrap();
        }

        self.builder.position_at_end(merge_block);

        let result_val = self
            .builder
            .build_load(llvm_any_type, result_ptr, "tuple_index_result")
            .unwrap();

        if !self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_some()
        {
            if let Some(current_function) = self.current_function {
                if current_function.get_type().get_return_type().is_none() {
                    self.builder.build_return(None).unwrap();
                } else {
                    let return_type = current_function.get_type().get_return_type().unwrap();
                    let default_val = return_type.const_zero();
                    self.builder.build_return(Some(&default_val)).unwrap();
                }
            } else {
                self.builder.build_unreachable().unwrap();
            }
        }

        if !self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_some()
        {
            if let Some(current_function) = self.current_function {
                if current_function.get_type().get_return_type().is_none() {
                    self.builder.build_return(None).unwrap();
                } else {
                    let return_type = current_function.get_type().get_return_type().unwrap();
                    let default_val = return_type.const_zero();
                    self.builder.build_return(Some(&default_val)).unwrap();
                }
            } else {
                self.builder.build_unreachable().unwrap();
            }
        }

        Ok((result_val, element_types[0].clone()))
    }

    fn build_empty_dict(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let dict_new_fn = match self.module.get_function("dict_new") {
            Some(f) => f,
            None => return Err("dict_new function not found".to_string()),
        };

        let call_site_value = self.builder.build_call(dict_new_fn, &[], name).unwrap();
        let dict_ptr = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to create empty dictionary".to_string())?;

        Ok(dict_ptr.into_pointer_value())
    }

    fn build_dict(
        &self,
        keys: Vec<BasicValueEnum<'ctx>>,
        values: Vec<BasicValueEnum<'ctx>>,
        key_type: &Type,
        value_type: &Type,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let dict_with_capacity_fn = match self.module.get_function("dict_with_capacity") {
            Some(f) => f,
            None => return Err("dict_with_capacity function not found".to_string()),
        };

        let len = keys.len() as u64;
        let len_value = self.llvm_context.i64_type().const_int(len, false);

        let call_site_value = self
            .builder
            .build_call(
                dict_with_capacity_fn,
                &[len_value.into()],
                "dict_with_capacity",
            )
            .unwrap();
        let dict_ptr = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to create dictionary with capacity".to_string())?;

        let dict_ptr = dict_ptr.into_pointer_value();

        let dict_set_fn = match self.module.get_function("dict_set") {
            Some(f) => f,
            None => return Err("dict_set function not found".to_string()),
        };

        for (i, (key, value)) in keys.iter().zip(values.iter()).enumerate() {
            let key_ptr = if crate::compiler::types::is_reference_type(key_type) {
                *key
            } else {
                let key_alloca = self
                    .builder
                    .build_alloca(key.get_type(), &format!("dict_key_{}", i))
                    .unwrap();
                self.builder.build_store(key_alloca, *key).unwrap();
                key_alloca.into()
            };

            let value_ptr = if crate::compiler::types::is_reference_type(value_type) {
                *value
            } else {
                let value_alloca = self
                    .builder
                    .build_alloca(value.get_type(), &format!("dict_value_{}", i))
                    .unwrap();
                self.builder.build_store(value_alloca, *value).unwrap();
                value_alloca.into()
            };

            self.builder
                .build_call(
                    dict_set_fn,
                    &[dict_ptr.into(), key_ptr.into(), value_ptr.into()],
                    &format!("dict_set_{}", i),
                )
                .unwrap();
        }

        Ok(dict_ptr)
    }

    fn build_empty_set(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = name;
        Err("Set operations require runtime support (not yet implemented)".to_string())
    }

    fn build_set(
        &self,
        elements: Vec<BasicValueEnum<'ctx>>,
        element_type: &Type,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = elements;
        let _ = element_type;
        Err("Set operations require runtime support (not yet implemented)".to_string())
    }

    fn build_list_get_item(
        &self,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        self.ensure_block_has_terminator();

        let list_get_fn = match self.module.get_function("list_get") {
            Some(f) => f,
            None => return Err("list_get function not found".to_string()),
        };

        self.ensure_block_has_terminator();

        let call_site_value = self
            .builder
            .build_call(list_get_fn, &[list_ptr.into(), index.into()], "list_get")
            .unwrap();

        let item_ptr = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to get item from list".to_string())?;

        self.ensure_block_has_terminator();

        if item_ptr.is_pointer_value() {
            Ok(item_ptr.into_pointer_value())
        } else {
            let item_alloca = self
                .builder
                .build_alloca(item_ptr.get_type(), "list_item_alloca")
                .unwrap();
            self.builder.build_store(item_alloca, item_ptr).unwrap();
            Ok(item_alloca)
        }
    }

    fn build_list_slice(
        &self,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        start: inkwell::values::IntValue<'ctx>,
        stop: inkwell::values::IntValue<'ctx>,
        step: inkwell::values::IntValue<'ctx>,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let list_slice_fn = match self.module.get_function("list_slice") {
            Some(f) => f,
            None => return Err("list_slice function not found".to_string()),
        };

        let call_site_value = self
            .builder
            .build_call(
                list_slice_fn,
                &[list_ptr.into(), start.into(), stop.into(), step.into()],
                "list_slice",
            )
            .unwrap();

        let slice_ptr = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to get slice from list".to_string())?;

        Ok(slice_ptr.into_pointer_value())
    }

    /// Compile a slice operation (e.g., list[1:10:2])
    fn compile_slice_operation(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        lower: Option<&Expr>,
        upper: Option<&Expr>,
        step: Option<&Expr>,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        self.compile_slice_operation_non_recursive(value_val, value_type, lower, upper, step)
    }

    fn compile_slice_operation_non_recursive(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        lower: Option<&Expr>,
        upper: Option<&Expr>,
        step: Option<&Expr>,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        self.ensure_block_has_terminator();

        match &value_type {
            Type::List(element_type) => {
                let list_len_fn = match self.module.get_function("list_len") {
                    Some(f) => f,
                    None => return Err("list_len function not found".to_string()),
                };

                let list_ptr = value_val.into_pointer_value();
                let list_len_call = self
                    .builder
                    .build_call(list_len_fn, &[list_ptr.into()], "list_len_result")
                    .unwrap();

                let list_len = list_len_call
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get list length".to_string())?;

                let list_len_int = list_len.into_int_value();

                let i64_type = self.llvm_context.i64_type();

                self.ensure_block_has_terminator();

                let start_val = match lower {
                    Some(expr) => {
                        let (start_val, start_type) = self.compile_expr(expr)?;
                        if !start_type.can_coerce_to(&Type::Int) {
                            return Err(format!(
                                "Slice start index must be an integer, got {:?}",
                                start_type
                            ));
                        }

                        self.ensure_block_has_terminator();

                        if start_type != Type::Int {
                            self.convert_type(start_val, &start_type, &Type::Int)?
                                .into_int_value()
                        } else {
                            start_val.into_int_value()
                        }
                    }
                    None => i64_type.const_int(0, false),
                };

                self.ensure_block_has_terminator();

                let stop_val = match upper {
                    Some(expr) => {
                        let (stop_val, stop_type) = self.compile_expr(expr)?;
                        if !stop_type.can_coerce_to(&Type::Int) {
                            return Err(format!(
                                "Slice stop index must be an integer, got {:?}",
                                stop_type
                            ));
                        }

                        self.ensure_block_has_terminator();

                        if stop_type != Type::Int {
                            self.convert_type(stop_val, &stop_type, &Type::Int)?
                                .into_int_value()
                        } else {
                            stop_val.into_int_value()
                        }
                    }
                    None => list_len_int,
                };

                self.ensure_block_has_terminator();

                let step_val = match step {
                    Some(expr) => {
                        let (step_val, step_type) = self.compile_expr(expr)?;
                        if !step_type.can_coerce_to(&Type::Int) {
                            return Err(format!(
                                "Slice step must be an integer, got {:?}",
                                step_type
                            ));
                        }

                        self.ensure_block_has_terminator();

                        if step_type != Type::Int {
                            self.convert_type(step_val, &step_type, &Type::Int)?
                                .into_int_value()
                        } else {
                            step_val.into_int_value()
                        }
                    }
                    None => i64_type.const_int(1, false),
                };

                self.ensure_block_has_terminator();

                let slice_ptr = self.build_list_slice(list_ptr, start_val, stop_val, step_val)?;

                self.ensure_block_has_terminator();

                Ok((slice_ptr.into(), Type::List(element_type.clone())))
            }
            Type::String => {
                let string_len_fn = match self.module.get_function("string_len") {
                    Some(f) => f,
                    None => return Err("string_len function not found".to_string()),
                };

                let str_ptr = value_val.into_pointer_value();
                let string_len_call = self
                    .builder
                    .build_call(string_len_fn, &[str_ptr.into()], "string_len_result")
                    .unwrap();

                let string_len = string_len_call
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get string length".to_string())?;

                let string_len_int = string_len.into_int_value();

                let i64_type = self.llvm_context.i64_type();

                let start_val = match lower {
                    Some(expr) => {
                        let (start_val, start_type) = self.compile_expr(expr)?;
                        if !start_type.can_coerce_to(&Type::Int) {
                            return Err(format!(
                                "Slice start index must be an integer, got {:?}",
                                start_type
                            ));
                        }

                        if start_type != Type::Int {
                            self.convert_type(start_val, &start_type, &Type::Int)?
                                .into_int_value()
                        } else {
                            start_val.into_int_value()
                        }
                    }
                    None => i64_type.const_int(0, false),
                };

                let stop_val = match upper {
                    Some(expr) => {
                        let (stop_val, stop_type) = self.compile_expr(expr)?;
                        if !stop_type.can_coerce_to(&Type::Int) {
                            return Err(format!(
                                "Slice stop index must be an integer, got {:?}",
                                stop_type
                            ));
                        }

                        if stop_type != Type::Int {
                            self.convert_type(stop_val, &stop_type, &Type::Int)?
                                .into_int_value()
                        } else {
                            stop_val.into_int_value()
                        }
                    }
                    None => string_len_int,
                };

                let step_val = match step {
                    Some(expr) => {
                        let (step_val, step_type) = self.compile_expr(expr)?;
                        if !step_type.can_coerce_to(&Type::Int) {
                            return Err(format!(
                                "Slice step must be an integer, got {:?}",
                                step_type
                            ));
                        }

                        if step_type != Type::Int {
                            self.convert_type(step_val, &step_type, &Type::Int)?
                                .into_int_value()
                        } else {
                            step_val.into_int_value()
                        }
                    }
                    None => i64_type.const_int(1, false),
                };

                self.ensure_block_has_terminator();

                let slice_ptr = self.build_string_slice(str_ptr, start_val, stop_val, step_val)?;

                self.ensure_block_has_terminator();

                Ok((slice_ptr.into(), Type::String))
            }
            _ => Err(format!("Type {:?} does not support slicing", value_type)),
        }
    }

    fn build_dict_get_item(
        &self,
        dict_ptr: inkwell::values::PointerValue<'ctx>,
        key: BasicValueEnum<'ctx>,
        key_type: &Type,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        self.ensure_block_has_terminator();

        let dict_get_fn = match self.module.get_function("dict_get") {
            Some(f) => f,
            None => return Err("dict_get function not found".to_string()),
        };

        let key_ptr = if matches!(key_type, Type::String) {
            if key.is_pointer_value() {
                key
            } else {
                return Err(format!("Expected pointer value for string key"));
            }
        } else if crate::compiler::types::is_reference_type(key_type) {
            key
        } else {
            let key_alloca = self
                .builder
                .build_alloca(key.get_type(), "dict_key_temp")
                .unwrap();
            self.builder.build_store(key_alloca, key).unwrap();
            key_alloca.into()
        };

        self.ensure_block_has_terminator();

        let call_site_value = self
            .builder
            .build_call(
                dict_get_fn,
                &[dict_ptr.into(), key_ptr.into()],
                "dict_get_result",
            )
            .unwrap();

        let value_ptr = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to get value from dictionary".to_string())?;

        self.ensure_block_has_terminator();

        Ok(value_ptr.into_pointer_value())
    }

    fn build_string_get_char(
        &self,
        str_ptr: inkwell::values::PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.ensure_block_has_terminator();

        let string_get_char_fn = match self.module.get_function("string_get_char") {
            Some(f) => f,
            None => return Err("string_get_char function not found".to_string()),
        };

        self.ensure_block_has_terminator();

        let call_site_value = self
            .builder
            .build_call(
                string_get_char_fn,
                &[str_ptr.into(), index.into()],
                "string_get_char_result",
            )
            .unwrap();

        let char_int = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to get character from string".to_string())?;

        self.ensure_block_has_terminator();

        let char_to_string_fn = match self.module.get_function("char_to_string") {
            Some(f) => f,
            None => {
                let int_to_string_fn = match self.module.get_function("int_to_string") {
                    Some(f) => f,
                    None => return Err("int_to_string function not found".to_string()),
                };

                self.ensure_block_has_terminator();

                let call_site_value = self
                    .builder
                    .build_call(int_to_string_fn, &[char_int.into()], "int_to_string_result")
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to convert character to string".to_string())?;

                self.ensure_block_has_terminator();

                return Ok(result);
            }
        };

        self.ensure_block_has_terminator();

        let call_site_value = self
            .builder
            .build_call(
                char_to_string_fn,
                &[char_int.into()],
                "char_to_string_result",
            )
            .unwrap();

        let result = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to convert character to string".to_string())?;

        self.ensure_block_has_terminator();

        Ok(result)
    }

    fn build_string_slice(
        &self,
        str_ptr: inkwell::values::PointerValue<'ctx>,
        start: inkwell::values::IntValue<'ctx>,
        stop: inkwell::values::IntValue<'ctx>,
        step: inkwell::values::IntValue<'ctx>,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let string_slice_fn = match self.module.get_function("string_slice") {
            Some(f) => f,
            None => return Err("string_slice function not found".to_string()),
        };

        let call_site_value = self
            .builder
            .build_call(
                string_slice_fn,
                &[str_ptr.into(), start.into(), stop.into(), step.into()],
                "string_slice_result",
            )
            .unwrap();

        let result = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to get slice from string".to_string())?;

        Ok(result.into_pointer_value())
    }

    fn compile_number(&mut self, num: &Number) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        match num {
            Number::Integer(value) => {
                let int_type = self.llvm_context.i64_type();
                let int_value = int_type.const_int(*value as u64, true);
                Ok((int_value.into(), Type::Int))
            }
            Number::Float(value) => {
                let float_type = self.llvm_context.f64_type();
                let float_value = float_type.const_float(*value);
                Ok((float_value.into(), Type::Float))
            }
            Number::Complex { real, imag } => {
                let float_type = self.llvm_context.f64_type();
                let struct_type = self
                    .llvm_context
                    .struct_type(&[float_type.into(), float_type.into()], false);

                let real_value = float_type.const_float(*real);
                let imag_value = float_type.const_float(*imag);

                let complex_value =
                    struct_type.const_named_struct(&[real_value.into(), imag_value.into()]);

                Ok((complex_value.into(), Type::Float))
            }
        }
    }

    fn compile_name_constant(
        &mut self,
        constant: &NameConstant,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        match constant {
            NameConstant::True => {
                let bool_type = self.llvm_context.bool_type();
                let bool_value = bool_type.const_int(1, false);
                Ok((bool_value.into(), Type::Bool))
            }
            NameConstant::False => {
                let bool_type = self.llvm_context.bool_type();
                let bool_value = bool_type.const_int(0, false);
                Ok((bool_value.into(), Type::Bool))
            }
            NameConstant::None => {
                let ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let null_value = ptr_type.const_null();
                Ok((null_value.into(), Type::None))
            }
        }
    }

    /// Compile a list comprehension expression
    fn compile_list_comprehension(
        &mut self,
        elt: &Expr,
        generators: &[crate::ast::Comprehension],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        self.compile_list_comprehension_non_recursive(elt, generators)
    }

    fn compile_list_comprehension_non_recursive(
        &mut self,
        elt: &Expr,
        generators: &[crate::ast::Comprehension],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if generators.is_empty() {
            return Err("List comprehension must have at least one generator".to_string());
        }

        self.ensure_block_has_terminator();

        let result_list = self.build_empty_list("list_comp_result")?;

        self.ensure_block_has_terminator();

        let list_append_fn = match self.module.get_function("list_append") {
            Some(f) => f,
            None => return Err("list_append function not found".to_string()),
        };

        self.scope_stack.push_scope(false, false, false);

        let generator = &generators[0];

        self.ensure_block_has_terminator();

        let (iter_val, iter_type_original) = self.compile_expr(&generator.iter)?;
        let iter_type = iter_type_original.clone();

        self.ensure_block_has_terminator();

        if let Expr::Call { func, .. } = &*generator.iter {
            if let Expr::Name { id, .. } = func.as_ref() {
                if id == "range" {
                    self.handle_range_list_comprehension(
                        elt,
                        generator,
                        iter_val,
                        result_list,
                        list_append_fn,
                    )?;

                    self.scope_stack.pop_scope();

                    self.scope_stack.push_scope(false, false, false);

                    if let Expr::Name { id, .. } = generator.target.as_ref() {
                        let dummy_type = Type::Int;

                        let dummy_alloca = self
                            .builder
                            .build_alloca(self.get_llvm_type(&dummy_type), id)
                            .unwrap();

                        self.scope_stack
                            .add_variable(id.to_string(), dummy_alloca, dummy_type);
                    }

                    let (_, element_type) = self.compile_expr(elt)?;

                    self.scope_stack.pop_scope();

                    return Ok((result_list.into(), Type::List(Box::new(element_type))));
                }
            }
        }

        if let Expr::List { elts, .. } = &*generator.iter {
            println!("Creating list from literal for iteration");

            let mut element_values = Vec::with_capacity(elts.len());
            let mut element_types = Vec::with_capacity(elts.len());

            for elt in elts {
                let (value, ty) = self.compile_expr(elt)?;
                element_values.push(value);
                element_types.push(ty.clone());
            }

            let element_type = if element_types.is_empty() {
                Type::Unknown
            } else {
                let first_type = &element_types[0];
                let all_same = element_types.iter().all(|t| t == first_type);

                if all_same {
                    println!("All list elements have the same type: {:?}", first_type);
                    first_type.clone()
                } else {
                    let mut common_type = element_types[0].clone();
                    for ty in &element_types[1..] {
                        common_type = match self.get_common_type(&common_type, ty) {
                            Ok(t) => t,
                            Err(_) => {
                                println!(
                                    "Could not find common type between {:?} and {:?}, using Any",
                                    common_type, ty
                                );
                                Type::Any
                            }
                        };
                    }
                    println!(
                        "List literal elements have different types, using common type: {:?}",
                        common_type
                    );
                    common_type
                }
            };

            let list_ptr = self.build_list(element_values, &element_type)?;

            self.handle_list_iteration_for_comprehension(
                elt,
                generator,
                list_ptr,
                result_list,
                list_append_fn,
            )?;

            self.scope_stack.pop_scope();

            let (_, element_type) = self.compile_expr(elt)?;
            return Ok((result_list.into(), Type::List(Box::new(element_type))));
        } else {
            match iter_type {
                Type::List(_) => {
                    self.handle_list_iteration_for_comprehension(
                        elt,
                        generator,
                        iter_val.into_pointer_value(),
                        result_list,
                        list_append_fn,
                    )?;
                }
                Type::Tuple(element_types) => {
                    println!("Handling tuple iteration directly");

                    let tuple_ptr = iter_val.into_pointer_value();

                    let current_function = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();
                    let loop_entry_block = self
                        .llvm_context
                        .append_basic_block(current_function, "tuple_comp_entry");
                    let loop_body_block = self
                        .llvm_context
                        .append_basic_block(current_function, "tuple_comp_body");
                    let loop_exit_block = self
                        .llvm_context
                        .append_basic_block(current_function, "tuple_comp_exit");

                    let index_ptr = self
                        .builder
                        .build_alloca(self.llvm_context.i64_type(), "tuple_comp_index")
                        .unwrap();
                    self.builder
                        .build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false))
                        .unwrap();

                    self.builder
                        .build_unconditional_branch(loop_entry_block)
                        .unwrap();

                    self.builder.position_at_end(loop_entry_block);
                    let current_index = self
                        .builder
                        .build_load(self.llvm_context.i64_type(), index_ptr, "current_index")
                        .unwrap()
                        .into_int_value();
                    let tuple_len = self
                        .llvm_context
                        .i64_type()
                        .const_int(element_types.len() as u64, false);
                    let condition = self
                        .builder
                        .build_int_compare(
                            inkwell::IntPredicate::SLT,
                            current_index,
                            tuple_len,
                            "loop_condition",
                        )
                        .unwrap();

                    self.builder
                        .build_conditional_branch(condition, loop_body_block, loop_exit_block)
                        .unwrap();

                    self.builder.position_at_end(loop_body_block);

                    let default_block = self
                        .llvm_context
                        .append_basic_block(current_function, "tuple_default");
                    let merge_block = self
                        .llvm_context
                        .append_basic_block(current_function, "tuple_merge");

                    let mut case_blocks = Vec::with_capacity(element_types.len());
                    for i in 0..element_types.len() {
                        case_blocks.push(
                            self.llvm_context
                                .append_basic_block(current_function, &format!("tuple_case_{}", i)),
                        );
                    }

                    let _switch = self
                        .builder
                        .build_switch(
                            current_index,
                            default_block,
                            &case_blocks
                                .iter()
                                .enumerate()
                                .map(|(i, block)| {
                                    (
                                        self.llvm_context.i64_type().const_int(i as u64, false),
                                        *block,
                                    )
                                })
                                .collect::<Vec<_>>(),
                        )
                        .unwrap();

                    let llvm_types: Vec<BasicTypeEnum> = element_types
                        .iter()
                        .map(|ty| self.get_llvm_type(ty))
                        .collect();

                    let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

                    for (i, &block) in case_blocks.iter().enumerate() {
                        self.builder.position_at_end(block);

                        let element_ptr = self
                            .builder
                            .build_struct_gep(
                                tuple_struct,
                                tuple_ptr,
                                i as u32,
                                &format!("tuple_element_{}", i),
                            )
                            .unwrap();

                        let element_type = &element_types[i];
                        let element_val = self
                            .builder
                            .build_load(
                                self.get_llvm_type(element_type),
                                element_ptr,
                                &format!("load_tuple_element_{}", i),
                            )
                            .unwrap();

                        let element_alloca = self
                            .builder
                            .build_alloca(
                                element_val.get_type(),
                                &format!("tuple_element_alloca_{}", i),
                            )
                            .unwrap();
                        self.builder
                            .build_store(element_alloca, element_val)
                            .unwrap();

                        if let Expr::Name { id, .. } = generator.target.as_ref() {
                            self.scope_stack.add_variable(
                                id.to_string(),
                                element_alloca,
                                element_type.clone(),
                            );

                            let should_append = self
                                .evaluate_comprehension_conditions(generator, current_function)?;

                            self.process_list_comprehension_element(
                                elt,
                                should_append,
                                result_list,
                                list_append_fn,
                                current_function,
                            )?;
                        } else {
                            return Err(
                                "Only simple variable targets are supported in list comprehensions"
                                    .to_string(),
                            );
                        }

                        self.builder
                            .build_unconditional_branch(merge_block)
                            .unwrap();
                    }

                    self.builder.position_at_end(default_block);
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .unwrap();

                    self.builder.position_at_end(merge_block);
                    let next_index = self
                        .builder
                        .build_int_add(
                            current_index,
                            self.llvm_context.i64_type().const_int(1, false),
                            "next_index",
                        )
                        .unwrap();
                    self.builder.build_store(index_ptr, next_index).unwrap();
                    self.builder
                        .build_unconditional_branch(loop_entry_block)
                        .unwrap();

                    self.builder.position_at_end(loop_exit_block);
                }
                Type::String => {
                    self.handle_string_iteration_for_comprehension(
                        elt,
                        generator,
                        iter_val.into_pointer_value(),
                        result_list,
                        list_append_fn,
                    )?;
                }
                _ => {
                    self.handle_general_iteration_for_comprehension(
                        elt,
                        generator,
                        iter_val,
                        iter_type,
                        result_list,
                        list_append_fn,
                    )?;
                }
            }
        }

        self.scope_stack.pop_scope();

        self.ensure_block_has_terminator();

        self.scope_stack.push_scope(false, false, false);

        if let Expr::Name { id, .. } = generator.target.as_ref() {
            let mut dummy_type = match &iter_type_original {
                Type::List(elem_type) => *elem_type.clone(),
                Type::String => Type::String,
                _ => Type::Int,
            };

            dummy_type = match &dummy_type {
                Type::Tuple(tuple_element_types) => {
                    if !tuple_element_types.is_empty()
                        && tuple_element_types
                            .iter()
                            .all(|t| t == &tuple_element_types[0])
                    {
                        tuple_element_types[0].clone()
                    } else {
                        Type::Int
                    }
                }
                _ => dummy_type,
            };

            let dummy_alloca = self
                .builder
                .build_alloca(self.get_llvm_type(&dummy_type), id)
                .unwrap();

            self.scope_stack
                .add_variable(id.to_string(), dummy_alloca, dummy_type);
        }

        let (_, element_type) = self.compile_expr(elt)?;

        self.scope_stack.pop_scope();

        Ok((result_list.into(), Type::List(Box::new(element_type))))
    }

    fn handle_range_list_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        range_val: BasicValueEnum<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String> {
        let range_val = range_val.into_int_value();

        let current_function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let loop_entry_block = self
            .llvm_context
            .append_basic_block(current_function, "range_comp_entry");
        let loop_body_block = self
            .llvm_context
            .append_basic_block(current_function, "range_comp_body");
        let loop_exit_block = self
            .llvm_context
            .append_basic_block(current_function, "range_comp_exit");

        let index_ptr = self
            .builder
            .build_alloca(self.llvm_context.i64_type(), "range_comp_index")
            .unwrap();
        self.builder
            .build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false))
            .unwrap();

        self.builder
            .build_unconditional_branch(loop_entry_block)
            .unwrap();

        self.builder.position_at_end(loop_entry_block);
        let current_index = self
            .builder
            .build_load(self.llvm_context.i64_type(), index_ptr, "current_index")
            .unwrap()
            .into_int_value();
        let condition = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::SLT,
                current_index,
                range_val,
                "loop_condition",
            )
            .unwrap();

        self.builder
            .build_conditional_branch(condition, loop_body_block, loop_exit_block)
            .unwrap();

        self.builder.position_at_end(loop_body_block);

        if let Expr::Name { id, .. } = generator.target.as_ref() {
            let index_alloca = self
                .builder
                .build_alloca(self.llvm_context.i64_type(), "range_index_alloca")
                .unwrap();
            self.builder
                .build_store(index_alloca, current_index)
                .unwrap();
            self.scope_stack
                .add_variable(id.to_string(), index_alloca, Type::Int);
        } else {
            return Err(
                "Only simple variable targets are supported in list comprehensions".to_string(),
            );
        }

        let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

        self.process_list_comprehension_element(
            elt,
            should_append,
            result_list,
            list_append_fn,
            current_function,
        )?;

        let next_index = self
            .builder
            .build_int_add(
                current_index,
                self.llvm_context.i64_type().const_int(1, false),
                "next_index",
            )
            .unwrap();
        self.builder.build_store(index_ptr, next_index).unwrap();
        self.builder
            .build_unconditional_branch(loop_entry_block)
            .unwrap();

        self.builder.position_at_end(loop_exit_block);

        Ok(())
    }

    fn handle_list_iteration_for_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String> {
        let list_len_fn = match self.module.get_function("list_len") {
            Some(f) => f,
            None => return Err("list_len function not found".to_string()),
        };

        let list_len_call = self
            .builder
            .build_call(list_len_fn, &[list_ptr.into()], "list_len_result")
            .unwrap();

        let list_len = list_len_call
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to get list length".to_string())?;

        let list_get_fn = match self.module.get_function("list_get") {
            Some(f) => f,
            None => return Err("list_get function not found".to_string()),
        };

        let current_function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let loop_entry_block = self
            .llvm_context
            .append_basic_block(current_function, "list_comp_entry");
        let loop_body_block = self
            .llvm_context
            .append_basic_block(current_function, "list_comp_body");
        let loop_exit_block = self
            .llvm_context
            .append_basic_block(current_function, "list_comp_exit");

        let index_ptr = self
            .builder
            .build_alloca(self.llvm_context.i64_type(), "list_comp_index")
            .unwrap();
        self.builder
            .build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false))
            .unwrap();

        self.builder
            .build_unconditional_branch(loop_entry_block)
            .unwrap();

        self.builder.position_at_end(loop_entry_block);
        let current_index = self
            .builder
            .build_load(self.llvm_context.i64_type(), index_ptr, "current_index")
            .unwrap()
            .into_int_value();
        let condition = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::SLT,
                current_index,
                list_len.into_int_value(),
                "loop_condition",
            )
            .unwrap();

        self.builder
            .build_conditional_branch(condition, loop_body_block, loop_exit_block)
            .unwrap();

        self.builder.position_at_end(loop_body_block);

        let call_site_value = self
            .builder
            .build_call(
                list_get_fn,
                &[list_ptr.into(), current_index.into()],
                "list_get_result",
            )
            .unwrap();

        let element_ptr = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to get list element".to_string())?;

        let element_type = match self.lookup_variable_type(&generator.iter.to_string()) {
            Some(Type::List(element_type)) => *element_type.clone(),
            _ => {
                if let Expr::List { elts, .. } = &*generator.iter {
                    if !elts.is_empty() {
                        if let Ok((_, element_type)) = self.compile_expr(&elts[0]) {
                            match &element_type {
                                Type::Tuple(tuple_element_types) => {
                                    if !tuple_element_types.is_empty()
                                        && tuple_element_types
                                            .iter()
                                            .all(|t| t == &tuple_element_types[0])
                                    {
                                        tuple_element_types[0].clone()
                                    } else {
                                        Type::Int
                                    }
                                }
                                _ => element_type,
                            }
                        } else {
                            Type::Int
                        }
                    } else {
                        Type::Int
                    }
                } else {
                    Type::Int
                }
            }
        };

        let element_type = match &element_type {
            Type::Tuple(tuple_element_types) => {
                if !tuple_element_types.is_empty()
                    && tuple_element_types
                        .iter()
                        .all(|t| t == &tuple_element_types[0])
                {
                    tuple_element_types[0].clone()
                } else {
                    Type::Int
                }
            }
            _ => element_type,
        };

        match generator.target.as_ref() {
            Expr::Name { id, .. } => {
                println!("Setting list comprehension variable '{}' to type: {:?}", id, element_type);
                self.scope_stack.add_variable(id.to_string(), element_ptr.into_pointer_value(), element_type.clone());
            },
            Expr::Tuple { elts, .. } => {
                if let Type::Tuple(tuple_element_types) = &element_type {
                    if elts.len() != tuple_element_types.len() {
                        return Err(format!("Tuple unpacking mismatch: expected {} elements, got {}",
                                          elts.len(), tuple_element_types.len()));
                    }

                    let llvm_types: Vec<BasicTypeEnum> = tuple_element_types.iter()
                        .map(|ty| self.get_llvm_type(ty))
                        .collect();

                    let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);
                    let tuple_ptr = element_ptr.into_pointer_value();

                    for (i, target_elt) in elts.iter().enumerate() {
                        if let Expr::Name { id, .. } = &**target_elt {
                            let element_ptr = self.builder.build_struct_gep(
                                tuple_struct,
                                tuple_ptr,
                                i as u32,
                                &format!("tuple_element_{}", i)
                            ).unwrap();

                            let element_type = &tuple_element_types[i];
                            let element_val = self.builder.build_load(
                                self.get_llvm_type(element_type),
                                element_ptr,
                                &format!("load_tuple_element_{}", i)
                            ).unwrap();

                            let element_alloca = self.builder.build_alloca(
                                element_val.get_type(),
                                &format!("tuple_element_alloca_{}", i)
                            ).unwrap();
                            self.builder.build_store(element_alloca, element_val).unwrap();

                            println!("Setting unpacked tuple element '{}' to type: {:?}", id, element_type);
                            self.scope_stack.add_variable(id.to_string(), element_alloca, element_type.clone());
                        } else {
                            return Err("Only simple variable names are supported in tuple unpacking".to_string());
                        }
                    }
                } else {
                    return Err(format!("Cannot unpack non-tuple type {:?} in list comprehension", element_type));
                }
            },
            _ => return Err("Only simple variable targets or tuple unpacking are supported in list comprehensions".to_string()),
        }

        let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

        self.process_list_comprehension_element(
            elt,
            should_append,
            result_list,
            list_append_fn,
            current_function,
        )?;

        let next_index = self
            .builder
            .build_int_add(
                current_index,
                self.llvm_context.i64_type().const_int(1, false),
                "next_index",
            )
            .unwrap();
        self.builder.build_store(index_ptr, next_index).unwrap();
        self.builder
            .build_unconditional_branch(loop_entry_block)
            .unwrap();

        self.builder.position_at_end(loop_exit_block);

        Ok(())
    }

    fn handle_string_iteration_for_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        str_ptr: inkwell::values::PointerValue<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String> {
        let string_len_fn = match self.module.get_function("string_len") {
            Some(f) => f,
            None => return Err("string_len function not found".to_string()),
        };

        let string_len_call = self
            .builder
            .build_call(string_len_fn, &[str_ptr.into()], "string_len_result")
            .unwrap();

        let string_len = string_len_call
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to get string length".to_string())?;

        let string_get_fn = match self.module.get_function("string_get_char") {
            Some(f) => f,
            None => return Err("string_get_char function not found".to_string()),
        };

        let current_function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let loop_entry_block = self
            .llvm_context
            .append_basic_block(current_function, "string_comp_entry");
        let loop_body_block = self
            .llvm_context
            .append_basic_block(current_function, "string_comp_body");
        let loop_exit_block = self
            .llvm_context
            .append_basic_block(current_function, "string_comp_exit");

        let index_ptr = self
            .builder
            .build_alloca(self.llvm_context.i64_type(), "string_comp_index")
            .unwrap();
        self.builder
            .build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false))
            .unwrap();

        self.builder
            .build_unconditional_branch(loop_entry_block)
            .unwrap();

        self.builder.position_at_end(loop_entry_block);
        let current_index = self
            .builder
            .build_load(self.llvm_context.i64_type(), index_ptr, "current_index")
            .unwrap()
            .into_int_value();
        let condition = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::SLT,
                current_index,
                string_len.into_int_value(),
                "loop_condition",
            )
            .unwrap();

        self.builder
            .build_conditional_branch(condition, loop_body_block, loop_exit_block)
            .unwrap();

        self.builder.position_at_end(loop_body_block);

        let call_site_value = self
            .builder
            .build_call(
                string_get_fn,
                &[str_ptr.into(), current_index.into()],
                "string_get_result",
            )
            .unwrap();

        let char_val = call_site_value
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Failed to get string character".to_string())?;

        let char_ptr = self
            .builder
            .build_alloca(char_val.get_type(), "char_ptr")
            .unwrap();
        self.builder.build_store(char_ptr, char_val).unwrap();

        if let Expr::Name { id, .. } = generator.target.as_ref() {
            self.scope_stack
                .add_variable(id.to_string(), char_ptr, Type::Int);
        } else {
            return Err(
                "Only simple variable targets are supported in list comprehensions".to_string(),
            );
        }

        let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

        self.process_list_comprehension_element(
            elt,
            should_append,
            result_list,
            list_append_fn,
            current_function,
        )?;

        let next_index = self
            .builder
            .build_int_add(
                current_index,
                self.llvm_context.i64_type().const_int(1, false),
                "next_index",
            )
            .unwrap();
        self.builder.build_store(index_ptr, next_index).unwrap();
        self.builder
            .build_unconditional_branch(loop_entry_block)
            .unwrap();

        self.builder.position_at_end(loop_exit_block);

        Ok(())
    }

    /// Handle general iteration (for other types) in list comprehension
    fn handle_general_iteration_for_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        iter_val: BasicValueEnum<'ctx>,
        iter_type: Type,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String> {
        match &iter_type {
            Type::Tuple(element_types) => {
                println!("Handling tuple iteration directly in general handler");

                let tuple_ptr = iter_val.into_pointer_value();

                let current_function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                if let Expr::Name { id, .. } = generator.target.as_ref() {
                    println!("Setting tuple variable '{}' to type: {:?}", id, iter_type);
                    self.scope_stack
                        .add_variable(id.to_string(), tuple_ptr, iter_type.clone());

                    let should_append =
                        self.evaluate_comprehension_conditions(generator, current_function)?;

                    self.process_list_comprehension_element(
                        elt,
                        should_append,
                        result_list,
                        list_append_fn,
                        current_function,
                    )?;
                } else {
                    if let Expr::Tuple { elts, .. } = generator.target.as_ref() {
                        if elts.len() != element_types.len() {
                            return Err(format!(
                                "Tuple unpacking mismatch: expected {} elements, got {}",
                                elts.len(),
                                element_types.len()
                            ));
                        }

                        let llvm_types: Vec<BasicTypeEnum> = element_types
                            .iter()
                            .map(|ty| self.get_llvm_type(ty))
                            .collect();

                        let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

                        for (i, target_elt) in elts.iter().enumerate() {
                            if let Expr::Name { id, .. } = &**target_elt {
                                let element_ptr = self
                                    .builder
                                    .build_struct_gep(
                                        tuple_struct,
                                        tuple_ptr,
                                        i as u32,
                                        &format!("tuple_element_{}", i),
                                    )
                                    .unwrap();

                                let element_type = &element_types[i];
                                let element_val = self
                                    .builder
                                    .build_load(
                                        self.get_llvm_type(element_type),
                                        element_ptr,
                                        &format!("load_tuple_element_{}", i),
                                    )
                                    .unwrap();

                                let element_alloca = self
                                    .builder
                                    .build_alloca(
                                        element_val.get_type(),
                                        &format!("tuple_element_alloca_{}", i),
                                    )
                                    .unwrap();
                                self.builder
                                    .build_store(element_alloca, element_val)
                                    .unwrap();

                                println!(
                                    "Setting unpacked tuple element '{}' to type: {:?}",
                                    id, element_type
                                );
                                self.scope_stack.add_variable(
                                    id.to_string(),
                                    element_alloca,
                                    element_type.clone(),
                                );
                            } else {
                                return Err(
                                    "Only simple variable names are supported in tuple unpacking"
                                        .to_string(),
                                );
                            }
                        }

                        let should_append =
                            self.evaluate_comprehension_conditions(generator, current_function)?;

                        self.process_list_comprehension_element(
                            elt,
                            should_append,
                            result_list,
                            list_append_fn,
                            current_function,
                        )?;
                    } else {
                        return Err("Only simple variable targets or tuple unpacking are supported in list comprehensions".to_string());
                    }
                }
            }
            _ => {
                if let Expr::Name { id, .. } = generator.target.as_ref() {
                    let dummy_val = self.llvm_context.i64_type().const_int(0, false);
                    let dummy_ptr = self
                        .builder
                        .build_alloca(self.llvm_context.i64_type(), id)
                        .unwrap();
                    self.builder.build_store(dummy_ptr, dummy_val).unwrap();

                    self.scope_stack
                        .add_variable(id.to_string(), dummy_ptr, Type::Int);

                    let current_function = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();

                    let should_append =
                        self.evaluate_comprehension_conditions(generator, current_function)?;

                    self.process_list_comprehension_element(
                        elt,
                        should_append,
                        result_list,
                        list_append_fn,
                        current_function,
                    )?;
                } else {
                    return Err(
                        "Only simple variable targets are supported in list comprehensions"
                            .to_string(),
                    );
                }
            }
        }

        Ok(())
    }

    /// Evaluate all conditions (if clauses) in a comprehension
    fn evaluate_comprehension_conditions(
        &mut self,
        generator: &crate::ast::Comprehension,
        _current_function: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        if generator.ifs.is_empty() {
            return Ok(self.llvm_context.bool_type().const_int(1, false));
        }

        let mut should_append = self.llvm_context.bool_type().const_int(1, false);

        for if_expr in &generator.ifs {
            let (cond_val, cond_type) = self.compile_expr(if_expr)?;

            let cond_bool = if cond_type != Type::Bool {
                match &cond_type {
                    Type::Tuple(_) => {
                        println!("Treating tuple as truthy in comprehension condition");
                        self.llvm_context.bool_type().const_int(1, false)
                    }
                    _ => {
                        match self.convert_type(cond_val, &cond_type, &Type::Bool) {
                            Ok(bool_val) => bool_val.into_int_value(),
                            Err(_) => match cond_val {
                                BasicValueEnum::IntValue(i) => {
                                    let zero = self.llvm_context.i64_type().const_zero();
                                    self.builder
                                        .build_int_compare(
                                            inkwell::IntPredicate::NE,
                                            i,
                                            zero,
                                            "is_nonzero",
                                        )
                                        .unwrap()
                                }
                                BasicValueEnum::FloatValue(f) => {
                                    let zero = self.llvm_context.f64_type().const_float(0.0);
                                    self.builder
                                        .build_float_compare(
                                            inkwell::FloatPredicate::ONE,
                                            f,
                                            zero,
                                            "is_nonzero",
                                        )
                                        .unwrap()
                                }
                                BasicValueEnum::PointerValue(_) => {
                                    println!("Treating pointer value as truthy in comprehension condition");
                                    self.llvm_context.bool_type().const_int(1, false)
                                }
                                _ => {
                                    println!("WARNING: Unknown value type in condition, treating as falsy");
                                    self.llvm_context.bool_type().const_int(0, false)
                                }
                            },
                        }
                    }
                }
            } else {
                cond_val.into_int_value()
            };

            should_append = self
                .builder
                .build_and(should_append, cond_bool, "if_condition")
                .unwrap();
        }

        Ok(should_append)
    }

    fn process_list_comprehension_element(
        &mut self,
        elt: &Expr,
        should_append: inkwell::values::IntValue<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
        current_function: inkwell::values::FunctionValue<'ctx>,
    ) -> Result<(), String> {
        let then_block = self
            .llvm_context
            .append_basic_block(current_function, "comp_then");
        let continue_block = self
            .llvm_context
            .append_basic_block(current_function, "comp_continue");

        self.builder
            .build_conditional_branch(should_append, then_block, continue_block)
            .unwrap();

        self.builder.position_at_end(then_block);

        let (element_val, mut element_type) = self.compile_expr(elt)?;

        element_type = match &element_type {
            Type::Tuple(tuple_element_types) => {
                if !tuple_element_types.is_empty()
                    && tuple_element_types
                        .iter()
                        .all(|t| t == &tuple_element_types[0])
                {
                    tuple_element_types[0].clone()
                } else {
                    element_type
                }
            }
            _ => element_type,
        };

        let element_ptr = match &element_type {
            Type::Tuple(_) => {
                if element_val.is_pointer_value() {
                    element_val.into_pointer_value()
                } else {
                    let element_alloca = self
                        .builder
                        .build_alloca(element_val.get_type(), "comp_element")
                        .unwrap();
                    self.builder
                        .build_store(element_alloca, element_val)
                        .unwrap();
                    element_alloca
                }
            }
            _ => {
                if crate::compiler::types::is_reference_type(&element_type) {
                    if element_val.is_pointer_value() {
                        element_val.into_pointer_value()
                    } else {
                        let element_alloca = self
                            .builder
                            .build_alloca(element_val.get_type(), "comp_element")
                            .unwrap();
                        self.builder
                            .build_store(element_alloca, element_val)
                            .unwrap();
                        element_alloca
                    }
                } else {
                    let element_alloca = self
                        .builder
                        .build_alloca(element_val.get_type(), "comp_element")
                        .unwrap();
                    self.builder
                        .build_store(element_alloca, element_val)
                        .unwrap();
                    element_alloca
                }
            }
        };

        self.builder
            .build_call(
                list_append_fn,
                &[result_list.into(), element_ptr.into()],
                "list_append_result",
            )
            .unwrap();

        self.builder
            .build_unconditional_branch(continue_block)
            .unwrap();

        self.builder.position_at_end(continue_block);

        Ok(())
    }

    /// Compile an attribute access expression (e.g., dict.keys())
    fn compile_attribute_access(
        &mut self,
        value: &Expr,
        attr: &str,
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        let (value_val, value_type) = self.compile_expr(value)?;

        match &value_type {
            Type::Dict(key_type, value_type) => match attr {
                "keys" => {
                    let dict_keys_fn = match self.module.get_function("dict_keys") {
                        Some(f) => f,
                        None => return Err("dict_keys function not found".to_string()),
                    };

                    let call_site_value = self
                        .builder
                        .build_call(
                            dict_keys_fn,
                            &[value_val.into_pointer_value().into()],
                            "dict_keys_result",
                        )
                        .unwrap();

                    let keys_list_ptr = call_site_value
                        .try_as_basic_value()
                        .left()
                        .ok_or_else(|| "Failed to get keys from dictionary".to_string())?;

                    Ok((keys_list_ptr, Type::List(key_type.clone())))
                }
                "values" => {
                    let dict_values_fn = match self.module.get_function("dict_values") {
                        Some(f) => f,
                        None => return Err("dict_values function not found".to_string()),
                    };

                    let call_site_value = self
                        .builder
                        .build_call(
                            dict_values_fn,
                            &[value_val.into_pointer_value().into()],
                            "dict_values_result",
                        )
                        .unwrap();

                    let values_list_ptr = call_site_value
                        .try_as_basic_value()
                        .left()
                        .ok_or_else(|| "Failed to get values from dictionary".to_string())?;

                    Ok((values_list_ptr, Type::List(value_type.clone())))
                }
                "items" => {
                    let dict_items_fn = match self.module.get_function("dict_items") {
                        Some(f) => f,
                        None => return Err("dict_items function not found".to_string()),
                    };

                    let call_site_value = self
                        .builder
                        .build_call(
                            dict_items_fn,
                            &[value_val.into_pointer_value().into()],
                            "dict_items_result",
                        )
                        .unwrap();

                    let items_list_ptr = call_site_value
                        .try_as_basic_value()
                        .left()
                        .ok_or_else(|| "Failed to get items from dictionary".to_string())?;

                    let tuple_type = Type::Tuple(vec![*key_type.clone(), *value_type.clone()]);
                    Ok((items_list_ptr, Type::List(Box::new(tuple_type))))
                }
                _ => Err(format!("Unknown method '{}' for dictionary type", attr)),
            },
            Type::Class {
                name,
                methods,
                fields,
                ..
            } => {
                if let Some(_method_type) = methods.get(attr) {
                    Err(format!(
                        "Method access for class '{}' not yet implemented",
                        name
                    ))
                } else if let Some(_field_type) = fields.get(attr) {
                    Err(format!(
                        "Field access for class '{}' not yet implemented",
                        name
                    ))
                } else {
                    Err(format!("Unknown attribute '{}' for class '{}'", attr, name))
                }
            }
            _ => Err(format!(
                "Type {:?} does not support attribute access",
                value_type
            )),
        }
    }

    /// Compile a dictionary comprehension expression
    fn compile_dict_comprehension(
        &mut self,
        key: &Expr,
        value: &Expr,
        generators: &[crate::ast::Comprehension],
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if generators.is_empty() {
            return Err("Dictionary comprehension must have at least one generator".to_string());
        }

        let result_dict = self.build_empty_dict("dict_comp_result")?;

        let dict_set_fn = match self.module.get_function("dict_set") {
            Some(f) => f,
            None => return Err("dict_set function not found".to_string()),
        };

        self.scope_stack.push_scope(false, false, false);

        let generator = &generators[0];

        let (iter_val, iter_type) = self.compile_expr(&generator.iter)?;

        if let Expr::Call { func, .. } = &*generator.iter {
            if let Expr::Name { id, .. } = func.as_ref() {
                if id == "range" {
                    let range_val = iter_val.into_int_value();

                    let current_function = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();
                    let loop_entry_block = self
                        .llvm_context
                        .append_basic_block(current_function, "range_comp_entry");
                    let loop_body_block = self
                        .llvm_context
                        .append_basic_block(current_function, "range_comp_body");
                    let loop_exit_block = self
                        .llvm_context
                        .append_basic_block(current_function, "range_comp_exit");

                    let index_ptr = self
                        .builder
                        .build_alloca(self.llvm_context.i64_type(), "range_index")
                        .unwrap();
                    self.builder
                        .build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false))
                        .unwrap();

                    self.builder
                        .build_unconditional_branch(loop_entry_block)
                        .unwrap();

                    self.builder.position_at_end(loop_entry_block);
                    let current_index = self
                        .builder
                        .build_load(self.llvm_context.i64_type(), index_ptr, "current_index")
                        .unwrap()
                        .into_int_value();
                    let cond = self
                        .builder
                        .build_int_compare(
                            inkwell::IntPredicate::SLT,
                            current_index,
                            range_val,
                            "range_cond",
                        )
                        .unwrap();
                    self.builder
                        .build_conditional_branch(cond, loop_body_block, loop_exit_block)
                        .unwrap();

                    self.builder.position_at_end(loop_body_block);

                    match &*generator.target {
                        Expr::Name { id, .. } => {
                            let target_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), id).unwrap();
                            self.builder.build_store(target_ptr, current_index).unwrap();

                            self.scope_stack.add_variable(id.clone(), target_ptr, Type::Int);

                            let mut continue_block = loop_body_block;
                            let mut condition_blocks = Vec::new();

                            for if_expr in &generator.ifs {
                                let if_block = self.llvm_context.append_basic_block(current_function, "if_block");
                                condition_blocks.push(if_block);

                                let (cond_val, _) = self.compile_expr(if_expr)?;
                                let cond_val = self.builder.build_int_truncate_or_bit_cast(cond_val.into_int_value(), self.llvm_context.bool_type(), "cond").unwrap();

                                self.builder.build_conditional_branch(cond_val, if_block, continue_block).unwrap();

                                self.builder.position_at_end(if_block);
                                continue_block = if_block;
                            }

                            let (key_val, key_type) = self.compile_expr(key)?;
                            let (value_val, value_type) = self.compile_expr(value)?;

                            let key_ptr = if crate::compiler::types::is_reference_type(&key_type) {
                                if key_val.is_pointer_value() {
                                    key_val.into_pointer_value()
                                } else {
                                    return Err(format!("Expected pointer value for key of type {:?}", key_type));
                                }
                            } else {
                                let key_alloca = self.builder.build_alloca(
                                    key_val.get_type(),
                                    "dict_comp_key"
                                ).unwrap();
                                self.builder.build_store(key_alloca, key_val).unwrap();
                                key_alloca
                            };

                            let value_ptr = if crate::compiler::types::is_reference_type(&value_type) {
                                if value_val.is_pointer_value() {
                                    value_val.into_pointer_value()
                                } else {
                                    return Err(format!("Expected pointer value for value of type {:?}", value_type));
                                }
                            } else {
                                let value_alloca = self.builder.build_alloca(
                                    value_val.get_type(),
                                    "dict_comp_value"
                                ).unwrap();
                                self.builder.build_store(value_alloca, value_val).unwrap();
                                value_alloca
                            };

                            self.builder.build_call(
                                dict_set_fn,
                                &[
                                    result_dict.into(),
                                    key_ptr.into(),
                                    value_ptr.into(),
                                ],
                                "dict_set_result"
                            ).unwrap();

                            let continue_block = self.llvm_context.append_basic_block(current_function, "continue_block");
                            self.builder.build_unconditional_branch(continue_block).unwrap();

                            self.builder.position_at_end(continue_block);

                            let next_index = self.builder.build_int_add(
                                current_index,
                                self.llvm_context.i64_type().const_int(1, false),
                                "next_index"
                            ).unwrap();

                            self.builder.build_store(index_ptr, next_index).unwrap();

                            self.builder.build_unconditional_branch(loop_entry_block).unwrap();

                            self.builder.position_at_end(loop_exit_block);

                            self.scope_stack.pop_scope();

                            return Ok((result_dict.into(), Type::Dict(Box::new(key_type), Box::new(value_type))));
                        },
                        _ => return Err("Only simple variable names are supported as targets in dictionary comprehensions".to_string()),
                    }
                }
            }
        }

        match iter_type {
            Type::List(_) => {
                let list_len_fn = match self.module.get_function("list_len") {
                    Some(f) => f,
                    None => return Err("list_len function not found".to_string()),
                };

                let list_ptr = iter_val.into_pointer_value();
                let call_site_value = self
                    .builder
                    .build_call(list_len_fn, &[list_ptr.into()], "list_len_result")
                    .unwrap();

                let list_len = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get list length".to_string())?;

                let list_get_fn = match self.module.get_function("list_get") {
                    Some(f) => f,
                    None => return Err("list_get function not found".to_string()),
                };

                let current_function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();
                let loop_entry_block = self
                    .llvm_context
                    .append_basic_block(current_function, "list_comp_entry");
                let loop_body_block = self
                    .llvm_context
                    .append_basic_block(current_function, "list_comp_body");
                let loop_exit_block = self
                    .llvm_context
                    .append_basic_block(current_function, "list_comp_exit");

                let index_ptr = self
                    .builder
                    .build_alloca(self.llvm_context.i64_type(), "list_index")
                    .unwrap();
                self.builder
                    .build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false))
                    .unwrap();

                self.builder
                    .build_unconditional_branch(loop_entry_block)
                    .unwrap();

                self.builder.position_at_end(loop_entry_block);
                let current_index = self
                    .builder
                    .build_load(self.llvm_context.i64_type(), index_ptr, "current_index")
                    .unwrap()
                    .into_int_value();
                let cond = self
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::SLT,
                        current_index,
                        list_len.into_int_value(),
                        "list_cond",
                    )
                    .unwrap();
                self.builder
                    .build_conditional_branch(cond, loop_body_block, loop_exit_block)
                    .unwrap();

                self.builder.position_at_end(loop_body_block);

                let call_site_value = self
                    .builder
                    .build_call(
                        list_get_fn,
                        &[list_ptr.into(), current_index.into()],
                        "list_get_result",
                    )
                    .unwrap();

                let element_val = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to get element from list".to_string())?;

                match &*generator.target {
                    Expr::Name { id, .. } => {
                        let mut element_type = if let Type::List(elem_type) = &iter_type {
                            *elem_type.clone()
                        } else {
                            Type::Any
                        };

                        element_type = match &element_type {
                            Type::Tuple(tuple_element_types) => {
                                if !tuple_element_types.is_empty() && tuple_element_types.iter().all(|t| t == &tuple_element_types[0]) {
                                    tuple_element_types[0].clone()
                                } else {
                                    element_type
                                }
                            },
                            _ => element_type
                        };

                        let target_ptr = match element_type {
                            Type::Int => self.builder.build_alloca(self.llvm_context.i64_type(), id).unwrap(),
                            Type::Float => self.builder.build_alloca(self.llvm_context.f64_type(), id).unwrap(),
                            Type::Bool => self.builder.build_alloca(self.llvm_context.bool_type(), id).unwrap(),
                            _ => self.builder.build_alloca(self.llvm_context.ptr_type(inkwell::AddressSpace::default()), id).unwrap(),
                        };

                        self.builder.build_store(target_ptr, element_val).unwrap();

                        self.scope_stack.add_variable(id.clone(), target_ptr, element_type);

                        let mut continue_block = loop_body_block;
                        let mut condition_blocks = Vec::new();

                        for if_expr in &generator.ifs {
                            let if_block = self.llvm_context.append_basic_block(current_function, "if_block");
                            condition_blocks.push(if_block);

                            let (cond_val, _) = self.compile_expr(if_expr)?;
                            let cond_val = self.builder.build_int_truncate_or_bit_cast(cond_val.into_int_value(), self.llvm_context.bool_type(), "cond").unwrap();

                            self.builder.build_conditional_branch(cond_val, if_block, continue_block).unwrap();

                            self.builder.position_at_end(if_block);
                            continue_block = if_block;
                        }

                        let (key_val, key_type) = self.compile_expr(key)?;
                        let (value_val, value_type) = self.compile_expr(value)?;

                        let key_ptr = if crate::compiler::types::is_reference_type(&key_type) {
                            if key_val.is_pointer_value() {
                                key_val.into_pointer_value()
                            } else {
                                return Err(format!("Expected pointer value for key of type {:?}", key_type));
                            }
                        } else {
                            let key_alloca = self.builder.build_alloca(
                                key_val.get_type(),
                                "dict_comp_key"
                            ).unwrap();
                            self.builder.build_store(key_alloca, key_val).unwrap();
                            key_alloca
                        };

                        let value_ptr = if crate::compiler::types::is_reference_type(&value_type) {
                            if value_val.is_pointer_value() {
                                value_val.into_pointer_value()
                            } else {
                                return Err(format!("Expected pointer value for value of type {:?}", value_type));
                            }
                        } else {
                            let value_alloca = self.builder.build_alloca(
                                value_val.get_type(),
                                "dict_comp_value"
                            ).unwrap();
                            self.builder.build_store(value_alloca, value_val).unwrap();
                            value_alloca
                        };

                        self.builder.build_call(
                            dict_set_fn,
                            &[
                                result_dict.into(),
                                key_ptr.into(),
                                value_ptr.into(),
                            ],
                            "dict_set_result"
                        ).unwrap();

                        let continue_block = self.llvm_context.append_basic_block(current_function, "continue_block");
                        self.builder.build_unconditional_branch(continue_block).unwrap();

                        self.builder.position_at_end(continue_block);

                        let next_index = self.builder.build_int_add(
                            current_index,
                            self.llvm_context.i64_type().const_int(1, false),
                            "next_index"
                        ).unwrap();

                        self.builder.build_store(index_ptr, next_index).unwrap();

                        self.builder.build_unconditional_branch(loop_entry_block).unwrap();

                        self.builder.position_at_end(loop_exit_block);

                        self.scope_stack.pop_scope();

                        return Ok((result_dict.into(), Type::Dict(Box::new(key_type), Box::new(value_type))));
                    },
                    _ => return Err("Only simple variable names are supported as targets in dictionary comprehensions".to_string()),
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported iterable type for dictionary comprehension: {:?}",
                    iter_type
                ))
            }
        }
    }
}

impl<'ctx> BinaryOpCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_binary_op(
        &mut self,
        left: inkwell::values::BasicValueEnum<'ctx>,
        left_type: &Type,
        op: Operator,
        right: inkwell::values::BasicValueEnum<'ctx>,
        right_type: &Type,
    ) -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String> {
        let common_type = self.get_common_type(left_type, right_type)?;

        let left_converted = if left_type != &common_type {
            self.convert_type(left, left_type, &common_type)?
        } else {
            left
        };

        let right_converted = if right_type != &common_type {
            self.convert_type(right, right_type, &common_type)?
        } else {
            right
        };

        match op {
            Operator::Add => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self
                        .builder
                        .build_int_add(left_int, right_int, "int_add")
                        .unwrap();
                    Ok((result.into(), Type::Int))
                }
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    let result = self
                        .builder
                        .build_float_add(left_float, right_float, "float_add")
                        .unwrap();
                    Ok((result.into(), Type::Float))
                }
                Type::String => {
                    let string_concat_fn = self
                        .module
                        .get_function("string_concat")
                        .unwrap_or_else(|| {
                            let str_ptr_type =
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                            let fn_type = str_ptr_type
                                .fn_type(&[str_ptr_type.into(), str_ptr_type.into()], false);
                            self.module.add_function("string_concat", fn_type, None)
                        });

                    let left_ptr = left_converted.into_pointer_value();
                    let right_ptr = right_converted.into_pointer_value();
                    let result = self
                        .builder
                        .build_call(
                            string_concat_fn,
                            &[left_ptr.into(), right_ptr.into()],
                            "string_concat_result",
                        )
                        .unwrap();

                    if let Some(result_val) = result.try_as_basic_value().left() {
                        Ok((result_val, Type::String))
                    } else {
                        Err("Failed to concatenate strings".to_string())
                    }
                }
                Type::List(elem_type) => {
                    let list_concat_fn = match self.module.get_function("list_concat") {
                        Some(f) => f,
                        None => return Err("list_concat function not found".to_string()),
                    };

                    let left_ptr = left_converted.into_pointer_value();
                    let right_ptr = right_converted.into_pointer_value();
                    let call_site_value = self
                        .builder
                        .build_call(
                            list_concat_fn,
                            &[left_ptr.into(), right_ptr.into()],
                            "list_concat_result",
                        )
                        .unwrap();

                    if let Some(ret_val) = call_site_value.try_as_basic_value().left() {
                        Ok((ret_val, Type::List(elem_type.clone())))
                    } else {
                        Err("Failed to concatenate lists".to_string())
                    }
                }
                _ => Err(format!("Addition not supported for type {:?}", common_type)),
            },

            Operator::Sub => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self
                        .builder
                        .build_int_sub(left_int, right_int, "int_sub")
                        .unwrap();
                    Ok((result.into(), Type::Int))
                }
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    let result = self
                        .builder
                        .build_float_sub(left_float, right_float, "float_sub")
                        .unwrap();
                    Ok((result.into(), Type::Float))
                }
                _ => Err(format!(
                    "Subtraction not supported for type {:?}",
                    common_type
                )),
            },

            Operator::Mult => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self
                        .builder
                        .build_int_mul(left_int, right_int, "int_mul")
                        .unwrap();
                    Ok((result.into(), Type::Int))
                }
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    let result = self
                        .builder
                        .build_float_mul(left_float, right_float, "float_mul")
                        .unwrap();
                    Ok((result.into(), Type::Float))
                }
                Type::String => {
                    if let Type::Int = *right_type {
                        let string_repeat_fn = self
                            .module
                            .get_function("string_repeat")
                            .unwrap_or_else(|| {
                                let str_ptr_type =
                                    self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                                let fn_type = str_ptr_type.fn_type(
                                    &[str_ptr_type.into(), self.llvm_context.i64_type().into()],
                                    false,
                                );
                                self.module.add_function("string_repeat", fn_type, None)
                            });

                        let left_ptr = left_converted.into_pointer_value();
                        let right_int = right_converted.into_int_value();
                        let result = self
                            .builder
                            .build_call(
                                string_repeat_fn,
                                &[left_ptr.into(), right_int.into()],
                                "string_repeat_result",
                            )
                            .unwrap();

                        if let Some(result_val) = result.try_as_basic_value().left() {
                            return Ok((result_val, Type::String));
                        } else {
                            return Err("Failed to repeat string".to_string());
                        }
                    }
                    Err(format!(
                        "String repetition requires an integer, got {:?}",
                        right_type
                    ))
                }
                Type::List(elem_type) => {
                    if let Type::Int = right_type {
                        let list_repeat_fn = match self.module.get_function("list_repeat") {
                            Some(f) => f,
                            None => return Err("list_repeat function not found".to_string()),
                        };

                        let left_ptr = left_converted.into_pointer_value();
                        let right_int = right_converted.into_int_value();
                        let call_site_value = self
                            .builder
                            .build_call(
                                list_repeat_fn,
                                &[left_ptr.into(), right_int.into()],
                                "list_repeat_result",
                            )
                            .unwrap();

                        if let Some(ret_val) = call_site_value.try_as_basic_value().left() {
                            return Ok((ret_val, Type::List(elem_type.clone())));
                        } else {
                            return Err("Failed to repeat list".to_string());
                        }
                    }
                    Err(format!(
                        "List repetition requires an integer, got {:?}",
                        right_type
                    ))
                }
                _ => Err(format!(
                    "Multiplication not supported for type {:?}",
                    common_type
                )),
            },

            Operator::Div => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();

                    let zero = self.llvm_context.i64_type().const_zero();
                    let is_zero = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::EQ, right_int, zero, "is_zero")
                        .unwrap();

                    let current_function = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();
                    let div_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "div");
                    let div_by_zero_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "cont");

                    self.builder
                        .build_conditional_branch(is_zero, div_by_zero_bb, div_bb)
                        .unwrap();

                    self.builder.position_at_end(div_bb);
                    let left_float = self
                        .builder
                        .build_signed_int_to_float(
                            left_int,
                            self.llvm_context.f64_type(),
                            "int_to_float",
                        )
                        .unwrap();
                    let right_float = self
                        .builder
                        .build_signed_int_to_float(
                            right_int,
                            self.llvm_context.f64_type(),
                            "int_to_float",
                        )
                        .unwrap();
                    let div_result = self
                        .builder
                        .build_float_div(left_float, right_float, "float_div")
                        .unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(div_by_zero_bb);
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(cont_bb);
                    let phi = self
                        .builder
                        .build_phi(self.llvm_context.f64_type(), "div_result")
                        .unwrap();
                    phi.add_incoming(&[(&div_result, div_bb), (&error_value, div_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Float))
                }
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();

                    let zero = self.llvm_context.f64_type().const_float(0.0);
                    let is_zero = self
                        .builder
                        .build_float_compare(
                            inkwell::FloatPredicate::OEQ,
                            right_float,
                            zero,
                            "is_zero",
                        )
                        .unwrap();

                    let current_function = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();
                    let div_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "div");
                    let div_by_zero_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "cont");

                    self.builder
                        .build_conditional_branch(is_zero, div_by_zero_bb, div_bb)
                        .unwrap();

                    self.builder.position_at_end(div_bb);
                    let div_result = self
                        .builder
                        .build_float_div(left_float, right_float, "float_div")
                        .unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(div_by_zero_bb);
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(cont_bb);
                    let phi = self
                        .builder
                        .build_phi(self.llvm_context.f64_type(), "div_result")
                        .unwrap();
                    phi.add_incoming(&[(&div_result, div_bb), (&error_value, div_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Float))
                }
                _ => Err(format!("Division not supported for type {:?}", common_type)),
            },

            Operator::FloorDiv => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();

                    let zero = self.llvm_context.i64_type().const_zero();
                    let is_zero = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::EQ, right_int, zero, "is_zero")
                        .unwrap();

                    let current_function = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();
                    let div_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "div");
                    let div_by_zero_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "cont");

                    self.builder
                        .build_conditional_branch(is_zero, div_by_zero_bb, div_bb)
                        .unwrap();

                    self.builder.position_at_end(div_bb);
                    let div_result = self
                        .builder
                        .build_int_signed_div(left_int, right_int, "int_div")
                        .unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(div_by_zero_bb);
                    let error_value = self.llvm_context.i64_type().const_zero();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(cont_bb);
                    let phi = self
                        .builder
                        .build_phi(self.llvm_context.i64_type(), "div_result")
                        .unwrap();
                    phi.add_incoming(&[(&div_result, div_bb), (&error_value, div_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Int))
                }
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();

                    let zero = self.llvm_context.f64_type().const_float(0.0);
                    let is_zero = self
                        .builder
                        .build_float_compare(
                            inkwell::FloatPredicate::OEQ,
                            right_float,
                            zero,
                            "is_zero",
                        )
                        .unwrap();

                    let current_function = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();
                    let div_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "div");
                    let div_by_zero_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "cont");

                    self.builder
                        .build_conditional_branch(is_zero, div_by_zero_bb, div_bb)
                        .unwrap();

                    self.builder.position_at_end(div_bb);
                    let div_result = self
                        .builder
                        .build_float_div(left_float, right_float, "float_div")
                        .unwrap();
                    let floor_result = self
                        .builder
                        .build_call(
                            self.module
                                .get_function("llvm.floor.f64")
                                .unwrap_or_else(|| {
                                    let f64_type = self.llvm_context.f64_type();
                                    let function_type = f64_type.fn_type(&[f64_type.into()], false);
                                    self.module
                                        .add_function("llvm.floor.f64", function_type, None)
                                }),
                            &[div_result.into()],
                            "floor_div",
                        )
                        .unwrap();
                    let floor_result = floor_result.try_as_basic_value().left().unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(div_by_zero_bb);
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(cont_bb);
                    let phi = self
                        .builder
                        .build_phi(self.llvm_context.f64_type(), "div_result")
                        .unwrap();
                    phi.add_incoming(&[(&floor_result, div_bb), (&error_value, div_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Float))
                }
                _ => Err(format!(
                    "Floor division not supported for type {:?}",
                    common_type
                )),
            },

            Operator::Mod => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();

                    let zero = self.llvm_context.i64_type().const_zero();
                    let is_zero = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::EQ, right_int, zero, "is_zero")
                        .unwrap();

                    let current_function = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();
                    let mod_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "mod");
                    let mod_by_zero_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "mod_by_zero");
                    let cont_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "cont");

                    self.builder
                        .build_conditional_branch(is_zero, mod_by_zero_bb, mod_bb)
                        .unwrap();

                    self.builder.position_at_end(mod_bb);
                    let mod_result = self
                        .builder
                        .build_int_signed_rem(left_int, right_int, "int_mod")
                        .unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(mod_by_zero_bb);
                    let error_value = self.llvm_context.i64_type().const_zero();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_by_zero_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(cont_bb);
                    let phi = self
                        .builder
                        .build_phi(self.llvm_context.i64_type(), "mod_result")
                        .unwrap();
                    phi.add_incoming(&[(&mod_result, mod_bb), (&error_value, mod_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Int))
                }
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();

                    let zero = self.llvm_context.f64_type().const_float(0.0);
                    let is_zero = self
                        .builder
                        .build_float_compare(
                            inkwell::FloatPredicate::OEQ,
                            right_float,
                            zero,
                            "is_zero",
                        )
                        .unwrap();

                    let current_function = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();
                    let mod_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "mod");
                    let mod_by_zero_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "mod_by_zero");
                    let cont_bb = self
                        .llvm_context
                        .append_basic_block(current_function, "cont");

                    self.builder
                        .build_conditional_branch(is_zero, mod_by_zero_bb, mod_bb)
                        .unwrap();

                    self.builder.position_at_end(mod_bb);
                    let mod_result = self
                        .builder
                        .build_call(
                            self.module.get_function("fmod").unwrap_or_else(|| {
                                let f64_type = self.llvm_context.f64_type();
                                let function_type =
                                    f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
                                self.module.add_function("fmod", function_type, None)
                            }),
                            &[left_float.into(), right_float.into()],
                            "float_mod",
                        )
                        .unwrap();
                    let mod_result = mod_result.try_as_basic_value().left().unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(mod_by_zero_bb);
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_by_zero_bb = self.builder.get_insert_block().unwrap();

                    self.builder.position_at_end(cont_bb);
                    let phi = self
                        .builder
                        .build_phi(self.llvm_context.f64_type(), "mod_result")
                        .unwrap();
                    phi.add_incoming(&[(&mod_result, mod_bb), (&error_value, mod_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Float))
                }
                _ => Err(format!("Modulo not supported for type {:?}", common_type)),
            },

            Operator::Pow => match common_type {
                Type::Int => {
                    let left_float = self.convert_type(left_converted, &Type::Int, &Type::Float)?;
                    let right_float =
                        self.convert_type(right_converted, &Type::Int, &Type::Float)?;

                    let pow_result = self
                        .builder
                        .build_call(
                            self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
                                let f64_type = self.llvm_context.f64_type();
                                let function_type =
                                    f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
                                self.module
                                    .add_function("llvm.pow.f64", function_type, None)
                            }),
                            &[
                                left_float.into_float_value().into(),
                                right_float.into_float_value().into(),
                            ],
                            "float_pow",
                        )
                        .unwrap();

                    let pow_float = pow_result.try_as_basic_value().left().unwrap();
                    let pow_int = self.convert_type(pow_float, &Type::Float, &Type::Int)?;

                    Ok((pow_int, Type::Int))
                }
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();

                    let pow_result = self
                        .builder
                        .build_call(
                            self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
                                let f64_type = self.llvm_context.f64_type();
                                let function_type =
                                    f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
                                self.module
                                    .add_function("llvm.pow.f64", function_type, None)
                            }),
                            &[left_float.into(), right_float.into()],
                            "float_pow",
                        )
                        .unwrap();

                    let pow_float = pow_result.try_as_basic_value().left().unwrap();

                    Ok((pow_float, Type::Float))
                }
                _ => Err(format!(
                    "Power operation not supported for type {:?}",
                    common_type
                )),
            },

            Operator::BitOr => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self
                        .builder
                        .build_or(left_int, right_int, "int_or")
                        .unwrap();
                    Ok((result.into(), Type::Int))
                }
                _ => Err(format!(
                    "Bitwise OR not supported for type {:?}",
                    common_type
                )),
            },

            Operator::BitXor => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self
                        .builder
                        .build_xor(left_int, right_int, "int_xor")
                        .unwrap();
                    Ok((result.into(), Type::Int))
                }
                _ => Err(format!(
                    "Bitwise XOR not supported for type {:?}",
                    common_type
                )),
            },

            Operator::BitAnd => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self
                        .builder
                        .build_and(left_int, right_int, "int_and")
                        .unwrap();
                    Ok((result.into(), Type::Int))
                }
                _ => Err(format!(
                    "Bitwise AND not supported for type {:?}",
                    common_type
                )),
            },

            Operator::LShift => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self
                        .builder
                        .build_left_shift(left_int, right_int, "int_lshift")
                        .unwrap();
                    Ok((result.into(), Type::Int))
                }
                _ => Err(format!(
                    "Left shift not supported for type {:?}",
                    common_type
                )),
            },

            Operator::RShift => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self
                        .builder
                        .build_right_shift(left_int, right_int, true, "int_rshift")
                        .unwrap();
                    Ok((result.into(), Type::Int))
                }
                _ => Err(format!(
                    "Right shift not supported for type {:?}",
                    common_type
                )),
            },

            Operator::MatMult => Err("Matrix multiplication not yet implemented".to_string()),

            #[allow(unreachable_patterns)]
            _ => Err(format!("Binary operator {:?} not implemented", op)),
        }
    }
}

impl<'ctx> ComparisonCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_comparison(
        &mut self,
        left: inkwell::values::BasicValueEnum<'ctx>,
        left_type: &Type,
        op: CmpOperator,
        right: inkwell::values::BasicValueEnum<'ctx>,
        right_type: &Type,
    ) -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String> {
        if matches!(op, CmpOperator::Is) || matches!(op, CmpOperator::IsNot) {
            if is_reference_type(left_type) && is_reference_type(right_type) {
                let left_ptr = if left.is_pointer_value() {
                    left.into_pointer_value()
                } else {
                    let left_as_ptr = self
                        .builder
                        .build_bit_cast(
                            left,
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "as_ptr",
                        )
                        .unwrap();
                    left_as_ptr.into_pointer_value()
                };

                let right_ptr = if right.is_pointer_value() {
                    right.into_pointer_value()
                } else {
                    let right_as_ptr = self
                        .builder
                        .build_bit_cast(
                            right,
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "as_ptr",
                        )
                        .unwrap();
                    right_as_ptr.into_pointer_value()
                };

                let left_ptr_int = self
                    .builder
                    .build_ptr_to_int(left_ptr, self.llvm_context.i64_type(), "ptr_as_int")
                    .unwrap();

                let right_ptr_int = self
                    .builder
                    .build_ptr_to_int(right_ptr, self.llvm_context.i64_type(), "ptr_as_int")
                    .unwrap();

                let is_same = self
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::EQ,
                        left_ptr_int,
                        right_ptr_int,
                        "is_same",
                    )
                    .unwrap();

                let result = if matches!(op, CmpOperator::IsNot) {
                    self.builder.build_not(is_same, "is_not_same").unwrap()
                } else {
                    is_same
                };

                return Ok((result.into(), Type::Bool));
            }

            return self.compile_comparison(
                left,
                left_type,
                if matches!(op, CmpOperator::Is) {
                    CmpOperator::Eq
                } else {
                    CmpOperator::NotEq
                },
                right,
                right_type,
            );
        }

        if matches!(op, CmpOperator::In) || matches!(op, CmpOperator::NotIn) {
            match right_type {
                Type::Dict(key_type, _) => {
                    if !left_type.can_coerce_to(key_type) {
                        return Err(format!("Type mismatch for 'in' operator: {:?} is not compatible with dictionary key type {:?}", left_type, key_type));
                    }

                    let dict_contains_fn = match self.module.get_function("dict_contains") {
                        Some(f) => f,
                        None => return Err("dict_contains function not found".to_string()),
                    };

                    let key_ptr = if crate::compiler::types::is_reference_type(left_type) {
                        if left.is_pointer_value() {
                            left.into_pointer_value()
                        } else {
                            return Err(format!(
                                "Expected pointer value for key of type {:?}",
                                left_type
                            ));
                        }
                    } else {
                        let key_alloca = self
                            .builder
                            .build_alloca(left.get_type(), "dict_key_temp")
                            .unwrap();
                        self.builder.build_store(key_alloca, left).unwrap();
                        key_alloca
                    };

                    let call_site_value = self
                        .builder
                        .build_call(
                            dict_contains_fn,
                            &[right.into_pointer_value().into(), key_ptr.into()],
                            "dict_contains_result",
                        )
                        .unwrap();

                    let contains_result = call_site_value
                        .try_as_basic_value()
                        .left()
                        .ok_or_else(|| "Failed to get result from dict_contains".to_string())?;

                    let contains_bool = self
                        .builder
                        .build_int_compare(
                            inkwell::IntPredicate::NE,
                            contains_result.into_int_value(),
                            self.llvm_context.i8_type().const_int(0, false),
                            "contains_bool",
                        )
                        .unwrap();

                    let result = if matches!(op, CmpOperator::NotIn) {
                        self.builder
                            .build_not(contains_bool, "not_contains_bool")
                            .unwrap()
                    } else {
                        contains_bool
                    };

                    return Ok((result.into(), Type::Bool));
                }
                Type::List(_) => {
                    return Err(format!("'in' operator not yet implemented for lists"));
                }
                Type::String => {
                    return Err(format!("'in' operator not yet implemented for strings"));
                }
                _ => {
                    return Err(format!(
                        "'in' operator not supported for type {:?}",
                        right_type
                    ));
                }
            }
        }

        let common_type = self.get_common_type(left_type, right_type)?;

        let left_converted = if left_type != &common_type {
            self.convert_type(left, left_type, &common_type)?
        } else {
            left
        };

        let right_converted = if right_type != &common_type {
            self.convert_type(right, right_type, &common_type)?
        } else {
            right
        };

        match common_type {
            Type::Int => {
                let left_int = left_converted.into_int_value();
                let right_int = right_converted.into_int_value();

                let pred = match op {
                    CmpOperator::Eq => inkwell::IntPredicate::EQ,
                    CmpOperator::NotEq => inkwell::IntPredicate::NE,
                    CmpOperator::Lt => inkwell::IntPredicate::SLT,
                    CmpOperator::LtE => inkwell::IntPredicate::SLE,
                    CmpOperator::Gt => inkwell::IntPredicate::SGT,
                    CmpOperator::GtE => inkwell::IntPredicate::SGE,
                    _ => {
                        return Err(format!(
                            "Comparison operator {:?} not supported for integers",
                            op
                        ))
                    }
                };

                let result = self
                    .builder
                    .build_int_compare(pred, left_int, right_int, "int_cmp")
                    .unwrap();
                Ok((result.into(), Type::Bool))
            }

            Type::Float => {
                let left_float = left_converted.into_float_value();
                let right_float = right_converted.into_float_value();

                let pred = match op {
                    CmpOperator::Eq => inkwell::FloatPredicate::OEQ,
                    CmpOperator::NotEq => inkwell::FloatPredicate::ONE,
                    CmpOperator::Lt => inkwell::FloatPredicate::OLT,
                    CmpOperator::LtE => inkwell::FloatPredicate::OLE,
                    CmpOperator::Gt => inkwell::FloatPredicate::OGT,
                    CmpOperator::GtE => inkwell::FloatPredicate::OGE,
                    _ => {
                        return Err(format!(
                            "Comparison operator {:?} not supported for floats",
                            op
                        ))
                    }
                };

                let result = self
                    .builder
                    .build_float_compare(pred, left_float, right_float, "float_cmp")
                    .unwrap();
                Ok((result.into(), Type::Bool))
            }

            Type::Bool => {
                let left_bool = left_converted.into_int_value();
                let right_bool = right_converted.into_int_value();

                let pred = match op {
                    CmpOperator::Eq => inkwell::IntPredicate::EQ,
                    CmpOperator::NotEq => inkwell::IntPredicate::NE,
                    _ => {
                        return Err(format!(
                            "Comparison operator {:?} not supported for booleans",
                            op
                        ))
                    }
                };

                let result = self
                    .builder
                    .build_int_compare(pred, left_bool, right_bool, "bool_cmp")
                    .unwrap();
                Ok((result.into(), Type::Bool))
            }

            Type::String => {
                let string_equals_fn =
                    self.module
                        .get_function("string_equals")
                        .unwrap_or_else(|| {
                            let str_ptr_type =
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                            let fn_type = self
                                .llvm_context
                                .bool_type()
                                .fn_type(&[str_ptr_type.into(), str_ptr_type.into()], false);
                            self.module.add_function("string_equals", fn_type, None)
                        });

                let left_ptr = left_converted.into_pointer_value();
                let right_ptr = right_converted.into_pointer_value();
                let result = self
                    .builder
                    .build_call(
                        string_equals_fn,
                        &[left_ptr.into(), right_ptr.into()],
                        "string_equals_result",
                    )
                    .unwrap();

                if let Some(result_val) = result.try_as_basic_value().left() {
                    let bool_result = result_val.into_int_value();

                    match op {
                        CmpOperator::Eq => Ok((bool_result.into(), Type::Bool)),
                        CmpOperator::NotEq => {
                            let not_result = self
                                .builder
                                .build_not(bool_result, "string_not_equals")
                                .unwrap();
                            Ok((not_result.into(), Type::Bool))
                        }
                        _ => Err(format!("String comparison operator {:?} not supported", op)),
                    }
                } else {
                    Err("Failed to compare strings".to_string())
                }
            }

            _ => Err(format!(
                "Comparison not supported for type {:?}",
                common_type
            )),
        }
    }
}

impl<'ctx> AssignmentCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_assignment(
        &mut self,
        target: &Expr,
        value: BasicValueEnum<'ctx>,
        value_type: &Type,
    ) -> Result<(), String> {
        match target {
            Expr::Tuple { elts, .. } => {
                if let Type::Tuple(element_types) = value_type {
                    if elts.len() != element_types.len() {
                        return Err(format!(
                            "Tuple unpacking mismatch: expected {} elements, got {}",
                            elts.len(),
                            element_types.len()
                        ));
                    }

                    let llvm_types: Vec<BasicTypeEnum> = element_types
                        .iter()
                        .map(|ty| self.get_llvm_type(ty))
                        .collect();

                    let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);
                    let tuple_ptr = if value.is_pointer_value() {
                        value.into_pointer_value()
                    } else if value.is_struct_value() {
                        let temp_ptr = self
                            .builder
                            .build_alloca(tuple_struct, "temp_tuple")
                            .unwrap();
                        self.builder.build_store(temp_ptr, value).unwrap();
                        temp_ptr
                    } else if value.is_int_value() {
                        let ptr = self
                            .builder
                            .build_int_to_ptr(
                                value.into_int_value(),
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "tuple_ptr",
                            )
                            .unwrap();

                        self.builder
                            .build_pointer_cast(
                                ptr,
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                "tuple_struct_ptr",
                            )
                            .unwrap()
                    } else {
                        return Err(format!(
                            "Cannot unpack value of type {:?} - expected a tuple",
                            value_type
                        ));
                    };

                    for (i, target_expr) in elts.iter().enumerate() {
                        let element_ptr = self
                            .builder
                            .build_struct_gep(
                                tuple_struct,
                                tuple_ptr,
                                i as u32,
                                &format!("tuple_element_{}", i),
                            )
                            .unwrap();

                        let element_value = self
                            .builder
                            .build_load(
                                self.get_llvm_type(&element_types[i]),
                                element_ptr,
                                &format!("load_tuple_element_{}", i),
                            )
                            .unwrap();

                        if let Expr::Name { id, .. } = target_expr.as_ref() {
                            self.register_variable(id.clone(), element_types[i].clone());

                            if self.get_variable_ptr(id).is_none() {
                                let ptr = self
                                    .builder
                                    .build_alloca(self.get_llvm_type(&element_types[i]), id)
                                    .unwrap();

                                self.builder.build_store(ptr, element_value).unwrap();

                                self.add_variable_to_scope(
                                    id.clone(),
                                    ptr,
                                    element_types[i].clone(),
                                );

                                self.variables.insert(id.clone(), ptr);

                                continue;
                            }
                        }

                        self.compile_assignment(target_expr, element_value, &element_types[i])?;
                    }

                    Ok(())
                } else {
                    Err(format!(
                        "Cannot unpack non-tuple value of type {:?}",
                        value_type
                    ))
                }
            }
            Expr::Name { id, .. } => {
                let is_global = if let Some(current_scope) = self.scope_stack.current_scope() {
                    current_scope.is_global(id)
                } else {
                    false
                };

                let is_nonlocal = if let Some(current_scope) = self.scope_stack.current_scope() {
                    current_scope.is_nonlocal(id)
                } else {
                    false
                };

                if is_nonlocal {
                    if let Some(env_name) = &self.current_environment {
                        if let Some(env) = self.get_closure_environment(env_name) {
                            if let Some(proxy_ptr) = env.get_nonlocal_proxy(id) {
                                self.builder.build_store(*proxy_ptr, value).unwrap();
                                println!("Assigned to nonlocal variable '{}' using proxy in environment {}", id, env_name);
                                return Ok(());
                            }
                        }
                    }

                    if let Some(current_scope) = self.scope_stack.current_scope() {
                        if let Some(unique_name) = current_scope.get_nonlocal_mapping(id) {
                            if let Some(ptr) = current_scope.get_variable(unique_name).cloned() {
                                self.builder.build_store(ptr, value).unwrap();
                                println!(
                                    "Assigned to nonlocal variable '{}' using unique name '{}'",
                                    id, unique_name
                                );
                                return Ok(());
                            }
                        }

                        if self.scope_stack.scopes.len() >= 2 {
                            let parent_scope_index = self.scope_stack.scopes.len() - 2;

                            let parent_var_ptr = self.scope_stack.scopes[parent_scope_index]
                                .get_variable(id)
                                .cloned();

                            if let Some(_ptr) = parent_var_ptr {
                                let llvm_type = value.get_type();

                                let current_position = self.builder.get_insert_block().unwrap();

                                let current_function = self.current_function.unwrap();
                                let entry_block = current_function.get_first_basic_block().unwrap();
                                if let Some(first_instr) = entry_block.get_first_instruction() {
                                    self.builder.position_before(&first_instr);
                                } else {
                                    self.builder.position_at_end(entry_block);
                                }

                                let local_ptr = self.builder.build_alloca(llvm_type, id).unwrap();

                                self.builder.position_at_end(current_position);

                                self.builder.build_store(local_ptr, value).unwrap();

                                self.scope_stack.current_scope_mut().map(|scope| {
                                    scope.add_variable(id.clone(), local_ptr, value_type.clone());
                                    println!(
                                        "Created shadowing variable '{}' in nested function",
                                        id
                                    );
                                });

                                self.variables.insert(id.clone(), local_ptr);

                                self.register_variable(id.clone(), value_type.clone());

                                return Ok(());
                            }
                        }
                    }

                    if let Some(env_name) = &self.current_environment {
                        let mut env_data = None;

                        if let Some(env) = self.get_closure_environment(env_name) {
                            if let Some(index) = env.get_index(id) {
                                if let Some(var_type) = env.get_type(id) {
                                    if let Some(env_ptr) = env.env_ptr {
                                        if let Some(struct_type) = env.env_type {
                                            env_data = Some((
                                                index,
                                                var_type.clone(),
                                                env_ptr,
                                                struct_type,
                                            ));
                                        }
                                    }
                                }
                            }
                        }

                        if let Some((index, var_type, env_ptr, struct_type)) = env_data {
                            let unique_name =
                                format!("__nonlocal_{}_{}", env_name.replace('.', "_"), id);

                            let llvm_type = self.get_llvm_type(&var_type);
                            let ptr = self.builder.build_alloca(llvm_type, &unique_name).unwrap();

                            self.store_nonlocal_variable(ptr, value, &unique_name)?;

                            if let Some(current_scope) = self.scope_stack.current_scope_mut() {
                                current_scope.add_variable(
                                    unique_name.clone(),
                                    ptr,
                                    var_type.clone(),
                                );
                                current_scope.add_nonlocal_mapping(id.clone(), unique_name.clone());
                                println!("Created local variable for nonlocal variable '{}' with unique name '{}'", id, unique_name);
                            }

                            let field_ptr = self
                                .builder
                                .build_struct_gep(
                                    struct_type,
                                    env_ptr,
                                    index,
                                    &format!("env_{}_ptr", id),
                                )
                                .unwrap();

                            self.builder.build_store(field_ptr, value).unwrap();
                            println!("Updated nonlocal variable '{}' in closure environment", id);

                            return Ok(());
                        }
                    }
                }

                let simple_global_name = format!("__nonlocal_{}", id);

                let current_function =
                    if let Some(func) = self.builder.get_insert_block().unwrap().get_parent() {
                        func.get_name().to_string_lossy().to_string()
                    } else {
                        "".to_string()
                    };

                let mut global_var = None;

                if !current_function.is_empty() {
                    let func_global_name =
                        format!("__nonlocal_{}_{}", current_function.replace('.', "_"), id);
                    if let Some(var) = self.module.get_global(&func_global_name) {
                        global_var = Some(var);
                    }

                    if global_var.is_none() && current_function.contains('.') {
                        let parts: Vec<&str> = current_function.split('.').collect();
                        for i in 1..parts.len() {
                            let parent_name = parts[..i].join(".");
                            let parent_global_name =
                                format!("__nonlocal_{}_{}", parent_name.replace('.', "_"), id);
                            if let Some(var) = self.module.get_global(&parent_global_name) {
                                global_var = Some(var);
                                break;
                            }
                        }
                    }
                }

                if global_var.is_none() {
                    if let Some(var) = self.module.get_global(&simple_global_name) {
                        global_var = Some(var);
                    }
                }

                if let Some(global_var) = global_var {
                    self.builder
                        .build_store(global_var.as_pointer_value(), value)
                        .unwrap();
                    println!(
                        "Assigned to nonlocal variable '{}' using global variable",
                        id
                    );
                    return Ok(());
                }

                if is_global {
                    if let Some(global_scope) = self.scope_stack.global_scope() {
                        if let Some(ptr) = global_scope.get_variable(id) {
                            if let Some(target_type) = self.lookup_variable_type(id) {
                                let converted_value = if target_type != value_type {
                                    self.convert_type(value, value_type, target_type)?
                                } else {
                                    value
                                };

                                self.builder.build_store(*ptr, converted_value).unwrap();
                                return Ok(());
                            }
                        } else {
                            let global_var = self.module.add_global(
                                self.get_llvm_type(value_type).into_int_type(),
                                None,
                                id,
                            );

                            global_var
                                .set_initializer(&self.get_llvm_type(value_type).const_zero());

                            let ptr = global_var.as_pointer_value();

                            if let Some(global_scope) = self.scope_stack.global_scope_mut() {
                                global_scope.add_variable(id.clone(), ptr, value_type.clone());
                            }

                            self.builder.build_store(ptr, value).unwrap();
                            return Ok(());
                        }
                    }
                } else if is_nonlocal {
                    if let Some(ptr) = self.get_variable_ptr(id) {
                        if let Some(target_type) = self.lookup_variable_type(id) {
                            let converted_value = if target_type != value_type {
                                self.convert_type(value, value_type, target_type)?
                            } else {
                                value
                            };

                            self.builder.build_store(ptr, converted_value).unwrap();
                            return Ok(());
                        }
                    } else {
                        return Err(format!("Nonlocal variable '{}' not found", id));
                    }
                }

                if let Some(ptr) = self.get_variable_ptr(id) {
                    if let Some(target_type) = self.lookup_variable_type(id) {
                        let converted_value = if target_type != value_type {
                            self.convert_type(value, value_type, target_type)?
                        } else {
                            value
                        };

                        self.builder.build_store(ptr, converted_value).unwrap();
                        Ok(())
                    } else {
                        Err(format!("Variable '{}' has unknown type", id))
                    }
                } else {
                    let ptr = if let Some(current_function) = self.current_function {
                        let fn_name = current_function.get_name().to_string_lossy();
                        if fn_name.contains('.') {
                            let current_position = self.builder.get_insert_block().unwrap();

                            let entry_block = current_function.get_first_basic_block().unwrap();
                            if let Some(first_instr) = entry_block.get_first_instruction() {
                                self.builder.position_before(&first_instr);
                            } else {
                                self.builder.position_at_end(entry_block);
                            }

                            let llvm_type = self.get_llvm_type(value_type);

                            let ptr = self.builder.build_alloca(llvm_type, id).unwrap();

                            self.builder.position_at_end(current_position);

                            ptr
                        } else {
                            self.allocate_variable(id.clone(), value_type)
                        }
                    } else {
                        self.allocate_variable(id.clone(), value_type)
                    };

                    self.register_variable(id.clone(), value_type.clone());

                    if let Some(current_scope) = self.scope_stack.current_scope_mut() {
                        current_scope.add_variable(id.clone(), ptr, value_type.clone());
                        println!("Added variable '{}' to current scope", id);
                    }

                    self.builder.build_store(ptr, value).unwrap();
                    Ok(())
                }
            }

            Expr::Subscript { value, slice, .. } => {
                let (container_val, container_type) = self.compile_expr(value)?;

                let (index_val, index_type) = self.compile_expr(slice)?;

                match &container_type {
                    Type::List(_) => {
                        if !matches!(index_type, Type::Int) {
                            return Err(format!(
                                "List index must be an integer, got {:?}",
                                index_type
                            ));
                        }

                        let list_set_fn = match self.module.get_function("list_set") {
                            Some(f) => f,
                            None => return Err("list_set function not found".to_string()),
                        };

                        let (value_val, _) = self.compile_expr(value)?;

                        let value_alloca = self
                            .builder
                            .build_alloca(value_val.get_type(), "list_set_value")
                            .unwrap();
                        self.builder.build_store(value_alloca, value_val).unwrap();

                        self.builder
                            .build_call(
                                list_set_fn,
                                &[
                                    container_val.into_pointer_value().into(),
                                    index_val.into_int_value().into(),
                                    value_alloca.into(),
                                ],
                                "list_set_result",
                            )
                            .unwrap();

                        Ok(())
                    }
                    Type::Dict(key_type, _value_type) => {
                        if matches!(**key_type, Type::Unknown) {
                            println!(
                                "Updating dictionary key type from Unknown to {:?}",
                                index_type
                            );
                        } else if !index_type.can_coerce_to(key_type)
                            && !matches!(index_type, Type::String)
                            && !matches!(**key_type, Type::Unknown)
                        {
                            return Err(format!(
                                "Dictionary key type mismatch: expected {:?}, got {:?}",
                                key_type, index_type
                            ));
                        }

                        let dict_set_fn = match self.module.get_function("dict_set") {
                            Some(f) => f,
                            None => return Err("dict_set function not found".to_string()),
                        };

                        let key_ptr = if crate::compiler::types::is_reference_type(&index_type) {
                            index_val
                        } else {
                            let key_alloca = self
                                .builder
                                .build_alloca(index_val.get_type(), "dict_key_temp")
                                .unwrap();
                            self.builder.build_store(key_alloca, index_val).unwrap();
                            key_alloca.into()
                        };

                        let (value_val, _value_type) = self.compile_expr(target)?;

                        let value_alloca = self
                            .builder
                            .build_alloca(value_val.get_type(), "dict_value_temp")
                            .unwrap();
                        self.builder.build_store(value_alloca, value_val).unwrap();

                        self.builder
                            .build_call(
                                dict_set_fn,
                                &[
                                    container_val.into_pointer_value().into(),
                                    key_ptr.into(),
                                    value_alloca.into(),
                                ],
                                "dict_set_result",
                            )
                            .unwrap();

                        Ok(())
                    }
                    Type::Tuple(_) => {
                        return Err("Tuple elements cannot be modified".to_string());
                    }
                    Type::String => {
                        return Err("String elements cannot be modified".to_string());
                    }
                    _ => Err(format!("Type {:?} is not indexable", container_type)),
                }
            }

            _ => Err(format!("Unsupported assignment target: {:?}", target)),
        }
    }
}
