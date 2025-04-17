use crate::ast::{CmpOperator, Expr, NameConstant, Number, Operator};
use crate::compiler::context::CompilationContext;
use crate::compiler::types::Type;
use crate::compiler::types::is_reference_type;
use inkwell::values::BasicValueEnum;
use inkwell::types::BasicTypeEnum;

/// Extension trait for handling expression code generation
pub trait ExprCompiler<'ctx> {
    fn evaluate_comprehension_conditions(&mut self, generator: &crate::ast::Comprehension, current_function: inkwell::values::FunctionValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, String>;
    fn handle_general_iteration_for_comprehension(&mut self, elt: &Expr, generator: &crate::ast::Comprehension, iter_val: BasicValueEnum<'ctx>, iter_type: Type, result_list: inkwell::values::PointerValue<'ctx>, list_append_fn: inkwell::values::FunctionValue<'ctx>) -> Result<(), String>;
    fn handle_string_iteration_for_comprehension(&mut self, elt: &Expr, generator: &crate::ast::Comprehension, str_ptr: inkwell::values::PointerValue<'ctx>, result_list: inkwell::values::PointerValue<'ctx>, list_append_fn: inkwell::values::FunctionValue<'ctx>) -> Result<(), String>;
    fn handle_list_iteration_for_comprehension(&mut self, elt: &Expr, generator: &crate::ast::Comprehension, list_ptr: inkwell::values::PointerValue<'ctx>, result_list: inkwell::values::PointerValue<'ctx>, list_append_fn: inkwell::values::FunctionValue<'ctx>) -> Result<(), String>;
    fn process_list_comprehension_element(&mut self, elt: &Expr, should_append: inkwell::values::IntValue<'ctx>, result_list: inkwell::values::PointerValue<'ctx>, list_append_fn: inkwell::values::FunctionValue<'ctx>, current_function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String>;
    fn handle_tuple_dynamic_index(&mut self, tuple_val: BasicValueEnum<'ctx>, tuple_type: Type, index_val: inkwell::values::IntValue<'ctx>, element_types: &[Type]) -> Result<(BasicValueEnum<'ctx>, Type), String>;
    fn build_empty_list(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_list(&self, elements: Vec<BasicValueEnum<'ctx>>, element_type: &Type) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_tuple(&self, elements: Vec<BasicValueEnum<'ctx>>, element_types: &[Type]) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_empty_tuple(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_empty_dict(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_dict(&self, keys: Vec<BasicValueEnum<'ctx>>, values: Vec<BasicValueEnum<'ctx>>, key_type: &Type, value_type: &Type) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_empty_set(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_set(&self, elements: Vec<BasicValueEnum<'ctx>>, element_type: &Type) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_list_get_item(&self, list_ptr: inkwell::values::PointerValue<'ctx>, index: inkwell::values::IntValue<'ctx>) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_list_slice(&self, list_ptr: inkwell::values::PointerValue<'ctx>, start: inkwell::values::IntValue<'ctx>, stop: inkwell::values::IntValue<'ctx>, step: inkwell::values::IntValue<'ctx>) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_dict_get_item(&self, dict_ptr: inkwell::values::PointerValue<'ctx>, key: BasicValueEnum<'ctx>, key_type: &Type) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn build_string_get_char(&self, str_ptr: inkwell::values::PointerValue<'ctx>, index: inkwell::values::IntValue<'ctx>) -> Result<BasicValueEnum<'ctx>, String>;
    fn build_string_slice(&self, str_ptr: inkwell::values::PointerValue<'ctx>, start: inkwell::values::IntValue<'ctx>, stop: inkwell::values::IntValue<'ctx>, step: inkwell::values::IntValue<'ctx>) -> Result<inkwell::values::PointerValue<'ctx>, String>;
    fn compile_slice_operation(&mut self, value_val: BasicValueEnum<'ctx>, value_type: Type, lower: Option<&Expr>, upper: Option<&Expr>, step: Option<&Expr>) -> Result<(BasicValueEnum<'ctx>, Type), String>;
    /// Compile an expression and return the resulting LLVM value with its type

    fn compile_slice_operation_non_recursive(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        lower: Option<&Expr>,
        upper: Option<&Expr>,
        step: Option<&Expr>
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    fn compile_expr(&mut self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a numeric literal
    fn compile_number(&mut self, num: &Number) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a name constant (True, False, None)
    fn compile_name_constant(&mut self, constant: &NameConstant) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a subscript expression (e.g., tuple[0])
    fn compile_subscript(&mut self, value: &Expr, slice: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    fn compile_subscript_non_recursive(&mut self, value: &Expr, slice: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a subscript expression with a pre-compiled value
    fn compile_subscript_with_value(&mut self, value_val: BasicValueEnum<'ctx>, value_type: Type, slice: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    fn compile_subscript_with_value_non_recursive(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        slice: &Expr
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    fn handle_range_list_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        range_val: BasicValueEnum<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>
    ) -> Result<(), String>;

    /// Compile a list comprehension expression
    fn compile_list_comprehension(&mut self, elt: &Expr, generators: &[crate::ast::Comprehension]) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    fn compile_list_comprehension_non_recursive(
        &mut self,
        elt: &Expr,
        generators: &[crate::ast::Comprehension]
    ) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a dictionary comprehension expression
    fn compile_dict_comprehension(&mut self, key: &Expr, value: &Expr, generators: &[crate::ast::Comprehension]) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Non-recursive implementation of dictionary comprehension compilation
    fn compile_dict_comprehension_non_recursive(&mut self, key: &Expr, value: &Expr, generators: &[crate::ast::Comprehension]) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile an attribute access expression (e.g., dict.keys())
    fn compile_attribute_access(&mut self, value: &Expr, attr: &str) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Non-recursive implementation of attribute access compilation
    fn compile_attribute_access_non_recursive(&mut self, value: &Expr, attr: &str) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a dict.keys() method call
    fn compile_dict_keys(&mut self, dict_ptr: inkwell::values::PointerValue<'ctx>, key_type: &Type) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a dict.values() method call
    fn compile_dict_values(&mut self, dict_ptr: inkwell::values::PointerValue<'ctx>, value_type: &Type) -> Result<(BasicValueEnum<'ctx>, Type), String>;

    /// Compile a dict.items() method call
    fn compile_dict_items(&mut self, dict_ptr: inkwell::values::PointerValue<'ctx>, key_type: &Type, value_type: &Type) -> Result<(BasicValueEnum<'ctx>, Type), String>;
}

pub trait AssignmentCompiler<'ctx> {
    /// Compile an assignment expression
    fn compile_assignment(&mut self, target: &Expr, value: BasicValueEnum<'ctx>,
                        value_type: &Type) -> Result<(), String>;
}

/// Extension trait for handling binary operations with type conversions
pub trait BinaryOpCompiler<'ctx> {
    /// Compile a binary operation with type conversion if needed
    fn compile_binary_op(&mut self, left: inkwell::values::BasicValueEnum<'ctx>, left_type: &Type,
                       op: Operator, right: inkwell::values::BasicValueEnum<'ctx>, right_type: &Type)
                       -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String>;
}

/// Extension trait for handling comparison operations with type conversions
pub trait ComparisonCompiler<'ctx> {
    /// Compile a comparison operation with type conversion if needed
    fn compile_comparison(&mut self, left: inkwell::values::BasicValueEnum<'ctx>, left_type: &Type,
                        op: CmpOperator, right: inkwell::values::BasicValueEnum<'ctx>, right_type: &Type)
                        -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String>;
}

impl<'ctx> ExprCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_expr(&mut self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Always use the non-recursive implementation to avoid stack overflow
        use crate::compiler::expr_non_recursive::ExprNonRecursive;
        self.compile_expr_non_recursive(expr)
    }

    fn compile_dict_keys(&mut self, dict_ptr: inkwell::values::PointerValue<'ctx>, key_type: &Type) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Get the dict_keys function
        let dict_keys_fn = match self.module.get_function("dict_keys") {
            Some(f) => f,
            None => return Err("dict_keys function not found".to_string()),
        };

        // Call the dict_keys function
        let call_site_value = self.builder.build_call(
            dict_keys_fn,
            &[dict_ptr.into()],
            "dict_keys_result"
        ).unwrap();

        // Get the result as a list pointer
        let result = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get keys from dictionary".to_string())?;

        // Return the result as a list of keys
        Ok((result, Type::List(Box::new(key_type.clone()))))
    }

    fn compile_dict_values(&mut self, dict_ptr: inkwell::values::PointerValue<'ctx>, value_type: &Type) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Get the dict_values function
        let dict_values_fn = match self.module.get_function("dict_values") {
            Some(f) => f,
            None => return Err("dict_values function not found".to_string()),
        };

        // Call the dict_values function
        let call_site_value = self.builder.build_call(
            dict_values_fn,
            &[dict_ptr.into()],
            "dict_values_result"
        ).unwrap();

        // Get the result as a list pointer
        let result = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get values from dictionary".to_string())?;

        // Return the result as a list of values
        Ok((result, Type::List(Box::new(value_type.clone()))))
    }

    fn compile_dict_items(&mut self, dict_ptr: inkwell::values::PointerValue<'ctx>, key_type: &Type, value_type: &Type) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Get the dict_items function
        let dict_items_fn = match self.module.get_function("dict_items") {
            Some(f) => f,
            None => return Err("dict_items function not found".to_string()),
        };

        // Call the dict_items function
        let call_site_value = self.builder.build_call(
            dict_items_fn,
            &[dict_ptr.into()],
            "dict_items_result"
        ).unwrap();

        // Get the result as a list pointer
        let result = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get items from dictionary".to_string())?;

        // Create a tuple type for the key-value pairs
        let tuple_types = vec![key_type.clone(), value_type.clone()];

        // Return the result as a list of tuples
        Ok((result, Type::List(Box::new(Type::Tuple(tuple_types)))))
    }

    // Placeholder methods for collection operations (to be implemented with runtime support)
    // These would be defined in your CompilationContext impl block

    fn build_empty_list(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Get the list_new function
        let list_new_fn = match self.module.get_function("list_new") {
            Some(f) => f,
            None => return Err("list_new function not found".to_string()),
        };

        // Call list_new to create an empty list
        let call_site_value = self.builder.build_call(list_new_fn, &[], name).unwrap();
        let list_ptr = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to create empty list".to_string())?;

        Ok(list_ptr.into_pointer_value())
    }

    fn build_list(
        &self,
        elements: Vec<BasicValueEnum<'ctx>>,
        element_type: &Type
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Create a list with the given capacity
        let list_with_capacity_fn = match self.module.get_function("list_with_capacity") {
            Some(f) => f,
            None => return Err("list_with_capacity function not found".to_string()),
        };

        // Get the length of the list
        let len = elements.len() as u64;
        let len_value = self.llvm_context.i64_type().const_int(len, false);

        // Call list_with_capacity to create a list with the given capacity
        let call_site_value = self.builder.build_call(list_with_capacity_fn, &[len_value.into()], "list_with_capacity").unwrap();
        let list_ptr = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to create list with capacity".to_string())?;

        let list_ptr = list_ptr.into_pointer_value();

        // Add each element to the list
        let list_append_fn = match self.module.get_function("list_append") {
            Some(f) => f,
            None => return Err("list_append function not found".to_string()),
        };

        for (i, element) in elements.iter().enumerate() {
            // Handle the element based on its type
            let element_ptr = if crate::compiler::types::is_reference_type(element_type) {
                *element
            } else {
                // For non-reference types, we need to allocate memory and store the value
                let element_alloca = self.builder.build_alloca(
                    element.get_type(),
                    &format!("list_element_{}", i)
                ).unwrap();
                self.builder.build_store(element_alloca, *element).unwrap();
                element_alloca.into()
            };

            // Call list_append to add the element to the list
            self.builder.build_call(
                list_append_fn,
                &[list_ptr.into(), element_ptr.into()],
                &format!("list_append_{}", i)
            ).unwrap();
        }

        Ok(list_ptr)
    }

    fn build_empty_tuple(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Create an empty tuple struct type
        let tuple_type = self.llvm_context.struct_type(&[], false);

        // Allocate memory for the tuple
        let tuple_ptr = self.builder.build_alloca(tuple_type, name).unwrap();

        Ok(tuple_ptr)
    }

    fn build_tuple(
        &self,
        elements: Vec<BasicValueEnum<'ctx>>,
        element_types: &[Type]
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Create LLVM types for each element
        let llvm_types: Vec<BasicTypeEnum> = element_types
            .iter()
            .map(|ty| self.get_llvm_type(ty))
            .collect();

        // Create a struct type for the tuple
        let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

        // Allocate memory for the tuple
        let tuple_ptr = self.builder.build_alloca(tuple_struct, "tuple").unwrap();

        // Store each element in the tuple
        for (i, element) in elements.iter().enumerate() {
            // Get a pointer to the i-th element of the tuple
            let element_ptr = self.builder.build_struct_gep(tuple_struct, tuple_ptr, i as u32, &format!("tuple_element_{}", i)).unwrap();

            // Store the element
            self.builder.build_store(element_ptr, *element).unwrap();
        }

        Ok(tuple_ptr)
    }

    /// Compile a subscript expression (e.g., tuple[0])
    fn compile_subscript(&mut self, value: &Expr, slice: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        self.compile_subscript_non_recursive(value, slice)
    }

    fn compile_subscript_non_recursive(&mut self, value: &Expr, slice: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Stack for tracking operations
        let mut work_stack = Vec::new();
        let mut value_stack = Vec::new();

        // Start with the base value
        work_stack.push((value, slice));

        while let Some((current_value, current_slice)) = work_stack.pop() {
            // Compile the value using the non-recursive implementation
            use crate::compiler::expr_non_recursive::ExprNonRecursive;
            let (value_val, value_type) = self.compile_expr_non_recursive(current_value)?;

            // Handle the slice
            let result = if let Expr::Slice { lower, upper, step, .. } = current_slice {
                self.compile_slice_operation_non_recursive(
                    value_val,
                    value_type,
                    lower.as_deref(),
                    upper.as_deref(),
                    step.as_deref()
                )?
            } else {
                self.compile_subscript_with_value_non_recursive(value_val, value_type, current_slice)?
            };

            value_stack.push(result);
        }

        value_stack.pop().ok_or_else(|| "Empty value stack".to_string())
    }

    fn compile_subscript_with_value(&mut self, value_val: BasicValueEnum<'ctx>, value_type: Type, slice: &Expr) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        self.compile_subscript_with_value_non_recursive(value_val, value_type, slice)
    }

    fn compile_subscript_with_value_non_recursive(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        slice: &Expr
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Handle slice expressions
        if let Expr::Slice { lower, upper, step, .. } = slice {
            return self.compile_slice_operation(
                value_val,
                value_type.clone(),
                lower.as_deref(),
                upper.as_deref(),
                step.as_deref()
            );
        }

        // Ensure the current block has a terminator before we create new blocks
        self.ensure_block_has_terminator();

        // Compile the index for regular subscript using the non-recursive implementation
        use crate::compiler::expr_non_recursive::ExprNonRecursive;
        let (index_val, index_type) = self.compile_expr_non_recursive(slice)?;

        // Ensure the current block has a terminator after compiling the index
        self.ensure_block_has_terminator();

        // Handle different container types for indexing
        let result = match &value_type {
            Type::List(element_type) => {
                // For lists, we need an integer index
                if !index_type.can_coerce_to(&Type::Int) {
                    return Err(format!("List index must be an integer, got {:?}", index_type));
                }

                // Convert index to integer if necessary
                let index_int = if index_type != Type::Int {
                    self.convert_type(index_val, &index_type, &Type::Int)?.into_int_value()
                } else {
                    index_val.into_int_value()
                };

                // Get the item from the list without recursion
                let item_ptr = self.build_list_get_item(
                    value_val.into_pointer_value(),
                    index_int
                )?;

                // Load the value from the pointer
                let element_type_ref = element_type.as_ref();

                // If the element type is a tuple, extract the element type if all elements are the same
                let actual_element_type = match element_type_ref {
                    Type::Tuple(tuple_element_types) => {
                        if !tuple_element_types.is_empty() && tuple_element_types.iter().all(|t| t == &tuple_element_types[0]) {
                            // All tuple elements have the same type, use that type
                            tuple_element_types[0].clone()
                        } else {
                            // Keep the original type
                            element_type_ref.clone()
                        }
                    },
                    _ => element_type_ref.clone()
                };

                let llvm_type = self.get_llvm_type(&actual_element_type);
                let item_val = self.builder.build_load(llvm_type, item_ptr, "list_item_load").unwrap();

                // Return the item and its type
                Ok((item_val, actual_element_type))
            },
            Type::Dict(key_type, value_type) => {
                // Special case for Unknown key type - allow any index type
                if matches!(**key_type, Type::Unknown) {
                    println!("Dictionary access with Unknown key type, allowing index type: {:?}", index_type);
                }
                // For dictionaries, check if the key type is compatible
                else if !index_type.can_coerce_to(key_type) && !matches!(index_type, Type::String) {
                    return Err(format!("Dictionary key type mismatch: expected {:?}, got {:?}", key_type, index_type));
                }

                // Get the value from the dictionary
                let value_ptr = self.build_dict_get_item(
                    value_val.into_pointer_value(),
                    index_val,
                    &index_type
                )?;

                // Return the value and its type
                Ok((value_ptr.into(), value_type.as_ref().clone()))
            },
            Type::String => {
                // For strings, we need an integer index
                if !index_type.can_coerce_to(&Type::Int) {
                    return Err(format!("String index must be an integer, got {:?}", index_type));
                }

                // Convert index to integer if necessary
                let index_int = if index_type != Type::Int {
                    self.convert_type(index_val, &index_type, &Type::Int)?.into_int_value()
                } else {
                    index_val.into_int_value()
                };

                // Get the character from the string
                let char_val = self.build_string_get_char(
                    value_val.into_pointer_value(),
                    index_int
                )?;

                // Return the character as a string (not an integer)
                // This ensures that string indexing returns a string, not an integer
                Ok((char_val, Type::String))
            },
            Type::Tuple(element_types) => {
                // For tuples, we need an integer index
                if !index_type.can_coerce_to(&Type::Int) {
                    return Err(format!("Tuple index must be an integer, got {:?}", index_type));
                }

                // Handle constant index case (most common)
                if let Expr::Num { value: Number::Integer(idx), .. } = slice {
                    let idx = *idx as usize;

                    // Check if the index is in bounds
                    if idx >= element_types.len() {
                        return Err(format!("Tuple index out of range: {} (tuple has {} elements)", idx, element_types.len()));
                    }

                    // Get the tuple struct type
                    let llvm_types: Vec<BasicTypeEnum> = element_types
                        .iter()
                        .map(|ty| self.get_llvm_type(ty))
                        .collect();

                    let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

                    // Get a pointer to the tuple
                    let tuple_ptr = if value_val.is_pointer_value() {
                        value_val.into_pointer_value()
                    } else {
                        // If the value is not a pointer, allocate memory for it
                        let llvm_type = self.get_llvm_type(&value_type);
                        let alloca = self.builder.build_alloca(llvm_type, "tuple_temp").unwrap();
                        self.builder.build_store(alloca, value_val).unwrap();
                        alloca
                    };

                    // Get a pointer to the indexed element
                    let element_ptr = self.builder.build_struct_gep(
                        tuple_struct,
                        tuple_ptr,
                        idx as u32,
                        &format!("tuple_element_{}", idx)
                    ).unwrap();

                    // Load the element
                    let element_type = &element_types[idx];
                    let element_val = self.builder.build_load(
                        self.get_llvm_type(element_type),
                        element_ptr,
                        &format!("load_tuple_element_{}", idx)
                    ).unwrap();

                    // Return the element and its type
                    return Ok((element_val, element_type.clone()));
                }

                // Handle dynamic index case - when index is not a constant
                // Convert index to integer
                let index_int = if index_type != Type::Int {
                    self.convert_type(index_val, &index_type, &Type::Int)?.into_int_value()
                } else {
                    index_val.into_int_value()
                };

                // Implement a switch-based approach for dynamic indexing
                self.handle_tuple_dynamic_index(value_val, value_type.clone(), index_int, element_types)
            },
            Type::Int => {
                // For integers, we need an integer index
                if !index_type.can_coerce_to(&Type::Int) {
                    return Err(format!("Integer index must be an integer, got {:?}", index_type));
                }

                // Convert index to integer if necessary
                let index_int = if index_type != Type::Int {
                    self.convert_type(index_val, &index_type, &Type::Int)?.into_int_value()
                } else {
                    index_val.into_int_value()
                };

                // Convert the integer to a string character
                // Get the int_to_string function
                let int_to_string_fn = match self.module.get_function("int_to_string") {
                    Some(f) => f,
                    None => return Err("int_to_string function not found".to_string()),
                };

                // Call int_to_string to convert the integer to a string
                let call_site_value = self.builder.build_call(
                    int_to_string_fn,
                    &[index_int.into()],
                    "int_to_string_result"
                ).unwrap();

                // Get the result as a string pointer
                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to convert integer to string".to_string())?;

                Ok((result, Type::String))
            },
            // Other types like Bytes, Set could be added here
            _ => Err(format!("Type {:?} is not indexable", value_type)),
        };

        // Ensure the current block has a terminator before returning
        self.ensure_block_has_terminator();

        result
    }

    // Helper method to handle dynamic tuple indexing without recursion
    fn handle_tuple_dynamic_index(
        &mut self,
        tuple_val: BasicValueEnum<'ctx>,
        tuple_type: Type,
        index_val: inkwell::values::IntValue<'ctx>,
        element_types: &[Type]
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // If there's only one element type, we can just return that
        if element_types.len() == 1 {
            let element_type = &element_types[0];

            // Get the tuple struct type
            let tuple_struct = self.llvm_context.struct_type(
                &[self.get_llvm_type(element_type)],
                false
            );

            // Get a pointer to the tuple
            let tuple_ptr = if tuple_val.is_pointer_value() {
                tuple_val.into_pointer_value()
            } else {
                // If not already a pointer, create a temporary
                let llvm_type = self.get_llvm_type(&tuple_type);
                let alloca = self.builder.build_alloca(llvm_type, "tuple_temp").unwrap();
                self.builder.build_store(alloca, tuple_val).unwrap();
                alloca
            };

            // Get a pointer to the first element
            let element_ptr = self.builder.build_struct_gep(
                tuple_struct,
                tuple_ptr,
                0,
                "tuple_element_0"
            ).unwrap();

            // Load the element
            let element_val = self.builder.build_load(
                self.get_llvm_type(element_type),
                element_ptr,
                "load_tuple_element_0"
            ).unwrap();

            return Ok((element_val, element_type.clone()));
        }

        // For multiple element types, use a switch-based approach
        let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();

        // Create a default block for out-of-bounds indices
        let default_block = self.llvm_context.append_basic_block(current_function, "tuple_index_default");

        // Create a merge block for all paths to converge
        let merge_block = self.llvm_context.append_basic_block(current_function, "tuple_index_merge");

        // Create blocks for each possible index value
        let mut case_blocks = Vec::with_capacity(element_types.len());
        for i in 0..element_types.len() {
            case_blocks.push(self.llvm_context.append_basic_block(current_function, &format!("tuple_index_{}", i)));
        }

        // Create a switch instruction to branch based on the index value
        let _switch = self.builder.build_switch(index_val, default_block, &case_blocks.iter().enumerate()
            .map(|(i, block)| (self.llvm_context.i64_type().const_int(i as u64, false), *block))
            .collect::<Vec<_>>()).unwrap();

        // Get the LLVM tuple type
        let llvm_types: Vec<BasicTypeEnum> = element_types.iter()
            .map(|ty| self.get_llvm_type(ty))
            .collect();

        let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

        // Get a pointer to the tuple
        let tuple_ptr = if tuple_val.is_pointer_value() {
            tuple_val.into_pointer_value()
        } else {
            // If the value is not a pointer, allocate memory for it
            let llvm_type = self.get_llvm_type(&tuple_type);
            let alloca = self.builder.build_alloca(llvm_type, "tuple_temp").unwrap();
            self.builder.build_store(alloca, tuple_val).unwrap();
            alloca
        };

        // Create a phi node to collect the results from each case
        // We'll use a struct to store both the value and its type index
        let result_struct_type = self.llvm_context.struct_type(
            &[
                self.llvm_context.i64_type().into(), // Type index
                self.llvm_context.ptr_type(inkwell::AddressSpace::default()).into() // Value pointer
            ],
            false
        );

        // Allocate memory for the result struct
        let result_struct_ptr = self.builder.build_alloca(result_struct_type, "tuple_index_result_struct").unwrap();

        // Handle each possible index case
        for (i, &block) in case_blocks.iter().enumerate() {
            self.builder.position_at_end(block);

            // Access the i-th element
            let element_ptr = self.builder.build_struct_gep(
                tuple_struct,
                tuple_ptr,
                i as u32,
                &format!("tuple_element_{}", i)
            ).unwrap();

            // Load the element
            let element_type = &element_types[i];
            let element_val = self.builder.build_load(
                self.get_llvm_type(element_type),
                element_ptr,
                &format!("load_tuple_element_{}", i)
            ).unwrap();

            // Store the element in a temporary allocation
            let element_alloca = self.builder.build_alloca(
                element_val.get_type(),
                &format!("tuple_element_{}_alloca", i)
            ).unwrap();
            self.builder.build_store(element_alloca, element_val).unwrap();

            // Store the type index in the result struct
            let type_idx_ptr = self.builder.build_struct_gep(
                result_struct_type,
                result_struct_ptr,
                0,
                "type_idx_ptr"
            ).unwrap();
            self.builder.build_store(
                type_idx_ptr,
                self.llvm_context.i64_type().const_int(i as u64, false)
            ).unwrap();

            // Store the element pointer in the result struct
            let element_ptr_ptr = self.builder.build_struct_gep(
                result_struct_type,
                result_struct_ptr,
                1,
                "element_ptr_ptr"
            ).unwrap();
            self.builder.build_store(
                element_ptr_ptr,
                element_alloca
            ).unwrap();

            // Branch to the merge block
            self.builder.build_unconditional_branch(merge_block).unwrap();
        }

        // Handle out-of-bounds indexing in the default block
        self.builder.position_at_end(default_block);

        // For out-of-bounds, we'll use a default value (first element type)
        // Store -1 as the type index to indicate out-of-bounds
        let type_idx_ptr = self.builder.build_struct_gep(
            result_struct_type,
            result_struct_ptr,
            0,
            "type_idx_ptr_default"
        ).unwrap();
        self.builder.build_store(
            type_idx_ptr,
            self.llvm_context.i64_type().const_int(u64::MAX, true) // -1 as i64
        ).unwrap();

        // Store null as the element pointer
        let element_ptr_ptr = self.builder.build_struct_gep(
            result_struct_type,
            result_struct_ptr,
            1,
            "element_ptr_ptr_default"
        ).unwrap();
        self.builder.build_store(
            element_ptr_ptr,
            self.llvm_context.ptr_type(inkwell::AddressSpace::default()).const_null()
        ).unwrap();

        // Branch to the merge block
        self.builder.build_unconditional_branch(merge_block).unwrap();

        // Position at the merge block
        self.builder.position_at_end(merge_block);

        // Load the type index from the result struct
        let type_idx_ptr = self.builder.build_struct_gep(
            result_struct_type,
            result_struct_ptr,
            0,
            "type_idx_ptr_load"
        ).unwrap();
        let type_idx = self.builder.build_load(
            self.llvm_context.i64_type(),
            type_idx_ptr,
            "type_idx_load"
        ).unwrap().into_int_value();

        // Load the element pointer from the result struct
        let element_ptr_ptr = self.builder.build_struct_gep(
            result_struct_type,
            result_struct_ptr,
            1,
            "element_ptr_ptr_load"
        ).unwrap();
        let element_ptr = self.builder.build_load(
            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
            element_ptr_ptr,
            "element_ptr_load"
        ).unwrap().into_pointer_value();

        // Check if the index is out of bounds
        let is_out_of_bounds = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type_idx,
            self.llvm_context.i64_type().const_int(u64::MAX, true), // -1 as i64
            "is_out_of_bounds"
        ).unwrap();

        // Create blocks for handling out-of-bounds and in-bounds cases
        let out_of_bounds_block = self.llvm_context.append_basic_block(current_function, "out_of_bounds");
        let in_bounds_block = self.llvm_context.append_basic_block(current_function, "in_bounds");
        let final_block = self.llvm_context.append_basic_block(current_function, "final");

        // Branch based on whether the index is out of bounds
        self.builder.build_conditional_branch(is_out_of_bounds, out_of_bounds_block, in_bounds_block).unwrap();

        // Handle out-of-bounds case
        self.builder.position_at_end(out_of_bounds_block);
        // For out-of-bounds, we'll return a default value (first element type)
        let default_type = &element_types[0];
        let default_val = self.get_llvm_type(default_type).const_zero();
        self.builder.build_unconditional_branch(final_block).unwrap();
        let out_of_bounds_block = self.builder.get_insert_block().unwrap();

        // Handle in-bounds case
        self.builder.position_at_end(in_bounds_block);
        // For in-bounds, we'll load the element from the pointer
        // We need to determine the element type based on the type index
        // For simplicity, we'll just use the first element type for now
        let element_type = &element_types[0];
        let element_val = self.builder.build_load(
            self.get_llvm_type(element_type),
            element_ptr,
            "element_val_load"
        ).unwrap();
        self.builder.build_unconditional_branch(final_block).unwrap();
        let in_bounds_block = self.builder.get_insert_block().unwrap();

        // Final block to merge results
        self.builder.position_at_end(final_block);
        let phi = self.builder.build_phi(
            self.get_llvm_type(element_type),
            "tuple_index_result"
        ).unwrap();
        phi.add_incoming(&[
            (&default_val, out_of_bounds_block),
            (&element_val, in_bounds_block)
        ]);

        // Make sure the final block has a terminator
        if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
            // Add a branch to a continuation block
            let continue_block = self.llvm_context.append_basic_block(current_function, "continue_block");
            self.builder.build_unconditional_branch(continue_block).unwrap();
            self.builder.position_at_end(continue_block);
        }

        // Return the result with the most specific type possible
        // For mixed-type tuples, we need to return a union type
        // For now, we'll just return the first element type as a simplification
        Ok((phi.as_basic_value(), element_types[0].clone()))
    }

    fn build_empty_dict(&self, name: &str) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Get the dict_new function
        let dict_new_fn = match self.module.get_function("dict_new") {
            Some(f) => f,
            None => return Err("dict_new function not found".to_string()),
        };

        // Call dict_new to create an empty dictionary
        let call_site_value = self.builder.build_call(dict_new_fn, &[], name).unwrap();
        let dict_ptr = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to create empty dictionary".to_string())?;

        Ok(dict_ptr.into_pointer_value())
    }

    fn build_dict(
        &self,
        keys: Vec<BasicValueEnum<'ctx>>,
        values: Vec<BasicValueEnum<'ctx>>,
        key_type: &Type,
        value_type: &Type
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Create a dictionary with the given capacity
        let dict_with_capacity_fn = match self.module.get_function("dict_with_capacity") {
            Some(f) => f,
            None => return Err("dict_with_capacity function not found".to_string()),
        };

        // Get the length of the dictionary
        let len = keys.len() as u64;
        let len_value = self.llvm_context.i64_type().const_int(len, false);

        // Call dict_with_capacity to create a dictionary with the given capacity
        let call_site_value = self.builder.build_call(dict_with_capacity_fn, &[len_value.into()], "dict_with_capacity").unwrap();
        let dict_ptr = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to create dictionary with capacity".to_string())?;

        let dict_ptr = dict_ptr.into_pointer_value();

        // Add each key-value pair to the dictionary
        let dict_set_fn = match self.module.get_function("dict_set") {
            Some(f) => f,
            None => return Err("dict_set function not found".to_string()),
        };

        for (i, (key, value)) in keys.iter().zip(values.iter()).enumerate() {
            // Convert the key and value to pointers if needed
            let key_ptr = if crate::compiler::types::is_reference_type(key_type) {
                *key
            } else {
                // For non-reference types, we need to allocate memory and store the value
                let key_alloca = self.builder.build_alloca(
                    key.get_type(),
                    &format!("dict_key_{}", i)
                ).unwrap();
                self.builder.build_store(key_alloca, *key).unwrap();
                key_alloca.into()
            };

            let value_ptr = if crate::compiler::types::is_reference_type(value_type) {
                *value
            } else {
                // For non-reference types, we need to allocate memory and store the value
                let value_alloca = self.builder.build_alloca(
                    value.get_type(),
                    &format!("dict_value_{}", i)
                ).unwrap();
                self.builder.build_store(value_alloca, *value).unwrap();
                value_alloca.into()
            };

            // Call dict_set to add the key-value pair to the dictionary
            self.builder.build_call(
                dict_set_fn,
                &[dict_ptr.into(), key_ptr.into(), value_ptr.into()],
                &format!("dict_set_{}", i)
            ).unwrap();
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
        element_type: &Type
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let _ = elements;
        let _ = element_type;
        Err("Set operations require runtime support (not yet implemented)".to_string())
    }

    fn build_list_get_item(
        &self,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Ensure the current block has a terminator before we create new blocks
        self.ensure_block_has_terminator();

        // Get the list_get function
        let list_get_fn = match self.module.get_function("list_get") {
            Some(f) => f,
            None => return Err("list_get function not found".to_string()),
        };

        // Ensure the current block has a terminator before calling list_get
        self.ensure_block_has_terminator();

        // Call list_get to get an item from the list
        let call_site_value = self.builder.build_call(
            list_get_fn,
            &[list_ptr.into(), index.into()],
            "list_get"
        ).unwrap();

        let item_ptr = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get item from list".to_string())?;

        // Ensure the current block has a terminator before returning
        self.ensure_block_has_terminator();

        // Check if the item is a pointer value
        if item_ptr.is_pointer_value() {
            Ok(item_ptr.into_pointer_value())
        } else {
            // If it's not a pointer value, allocate memory for it and store it
            let item_alloca = self.builder.build_alloca(
                item_ptr.get_type(),
                "list_item_alloca"
            ).unwrap();
            self.builder.build_store(item_alloca, item_ptr).unwrap();
            Ok(item_alloca)
        }
    }

    fn build_list_slice(
        &self,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        start: inkwell::values::IntValue<'ctx>,
        stop: inkwell::values::IntValue<'ctx>,
        step: inkwell::values::IntValue<'ctx>
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Get the list_slice function
        let list_slice_fn = match self.module.get_function("list_slice") {
            Some(f) => f,
            None => return Err("list_slice function not found".to_string()),
        };

        // Call list_slice to get a slice from the list
        let call_site_value = self.builder.build_call(
            list_slice_fn,
            &[list_ptr.into(), start.into(), stop.into(), step.into()],
            "list_slice"
        ).unwrap();

        let slice_ptr = call_site_value.try_as_basic_value().left()
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
        step: Option<&Expr>
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        self.compile_slice_operation_non_recursive(value_val, value_type, lower, upper, step)
    }

    fn compile_slice_operation_non_recursive(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
        lower: Option<&Expr>,
        upper: Option<&Expr>,
        step: Option<&Expr>
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Ensure the current block has a terminator before we create new blocks
        self.ensure_block_has_terminator();

        // Only certain types support slicing
        match &value_type {
            Type::List(element_type) => {
                // Get the list length
                let list_len_fn = match self.module.get_function("list_len") {
                    Some(f) => f,
                    None => return Err("list_len function not found".to_string()),
                };

                let list_ptr = value_val.into_pointer_value();
                let list_len_call = self.builder.build_call(
                    list_len_fn,
                    &[list_ptr.into()],
                    "list_len_result"
                ).unwrap();

                let list_len = list_len_call.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to get list length".to_string())?;

                let list_len_int = list_len.into_int_value();

                // Compile the slice bounds without recursion
                let i64_type = self.llvm_context.i64_type();

                // Ensure the current block has a terminator before compiling the start index
                self.ensure_block_has_terminator();

                // Handle start index (default = 0)
                let start_val = match lower {
                    Some(expr) => {
                        // Use non-recursive implementation
                        use crate::compiler::expr_non_recursive::ExprNonRecursive;
                        let (start_val, start_type) = self.compile_expr_non_recursive(expr)?;
                        if !start_type.can_coerce_to(&Type::Int) {
                            return Err(format!("Slice start index must be an integer, got {:?}", start_type));
                        }

                        // Ensure the current block has a terminator after compiling the start index
                        self.ensure_block_has_terminator();

                        // Convert to integer if needed
                        if start_type != Type::Int {
                            self.convert_type(start_val, &start_type, &Type::Int)?.into_int_value()
                        } else {
                            start_val.into_int_value()
                        }
                    },
                    None => i64_type.const_int(0, false)
                };

                // Ensure the current block has a terminator before compiling the stop index
                self.ensure_block_has_terminator();

                // Handle stop index (default = list length)
                let stop_val = match upper {
                    Some(expr) => {
                        // Use non-recursive implementation
                        use crate::compiler::expr_non_recursive::ExprNonRecursive;
                        let (stop_val, stop_type) = self.compile_expr_non_recursive(expr)?;
                        if !stop_type.can_coerce_to(&Type::Int) {
                            return Err(format!("Slice stop index must be an integer, got {:?}", stop_type));
                        }

                        // Ensure the current block has a terminator after compiling the stop index
                        self.ensure_block_has_terminator();

                        // Convert to integer if needed
                        if stop_type != Type::Int {
                            self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                        } else {
                            stop_val.into_int_value()
                        }
                    },
                    None => list_len_int
                };

                // Ensure the current block has a terminator before compiling the step index
                self.ensure_block_has_terminator();

                // Handle step (default = 1)
                let step_val = match step {
                    Some(expr) => {
                        // Use non-recursive implementation
                        use crate::compiler::expr_non_recursive::ExprNonRecursive;
                        let (step_val, step_type) = self.compile_expr_non_recursive(expr)?;
                        if !step_type.can_coerce_to(&Type::Int) {
                            return Err(format!("Slice step must be an integer, got {:?}", step_type));
                        }

                        // Ensure the current block has a terminator after compiling the step index
                        self.ensure_block_has_terminator();

                        // Convert to integer if needed
                        if step_type != Type::Int {
                            self.convert_type(step_val, &step_type, &Type::Int)?.into_int_value()
                        } else {
                            step_val.into_int_value()
                        }
                    },
                    None => i64_type.const_int(1, false)
                };

                // Ensure the current block has a terminator before calling list_slice
                self.ensure_block_has_terminator();

                // Call the list_slice function without recursion
                let slice_ptr = self.build_list_slice(list_ptr, start_val, stop_val, step_val)?;

                // Ensure the current block has a terminator before returning
                self.ensure_block_has_terminator();

                // Return the slice and its type
                Ok((slice_ptr.into(), Type::List(element_type.clone())))
            },
            Type::String => {
                // Get the string length
                let string_len_fn = match self.module.get_function("string_len") {
                    Some(f) => f,
                    None => return Err("string_len function not found".to_string()),
                };

                let str_ptr = value_val.into_pointer_value();
                let string_len_call = self.builder.build_call(
                    string_len_fn,
                    &[str_ptr.into()],
                    "string_len_result"
                ).unwrap();

                let string_len = string_len_call.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to get string length".to_string())?;

                let string_len_int = string_len.into_int_value();

                // Compile the slice bounds without recursion
                let i64_type = self.llvm_context.i64_type();

                // Handle start index (default = 0)
                let start_val = match lower {
                    Some(expr) => {
                        // Use non-recursive implementation
                        use crate::compiler::expr_non_recursive::ExprNonRecursive;
                        let (start_val, start_type) = self.compile_expr_non_recursive(expr)?;
                        if !start_type.can_coerce_to(&Type::Int) {
                            return Err(format!("Slice start index must be an integer, got {:?}", start_type));
                        }

                        // Convert to integer if needed
                        if start_type != Type::Int {
                            self.convert_type(start_val, &start_type, &Type::Int)?.into_int_value()
                        } else {
                            start_val.into_int_value()
                        }
                    },
                    None => i64_type.const_int(0, false)
                };

                // Handle stop index (default = string length)
                let stop_val = match upper {
                    Some(expr) => {
                        // Use non-recursive implementation
                        use crate::compiler::expr_non_recursive::ExprNonRecursive;
                        let (stop_val, stop_type) = self.compile_expr_non_recursive(expr)?;
                        if !stop_type.can_coerce_to(&Type::Int) {
                            return Err(format!("Slice stop index must be an integer, got {:?}", stop_type));
                        }

                        // Convert to integer if needed
                        if stop_type != Type::Int {
                            self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                        } else {
                            stop_val.into_int_value()
                        }
                    },
                    None => string_len_int
                };

                // Handle step (default = 1)
                let step_val = match step {
                    Some(expr) => {
                        // Use non-recursive implementation
                        use crate::compiler::expr_non_recursive::ExprNonRecursive;
                        let (step_val, step_type) = self.compile_expr_non_recursive(expr)?;
                        if !step_type.can_coerce_to(&Type::Int) {
                            return Err(format!("Slice step must be an integer, got {:?}", step_type));
                        }

                        // Convert to integer if needed
                        if step_type != Type::Int {
                            self.convert_type(step_val, &step_type, &Type::Int)?.into_int_value()
                        } else {
                            step_val.into_int_value()
                        }
                    },
                    None => i64_type.const_int(1, false)
                };

                // Ensure the current block has a terminator before calling string_slice
                self.ensure_block_has_terminator();

                // Call the string_slice function without recursion
                let slice_ptr = self.build_string_slice(str_ptr, start_val, stop_val, step_val)?;

                // Ensure the current block has a terminator before returning
                self.ensure_block_has_terminator();

                // Return the slice and its type
                Ok((slice_ptr.into(), Type::String))
            },
            // Could add support for other sliceable types here (e.g., Bytes)
            _ => Err(format!("Type {:?} does not support slicing", value_type)),
        }
    }

    fn build_dict_get_item(
        &self,
        dict_ptr: inkwell::values::PointerValue<'ctx>,
        key: BasicValueEnum<'ctx>,
        key_type: &Type
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Ensure the current block has a terminator before we create new blocks
        self.ensure_block_has_terminator();

        // Get the dict_get function
        let dict_get_fn = match self.module.get_function("dict_get") {
            Some(f) => f,
            None => return Err("dict_get function not found".to_string()),
        };

        // Special handling for string keys
        let key_ptr = if matches!(key_type, Type::String) {
            // For string keys, we can use the pointer directly
            if key.is_pointer_value() {
                key
            } else {
                return Err(format!("Expected pointer value for string key"));
            }
        } else if crate::compiler::types::is_reference_type(key_type) {
            // For other reference types, use the pointer directly
            key
        } else {
            // For non-reference types, we need to allocate memory and store the value
            let key_alloca = self.builder.build_alloca(
                key.get_type(),
                "dict_key_temp"
            ).unwrap();
            self.builder.build_store(key_alloca, key).unwrap();
            key_alloca.into()
        };

        // Ensure the current block has a terminator before calling dict_get
        self.ensure_block_has_terminator();

        // Call dict_get to get the value from the dictionary
        let call_site_value = self.builder.build_call(
            dict_get_fn,
            &[dict_ptr.into(), key_ptr.into()],
            "dict_get_result"
        ).unwrap();

        let value_ptr = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get value from dictionary".to_string())?;

        // Ensure the current block has a terminator before returning
        self.ensure_block_has_terminator();

        Ok(value_ptr.into_pointer_value())
    }

    fn build_string_get_char(
        &self,
        str_ptr: inkwell::values::PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Ensure the current block has a terminator before we create new blocks
        self.ensure_block_has_terminator();

        // Get the string_get_char function
        let string_get_char_fn = match self.module.get_function("string_get_char") {
            Some(f) => f,
            None => return Err("string_get_char function not found".to_string()),
        };

        // Ensure the current block has a terminator before calling string_get_char
        self.ensure_block_has_terminator();

        // Call the string_get_char function to get the character as an integer
        let call_site_value = self.builder.build_call(
            string_get_char_fn,
            &[str_ptr.into(), index.into()],
            "string_get_char_result"
        ).unwrap();

        // Convert the result to an integer value
        let char_int = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get character from string".to_string())?;

        // Ensure the current block has a terminator before getting char_to_string
        self.ensure_block_has_terminator();

        // Get the char_to_string function
        let char_to_string_fn = match self.module.get_function("char_to_string") {
            Some(f) => f,
            None => {
                // If char_to_string doesn't exist, fall back to int_to_string
                let int_to_string_fn = match self.module.get_function("int_to_string") {
                    Some(f) => f,
                    None => return Err("int_to_string function not found".to_string()),
                };

                // Ensure the current block has a terminator before calling int_to_string
                self.ensure_block_has_terminator();

                // Call int_to_string to convert the character to a string
                let call_site_value = self.builder.build_call(
                    int_to_string_fn,
                    &[char_int.into()],
                    "int_to_string_result"
                ).unwrap();

                // Get the result as a string pointer
                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to convert character to string".to_string())?;

                // Ensure the current block has a terminator before returning
                self.ensure_block_has_terminator();

                return Ok(result);
            }
        };

        // Ensure the current block has a terminator before calling char_to_string
        self.ensure_block_has_terminator();

        // Call char_to_string to convert the character to a string
        let call_site_value = self.builder.build_call(
            char_to_string_fn,
            &[char_int.into()],
            "char_to_string_result"
        ).unwrap();

        // Get the result as a string pointer
        let result = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to convert character to string".to_string())?;

        // Ensure the current block has a terminator before returning
        self.ensure_block_has_terminator();

        Ok(result)
    }

    fn build_string_slice(
        &self,
        str_ptr: inkwell::values::PointerValue<'ctx>,
        start: inkwell::values::IntValue<'ctx>,
        stop: inkwell::values::IntValue<'ctx>,
        step: inkwell::values::IntValue<'ctx>
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        // Get the string_slice function
        let string_slice_fn = match self.module.get_function("string_slice") {
            Some(f) => f,
            None => return Err("string_slice function not found".to_string()),
        };

        // Call the string_slice function
        let call_site_value = self.builder.build_call(
            string_slice_fn,
            &[str_ptr.into(), start.into(), stop.into(), step.into()],
            "string_slice_result"
        ).unwrap();

        // Convert the result to a pointer value
        let result = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get slice from string".to_string())?;

        Ok(result.into_pointer_value())
    }

    fn compile_number(&mut self, num: &Number) -> Result<(BasicValueEnum<'ctx>, Type), String> {
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

    fn compile_name_constant(&mut self, constant: &NameConstant) -> Result<(BasicValueEnum<'ctx>, Type), String> {
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

    /// Compile a list comprehension expression
    fn compile_list_comprehension(&mut self, elt: &Expr, generators: &[crate::ast::Comprehension]) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Always use the non-recursive implementation to avoid stack overflow
        self.compile_list_comprehension_non_recursive(elt, generators)
    }

    fn compile_list_comprehension_non_recursive(
        &mut self,
        elt: &Expr,
        generators: &[crate::ast::Comprehension]
    ) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if generators.is_empty() {
            return Err("List comprehension must have at least one generator".to_string());
        }

        // Ensure the current block has a terminator before creating new blocks
        self.ensure_block_has_terminator();

        // Create an empty list to store the results
        let result_list = self.build_empty_list("list_comp_result")?;

        // Ensure the current block has a terminator after creating the empty list
        self.ensure_block_has_terminator();

        // Get the list_append function
        let list_append_fn = match self.module.get_function("list_append") {
            Some(f) => f,
            None => return Err("list_append function not found".to_string()),
        };

        // Create a new scope for the list comprehension
        self.scope_stack.push_scope(false, false, false);

        // We'll handle only the first generator for now
        // A full implementation would need to handle multiple generators
        let generator = &generators[0];

        // Ensure the current block has a terminator before compiling the iterable expression
        self.ensure_block_has_terminator();

        // Compile the iterable expression
        let (iter_val, iter_type_original) = self.compile_expr(&generator.iter)?;
        let iter_type = iter_type_original.clone();

        // Ensure the current block has a terminator after compiling the iterable expression
        self.ensure_block_has_terminator();

        // Special case for range function
        if let Expr::Call { func, .. } = &*generator.iter {
            if let Expr::Name { id, .. } = func.as_ref() {
                if id == "range" {
                    // Recompile the range function call to get the range size directly
                    // This avoids the issue with the range function returning a pointer to a list
                    let range_fn_name = match generator.iter.as_ref() {
                        Expr::Call { args, .. } => {
                            match args.len() {
                                1 => "range_1",
                                2 => "range_2",
                                3 => "range_3",
                                _ => return Err(format!("range() takes 1-3 arguments, got {}", args.len())),
                            }
                        },
                        _ => return Err("Expected range function call".to_string()),
                    };

                    // Get the range function
                    let range_fn = self.module.get_function(range_fn_name)
                        .ok_or_else(|| format!("{} function not found", range_fn_name))?;

                    // Compile the arguments
                    let args = match generator.iter.as_ref() {
                        Expr::Call { args, .. } => args,
                        _ => return Err("Expected range function call".to_string()),
                    };

                    let mut call_args = Vec::with_capacity(args.len());
                    for arg in args {
                        let (arg_val, _) = self.compile_expr(arg)?;
                        call_args.push(arg_val.into());
                    }

                    // Call the range function to get the range size
                    let call = self.builder.build_call(
                        range_fn,
                        &call_args,
                        &format!("call_{}", range_fn_name)
                    ).unwrap();

                    // Get the range size
                    let range_size = call.try_as_basic_value().left().ok_or("range call failed")?;

                    // Make sure the range size is an integer
                    if !range_size.is_int_value() {
                        return Err(format!("Expected integer value from range function, got {:?}", range_size));
                    }

                    let range_val = range_size.into_int_value();

                    // Create basic blocks for the loop
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let loop_entry_block = self.llvm_context.append_basic_block(current_function, "range_comp_entry");
                    let loop_body_block = self.llvm_context.append_basic_block(current_function, "range_comp_body");
                    let loop_exit_block = self.llvm_context.append_basic_block(current_function, "range_comp_exit");

                    // Create an index variable
                    let index_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), "range_comp_index").unwrap();
                    self.builder.build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false)).unwrap();

                    // Branch to the loop entry
                    self.builder.build_unconditional_branch(loop_entry_block).unwrap();

                    // Loop entry block - check if we've reached the end of the range
                    self.builder.position_at_end(loop_entry_block);
                    let current_index = self.builder.build_load(self.llvm_context.i64_type(), index_ptr, "current_index").unwrap().into_int_value();
                    let condition = self.builder.build_int_compare(
                        inkwell::IntPredicate::SLT,
                        current_index,
                        range_val,
                        "loop_condition"
                    ).unwrap();

                    self.builder.build_conditional_branch(condition, loop_body_block, loop_exit_block).unwrap();

                    // Loop body block - process the current index
                    self.builder.position_at_end(loop_body_block);

                    // Bind the target variable to the current index
                    if let Expr::Name { id, .. } = generator.target.as_ref() {
                        // Declare the target variable in the current scope
                        let index_alloca = self.builder.build_alloca(self.llvm_context.i64_type(), "range_index_alloca").unwrap();
                        self.builder.build_store(index_alloca, current_index).unwrap();
                        self.scope_stack.add_variable(id.to_string(), index_alloca, Type::Int);
                    } else {
                        return Err("Only simple variable targets are supported in list comprehensions".to_string());
                    }

                    // Check if all conditions (if clauses) are met
                    let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

                    // Create the element and append it to the result list if conditions are met
                    self.process_list_comprehension_element(
                        elt,
                        should_append,
                        result_list,
                        list_append_fn,
                        current_function
                    )?;

                    // Increment the index and continue to the next iteration
                    let next_index = self.builder.build_int_add(
                        current_index,
                        self.llvm_context.i64_type().const_int(1, false),
                        "next_index"
                    ).unwrap();
                    self.builder.build_store(index_ptr, next_index).unwrap();
                    self.builder.build_unconditional_branch(loop_entry_block).unwrap();

                    // Exit block
                    self.builder.position_at_end(loop_exit_block);

                    // Pop the scope for the list comprehension
                    self.scope_stack.pop_scope();

                    // Create a temporary scope to compile the element expression
                    self.scope_stack.push_scope(false, false, false);

                    // Bind the target variable to a dummy value for type inference
                    if let Expr::Name { id, .. } = generator.target.as_ref() {
                        // Create a dummy Int variable for range iteration
                        let dummy_type = Type::Int;

                        // Allocate space for the dummy variable
                        let dummy_alloca = self.builder.build_alloca(
                            self.get_llvm_type(&dummy_type),
                            id
                        ).unwrap();

                        // Add the variable to the scope
                        self.scope_stack.add_variable(id.to_string(), dummy_alloca, dummy_type);
                    }

                    // Compile the element expression to determine the element type
                    let (_, element_type) = self.compile_expr(elt)?;

                    // Pop the temporary scope
                    self.scope_stack.pop_scope();

                    // Return the result list with the correct element type
                    return Ok((result_list.into(), Type::List(Box::new(element_type))));
                }
            }
        }

        // Check if the iterator is a list literal
        if let Expr::List { elts, .. } = &*generator.iter {
            // Create a list from the literal
            println!("Creating list from literal for iteration");

            // Compile each element of the list
            let mut element_values = Vec::with_capacity(elts.len());
            let mut element_types = Vec::with_capacity(elts.len());

            for elt in elts {
                let (value, ty) = self.compile_expr(elt)?;
                element_values.push(value);
                element_types.push(ty.clone());
            }

            // Determine the common element type
            let element_type = if element_types.is_empty() {
                Type::Unknown
            } else {
                // Check if all elements are the same type
                let first_type = &element_types[0];
                let all_same = element_types.iter().all(|t| t == first_type);

                if all_same {
                    // If all elements are the same type, use that type
                    println!("All list elements have the same type: {:?}", first_type);
                    first_type.clone()
                } else {
                    // If elements have different types, find a common type
                    let mut common_type = element_types[0].clone();
                    for ty in &element_types[1..] {
                        common_type = match self.get_common_type(&common_type, ty) {
                            Ok(t) => t,
                            Err(_) => {
                                println!("Could not find common type between {:?} and {:?}, using Any", common_type, ty);
                                Type::Any
                            },
                        };
                    }
                    println!("List literal elements have different types, using common type: {:?}", common_type);
                    common_type
                }
            };

            // Build the list
            let list_ptr = self.build_list(element_values, &element_type)?;

            // Now handle the list iteration
            self.handle_list_iteration_for_comprehension(
                elt,
                generator,
                list_ptr,
                result_list,
                list_append_fn
            )?;

            // Pop the scope for the list comprehension
            self.scope_stack.pop_scope();

            // Return the result list with the correct element type
            let (_, element_type) = self.compile_expr(elt)?;
            return Ok((result_list.into(), Type::List(Box::new(element_type))));
        } else {
            // Handle different iterable types
            match iter_type {
                Type::List(_) => {
                    self.handle_list_iteration_for_comprehension(
                        elt,
                        generator,
                        iter_val.into_pointer_value(),
                        result_list,
                        list_append_fn
                    )?;
                },
                Type::Tuple(element_types) => {
                    // For tuples, we need to handle them directly
                    println!("Handling tuple iteration directly");

                    // Get the tuple elements
                    let tuple_ptr = iter_val.into_pointer_value();

                    // Create basic blocks for the loop
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let loop_entry_block = self.llvm_context.append_basic_block(current_function, "tuple_comp_entry");
                    let loop_body_block = self.llvm_context.append_basic_block(current_function, "tuple_comp_body");
                    let loop_exit_block = self.llvm_context.append_basic_block(current_function, "tuple_comp_exit");

                    // Create an index variable
                    let index_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), "tuple_comp_index").unwrap();
                    self.builder.build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false)).unwrap();

                    // Branch to the loop entry
                    self.builder.build_unconditional_branch(loop_entry_block).unwrap();

                    // Loop entry block - check if we've reached the end of the tuple
                    self.builder.position_at_end(loop_entry_block);
                    let current_index = self.builder.build_load(self.llvm_context.i64_type(), index_ptr, "current_index").unwrap().into_int_value();
                    let tuple_len = self.llvm_context.i64_type().const_int(element_types.len() as u64, false);
                    let condition = self.builder.build_int_compare(
                        inkwell::IntPredicate::SLT,
                        current_index,
                        tuple_len,
                        "loop_condition"
                    ).unwrap();

                    self.builder.build_conditional_branch(condition, loop_body_block, loop_exit_block).unwrap();

                    // Loop body block - get the current element and process it
                    self.builder.position_at_end(loop_body_block);

                    // Create a switch to handle different indices
                    let default_block = self.llvm_context.append_basic_block(current_function, "tuple_default");
                    let merge_block = self.llvm_context.append_basic_block(current_function, "tuple_merge");

                    // Create blocks for each tuple element
                    let mut case_blocks = Vec::with_capacity(element_types.len());
                    for i in 0..element_types.len() {
                        case_blocks.push(self.llvm_context.append_basic_block(current_function, &format!("tuple_case_{}", i)));
                    }

                    // Create a switch instruction
                    let _switch = self.builder.build_switch(
                        current_index,
                        default_block,
                        &case_blocks.iter().enumerate()
                            .map(|(i, block)| (self.llvm_context.i64_type().const_int(i as u64, false), *block))
                            .collect::<Vec<_>>()
                    ).unwrap();

                    // Get the LLVM tuple type
                    let llvm_types: Vec<BasicTypeEnum> = element_types.iter()
                        .map(|ty| self.get_llvm_type(ty))
                        .collect();

                    let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

                    // Handle each case
                    for (i, &block) in case_blocks.iter().enumerate() {
                        self.builder.position_at_end(block);

                        // Get the element from the tuple
                        let element_ptr = self.builder.build_struct_gep(
                            tuple_struct,
                            tuple_ptr,
                            i as u32,
                            &format!("tuple_element_{}", i)
                        ).unwrap();

                        // Load the element
                        let element_type = &element_types[i];
                        let element_val = self.builder.build_load(
                            self.get_llvm_type(element_type),
                            element_ptr,
                            &format!("load_tuple_element_{}", i)
                        ).unwrap();

                        // Allocate memory for the element
                        let element_alloca = self.builder.build_alloca(
                            element_val.get_type(),
                            &format!("tuple_element_alloca_{}", i)
                        ).unwrap();
                        self.builder.build_store(element_alloca, element_val).unwrap();

                        // Bind the target variable to the current element
                        if let Expr::Name { id, .. } = generator.target.as_ref() {
                            // Add the variable to the scope
                            self.scope_stack.add_variable(id.to_string(), element_alloca, element_type.clone());

                            // Check if all conditions are met
                            let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

                            // Process the element
                            self.process_list_comprehension_element(
                                elt,
                                should_append,
                                result_list,
                                list_append_fn,
                                current_function
                            )?;
                        } else {
                            return Err("Only simple variable targets are supported in list comprehensions".to_string());
                        }

                        // Branch to the merge block
                        self.builder.build_unconditional_branch(merge_block).unwrap();
                    }

                    // Default block - just branch to merge
                    self.builder.position_at_end(default_block);
                    self.builder.build_unconditional_branch(merge_block).unwrap();

                    // Merge block - increment index and continue
                    self.builder.position_at_end(merge_block);
                    let next_index = self.builder.build_int_add(
                        current_index,
                        self.llvm_context.i64_type().const_int(1, false),
                        "next_index"
                    ).unwrap();
                    self.builder.build_store(index_ptr, next_index).unwrap();
                    self.builder.build_unconditional_branch(loop_entry_block).unwrap();

                    // Exit block
                    self.builder.position_at_end(loop_exit_block);
                },
                Type::String => {
                    self.handle_string_iteration_for_comprehension(
                        elt,
                        generator,
                        iter_val.into_pointer_value(),
                        result_list,
                        list_append_fn
                    )?;
                },
                // For simplicity, treat other types like lists
                _ => {
                    self.handle_general_iteration_for_comprehension(
                        elt,
                        generator,
                        iter_val,
                        iter_type,
                        result_list,
                        list_append_fn
                    )?;
                }
            }
        }

        // Pop the scope for the list comprehension
        self.scope_stack.pop_scope();

        // Ensure the current block has a terminator before returning
        self.ensure_block_has_terminator();

        // Create a temporary scope to compile the element expression
        self.scope_stack.push_scope(false, false, false);

        // Bind the target variable to a dummy value for type inference
        if let Expr::Name { id, .. } = generator.target.as_ref() {
            // Create a dummy variable of the appropriate type
            let mut dummy_type = match &iter_type_original {
                Type::List(elem_type) => *elem_type.clone(),
                Type::String => Type::String,
                _ => Type::Int, // Default to Int for range and other types
            };

            // If the dummy type is a tuple, extract the element type if all elements are the same
            dummy_type = match &dummy_type {
                Type::Tuple(tuple_element_types) => {
                    if !tuple_element_types.is_empty() && tuple_element_types.iter().all(|t| t == &tuple_element_types[0]) {
                        // All tuple elements have the same type, use that type
                        tuple_element_types[0].clone()
                    } else {
                        // If tuple elements have different types, use Int as a fallback
                        Type::Int
                    }
                },
                _ => dummy_type
            };

            // Allocate space for the dummy variable
            let dummy_alloca = self.builder.build_alloca(
                self.get_llvm_type(&dummy_type),
                id
            ).unwrap();

            // Add the variable to the scope
            self.scope_stack.add_variable(id.to_string(), dummy_alloca, dummy_type);
        }

        // Compile the element expression to determine the element type
        let (_, element_type) = self.compile_expr(elt)?;

        // Pop the temporary scope
        self.scope_stack.pop_scope();

        // Return the result list with the correct element type
        Ok((result_list.into(), Type::List(Box::new(element_type))))
    }

    fn handle_range_list_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        range_val: BasicValueEnum<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>
    ) -> Result<(), String> {
        // For range, we need to create a loop from 0 to the range value
        // The range functions return an i64 value (the size of the range), not a pointer
        let range_val = if range_val.is_int_value() {
            range_val.into_int_value()
        } else {
            return Err(format!("Expected integer value from range function, got {:?}", range_val));
        };

        // Create basic blocks for the loop
        let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let loop_entry_block = self.llvm_context.append_basic_block(current_function, "range_comp_entry");
        let loop_body_block = self.llvm_context.append_basic_block(current_function, "range_comp_body");
        let loop_exit_block = self.llvm_context.append_basic_block(current_function, "range_comp_exit");

        // Create an index variable
        let index_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), "range_comp_index").unwrap();
        self.builder.build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false)).unwrap();

        // Branch to the loop entry
        self.builder.build_unconditional_branch(loop_entry_block).unwrap();

        // Loop entry block - check if we've reached the end of the range
        self.builder.position_at_end(loop_entry_block);
        let current_index = self.builder.build_load(self.llvm_context.i64_type(), index_ptr, "current_index").unwrap().into_int_value();
        let condition = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            current_index,
            range_val,
            "loop_condition"
        ).unwrap();

        self.builder.build_conditional_branch(condition, loop_body_block, loop_exit_block).unwrap();

        // Loop body block - process the current index
        self.builder.position_at_end(loop_body_block);

        // Bind the target variable to the current index
        if let Expr::Name { id, .. } = generator.target.as_ref() {
            // Declare the target variable in the current scope
            let index_alloca = self.builder.build_alloca(self.llvm_context.i64_type(), "range_index_alloca").unwrap();
            self.builder.build_store(index_alloca, current_index).unwrap();
            self.scope_stack.add_variable(id.to_string(), index_alloca, Type::Int);
        } else {
            return Err("Only simple variable targets are supported in list comprehensions".to_string());
        }

        // Check if all conditions (if clauses) are met
        let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

        // Create the element and append it to the result list if conditions are met
        self.process_list_comprehension_element(
            elt,
            should_append,
            result_list,
            list_append_fn,
            current_function
        )?;

        // Increment the index and continue to the next iteration
        let next_index = self.builder.build_int_add(
            current_index,
            self.llvm_context.i64_type().const_int(1, false),
            "next_index"
        ).unwrap();
        self.builder.build_store(index_ptr, next_index).unwrap();
        self.builder.build_unconditional_branch(loop_entry_block).unwrap();

        // Exit block
        self.builder.position_at_end(loop_exit_block);

        Ok(())
    }

    fn handle_list_iteration_for_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        list_ptr: inkwell::values::PointerValue<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>
    ) -> Result<(), String> {
        // Get the list length
        let list_len_fn = match self.module.get_function("list_len") {
            Some(f) => f,
            None => return Err("list_len function not found".to_string()),
        };

        let list_len_call = self.builder.build_call(
            list_len_fn,
            &[list_ptr.into()],
            "list_len_result"
        ).unwrap();

        let list_len = list_len_call.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get list length".to_string())?;

        // Create a loop to iterate over the list
        let list_get_fn = match self.module.get_function("list_get") {
            Some(f) => f,
            None => return Err("list_get function not found".to_string()),
        };

        // Create basic blocks for the loop
        let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let loop_entry_block = self.llvm_context.append_basic_block(current_function, "list_comp_entry");
        let loop_body_block = self.llvm_context.append_basic_block(current_function, "list_comp_body");
        let loop_exit_block = self.llvm_context.append_basic_block(current_function, "list_comp_exit");

        // Create an index variable
        let index_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), "list_comp_index").unwrap();
        self.builder.build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false)).unwrap();

        // Branch to the loop entry
        self.builder.build_unconditional_branch(loop_entry_block).unwrap();

        // Loop entry block - check if we've reached the end of the list
        self.builder.position_at_end(loop_entry_block);
        let current_index = self.builder.build_load(self.llvm_context.i64_type(), index_ptr, "current_index").unwrap().into_int_value();
        let condition = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            current_index,
            list_len.into_int_value(),
            "loop_condition"
        ).unwrap();

        self.builder.build_conditional_branch(condition, loop_body_block, loop_exit_block).unwrap();

        // Loop body block - get the current element and process it
        self.builder.position_at_end(loop_body_block);

        // Get the current element from the list
        let call_site_value = self.builder.build_call(
            list_get_fn,
            &[list_ptr.into(), current_index.into()],
            "list_get_result"
        ).unwrap();

        let element_ptr = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get list element".to_string())?;

        // Determine the element type based on the list type
        let element_type = match self.lookup_variable_type(&generator.iter.to_string()) {
            Some(Type::List(element_type)) => {
                // For list types, use the element type directly
                *element_type.clone()
            },
            _ => {
                // If we can't determine the type from the variable, try to infer it
                // Check if the iterator is a list literal
                if let Expr::List { elts, .. } = &*generator.iter {
                    if !elts.is_empty() {
                        // Compile the first element to determine its type
                        if let Ok((_, element_type)) = self.compile_expr(&elts[0]) {
                            // If the element type is a tuple, extract the element type if all elements are the same
                            match &element_type {
                                Type::Tuple(tuple_element_types) => {
                                    if !tuple_element_types.is_empty() && tuple_element_types.iter().all(|t| t == &tuple_element_types[0]) {
                                        // All tuple elements have the same type, use that type
                                        tuple_element_types[0].clone()
                                    } else {
                                        // If tuple elements have different types, use Int as a fallback
                                        Type::Int
                                    }
                                },
                                _ => element_type
                            }
                        } else {
                            // Default to Int if we can't determine the element type
                            Type::Int
                        }
                    } else {
                        // Empty list, default to Int
                        Type::Int
                    }
                } else {
                    // For now, default to Int which is common
                    Type::Int
                }
            }
        };

        // If the element type is a tuple, extract the element type if all elements are the same
        let element_type = match &element_type {
            Type::Tuple(tuple_element_types) => {
                if !tuple_element_types.is_empty() && tuple_element_types.iter().all(|t| t == &tuple_element_types[0]) {
                    // All tuple elements have the same type, use that type
                    tuple_element_types[0].clone()
                } else {
                    // If tuple elements have different types, use Int as a fallback
                    Type::Int
                }
            },
            _ => element_type
        };

        // Handle the target variable based on its type
        match generator.target.as_ref() {
            Expr::Name { id, .. } => {
                // Simple variable target - just bind the element directly
                println!("Setting list comprehension variable '{}' to type: {:?}", id, element_type);
                self.scope_stack.add_variable(id.to_string(), element_ptr.into_pointer_value(), element_type.clone());
            },
            Expr::Tuple { elts, .. } => {
                // Tuple unpacking - only supported for tuple elements
                if let Type::Tuple(tuple_element_types) = &element_type {
                    // Check if the number of elements match
                    if elts.len() != tuple_element_types.len() {
                        return Err(format!("Tuple unpacking mismatch: expected {} elements, got {}",
                                          elts.len(), tuple_element_types.len()));
                    }

                    // Get the LLVM tuple type
                    let llvm_types: Vec<BasicTypeEnum> = tuple_element_types.iter()
                        .map(|ty| self.get_llvm_type(ty))
                        .collect();

                    let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);
                    let tuple_ptr = element_ptr.into_pointer_value();

                    // Unpack each element and bind it to the corresponding target
                    for (i, target_elt) in elts.iter().enumerate() {
                        if let Expr::Name { id, .. } = &**target_elt {
                            // Get the element from the tuple
                            let element_ptr = self.builder.build_struct_gep(
                                tuple_struct,
                                tuple_ptr,
                                i as u32,
                                &format!("tuple_element_{}", i)
                            ).unwrap();

                            // Load the element
                            let element_type = &tuple_element_types[i];
                            let element_val = self.builder.build_load(
                                self.get_llvm_type(element_type),
                                element_ptr,
                                &format!("load_tuple_element_{}", i)
                            ).unwrap();

                            // Allocate memory for the element
                            let element_alloca = self.builder.build_alloca(
                                element_val.get_type(),
                                &format!("tuple_element_alloca_{}", i)
                            ).unwrap();
                            self.builder.build_store(element_alloca, element_val).unwrap();

                            // Add the variable to the scope
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

        // Check if all conditions (if clauses) are met
        let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

        // Create the element and append it to the result list if conditions are met
        self.process_list_comprehension_element(
            elt,
            should_append,
            result_list,
            list_append_fn,
            current_function
        )?;

        // Increment the index and continue to the next iteration
        let next_index = self.builder.build_int_add(
            current_index,
            self.llvm_context.i64_type().const_int(1, false),
            "next_index"
        ).unwrap();
        self.builder.build_store(index_ptr, next_index).unwrap();
        self.builder.build_unconditional_branch(loop_entry_block).unwrap();

        // Exit block
        self.builder.position_at_end(loop_exit_block);

        Ok(())
    }

    fn handle_string_iteration_for_comprehension(
        &mut self,
        elt: &Expr,
        generator: &crate::ast::Comprehension,
        str_ptr: inkwell::values::PointerValue<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>
    ) -> Result<(), String> {
        // Get the string length
        let string_len_fn = match self.module.get_function("string_len") {
            Some(f) => f,
            None => return Err("string_len function not found".to_string()),
        };

        let string_len_call = self.builder.build_call(
            string_len_fn,
            &[str_ptr.into()],
            "string_len_result"
        ).unwrap();

        let string_len = string_len_call.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get string length".to_string())?;

        // Create a loop to iterate over the string
        let string_get_fn = match self.module.get_function("string_get_char") {
            Some(f) => f,
            None => return Err("string_get_char function not found".to_string()),
        };

        // Create basic blocks for the loop
        let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let loop_entry_block = self.llvm_context.append_basic_block(current_function, "string_comp_entry");
        let loop_body_block = self.llvm_context.append_basic_block(current_function, "string_comp_body");
        let loop_exit_block = self.llvm_context.append_basic_block(current_function, "string_comp_exit");

        // Create an index variable
        let index_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), "string_comp_index").unwrap();
        self.builder.build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false)).unwrap();

        // Branch to the loop entry
        self.builder.build_unconditional_branch(loop_entry_block).unwrap();

        // Loop entry block - check if we've reached the end of the string
        self.builder.position_at_end(loop_entry_block);
        let current_index = self.builder.build_load(self.llvm_context.i64_type(), index_ptr, "current_index").unwrap().into_int_value();
        let condition = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            current_index,
            string_len.into_int_value(),
            "loop_condition"
        ).unwrap();

        self.builder.build_conditional_branch(condition, loop_body_block, loop_exit_block).unwrap();

        // Loop body block - get the current character and process it
        self.builder.position_at_end(loop_body_block);

        // Get the current character from the string
        let call_site_value = self.builder.build_call(
            string_get_fn,
            &[str_ptr.into(), current_index.into()],
            "string_get_result"
        ).unwrap();

        let char_val = call_site_value.try_as_basic_value().left()
            .ok_or_else(|| "Failed to get string character".to_string())?;

        // Allocate memory for the character
        let char_ptr = self.builder.build_alloca(char_val.get_type(), "char_ptr").unwrap();
        self.builder.build_store(char_ptr, char_val).unwrap();

        // Bind the target variable to the current character
        if let Expr::Name { id, .. } = generator.target.as_ref() {
            // Declare the target variable in the current scope
            self.scope_stack.add_variable(id.to_string(), char_ptr, Type::Int);
        } else {
            return Err("Only simple variable targets are supported in list comprehensions".to_string());
        }

        // Check if all conditions (if clauses) are met
        let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

        // Create the element and append it to the result list if conditions are met
        self.process_list_comprehension_element(
            elt,
            should_append,
            result_list,
            list_append_fn,
            current_function
        )?;

        // Increment the index and continue to the next iteration
        let next_index = self.builder.build_int_add(
            current_index,
            self.llvm_context.i64_type().const_int(1, false),
            "next_index"
        ).unwrap();
        self.builder.build_store(index_ptr, next_index).unwrap();
        self.builder.build_unconditional_branch(loop_entry_block).unwrap();

        // Exit block
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
        list_append_fn: inkwell::values::FunctionValue<'ctx>
    ) -> Result<(), String> {
        // Handle different types of iterables
        match &iter_type {
            Type::Tuple(element_types) => {
                // For tuples, we need to handle them directly
                println!("Handling tuple iteration directly in general handler");

                // Get the tuple elements
                let tuple_ptr = iter_val.into_pointer_value();

                // Create basic blocks for the loop
                let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();

                // For tuples, we'll treat the entire tuple as a single element
                // This is simpler and more reliable than trying to iterate through tuple elements
                if let Expr::Name { id, .. } = generator.target.as_ref() {
                    // Add the variable to the scope with the tuple type
                    println!("Setting tuple variable '{}' to type: {:?}", id, iter_type);
                    self.scope_stack.add_variable(id.to_string(), tuple_ptr, iter_type.clone());

                    // Check if all conditions are met
                    let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

                    // Process the element
                    self.process_list_comprehension_element(
                        elt,
                        should_append,
                        result_list,
                        list_append_fn,
                        current_function
                    )?;
                } else {
                    // Handle tuple unpacking if the target is a tuple pattern
                    if let Expr::Tuple { elts, .. } = generator.target.as_ref() {
                        // Check if the number of elements match
                        if elts.len() != element_types.len() {
                            return Err(format!("Tuple unpacking mismatch: expected {} elements, got {}",
                                              elts.len(), element_types.len()));
                        }

                        // Get the LLVM tuple type
                        let llvm_types: Vec<BasicTypeEnum> = element_types.iter()
                            .map(|ty| self.get_llvm_type(ty))
                            .collect();

                        let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

                        // Unpack each element and bind it to the corresponding target
                        for (i, target_elt) in elts.iter().enumerate() {
                            if let Expr::Name { id, .. } = &**target_elt {
                                // Get the element from the tuple
                                let element_ptr = self.builder.build_struct_gep(
                                    tuple_struct,
                                    tuple_ptr,
                                    i as u32,
                                    &format!("tuple_element_{}", i)
                                ).unwrap();

                                // Load the element
                                let element_type = &element_types[i];
                                let element_val = self.builder.build_load(
                                    self.get_llvm_type(element_type),
                                    element_ptr,
                                    &format!("load_tuple_element_{}", i)
                                ).unwrap();

                                // Allocate memory for the element
                                let element_alloca = self.builder.build_alloca(
                                    element_val.get_type(),
                                    &format!("tuple_element_alloca_{}", i)
                                ).unwrap();
                                self.builder.build_store(element_alloca, element_val).unwrap();

                                // Add the variable to the scope
                                println!("Setting unpacked tuple element '{}' to type: {:?}", id, element_type);
                                self.scope_stack.add_variable(id.to_string(), element_alloca, element_type.clone());
                            } else {
                                return Err("Only simple variable names are supported in tuple unpacking".to_string());
                            }
                        }

                        // Check if all conditions are met
                        let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

                        // Process the element
                        self.process_list_comprehension_element(
                            elt,
                            should_append,
                            result_list,
                            list_append_fn,
                            current_function
                        )?;
                    } else {
                        return Err("Only simple variable targets or tuple unpacking are supported in list comprehensions".to_string());
                    }
                }
            },
            _ => {
                // For simplicity, we'll just delegate to the list handling method
                // In a real implementation, we'd need to handle different iterable types
                if let Expr::Name { id, .. } = generator.target.as_ref() {
                    // Create a dummy variable for the target
                    let dummy_val = self.llvm_context.i64_type().const_int(0, false);
                    let dummy_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), id).unwrap();
                    self.builder.build_store(dummy_ptr, dummy_val).unwrap();

                    // Add the variable to the scope
                    self.scope_stack.add_variable(id.to_string(), dummy_ptr, Type::Int);

                    // Create basic blocks for a simple loop
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();

                    // Check if all conditions (if clauses) are met
                    let should_append = self.evaluate_comprehension_conditions(generator, current_function)?;

                    // Process the element once (as if the iterable had just one element)
                    self.process_list_comprehension_element(
                        elt,
                        should_append,
                        result_list,
                        list_append_fn,
                        current_function
                    )?;
                } else {
                    return Err("Only simple variable targets are supported in list comprehensions".to_string());
                }
            }
        }

        Ok(())
    }

    /// Evaluate all conditions (if clauses) in a comprehension
    fn evaluate_comprehension_conditions(
        &mut self,
        generator: &crate::ast::Comprehension,
        _current_function: inkwell::values::FunctionValue<'ctx>
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        // If there are no conditions, always append
        if generator.ifs.is_empty() {
            return Ok(self.llvm_context.bool_type().const_int(1, false));
        }

        // Start with true (1)
        let mut should_append = self.llvm_context.bool_type().const_int(1, false);

        // For each condition, create blocks and evaluate
        for if_expr in &generator.ifs {
            // Compile the condition non-recursively
            let (cond_val, cond_type) = self.compile_expr(if_expr)?;

            // Convert to boolean if needed
            let cond_bool = if cond_type != Type::Bool {
                match &cond_type {
                    // Special handling for tuples - always treat them as truthy
                    Type::Tuple(_) => {
                        println!("Treating tuple as truthy in comprehension condition");
                        self.llvm_context.bool_type().const_int(1, false)
                    },
                    // For other types, try to convert to bool
                    _ => match self.convert_type(cond_val, &cond_type, &Type::Bool) {
                        Ok(bool_val) => bool_val.into_int_value(),
                        Err(_) => {
                            // If conversion fails, just use the original value
                            // For safety, we'll treat non-zero as true
                            match cond_val {
                                BasicValueEnum::IntValue(i) => {
                                    let zero = self.llvm_context.i64_type().const_zero();
                                    self.builder.build_int_compare(
                                        inkwell::IntPredicate::NE,
                                        i,
                                        zero,
                                        "is_nonzero"
                                    ).unwrap()
                                },
                                BasicValueEnum::FloatValue(f) => {
                                    let zero = self.llvm_context.f64_type().const_float(0.0);
                                    self.builder.build_float_compare(
                                        inkwell::FloatPredicate::ONE,
                                        f,
                                        zero,
                                        "is_nonzero"
                                    ).unwrap()
                                },
                                BasicValueEnum::PointerValue(_) => {
                                    // For pointers (including tuples), we'll consider them truthy
                                    // Non-empty tuples are always truthy in Python
                                    println!("Treating pointer value as truthy in comprehension condition");
                                    self.llvm_context.bool_type().const_int(1, false)
                                },
                                _ => {
                                    println!("WARNING: Unknown value type in condition, treating as falsy");
                                    self.llvm_context.bool_type().const_int(0, false)
                                },
                            }
                        }
                    }
                }
            } else {
                cond_val.into_int_value()
            };

            // AND with the current condition
            should_append = self.builder.build_and(should_append, cond_bool, "if_condition").unwrap();
        }

        Ok(should_append)
    }

    fn process_list_comprehension_element(
        &mut self,
        elt: &Expr,
        should_append: inkwell::values::IntValue<'ctx>,
        result_list: inkwell::values::PointerValue<'ctx>,
        list_append_fn: inkwell::values::FunctionValue<'ctx>,
        current_function: inkwell::values::FunctionValue<'ctx>
    ) -> Result<(), String> {
        // Create blocks for the conditional append
        let then_block = self.llvm_context.append_basic_block(current_function, "comp_then");
        let continue_block = self.llvm_context.append_basic_block(current_function, "comp_continue");

        // Branch based on the conditions
        self.builder.build_conditional_branch(should_append, then_block, continue_block).unwrap();

        // Then block - compile the element expression and append to the result list
        self.builder.position_at_end(then_block);

        // Compile the element expression non-recursively
        use crate::compiler::expr_non_recursive::ExprNonRecursive;
        let (element_val, mut element_type) = self.compile_expr_non_recursive(elt)?;

        // If the element type is a tuple, extract the element type if all elements are the same
        element_type = match &element_type {
            Type::Tuple(tuple_element_types) => {
                if !tuple_element_types.is_empty() && tuple_element_types.iter().all(|t| t == &tuple_element_types[0]) {
                    // All tuple elements have the same type, use that type
                    tuple_element_types[0].clone()
                } else {
                    // Keep the original type
                    element_type
                }
            },
            _ => element_type
        };

        // Handle the element based on its type
        let element_ptr = match &element_type {
            // For tuple types, we need to handle them as-is without trying to extract elements
            Type::Tuple(_) => {
                // For tuples, we'll just allocate memory and store the value directly
                // without trying to extract elements
                if element_val.is_pointer_value() {
                    element_val.into_pointer_value()
                } else {
                    // If not a pointer, allocate memory
                    let element_alloca = self.builder.build_alloca(
                        element_val.get_type(),
                        "comp_element"
                    ).unwrap();
                    self.builder.build_store(element_alloca, element_val).unwrap();
                    element_alloca
                }
            },
            // For other types, use the normal handling
            _ => {
                if crate::compiler::types::is_reference_type(&element_type) {
                    if element_val.is_pointer_value() {
                        element_val.into_pointer_value()
                    } else {
                        // If not a pointer, allocate memory
                        let element_alloca = self.builder.build_alloca(
                            element_val.get_type(),
                            "comp_element"
                        ).unwrap();
                        self.builder.build_store(element_alloca, element_val).unwrap();
                        element_alloca
                    }
                } else {
                    // For non-reference types, we need to allocate memory and store the value
                    let element_alloca = self.builder.build_alloca(
                        element_val.get_type(),
                        "comp_element"
                    ).unwrap();
                    self.builder.build_store(element_alloca, element_val).unwrap();
                    element_alloca
                }
            }
        };

        // Append the element to the result list
        self.builder.build_call(
            list_append_fn,
            &[result_list.into(), element_ptr.into()],
            "list_append_result"
        ).unwrap();

        // Branch to the continue block
        self.builder.build_unconditional_branch(continue_block).unwrap();

        // Continue block
        self.builder.position_at_end(continue_block);

        Ok(())
    }


    /// Compile an attribute access expression (e.g., dict.keys())
    fn compile_attribute_access(&mut self, value: &Expr, attr: &str) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Always use the non-recursive implementation to avoid stack overflow
        self.compile_attribute_access_non_recursive(value, attr)
    }

    /// Non-recursive implementation of attribute access compilation
    fn compile_attribute_access_non_recursive(&mut self, value: &Expr, attr: &str) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Compile the value being accessed using the non-recursive implementation
        use crate::compiler::expr_non_recursive::ExprNonRecursive;
        let (value_val, value_type) = self.compile_expr_non_recursive(value)?;

        // Handle different types of attribute access
        match &value_type {
            Type::Dict(key_type, value_type) => {
                // Handle dictionary methods
                match attr {
                    "keys" => {
                        // Use the compile_dict_keys method
                        self.compile_dict_keys(value_val.into_pointer_value(), key_type)
                    },
                    "values" => {
                        // Use the compile_dict_values method
                        self.compile_dict_values(value_val.into_pointer_value(), value_type)
                    },
                    "items" => {
                        // Use the compile_dict_items method
                        self.compile_dict_items(value_val.into_pointer_value(), key_type, value_type)
                    },

                    _ => Err(format!("Unknown method '{}' for dictionary type", attr)),
                }
            },
            Type::Class { name, methods, fields, .. } => {
                // Handle class attributes
                if let Some(_method_type) = methods.get(attr) {
                    // Method access not yet implemented
                    Err(format!("Method access for class '{}' not yet implemented", name))
                } else if let Some(_field_type) = fields.get(attr) {
                    // Field access not yet implemented
                    Err(format!("Field access for class '{}' not yet implemented", name))
                } else {
                    Err(format!("Unknown attribute '{}' for class '{}'", attr, name))
                }
            },
            _ => Err(format!("Type {:?} does not support attribute access", value_type)),
        }
    }

    /// Compile a dictionary comprehension expression
    fn compile_dict_comprehension(&mut self, key: &Expr, value: &Expr, generators: &[crate::ast::Comprehension]) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        // Always use the non-recursive implementation to avoid stack overflow
        self.compile_dict_comprehension_non_recursive(key, value, generators)
    }

    /// Non-recursive implementation of dictionary comprehension compilation
    fn compile_dict_comprehension_non_recursive(&mut self, key: &Expr, value: &Expr, generators: &[crate::ast::Comprehension]) -> Result<(BasicValueEnum<'ctx>, Type), String> {
        if generators.is_empty() {
            return Err("Dictionary comprehension must have at least one generator".to_string());
        }

        // Create an empty dictionary to store the results
        let result_dict = self.build_empty_dict("dict_comp_result")?;

        // Get the dict_set function
        let dict_set_fn = match self.module.get_function("dict_set") {
            Some(f) => f,
            None => return Err("dict_set function not found".to_string()),
        };

        // Create a new scope for the dictionary comprehension
        self.scope_stack.push_scope(false, false, false);

        // Compile the first generator
        let generator = &generators[0];

        // Compile the iterable expression using the non-recursive implementation
        use crate::compiler::expr_non_recursive::ExprNonRecursive;
        let (iter_val, iter_type) = self.compile_expr_non_recursive(&generator.iter)?;

        // Special case for range function
        if let Expr::Call { func, .. } = &*generator.iter {
            if let Expr::Name { id, .. } = func.as_ref() {
                if id == "range" {
                    // Recompile the range function call to get the range size directly
                    // This avoids the issue with the range function returning a pointer to a list
                    let range_fn_name = match generator.iter.as_ref() {
                        Expr::Call { args, .. } => {
                            match args.len() {
                                1 => "range_1",
                                2 => "range_2",
                                3 => "range_3",
                                _ => return Err(format!("range() takes 1-3 arguments, got {}", args.len())),
                            }
                        },
                        _ => return Err("Expected range function call".to_string()),
                    };

                    // Get the range function
                    let range_fn = self.module.get_function(range_fn_name)
                        .ok_or_else(|| format!("{} function not found", range_fn_name))?;

                    // Compile the arguments
                    let args = match generator.iter.as_ref() {
                        Expr::Call { args, .. } => args,
                        _ => return Err("Expected range function call".to_string()),
                    };

                    let mut call_args = Vec::with_capacity(args.len());
                    for arg in args {
                        let (arg_val, _) = self.compile_expr_non_recursive(arg)?;
                        call_args.push(arg_val.into());
                    }

                    // Call the range function to get the range size
                    let call = self.builder.build_call(
                        range_fn,
                        &call_args,
                        &format!("call_{}", range_fn_name)
                    ).unwrap();

                    // Get the range size
                    let range_size = call.try_as_basic_value().left().ok_or("range call failed")?;

                    // Make sure the range size is an integer
                    if !range_size.is_int_value() {
                        return Err(format!("Expected integer value from range function, got {:?}", range_size));
                    }

                    let range_val = range_size.into_int_value();

                    // Create basic blocks for the loop
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let loop_entry_block = self.llvm_context.append_basic_block(current_function, "range_comp_entry");
                    let loop_body_block = self.llvm_context.append_basic_block(current_function, "range_comp_body");
                    let loop_exit_block = self.llvm_context.append_basic_block(current_function, "range_comp_exit");

                    // Create an index variable
                    let index_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), "range_index").unwrap();
                    self.builder.build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false)).unwrap();

                    // Branch to the loop entry
                    self.builder.build_unconditional_branch(loop_entry_block).unwrap();

                    // Loop entry block - check if we've reached the end of the range
                    self.builder.position_at_end(loop_entry_block);
                    let current_index = self.builder.build_load(self.llvm_context.i64_type(), index_ptr, "current_index").unwrap().into_int_value();
                    let cond = self.builder.build_int_compare(inkwell::IntPredicate::SLT, current_index, range_val, "range_cond").unwrap();
                    self.builder.build_conditional_branch(cond, loop_body_block, loop_exit_block).unwrap();

                    // Loop body block - set the target variable and evaluate the element
                    self.builder.position_at_end(loop_body_block);

                    // Set the target variable to the current index
                    match &*generator.target {
                        Expr::Name { id, .. } => {
                            // Allocate space for the target variable
                            let target_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), id).unwrap();
                            self.builder.build_store(target_ptr, current_index).unwrap();

                            // Register the variable in the current scope
                            self.scope_stack.add_variable(id.clone(), target_ptr, Type::Int);

                            // Check if conditions
                            let mut continue_block = loop_body_block;
                            let mut condition_blocks = Vec::new();

                            for if_expr in &generator.ifs {
                                let if_block = self.llvm_context.append_basic_block(current_function, "if_block");
                                condition_blocks.push(if_block);

                                // Compile the condition using the non-recursive implementation
                                let (cond_val, _) = self.compile_expr_non_recursive(if_expr)?;
                                let cond_val = self.builder.build_int_truncate_or_bit_cast(cond_val.into_int_value(), self.llvm_context.bool_type(), "cond").unwrap();

                                // Branch based on the condition
                                self.builder.build_conditional_branch(cond_val, if_block, continue_block).unwrap();

                                // Position at the if block
                                self.builder.position_at_end(if_block);
                                continue_block = if_block;
                            }

                            // Compile the key and value expressions using the non-recursive implementation
                            let (key_val, key_type) = self.compile_expr_non_recursive(key)?;
                            let (value_val, value_type) = self.compile_expr_non_recursive(value)?;

                            // Convert the key and value to the appropriate types for dict_set
                            // For dictionary comprehensions, we don't need to convert Int to Any
                            // We'll just use the key as-is
                            let key_ptr = if crate::compiler::types::is_reference_type(&key_type) {
                                // For reference types, use the pointer directly
                                if key_val.is_pointer_value() {
                                    key_val.into_pointer_value()
                                } else {
                                    return Err(format!("Expected pointer value for key of type {:?}", key_type));
                                }
                            } else {
                                // For non-reference types, we need to allocate memory and store the value
                                let key_alloca = self.builder.build_alloca(
                                    key_val.get_type(),
                                    "dict_comp_key"
                                ).unwrap();
                                self.builder.build_store(key_alloca, key_val).unwrap();
                                key_alloca
                            };

                            // Convert the value to the appropriate type for dict_set
                            let value_ptr = if crate::compiler::types::is_reference_type(&value_type) {
                                // For reference types, use the pointer directly
                                if value_val.is_pointer_value() {
                                    value_val.into_pointer_value()
                                } else {
                                    return Err(format!("Expected pointer value for value of type {:?}", value_type));
                                }
                            } else {
                                // For non-reference types, we need to allocate memory and store the value
                                let value_alloca = self.builder.build_alloca(
                                    value_val.get_type(),
                                    "dict_comp_value"
                                ).unwrap();
                                self.builder.build_store(value_alloca, value_val).unwrap();
                                value_alloca
                            };

                            // Add the key-value pair to the dictionary
                            self.builder.build_call(
                                dict_set_fn,
                                &[
                                    result_dict.into(),
                                    key_ptr.into(),
                                    value_ptr.into(),
                                ],
                                "dict_set_result"
                            ).unwrap();

                            // Create a continue block
                            let continue_block = self.llvm_context.append_basic_block(current_function, "continue_block");
                            self.builder.build_unconditional_branch(continue_block).unwrap();

                            // Continue block - increment the index and continue the loop
                            self.builder.position_at_end(continue_block);

                            // Increment the index
                            let next_index = self.builder.build_int_add(
                                current_index,
                                self.llvm_context.i64_type().const_int(1, false),
                                "next_index"
                            ).unwrap();

                            self.builder.build_store(index_ptr, next_index).unwrap();

                            // Branch back to the loop entry
                            self.builder.build_unconditional_branch(loop_entry_block).unwrap();

                            // Exit block - return the result dictionary
                            self.builder.position_at_end(loop_exit_block);

                            // Pop the scope for the dictionary comprehension
                            self.scope_stack.pop_scope();

                            // Return the result dictionary
                            return Ok((result_dict.into(), Type::Dict(Box::new(key_type), Box::new(value_type))));
                        },
                        _ => return Err("Only simple variable names are supported as targets in dictionary comprehensions".to_string()),
                    }
                }
            }
        }

        // Check if the iterable is a list, string, or range
        match iter_type {
            Type::List(_) => {
                // Get the list length
                let list_len_fn = match self.module.get_function("list_len") {
                    Some(f) => f,
                    None => return Err("list_len function not found".to_string()),
                };

                let list_ptr = iter_val.into_pointer_value();
                let call_site_value = self.builder.build_call(
                    list_len_fn,
                    &[list_ptr.into()],
                    "list_len_result"
                ).unwrap();

                let list_len = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to get list length".to_string())?;

                // Create a loop to iterate over the list
                let list_get_fn = match self.module.get_function("list_get") {
                    Some(f) => f,
                    None => return Err("list_get function not found".to_string()),
                };

                // Create basic blocks for the loop
                let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                let loop_entry_block = self.llvm_context.append_basic_block(current_function, "list_comp_entry");
                let loop_body_block = self.llvm_context.append_basic_block(current_function, "list_comp_body");
                let loop_exit_block = self.llvm_context.append_basic_block(current_function, "list_comp_exit");

                // Create an index variable
                let index_ptr = self.builder.build_alloca(self.llvm_context.i64_type(), "list_index").unwrap();
                self.builder.build_store(index_ptr, self.llvm_context.i64_type().const_int(0, false)).unwrap();

                // Branch to the loop entry
                self.builder.build_unconditional_branch(loop_entry_block).unwrap();

                // Loop entry block - check if we've reached the end of the list
                self.builder.position_at_end(loop_entry_block);
                let current_index = self.builder.build_load(self.llvm_context.i64_type(), index_ptr, "current_index").unwrap().into_int_value();
                let cond = self.builder.build_int_compare(inkwell::IntPredicate::SLT, current_index, list_len.into_int_value(), "list_cond").unwrap();
                self.builder.build_conditional_branch(cond, loop_body_block, loop_exit_block).unwrap();

                // Loop body block - get the current element and evaluate the element expression
                self.builder.position_at_end(loop_body_block);

                // Get the current element from the list
                let call_site_value = self.builder.build_call(
                    list_get_fn,
                    &[list_ptr.into(), current_index.into()],
                    "list_get_result"
                ).unwrap();

                let element_val = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to get element from list".to_string())?;

                // Set the target variable to the current element
                match &*generator.target {
                    Expr::Name { id, .. } => {
                        // Determine the element type based on the list type
                        let mut element_type = if let Type::List(elem_type) = &iter_type {
                            *elem_type.clone()
                        } else {
                            Type::Any
                        };

                        // If the element type is a tuple, extract the element type if all elements are the same
                        element_type = match &element_type {
                            Type::Tuple(tuple_element_types) => {
                                if !tuple_element_types.is_empty() && tuple_element_types.iter().all(|t| t == &tuple_element_types[0]) {
                                    // All tuple elements have the same type, use that type
                                    tuple_element_types[0].clone()
                                } else {
                                    // Keep the original type
                                    element_type
                                }
                            },
                            _ => element_type
                        };

                        // Allocate space for the target variable
                        let target_ptr = match element_type {
                            Type::Int => self.builder.build_alloca(self.llvm_context.i64_type(), id).unwrap(),
                            Type::Float => self.builder.build_alloca(self.llvm_context.f64_type(), id).unwrap(),
                            Type::Bool => self.builder.build_alloca(self.llvm_context.bool_type(), id).unwrap(),
                            _ => self.builder.build_alloca(self.llvm_context.ptr_type(inkwell::AddressSpace::default()), id).unwrap(),
                        };

                        // Store the element in the target variable
                        self.builder.build_store(target_ptr, element_val).unwrap();

                        // Register the variable in the current scope
                        self.scope_stack.add_variable(id.clone(), target_ptr, element_type);

                        // Check if conditions
                        let mut continue_block = loop_body_block;
                        let mut condition_blocks = Vec::new();

                        for if_expr in &generator.ifs {
                            let if_block = self.llvm_context.append_basic_block(current_function, "if_block");
                            condition_blocks.push(if_block);

                            // Compile the condition using the non-recursive implementation
                            let (cond_val, _) = self.compile_expr_non_recursive(if_expr)?;
                            let cond_val = self.builder.build_int_truncate_or_bit_cast(cond_val.into_int_value(), self.llvm_context.bool_type(), "cond").unwrap();

                            // Branch based on the condition
                            self.builder.build_conditional_branch(cond_val, if_block, continue_block).unwrap();

                            // Position at the if block
                            self.builder.position_at_end(if_block);
                            continue_block = if_block;
                        }

                        // Compile the key and value expressions using the non-recursive implementation
                        let (key_val, key_type) = self.compile_expr_non_recursive(key)?;
                        let (value_val, value_type) = self.compile_expr_non_recursive(value)?;

                        // Convert the key and value to the appropriate types for dict_set
                        // For dictionary comprehensions, we don't need to convert Int to Any
                        // We'll just use the key as-is
                        let key_ptr = if crate::compiler::types::is_reference_type(&key_type) {
                            // For reference types, use the pointer directly
                            if key_val.is_pointer_value() {
                                key_val.into_pointer_value()
                            } else {
                                return Err(format!("Expected pointer value for key of type {:?}", key_type));
                            }
                        } else {
                            // For non-reference types, we need to allocate memory and store the value
                            let key_alloca = self.builder.build_alloca(
                                key_val.get_type(),
                                "dict_comp_key"
                            ).unwrap();
                            self.builder.build_store(key_alloca, key_val).unwrap();
                            key_alloca
                        };

                        // Convert the value to the appropriate type for dict_set
                        let value_ptr = if crate::compiler::types::is_reference_type(&value_type) {
                            // For reference types, use the pointer directly
                            if value_val.is_pointer_value() {
                                value_val.into_pointer_value()
                            } else {
                                return Err(format!("Expected pointer value for value of type {:?}", value_type));
                            }
                        } else {
                            // For non-reference types, we need to allocate memory and store the value
                            let value_alloca = self.builder.build_alloca(
                                value_val.get_type(),
                                "dict_comp_value"
                            ).unwrap();
                            self.builder.build_store(value_alloca, value_val).unwrap();
                            value_alloca
                        };

                        // Add the key-value pair to the dictionary
                        self.builder.build_call(
                            dict_set_fn,
                            &[
                                result_dict.into(),
                                key_ptr.into(),
                                value_ptr.into(),
                            ],
                            "dict_set_result"
                        ).unwrap();

                        // Create a continue block
                        let continue_block = self.llvm_context.append_basic_block(current_function, "continue_block");
                        self.builder.build_unconditional_branch(continue_block).unwrap();

                        // Continue block - increment the index and continue the loop
                        self.builder.position_at_end(continue_block);

                        // Increment the index
                        let next_index = self.builder.build_int_add(
                            current_index,
                            self.llvm_context.i64_type().const_int(1, false),
                            "next_index"
                        ).unwrap();

                        self.builder.build_store(index_ptr, next_index).unwrap();

                        // Branch back to the loop entry
                        self.builder.build_unconditional_branch(loop_entry_block).unwrap();

                        // Exit block - return the result dictionary
                        self.builder.position_at_end(loop_exit_block);

                        // Pop the scope for the dictionary comprehension
                        self.scope_stack.pop_scope();

                        // Return the result dictionary
                        return Ok((result_dict.into(), Type::Dict(Box::new(key_type), Box::new(value_type))));
                    },
                    _ => return Err("Only simple variable names are supported as targets in dictionary comprehensions".to_string()),
                }
            },
            _ => return Err(format!("Unsupported iterable type for dictionary comprehension: {:?}", iter_type)),
        }
    }
}



impl<'ctx> BinaryOpCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_binary_op(&mut self, left: inkwell::values::BasicValueEnum<'ctx>, left_type: &Type,
        op: Operator, right: inkwell::values::BasicValueEnum<'ctx>, right_type: &Type)
        -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String> {
        // Get the common type for this operation
        let common_type = self.get_common_type(left_type, right_type)?;

        // Convert operands to common type if needed
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

        // Perform the operation on converted values
        match op {
            Operator::Add => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self.builder.build_int_add(left_int, right_int, "int_add").unwrap();
                    Ok((result.into(), Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    let result = self.builder.build_float_add(left_float, right_float, "float_add").unwrap();
                    Ok((result.into(), Type::Float))
                },
                Type::String => {
                    // Get or create the string_concat function
                    let string_concat_fn = self.module.get_function("string_concat").unwrap_or_else(|| {
                        // Define the function signature: string_concat(string*, string*) -> string*
                        let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                        let fn_type = str_ptr_type.fn_type(&[str_ptr_type.into(), str_ptr_type.into()], false);
                        self.module.add_function("string_concat", fn_type, None)
                    });

                    // Build the function call
                    let left_ptr = left_converted.into_pointer_value();
                    let right_ptr = right_converted.into_pointer_value();
                    let result = self.builder.build_call(
                        string_concat_fn,
                        &[left_ptr.into(), right_ptr.into()],
                        "string_concat_result"
                    ).unwrap();

                    // Get the result value
                    if let Some(result_val) = result.try_as_basic_value().left() {
                        Ok((result_val, Type::String))
                    } else {
                        Err("Failed to concatenate strings".to_string())
                    }
                },
                Type::List(elem_type) => {
                    // Get the list_concat function
                    let list_concat_fn = match self.module.get_function("list_concat") {
                        Some(f) => f,
                        None => return Err("list_concat function not found".to_string()),
                    };

                    // Build the function call
                    let left_ptr = left_converted.into_pointer_value();
                    let right_ptr = right_converted.into_pointer_value();
                    let call_site_value = self.builder.build_call(
                        list_concat_fn,
                        &[left_ptr.into(), right_ptr.into()],
                        "list_concat_result"
                    ).unwrap();

                    if let Some(ret_val) = call_site_value.try_as_basic_value().left() {
                        Ok((ret_val, Type::List(elem_type.clone())))
                    } else {
                        Err("Failed to concatenate lists".to_string())
                    }
                },
                _ => Err(format!("Addition not supported for type {:?}", common_type)),
            },

            Operator::Sub => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self.builder.build_int_sub(left_int, right_int, "int_sub").unwrap();
                    Ok((result.into(), Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    let result = self.builder.build_float_sub(left_float, right_float, "float_sub").unwrap();
                    Ok((result.into(), Type::Float))
                },
                _ => Err(format!("Subtraction not supported for type {:?}", common_type)),
            },

            Operator::Mult => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self.builder.build_int_mul(left_int, right_int, "int_mul").unwrap();
                    Ok((result.into(), Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();
                    let result = self.builder.build_float_mul(left_float, right_float, "float_mul").unwrap();
                    Ok((result.into(), Type::Float))
                },
                Type::String => {
                    // String repetition (string * int)
                    if let Type::Int = *right_type {
                        // Get or create the string_repeat function
                        let string_repeat_fn = self.module.get_function("string_repeat").unwrap_or_else(|| {
                            // Define the function signature: string_repeat(string*, int) -> string*
                            let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                            let fn_type = str_ptr_type.fn_type(&[
                                str_ptr_type.into(),
                                self.llvm_context.i64_type().into()
                            ], false);
                            self.module.add_function("string_repeat", fn_type, None)
                        });

                        // Build the function call
                        let left_ptr = left_converted.into_pointer_value();
                        let right_int = right_converted.into_int_value();
                        let result = self.builder.build_call(
                            string_repeat_fn,
                            &[left_ptr.into(), right_int.into()],
                            "string_repeat_result"
                        ).unwrap();

                        // Get the result value
                        if let Some(result_val) = result.try_as_basic_value().left() {
                            return Ok((result_val, Type::String));
                        } else {
                            return Err("Failed to repeat string".to_string());
                        }
                    }
                    Err(format!("String repetition requires an integer, got {:?}", right_type))
                },
                Type::List(elem_type) => {
                    // List repetition (list * int)
                    if let Type::Int = right_type {
                        // Get the list_repeat function
                        let list_repeat_fn = match self.module.get_function("list_repeat") {
                            Some(f) => f,
                            None => return Err("list_repeat function not found".to_string()),
                        };

                        // Build the function call
                        let left_ptr = left_converted.into_pointer_value();
                        let right_int = right_converted.into_int_value();
                        let call_site_value = self.builder.build_call(
                            list_repeat_fn,
                            &[left_ptr.into(), right_int.into()],
                            "list_repeat_result"
                        ).unwrap();

                        if let Some(ret_val) = call_site_value.try_as_basic_value().left() {
                            return Ok((ret_val, Type::List(elem_type.clone())));
                        } else {
                            return Err("Failed to repeat list".to_string());
                        }
                    }
                    Err(format!("List repetition requires an integer, got {:?}", right_type))
                },
                _ => Err(format!("Multiplication not supported for type {:?}", common_type)),
            },

            Operator::Div => match common_type {
                Type::Int => {
                    // Convert to float for division to avoid integer division issues
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();

                    // Check for division by zero
                    let zero = self.llvm_context.i64_type().const_zero();
                    let is_zero = self.builder.build_int_compare(
                        inkwell::IntPredicate::EQ,
                        right_int,
                        zero,
                        "is_zero"
                    ).unwrap();

                    // Create basic blocks for division by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let div_bb = self.llvm_context.append_basic_block(current_function, "div");
                    let div_by_zero_bb = self.llvm_context.append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");

                    // Branch based on division by zero check
                    self.builder.build_conditional_branch(is_zero, div_by_zero_bb, div_bb).unwrap();

                    // Normal division block
                    self.builder.position_at_end(div_bb);
                    let left_float = self.builder.build_signed_int_to_float(left_int, self.llvm_context.f64_type(), "int_to_float").unwrap();
                    let right_float = self.builder.build_signed_int_to_float(right_int, self.llvm_context.f64_type(), "int_to_float").unwrap();
                    let div_result = self.builder.build_float_div(left_float, right_float, "float_div").unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();

                    // Division by zero error block
                    self.builder.position_at_end(div_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();

                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.f64_type(), "div_result").unwrap();
                    phi.add_incoming(&[(&div_result, div_bb), (&error_value, div_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Float))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();

                    // Check for division by zero
                    let zero = self.llvm_context.f64_type().const_float(0.0);
                    let is_zero = self.builder.build_float_compare(
                        inkwell::FloatPredicate::OEQ,
                        right_float,
                        zero,
                        "is_zero"
                    ).unwrap();

                    // Create basic blocks for division by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let div_bb = self.llvm_context.append_basic_block(current_function, "div");
                    let div_by_zero_bb = self.llvm_context.append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");

                    // Branch based on division by zero check
                    self.builder.build_conditional_branch(is_zero, div_by_zero_bb, div_bb).unwrap();

                    // Normal division block
                    self.builder.position_at_end(div_bb);
                    let div_result = self.builder.build_float_div(left_float, right_float, "float_div").unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();

                    // Division by zero error block
                    self.builder.position_at_end(div_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();

                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.f64_type(), "div_result").unwrap();
                    phi.add_incoming(&[(&div_result, div_bb), (&error_value, div_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Float))
                },
                _ => Err(format!("Division not supported for type {:?}", common_type)),
            },

            Operator::FloorDiv => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();

                    // Check for division by zero
                    let zero = self.llvm_context.i64_type().const_zero();
                    let is_zero = self.builder.build_int_compare(
                        inkwell::IntPredicate::EQ,
                        right_int,
                        zero,
                        "is_zero"
                    ).unwrap();

                    // Create basic blocks for division by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let div_bb = self.llvm_context.append_basic_block(current_function, "div");
                    let div_by_zero_bb = self.llvm_context.append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");

                    // Branch based on division by zero check
                    self.builder.build_conditional_branch(is_zero, div_by_zero_bb, div_bb).unwrap();

                    // Normal division block
                    self.builder.position_at_end(div_bb);
                    let div_result = self.builder.build_int_signed_div(left_int, right_int, "int_div").unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();

                    // Division by zero error block
                    self.builder.position_at_end(div_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.i64_type().const_zero();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();

                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.i64_type(), "div_result").unwrap();
                    phi.add_incoming(&[(&div_result, div_bb), (&error_value, div_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();

                    // Check for division by zero
                    let zero = self.llvm_context.f64_type().const_float(0.0);
                    let is_zero = self.builder.build_float_compare(
                        inkwell::FloatPredicate::OEQ,
                        right_float,
                        zero,
                        "is_zero"
                    ).unwrap();

                    // Create basic blocks for division by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let div_bb = self.llvm_context.append_basic_block(current_function, "div");
                    let div_by_zero_bb = self.llvm_context.append_basic_block(current_function, "div_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");

                    // Branch based on division by zero check
                    self.builder.build_conditional_branch(is_zero, div_by_zero_bb, div_bb).unwrap();

                    // Normal division block
                    self.builder.position_at_end(div_bb);
                    let div_result = self.builder.build_float_div(left_float, right_float, "float_div").unwrap();
                    let floor_result = self.builder.build_call(
                        self.module.get_function("llvm.floor.f64").unwrap_or_else(|| {
                            // Create floor function declaration if it doesn't exist
                            let f64_type = self.llvm_context.f64_type();
                            let function_type = f64_type.fn_type(&[f64_type.into()], false);
                            self.module.add_function("llvm.floor.f64", function_type, None)
                        }),
                        &[div_result.into()],
                        "floor_div"
                    ).unwrap();
                    let floor_result = floor_result.try_as_basic_value().left().unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_bb = self.builder.get_insert_block().unwrap();

                    // Division by zero error block
                    self.builder.position_at_end(div_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let div_by_zero_bb = self.builder.get_insert_block().unwrap();

                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.f64_type(), "div_result").unwrap();
                    phi.add_incoming(&[(&floor_result, div_bb), (&error_value, div_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Float))
                },
                _ => Err(format!("Floor division not supported for type {:?}", common_type)),
            },

            Operator::Mod => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();

                    // Check for modulo by zero
                    let zero = self.llvm_context.i64_type().const_zero();
                    let is_zero = self.builder.build_int_compare(
                        inkwell::IntPredicate::EQ,
                        right_int,
                        zero,
                        "is_zero"
                    ).unwrap();

                    // Create basic blocks for modulo by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let mod_bb = self.llvm_context.append_basic_block(current_function, "mod");
                    let mod_by_zero_bb = self.llvm_context.append_basic_block(current_function, "mod_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");

                    // Branch based on modulo by zero check
                    self.builder.build_conditional_branch(is_zero, mod_by_zero_bb, mod_bb).unwrap();

                    // Normal modulo block
                    self.builder.position_at_end(mod_bb);
                    let mod_result = self.builder.build_int_signed_rem(left_int, right_int, "int_mod").unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_bb = self.builder.get_insert_block().unwrap();

                    // Modulo by zero error block
                    self.builder.position_at_end(mod_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.i64_type().const_zero();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_by_zero_bb = self.builder.get_insert_block().unwrap();

                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.i64_type(), "mod_result").unwrap();
                    phi.add_incoming(&[(&mod_result, mod_bb), (&error_value, mod_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();

                    // Check for modulo by zero
                    let zero = self.llvm_context.f64_type().const_float(0.0);
                    let is_zero = self.builder.build_float_compare(
                        inkwell::FloatPredicate::OEQ,
                        right_float,
                        zero,
                        "is_zero"
                    ).unwrap();

                    // Create basic blocks for modulo by zero handling
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let mod_bb = self.llvm_context.append_basic_block(current_function, "mod");
                    let mod_by_zero_bb = self.llvm_context.append_basic_block(current_function, "mod_by_zero");
                    let cont_bb = self.llvm_context.append_basic_block(current_function, "cont");

                    // Branch based on modulo by zero check
                    self.builder.build_conditional_branch(is_zero, mod_by_zero_bb, mod_bb).unwrap();

                    // Normal modulo block
                    self.builder.position_at_end(mod_bb);
                    let mod_result = self.builder.build_call(
                        self.module.get_function("fmod").unwrap_or_else(|| {
                            // Create fmod function declaration if it doesn't exist
                            let f64_type = self.llvm_context.f64_type();
                            let function_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
                            self.module.add_function("fmod", function_type, None)
                        }),
                        &[left_float.into(), right_float.into()],
                        "float_mod"
                    ).unwrap();
                    let mod_result = mod_result.try_as_basic_value().left().unwrap();
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_bb = self.builder.get_insert_block().unwrap();

                    // Modulo by zero error block
                    self.builder.position_at_end(mod_by_zero_bb);
                    // In a real implementation, you would call a runtime error function here
                    let error_value = self.llvm_context.f64_type().const_float(f64::NAN);
                    self.builder.build_unconditional_branch(cont_bb).unwrap();
                    let mod_by_zero_bb = self.builder.get_insert_block().unwrap();

                    // Continuation block to merge results
                    self.builder.position_at_end(cont_bb);
                    let phi = self.builder.build_phi(self.llvm_context.f64_type(), "mod_result").unwrap();
                    phi.add_incoming(&[(&mod_result, mod_bb), (&error_value, mod_by_zero_bb)]);

                    Ok((phi.as_basic_value(), Type::Float))
                },
                _ => Err(format!("Modulo not supported for type {:?}", common_type)),
            },

            Operator::Pow => match common_type {
                Type::Int => {
                    // For integer exponentiation, we'll use a runtime function or convert to float
                    let left_float = self.convert_type(left_converted, &Type::Int, &Type::Float)?;
                    let right_float = self.convert_type(right_converted, &Type::Int, &Type::Float)?;

                    let pow_result = self.builder.build_call(
                        self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
                            // Create pow function declaration if it doesn't exist
                            let f64_type = self.llvm_context.f64_type();
                            let function_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
                            self.module.add_function("llvm.pow.f64", function_type, None)
                        }),
                        &[left_float.into_float_value().into(), right_float.into_float_value().into()],
                        "float_pow"
                    ).unwrap();

                    // Convert back to integer
                    let pow_float = pow_result.try_as_basic_value().left().unwrap();
                    let pow_int = self.convert_type(pow_float, &Type::Float, &Type::Int)?;

                    Ok((pow_int, Type::Int))
                },
                Type::Float => {
                    let left_float = left_converted.into_float_value();
                    let right_float = right_converted.into_float_value();

                    let pow_result = self.builder.build_call(
                        self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
                            // Create pow function declaration if it doesn't exist
                            let f64_type = self.llvm_context.f64_type();
                            let function_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
                            self.module.add_function("llvm.pow.f64", function_type, None)
                        }),
                        &[left_float.into(), right_float.into()],
                        "float_pow"
                    ).unwrap();

                    let pow_float = pow_result.try_as_basic_value().left().unwrap();

                    Ok((pow_float, Type::Float))
                },
                _ => Err(format!("Power operation not supported for type {:?}", common_type)),
            },

            // Bitwise operations
            Operator::BitOr => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self.builder.build_or(left_int, right_int, "int_or").unwrap();
                    Ok((result.into(), Type::Int))
                },
                _ => Err(format!("Bitwise OR not supported for type {:?}", common_type)),
            },

            Operator::BitXor => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self.builder.build_xor(left_int, right_int, "int_xor").unwrap();
                    Ok((result.into(), Type::Int))
                },
                _ => Err(format!("Bitwise XOR not supported for type {:?}", common_type)),
            },

            Operator::BitAnd => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self.builder.build_and(left_int, right_int, "int_and").unwrap();
                    Ok((result.into(), Type::Int))
                },
                _ => Err(format!("Bitwise AND not supported for type {:?}", common_type)),
            },

            // Shift operations
            Operator::LShift => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    let result = self.builder.build_left_shift(left_int, right_int, "int_lshift").unwrap();
                    Ok((result.into(), Type::Int))
                },
                _ => Err(format!("Left shift not supported for type {:?}", common_type)),
            },

            Operator::RShift => match common_type {
                Type::Int => {
                    let left_int = left_converted.into_int_value();
                    let right_int = right_converted.into_int_value();
                    // Use arithmetic right shift (preserves sign bit)
                    let result = self.builder.build_right_shift(left_int, right_int, true, "int_rshift").unwrap();
                    Ok((result.into(), Type::Int))
                },
                _ => Err(format!("Right shift not supported for type {:?}", common_type)),
            },

            // Matrix multiplication
            Operator::MatMult => {
                // Matrix multiplication requires runtime support or specialized libraries
                // For now, we'll just return an error
                Err("Matrix multiplication not yet implemented".to_string())
            },

            // All operators are now handled, but we'll keep this for future additions
            #[allow(unreachable_patterns)]
            _ => Err(format!("Binary operator {:?} not implemented", op)),
        }
    }
}

impl<'ctx> ComparisonCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_comparison(&mut self, left: inkwell::values::BasicValueEnum<'ctx>, left_type: &Type,
                        op: CmpOperator, right: inkwell::values::BasicValueEnum<'ctx>, right_type: &Type)
                        -> Result<(inkwell::values::BasicValueEnum<'ctx>, Type), String> {
        // Special cases for identity comparisons (is, is not)
        if matches!(op, CmpOperator::Is) || matches!(op, CmpOperator::IsNot) {
            // For reference types, compare pointers
            if is_reference_type(left_type) && is_reference_type(right_type) {
                let left_ptr = if left.is_pointer_value() {
                    left.into_pointer_value()
                } else {
                    // Convert to pointer value if needed
                    let left_as_ptr = self.builder.build_bit_cast(
                        left,
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "as_ptr"
                    ).unwrap();
                    left_as_ptr.into_pointer_value()
                };

                let right_ptr = if right.is_pointer_value() {
                    right.into_pointer_value()
                } else {
                    // Convert to pointer value if needed
                    let right_as_ptr = self.builder.build_bit_cast(
                        right,
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "as_ptr"
                    ).unwrap();
                    right_as_ptr.into_pointer_value()
                };

                // Convert pointers to integers for comparison
                let left_ptr_int = self.builder.build_ptr_to_int(
                    left_ptr,
                    self.llvm_context.i64_type(),
                    "ptr_as_int"
                ).unwrap();

                let right_ptr_int = self.builder.build_ptr_to_int(
                    right_ptr,
                    self.llvm_context.i64_type(),
                    "ptr_as_int"
                ).unwrap();

                let is_same = self.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    left_ptr_int,
                    right_ptr_int,
                    "is_same"
                ).unwrap();

                // For "is not", negate the result
                let result = if matches!(op, CmpOperator::IsNot) {
                    self.builder.build_not(is_same, "is_not_same").unwrap()
                } else {
                    is_same
                };

                return Ok((result.into(), Type::Bool));
            }

            // For primitive types, just compare values
            return self.compile_comparison(left, left_type,
                                          if matches!(op, CmpOperator::Is) { CmpOperator::Eq } else { CmpOperator::NotEq },
                                          right, right_type);
        }

        // Special cases for 'in' and 'not in' operators
        if matches!(op, CmpOperator::In) || matches!(op, CmpOperator::NotIn) {
            match right_type {
                // Dictionary membership testing
                Type::Dict(key_type, _) => {
                    // Check if the left type is compatible with the dictionary key type
                    if !left_type.can_coerce_to(key_type) {
                        return Err(format!("Type mismatch for 'in' operator: {:?} is not compatible with dictionary key type {:?}", left_type, key_type));
                    }

                    // Get the dict_contains function
                    let dict_contains_fn = match self.module.get_function("dict_contains") {
                        Some(f) => f,
                        None => return Err("dict_contains function not found".to_string()),
                    };

                    // Convert the key to a pointer if needed
                    let key_ptr = if crate::compiler::types::is_reference_type(left_type) {
                        // For reference types, use the pointer directly
                        if left.is_pointer_value() {
                            left.into_pointer_value()
                        } else {
                            return Err(format!("Expected pointer value for key of type {:?}", left_type));
                        }
                    } else {
                        // For non-reference types, we need to allocate memory and store the value
                        let key_alloca = self.builder.build_alloca(
                            left.get_type(),
                            "dict_key_temp"
                        ).unwrap();
                        self.builder.build_store(key_alloca, left).unwrap();
                        key_alloca
                    };

                    // Call dict_contains to check if the key exists in the dictionary
                    let call_site_value = self.builder.build_call(
                        dict_contains_fn,
                        &[
                            right.into_pointer_value().into(),
                            key_ptr.into(),
                        ],
                        "dict_contains_result"
                    ).unwrap();

                    // Get the result as a boolean value
                    let contains_result = call_site_value.try_as_basic_value().left()
                        .ok_or_else(|| "Failed to get result from dict_contains".to_string())?;

                    // Convert the i8 result to a boolean
                    let contains_bool = self.builder.build_int_compare(
                        inkwell::IntPredicate::NE,
                        contains_result.into_int_value(),
                        self.llvm_context.i8_type().const_int(0, false),
                        "contains_bool"
                    ).unwrap();

                    // If it's a 'not in' operator, negate the result
                    let result = if matches!(op, CmpOperator::NotIn) {
                        self.builder.build_not(contains_bool, "not_contains_bool").unwrap()
                    } else {
                        contains_bool
                    };

                    return Ok((result.into(), Type::Bool));
                },
                // List membership testing (already implemented)
                Type::List(_) => {
                    // Use the existing list_contains function
                    // This is a placeholder - implement list membership testing if needed
                    return Err(format!("'in' operator not yet implemented for lists"));
                },
                // String membership testing (already implemented)
                Type::String => {
                    // Use the existing string_contains function
                    // This is a placeholder - implement string membership testing if needed
                    return Err(format!("'in' operator not yet implemented for strings"));
                },
                // Other types
                _ => {
                    return Err(format!("'in' operator not supported for type {:?}", right_type));
                }
            }
        }

        // For regular comparisons, get the common type
        let common_type = self.get_common_type(left_type, right_type)?;

        // Convert operands to common type if needed
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

        // Perform the comparison on converted values
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
                    _ => return Err(format!("Comparison operator {:?} not supported for integers", op)),
                };

                let result = self.builder.build_int_compare(pred, left_int, right_int, "int_cmp").unwrap();
                Ok((result.into(), Type::Bool))
            },

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
                    _ => return Err(format!("Comparison operator {:?} not supported for floats", op)),
                };

                let result = self.builder.build_float_compare(pred, left_float, right_float, "float_cmp").unwrap();
                Ok((result.into(), Type::Bool))
            },

            Type::Bool => {
                let left_bool = left_converted.into_int_value();
                let right_bool = right_converted.into_int_value();

                let pred = match op {
                    CmpOperator::Eq => inkwell::IntPredicate::EQ,
                    CmpOperator::NotEq => inkwell::IntPredicate::NE,
                    _ => return Err(format!("Comparison operator {:?} not supported for booleans", op)),
                };

                let result = self.builder.build_int_compare(pred, left_bool, right_bool, "bool_cmp").unwrap();
                Ok((result.into(), Type::Bool))
            },

            Type::String => {
                // Get or create the string_equals function
                let string_equals_fn = self.module.get_function("string_equals").unwrap_or_else(|| {
                    // Define the function signature: string_equals(string*, string*) -> bool
                    let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                    let fn_type = self.llvm_context.bool_type().fn_type(&[str_ptr_type.into(), str_ptr_type.into()], false);
                    self.module.add_function("string_equals", fn_type, None)
                });

                // Build the function call
                let left_ptr = left_converted.into_pointer_value();
                let right_ptr = right_converted.into_pointer_value();
                let result = self.builder.build_call(
                    string_equals_fn,
                    &[left_ptr.into(), right_ptr.into()],
                    "string_equals_result"
                ).unwrap();

                // Get the result value
                if let Some(result_val) = result.try_as_basic_value().left() {
                    let bool_result = result_val.into_int_value();

                    // Apply the comparison operator
                    match op {
                        CmpOperator::Eq => Ok((bool_result.into(), Type::Bool)),
                        CmpOperator::NotEq => {
                            // Negate the result for not equal
                            let not_result = self.builder.build_not(bool_result, "string_not_equals").unwrap();
                            Ok((not_result.into(), Type::Bool))
                        },
                        _ => Err(format!("String comparison operator {:?} not supported", op)),
                    }
                } else {
                    Err("Failed to compare strings".to_string())
                }
            },

            _ => Err(format!("Comparison not supported for type {:?}", common_type)),
        }
    }
}

impl<'ctx> AssignmentCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_assignment(&mut self, target: &Expr, value: BasicValueEnum<'ctx>,
        value_type: &Type) -> Result<(), String> {
        match target {
            Expr::Tuple { elts, .. } => {
                // Handle tuple unpacking
                if let Type::Tuple(element_types) = value_type {
                    // Check if the number of elements match
                    if elts.len() != element_types.len() {
                        return Err(format!("Tuple unpacking mismatch: expected {} elements, got {}", elts.len(), element_types.len()));
                    }

                    // Get the tuple struct type
                    let llvm_types: Vec<BasicTypeEnum> = element_types
                        .iter()
                        .map(|ty| self.get_llvm_type(ty))
                        .collect();

                    let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);
                    // For tuple unpacking, we need to handle both pointer values and struct values
                    let tuple_ptr = if value.is_pointer_value() {
                        // If it's already a pointer, use it directly
                        value.into_pointer_value()
                    } else if value.is_struct_value() {
                        // If it's a struct value, we need to create a temporary variable to store it
                        let temp_ptr = self.builder.build_alloca(tuple_struct, "temp_tuple").unwrap();
                        self.builder.build_store(temp_ptr, value).unwrap();
                        temp_ptr
                    } else if value.is_int_value() {
                        // Special case for function return values that might be tuples
                        // Convert the integer value to a pointer
                        let ptr = self.builder.build_int_to_ptr(
                            value.into_int_value(),
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "tuple_ptr"
                        ).unwrap();

                        // Cast the pointer to the tuple struct type
                        self.builder.build_pointer_cast(
                            ptr,
                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                            "tuple_struct_ptr"
                        ).unwrap()
                    } else {
                        return Err(format!("Cannot unpack value of type {:?} - expected a tuple", value_type));
                    };

                    // Unpack each element and assign it to the corresponding target
                    for (i, target_expr) in elts.iter().enumerate() {
                        // Get a pointer to the i-th element of the tuple
                        let element_ptr = self.builder.build_struct_gep(tuple_struct, tuple_ptr, i as u32, &format!("tuple_element_{}", i)).unwrap();

                        // Load the element
                        let element_value = self.builder.build_load(self.get_llvm_type(&element_types[i]), element_ptr, &format!("load_tuple_element_{}", i)).unwrap();

                        // Handle variable registration and allocation for name expressions
                        if let Expr::Name { id, .. } = target_expr.as_ref() {
                            // Register the variable type in the type environment
                            self.register_variable(id.clone(), element_types[i].clone());

                            // Check if the variable already exists in the current scope
                            if self.get_variable_ptr(id).is_none() {
                                // Variable doesn't exist yet, allocate storage for it
                                let ptr = self.builder.build_alloca(
                                    self.get_llvm_type(&element_types[i]),
                                    id
                                ).unwrap();

                                // Store the element value in the allocated memory
                                self.builder.build_store(ptr, element_value).unwrap();

                                // Add the variable to the current scope
                                self.add_variable_to_scope(id.clone(), ptr, element_types[i].clone());

                                // Also add it to the variables map for backward compatibility
                                self.variables.insert(id.clone(), ptr);

                                // We've already handled the assignment, so continue to the next element
                                continue;
                            }
                        }

                        // For non-name expressions or existing variables, use the regular assignment
                        self.compile_assignment(target_expr, element_value, &element_types[i])?;
                    }

                    Ok(())
                } else {
                    Err(format!("Cannot unpack non-tuple value of type {:?}", value_type))
                }
            },
            Expr::Name { id, .. } => {
                // Check if the variable is declared as global in the current scope
                let is_global = if let Some(current_scope) = self.scope_stack.current_scope() {
                    current_scope.is_global(id)
                } else {
                    false
                };

                // Check if the variable is declared as nonlocal in the current scope
                let is_nonlocal = if let Some(current_scope) = self.scope_stack.current_scope() {
                    current_scope.is_nonlocal(id)
                } else {
                    false
                };

                // If the variable is nonlocal, first try to use our proxy system
                if is_nonlocal {
                    // Check if we have a current environment
                    if let Some(env_name) = &self.current_environment {
                        if let Some(env) = self.get_closure_environment(env_name) {
                            // Check if we have a proxy for this nonlocal variable
                            if let Some(proxy_ptr) = env.get_nonlocal_proxy(id) {
                                // Store the value in the proxy
                                self.builder.build_store(*proxy_ptr, value).unwrap();
                                println!("Assigned to nonlocal variable '{}' using proxy in environment {}", id, env_name);
                                return Ok(());
                            }
                        }
                    }

                    // Fall back to the old method if we don't have a proxy
                    if let Some(current_scope) = self.scope_stack.current_scope() {
                        if let Some(unique_name) = current_scope.get_nonlocal_mapping(id) {
                            // Use the unique name instead of the original name
                            if let Some(ptr) = current_scope.get_variable(unique_name).cloned() {
                                // First, store the value directly without using the helper method
                                // This avoids the mutable borrow issue
                                self.builder.build_store(ptr, value).unwrap();
                                println!("Assigned to nonlocal variable '{}' using unique name '{}'", id, unique_name);
                                return Ok(());
                            }
                        }

                        // Special handling for shadowing cases
                        // If we're in a nested function and the nonlocal variable isn't found in the current scope,
                        // look for it in the parent scope
                        if self.scope_stack.scopes.len() >= 2 {
                            let parent_scope_index = self.scope_stack.scopes.len() - 2;

                            // First, check if the variable exists in the parent scope
                            let parent_var_ptr = self.scope_stack.scopes[parent_scope_index].get_variable(id).cloned();

                            if let Some(_ptr) = parent_var_ptr {
                                // For shadowing, we don't need a unique name anymore
                                // We're creating a new variable with the same name that shadows the outer one

                                // For shadowing cases, we want to create a new local variable with the same name
                                // instead of trying to access the outer variable

                                // Get the LLVM type for the value
                                let llvm_type = value.get_type();

                                // Create a local variable with the original name (not a unique name) at the beginning of the function
                                // This is the key difference - we're creating a new variable that shadows the outer one

                                // Save current position
                                let current_position = self.builder.get_insert_block().unwrap();

                                // Move to the beginning of the entry block
                                let current_function = self.current_function.unwrap();
                                let entry_block = current_function.get_first_basic_block().unwrap();
                                if let Some(first_instr) = entry_block.get_first_instruction() {
                                    self.builder.position_before(&first_instr);
                                } else {
                                    self.builder.position_at_end(entry_block);
                                }

                                // Create the alloca at the beginning of the function
                                let local_ptr = self.builder.build_alloca(llvm_type, id).unwrap();

                                // Restore position
                                self.builder.position_at_end(current_position);

                                // Store the value in the local variable
                                self.builder.build_store(local_ptr, value).unwrap();

                                // Add the variable to the current scope with the original name
                                self.scope_stack.current_scope_mut().map(|scope| {
                                    scope.add_variable(id.clone(), local_ptr, value_type.clone());
                                    println!("Created shadowing variable '{}' in nested function", id);
                                });

                                // Also add it to the variables map for backward compatibility
                                self.variables.insert(id.clone(), local_ptr);

                                // Register the variable type
                                self.register_variable(id.clone(), value_type.clone());

                                return Ok(());
                            }
                        }
                    }

                    // If we didn't find a mapping but we're in a nested function with an environment,
                    // try to create a local variable for the nonlocal variable
                    if let Some(env_name) = &self.current_environment {
                        // Get all the data we need from the environment first
                        let mut env_data = None;

                        if let Some(env) = self.get_closure_environment(env_name) {
                            if let Some(index) = env.get_index(id) {
                                if let Some(var_type) = env.get_type(id) {
                                    if let Some(env_ptr) = env.env_ptr {
                                        if let Some(struct_type) = env.env_type {
                                            // Collect all the data we need
                                            env_data = Some((index, var_type.clone(), env_ptr, struct_type));
                                        }
                                    }
                                }
                            }
                        }

                        // Now process the data if we found it
                        if let Some((index, var_type, env_ptr, struct_type)) = env_data {
                            // Create a unique name for the nonlocal variable
                            let unique_name = format!("__nonlocal_{}_{}", env_name.replace('.', "_"), id);

                            // Allocate space for the variable
                            let llvm_type = self.get_llvm_type(&var_type);
                            let ptr = self.builder.build_alloca(llvm_type, &unique_name).unwrap();

                            // Store the value in the local variable using our safe helper method
                            self.store_nonlocal_variable(ptr, value, &unique_name)?;

                            // Add the variable to the current scope with the unique name
                            if let Some(current_scope) = self.scope_stack.current_scope_mut() {
                                current_scope.add_variable(unique_name.clone(), ptr, var_type.clone());
                                current_scope.add_nonlocal_mapping(id.clone(), unique_name.clone());
                                println!("Created local variable for nonlocal variable '{}' with unique name '{}'", id, unique_name);
                            }

                            // Get a pointer to the field in the environment struct
                            let field_ptr = self.builder.build_struct_gep(
                                struct_type,
                                env_ptr,
                                index,
                                &format!("env_{}_ptr", id)
                            ).unwrap();

                            // Store the value directly in the environment
                            // This avoids the mutable borrow issue
                            self.builder.build_store(field_ptr, value).unwrap();
                            println!("Updated nonlocal variable '{}' in closure environment", id);

                            return Ok(());
                        }
                    }
                }

                // If we didn't find a mapping, check if there's a global variable with a name that matches this variable
                // Try different formats of the global name (with and without function prefix)
                let simple_global_name = format!("__nonlocal_{}", id);

                // Get the current function name if available
                let current_function = if let Some(func) = self.builder.get_insert_block().unwrap().get_parent() {
                    func.get_name().to_string_lossy().to_string()
                } else {
                    "".to_string()
                };

                // Try to find the global variable with different name patterns
                let mut global_var = None;

                // First try with the current function name
                if !current_function.is_empty() {
                    let func_global_name = format!("__nonlocal_{}_{}", current_function.replace('.', "_"), id);
                    if let Some(var) = self.module.get_global(&func_global_name) {
                        global_var = Some(var);
                    }

                    // Try with parent function names if this is a nested function
                    if global_var.is_none() && current_function.contains('.') {
                        let parts: Vec<&str> = current_function.split('.').collect();
                        for i in 1..parts.len() {
                            let parent_name = parts[..i].join(".");
                            let parent_global_name = format!("__nonlocal_{}_{}", parent_name.replace('.', "_"), id);
                            if let Some(var) = self.module.get_global(&parent_global_name) {
                                global_var = Some(var);
                                break;
                            }
                        }
                    }
                }

                // Finally try with the simple name
                if global_var.is_none() {
                    if let Some(var) = self.module.get_global(&simple_global_name) {
                        global_var = Some(var);
                    }
                }

                if let Some(global_var) = global_var {
                    // Store the value directly to the global variable
                    self.builder.build_store(global_var.as_pointer_value(), value).unwrap();
                    println!("Assigned to nonlocal variable '{}' using global variable", id);
                    return Ok(());
                }

                if is_global {
                    // Handle global variable assignment
                    if let Some(global_scope) = self.scope_stack.global_scope() {
                        // Check if the global variable exists
                        if let Some(ptr) = global_scope.get_variable(id) {
                            // Check if types are compatible
                            if let Some(target_type) = self.lookup_variable_type(id) {
                                // Convert value to target type if needed
                                let converted_value = if target_type != value_type {
                                    self.convert_type(value, value_type, target_type)?
                                } else {
                                    value
                                };

                                // Store the value to the global variable
                                self.builder.build_store(*ptr, converted_value).unwrap();
                                return Ok(());
                            }
                        } else {
                            // Global variable doesn't exist yet, allocate it in the global scope
                            let global_var = self.module.add_global(
                                self.get_llvm_type(value_type).into_int_type(),
                                None,
                                id
                            );

                            // Initialize with the value
                            global_var.set_initializer(&self.get_llvm_type(value_type).const_zero());

                            // Get a pointer to the global variable
                            let ptr = global_var.as_pointer_value();

                            // Store the variable's storage location in the global scope
                            if let Some(global_scope) = self.scope_stack.global_scope_mut() {
                                global_scope.add_variable(id.clone(), ptr, value_type.clone());
                            }

                            // Store the value to the global variable
                            self.builder.build_store(ptr, value).unwrap();
                            return Ok(());
                        }
                    }
                } else if is_nonlocal {
                    // Handle nonlocal variable assignment
                    // For nonlocal variables, we use the same approach as for normal variables
                    // because we've already set up the variable in the current scope to point to the outer scope
                    if let Some(ptr) = self.get_variable_ptr(id) {
                        // Check if types are compatible
                        if let Some(target_type) = self.lookup_variable_type(id) {
                            // Convert value to target type if needed
                            let converted_value = if target_type != value_type {
                                self.convert_type(value, value_type, target_type)?
                            } else {
                                value
                            };

                            // Store the value to the nonlocal variable
                            self.builder.build_store(ptr, converted_value).unwrap();
                            return Ok(());
                        }
                    } else {
                        return Err(format!("Nonlocal variable '{}' not found", id));
                    }
                }

                // Normal variable assignment (not global or nonlocal)
                if let Some(ptr) = self.get_variable_ptr(id) {
                    // Check if types are compatible
                    if let Some(target_type) = self.lookup_variable_type(id) {
                        // Convert value to target type if needed
                        let converted_value = if target_type != value_type {
                            self.convert_type(value, value_type, target_type)?
                        } else {
                            value
                        };

                        // Store the value to the variable
                        self.builder.build_store(ptr, converted_value).unwrap();
                        Ok(())
                    } else {
                        Err(format!("Variable '{}' has unknown type", id))
                    }
                } else {
                    // Variable doesn't exist yet, allocate storage for it
                    // For nested functions, we need to ensure the allocation happens at the beginning of the function
                    let ptr = if let Some(current_function) = self.current_function {
                        let fn_name = current_function.get_name().to_string_lossy();
                        if fn_name.contains('.') {
                            // For nested functions, we need to allocate at the beginning of the function
                            // Save current position
                            let current_position = self.builder.get_insert_block().unwrap();

                            // Move to the beginning of the entry block
                            let entry_block = current_function.get_first_basic_block().unwrap();
                            if let Some(first_instr) = entry_block.get_first_instruction() {
                                self.builder.position_before(&first_instr);
                            } else {
                                self.builder.position_at_end(entry_block);
                            }

                            // Get the LLVM type for the variable
                            let llvm_type = self.get_llvm_type(value_type);

                            // Create the alloca at the beginning of the function
                            let ptr = self.builder.build_alloca(llvm_type, id).unwrap();

                            // Restore position
                            self.builder.position_at_end(current_position);

                            ptr
                        } else {
                            // For regular functions, use the normal allocation method
                            self.allocate_variable(id.clone(), value_type)
                        }
                    } else {
                        // If not in a function, use the normal allocation method
                        self.allocate_variable(id.clone(), value_type)
                    };

                    // Register the variable type in the type environment
                    self.register_variable(id.clone(), value_type.clone());

                    // Add the variable to the current scope
                    if let Some(current_scope) = self.scope_stack.current_scope_mut() {
                        current_scope.add_variable(id.clone(), ptr, value_type.clone());
                        println!("Added variable '{}' to current scope", id);
                    }

                    // Store the value to the newly created variable
                    self.builder.build_store(ptr, value).unwrap();
                    Ok(())
                }
            },

            // Handle subscript assignment (e.g., list[0] = value)
            Expr::Subscript { value, slice, .. } => {
                // Compile the value being indexed
                let (container_val, container_type) = self.compile_expr(value)?;

                // Compile the index
                let (index_val, index_type) = self.compile_expr(slice)?;

                // Check if the container is indexable
                match &container_type {
                    Type::List(_) => {
                        // Check if the index is an integer
                        if !matches!(index_type, Type::Int) {
                            return Err(format!("List index must be an integer, got {:?}", index_type));
                        }

                        // Get the list_set function
                        let list_set_fn = match self.module.get_function("list_set") {
                            Some(f) => f,
                            None => return Err("list_set function not found".to_string()),
                        };

                        // Compile the value expression
                        let (value_val, _) = self.compile_expr(value)?;

                        // Store the value in memory and pass a pointer to it
                        let value_alloca = self.builder.build_alloca(
                            value_val.get_type(),
                            "list_set_value"
                        ).unwrap();
                        self.builder.build_store(value_alloca, value_val).unwrap();

                        // Call list_set to set the item in the list
                        self.builder.build_call(
                            list_set_fn,
                            &[
                                container_val.into_pointer_value().into(),
                                index_val.into_int_value().into(),
                                value_alloca.into()
                            ],
                            "list_set_result"
                        ).unwrap();

                        Ok(())
                    },
                    Type::Dict(key_type, _value_type) => {
                        // Special case for Unknown key type - update the dictionary's key type
                        if matches!(**key_type, Type::Unknown) {
                            // Update the dictionary's key type to match the index type
                            println!("Updating dictionary key type from Unknown to {:?}", index_type);

                            // We can't actually modify the Type enum directly, but we can update our type tracking
                            // For now, just allow the assignment to proceed
                        }
                        // For dictionaries, we need to check if the key type is compatible
                        // For string keys, we're more permissive to allow for nested dictionary access
                        // Also allow Unknown key type to be compatible with any index type
                        else if !index_type.can_coerce_to(key_type) && !matches!(index_type, Type::String) && !matches!(**key_type, Type::Unknown) {
                            return Err(format!("Dictionary key type mismatch: expected {:?}, got {:?}", key_type, index_type));
                        }

                        // Get the dict_set function
                        let dict_set_fn = match self.module.get_function("dict_set") {
                            Some(f) => f,
                            None => return Err("dict_set function not found".to_string()),
                        };

                        // Convert the key to a pointer if needed
                        let key_ptr = if crate::compiler::types::is_reference_type(&index_type) {
                            index_val
                        } else {
                            // For non-reference types, we need to allocate memory and store the value
                            let key_alloca = self.builder.build_alloca(
                                index_val.get_type(),
                                "dict_key_temp"
                            ).unwrap();
                            self.builder.build_store(key_alloca, index_val).unwrap();
                            key_alloca.into()
                        };

                        // Compile the value to be stored
                        let (value_val, _value_type) = self.compile_expr(target)?;

                        // Store the value in memory and pass a pointer to it
                        let value_alloca = self.builder.build_alloca(
                            value_val.get_type(),
                            "dict_value_temp"
                        ).unwrap();
                        self.builder.build_store(value_alloca, value_val).unwrap();

                        // Call dict_set to set the item in the dictionary
                        self.builder.build_call(
                            dict_set_fn,
                            &[
                                container_val.into_pointer_value().into(),
                                key_ptr.into(),
                                value_alloca.into()
                            ],
                            "dict_set_result"
                        ).unwrap();

                        Ok(())
                    },
                    Type::Tuple(_) => {
                        return Err("Tuple elements cannot be modified".to_string());
                    },
                    Type::String => {
                        return Err("String elements cannot be modified".to_string());
                    },
                    _ => Err(format!("Type {:?} is not indexable", container_type)),
                }
            },

            // Handle other assignment targets (attributes, etc.)
            _ => Err(format!("Unsupported assignment target: {:?}", target)),
        }
    }
}
