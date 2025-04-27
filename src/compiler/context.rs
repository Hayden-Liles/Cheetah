use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;
// use inkwell::types::BasicType;
use crate::ast;
use crate::compiler::closure::ClosureEnvironment;
use crate::compiler::scope::ScopeStack;
use crate::compiler::stmt::StmtCompiler;
use crate::compiler::types::is_reference_type;
use crate::compiler::types::Type;

/// Loop context for managing break and continue statements
pub struct LoopContext<'ctx> {
    pub continue_block: BasicBlock<'ctx>,
    pub break_block: BasicBlock<'ctx>,
}

/// Compilation context that manages types and values during code generation
pub struct CompilationContext<'ctx> {
    /// LLVM context
    pub llvm_context: &'ctx Context,

    /// LLVM module being built
    pub module: Module<'ctx>,

    /// LLVM IR builder
    pub builder: Builder<'ctx>,

    /// Type environment mapping variable names to their types
    pub type_env: HashMap<String, Type>,

    /// Map of function names to their LLVM function values
    pub functions: HashMap<String, inkwell::values::FunctionValue<'ctx>>,

    /// Map of class names to their LLVM struct types
    pub class_types: HashMap<String, inkwell::types::StructType<'ctx>>,

    /// Map of variable names to their LLVM pointer values (storage locations)
    pub variables: HashMap<String, inkwell::values::PointerValue<'ctx>>,

    /// Stack of loop contexts for break/continue statements
    pub loop_stack: Vec<LoopContext<'ctx>>,

    /// Map of polymorphic function names to their implementation variants by argument type
    pub polymorphic_functions: HashMap<String, HashMap<Type, inkwell::values::FunctionValue<'ctx>>>,

    /// Currently active function (if any)
    pub current_function: Option<inkwell::values::FunctionValue<'ctx>>,

    /// Local variables in the current function scope
    pub local_vars: HashMap<String, inkwell::values::PointerValue<'ctx>>,

    /// Stack of variable scopes
    pub scope_stack: ScopeStack<'ctx>,

    /// Map of function names to their closure environments
    pub closure_environments: HashMap<String, ClosureEnvironment<'ctx>>,

    /// Currently active closure environment (if any)
    pub current_environment: Option<String>,

    /// Unique ID counter for generating unique names
    pub unique_id_counter: usize,

    /// Pending method calls (object_ptr, method_name, element_type)
    pub pending_method_calls: HashMap<String, (String, Box<Type>)>,

    /// Whether to use BoxedAny values
    pub use_boxed_values: bool,
}

impl<'ctx> CompilationContext<'ctx> {
    /// Create a new compilation context
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let mut module = context.create_module(module_name);
        let builder = context.create_builder();

        // Register runtime functions
        crate::compiler::runtime::register_runtime_functions(context, &mut module);

        Self {
            llvm_context: context,
            module,
            builder,
            type_env: HashMap::new(),
            functions: HashMap::new(),
            class_types: HashMap::new(),
            variables: HashMap::new(),
            loop_stack: Vec::new(),
            polymorphic_functions: HashMap::new(),
            current_function: None,
            local_vars: HashMap::new(),
            scope_stack: ScopeStack::new(),
            closure_environments: HashMap::new(),
            current_environment: None,
            unique_id_counter: 0,
            pending_method_calls: HashMap::new(),
            use_boxed_values: true, // Default to using BoxedAny values
        }
    }

    /// Set whether to use BoxedAny values
    pub fn set_use_boxed_values(&mut self, use_boxed_values: bool) {
        self.use_boxed_values = use_boxed_values;
    }

    /// Get or create a type in the LLVM context
    pub fn get_llvm_type(&self, ty: &Type) -> inkwell::types::BasicTypeEnum<'ctx> {
        ty.to_llvm_type(self.llvm_context)
    }

    /// Register a variable with its type
    pub fn register_variable(&mut self, name: String, ty: Type) {
        self.type_env.insert(name, ty);
    }

    /// Look up a variable's type
    pub fn lookup_variable_type(&self, name: &str) -> Option<&Type> {
        if let Some(ty) = self.scope_stack.get_type(name) {
            return Some(ty);
        }

        self.type_env.get(name)
    }

    /// Register a class type
    pub fn register_class(&mut self, name: String, fields: HashMap<String, Type>) {
        let ty = Type::Class {
            name: name.clone(),
            base_classes: vec![],
            methods: HashMap::new(),
            fields: fields.clone(),
        };

        if let Type::Class { ref name, .. } = ty {
            let struct_type = ty.create_class_type(self.llvm_context, name, &fields);
            self.class_types.insert(name.clone(), struct_type);
        }

        self.type_env.insert(name, ty);
    }

    pub fn declare_variable(
        &mut self,
        name: String,
        init_value: BasicValueEnum<'ctx>,
        value_type: &Type,
    ) -> Result<(), String> {
        let ptr = self.allocate_variable(name.clone(), value_type);

        self.builder.build_store(ptr, init_value).unwrap();

        self.scope_stack.add_variable(name, ptr, value_type.clone());

        Ok(())
    }

    pub fn allocate_variable(
        &mut self,
        name: String,
        ty: &Type,
    ) -> inkwell::values::PointerValue<'ctx> {
        let current_function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let entry_bb = current_function.get_first_basic_block().unwrap();

        let current_position = self.builder.get_insert_block().unwrap();
        if let Some(first_instr) = entry_bb.get_first_instruction() {
            self.builder.position_before(&first_instr);
        } else {
            self.builder.position_at_end(entry_bb);
        }

        let llvm_type = self.get_llvm_type(ty);

        let var_name = if let Some(current_function) = self.current_function {
            let fn_name = current_function.get_name().to_string_lossy();
            if fn_name.contains('.') {
                format!("{}.{}", fn_name, name)
            } else {
                name.clone()
            }
        } else {
            name.clone()
        };

        let ptr = self.builder.build_alloca(llvm_type, &var_name).unwrap();

        self.builder.position_at_end(current_position);

        self.variables.insert(name.clone(), ptr);

        self.add_variable_to_scope(name.clone(), ptr, ty.clone());

        println!("Added variable '{}' to current scope", name);

        if !self.type_env.contains_key(&name) {
            self.register_variable(name, ty.clone());
        }

        ptr
    }

    /// Get the storage location for a variable
    pub fn get_variable_ptr(&self, name: &str) -> Option<inkwell::values::PointerValue<'ctx>> {
        if let Some(ptr) = self.scope_stack.get_variable_respecting_declarations(name) {
            return Some(*ptr);
        }

        if let Some(&ptr) = self.local_vars.get(name) {
            return Some(ptr);
        }

        self.variables.get(name).copied()
    }

    /// Ensure a variable exists in the current scope or create it if it's a global variable
    pub fn ensure_variable(&mut self, name: &str) -> Option<inkwell::values::PointerValue<'ctx>> {
        if let Some(ptr) = self.get_variable_ptr(name) {
            return Some(ptr);
        }

        if self.type_env.contains_key(name) {
            let ty = self.type_env.get(name).unwrap().clone();
            let ptr = self.allocate_variable(name.to_string(), &ty);
            return Some(ptr);
        }

        None
    }

    /// Ensure the current block has a terminator
    /// If it doesn't, add a branch to a new block and position at that block
    pub fn ensure_block_has_terminator(&self) -> Option<BasicBlock<'ctx>> {
        let current_block = self.builder.get_insert_block().unwrap();
        if current_block.get_terminator().is_none() {
            if let Some(current_function) = current_block.get_parent() {
                let new_block = self
                    .llvm_context
                    .append_basic_block(current_function, "continue_block");

                self.builder.build_unconditional_branch(new_block).unwrap();

                self.builder.position_at_end(new_block);

                return Some(new_block);
            }
        }
        None
    }

    /// Create a shadow variable in the entry block of the current function
    /// This ensures proper dominance for all uses of the variable
    fn create_shadow_variable(
        &self,
        _original_ptr: inkwell::values::PointerValue<'ctx>,
        var_type: inkwell::types::BasicTypeEnum<'ctx>,
        name: &str,
    ) -> inkwell::values::PointerValue<'ctx> {
        let current_block = self.builder.get_insert_block().unwrap();
        let current_function = current_block.get_parent().unwrap();

        let current_position = current_block;

        let entry_block = current_function.get_first_basic_block().unwrap();

        if let Some(first_instr) = entry_block.get_first_instruction() {
            self.builder.position_before(&first_instr);
        } else {
            self.builder.position_at_end(entry_block);
        }

        let shadow_ptr = self
            .builder
            .build_alloca(var_type, &format!("shadow_{}", name))
            .unwrap();

        match var_type {
            inkwell::types::BasicTypeEnum::IntType(int_type) => {
                self.builder
                    .build_store(shadow_ptr, int_type.const_zero())
                    .unwrap();
            }
            inkwell::types::BasicTypeEnum::FloatType(float_type) => {
                self.builder
                    .build_store(shadow_ptr, float_type.const_zero())
                    .unwrap();
            }
            inkwell::types::BasicTypeEnum::PointerType(ptr_type) => {
                self.builder
                    .build_store(shadow_ptr, ptr_type.const_null())
                    .unwrap();
            }
            _ => {}
        }

        self.builder.position_at_end(current_position);

        shadow_ptr
    }

    /// Load a nonlocal variable safely, ensuring proper dominance
    pub fn load_nonlocal_variable(
        &mut self,
        ptr: inkwell::values::PointerValue<'ctx>,
        var_type: &Type,
        name: &str,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let llvm_type = self.get_llvm_type(var_type);

        let current_block = self.builder.get_insert_block().unwrap();
        let current_function = current_block.get_parent().unwrap();

        let function_name = current_function.get_name().to_str().unwrap_or("unknown");
        let shadow_name = format!("shadow_{}_{}", function_name, name);

        let shadow_ptr = if let Some(shadow) = self.variables.get(&shadow_name) {
            *shadow
        } else {
            let shadow = self.create_shadow_variable(ptr, llvm_type, name);

            let current_position = self.builder.get_insert_block().unwrap();
            let entry_block = current_function.get_first_basic_block().unwrap();

            if let Some(first_instr) = entry_block.get_first_instruction() {
                self.builder.position_before(&first_instr);
            } else {
                self.builder.position_at_end(entry_block);
            }

            let initial_value = self
                .builder
                .build_load(llvm_type, ptr, &format!("initial_{}", name))
                .unwrap();

            self.builder.build_store(shadow, initial_value).unwrap();

            self.builder.position_at_end(current_position);

            self.variables.insert(shadow_name.clone(), shadow);

            shadow
        };

        let value = self
            .builder
            .build_load(llvm_type, shadow_ptr, &format!("load_{}", name))
            .unwrap();

        Ok(value)
    }

    /// Store a value to a nonlocal variable safely, ensuring proper dominance
    pub fn store_nonlocal_variable(
        &mut self,
        ptr: inkwell::values::PointerValue<'ctx>,
        value: inkwell::values::BasicValueEnum<'ctx>,
        name: &str,
    ) -> Result<(), String> {
        let current_block = self.builder.get_insert_block().unwrap();
        let current_function = current_block.get_parent().unwrap();

        let function_name = current_function.get_name().to_str().unwrap_or("unknown");
        let shadow_name = format!("shadow_{}_{}", function_name, name);

        let shadow_ptr = if let Some(shadow) = self.variables.get(&shadow_name) {
            *shadow
        } else {
            let shadow = self.create_shadow_variable(ptr, value.get_type(), name);

            self.variables.insert(shadow_name.clone(), shadow);

            shadow
        };

        self.builder.build_store(shadow_ptr, value).unwrap();
        self.builder.build_store(ptr, value).unwrap();

        Ok(())
    }

    /// Push a new loop context onto the stack
    pub fn push_loop(&mut self, continue_block: BasicBlock<'ctx>, break_block: BasicBlock<'ctx>) {
        self.loop_stack.push(LoopContext {
            continue_block,
            break_block,
        });
    }

    /// Pop the top loop context off the stack
    pub fn pop_loop(&mut self) -> Option<LoopContext<'ctx>> {
        self.loop_stack.pop()
    }

    /// Get the current loop's continue block
    pub fn current_continue_block(&self) -> Option<BasicBlock<'ctx>> {
        self.loop_stack.last().map(|ctx| ctx.continue_block)
    }

    /// Get the current loop's break block
    pub fn current_break_block(&self) -> Option<BasicBlock<'ctx>> {
        self.loop_stack.last().map(|ctx| ctx.break_block)
    }

    /// Get a unique ID for generating unique names
    pub fn get_unique_id(&mut self) -> usize {
        let id = self.unique_id_counter;
        self.unique_id_counter += 1;
        id
    }

    /// Set a pending method call for later processing
    pub fn set_pending_method_call(&mut self, object_ptr: String, method_name: String, element_type: Box<Type>) {
        self.pending_method_calls.insert(object_ptr, (method_name, element_type));
    }

    /// Convert a value from one type to another
    pub fn convert_type(
        &self,
        value: inkwell::values::BasicValueEnum<'ctx>,
        from_type: &Type,
        to_type: &Type,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        if from_type == to_type {
            return Ok(value);
        }

        if let (Type::Int, Type::List(_)) = (from_type, to_type) {
            return Ok(value);
        }

        if let Type::Tuple(_) = from_type {
            println!(
                "WARNING: Attempted to convert tuple to {:?}, returning original value",
                to_type
            );
            return Ok(value);
        }

        if !from_type.can_coerce_to(to_type) {
            return Err(format!(
                "Cannot convert from {:?} to {:?}",
                from_type, to_type
            ));
        }

        match (from_type, to_type) {
            (Type::Bool, Type::Int) => {
                let bool_val = value.into_int_value();
                let int_val = self
                    .builder
                    .build_int_z_extend(bool_val, self.llvm_context.i64_type(), "bool_to_int")
                    .unwrap();
                Ok(int_val.into())
            }

            (Type::Int, Type::Bool) => {
                let int_val = value.into_int_value();
                let zero = self.llvm_context.i64_type().const_zero();
                let bool_val = self
                    .builder
                    .build_int_compare(inkwell::IntPredicate::NE, int_val, zero, "int_to_bool")
                    .unwrap();
                Ok(bool_val.into())
            }

            (Type::Int, Type::Float) => {
                let int_val = value.into_int_value();
                let float_val = self
                    .builder
                    .build_signed_int_to_float(
                        int_val,
                        self.llvm_context.f64_type(),
                        "int_to_float",
                    )
                    .unwrap();
                Ok(float_val.into())
            }

            (Type::Float, Type::Int) => {
                let float_val = value.into_float_value();
                let int_val = self
                    .builder
                    .build_float_to_signed_int(
                        float_val,
                        self.llvm_context.i64_type(),
                        "float_to_int",
                    )
                    .unwrap();
                Ok(int_val.into())
            }

            (Type::Bool, Type::Float) => {
                let bool_val = value.into_int_value();
                let int_val = self
                    .builder
                    .build_int_z_extend(bool_val, self.llvm_context.i64_type(), "bool_to_int")
                    .unwrap();

                let float_val = self
                    .builder
                    .build_signed_int_to_float(
                        int_val,
                        self.llvm_context.f64_type(),
                        "int_to_float",
                    )
                    .unwrap();
                Ok(float_val.into())
            }

            (Type::Float, Type::Bool) => {
                let float_val = value.into_float_value();
                let zero = self.llvm_context.f64_type().const_float(0.0);
                let bool_val = self
                    .builder
                    .build_float_compare(
                        inkwell::FloatPredicate::ONE,
                        float_val,
                        zero,
                        "float_to_bool",
                    )
                    .unwrap();
                Ok(bool_val.into())
            }

            (Type::None, _) if is_reference_type(to_type) => {
                let ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let null_ptr = ptr_type.const_null();
                Ok(null_ptr.into())
            }

            (Type::Int, Type::String) => self.build_int_to_string_call(value.into_int_value()),

            (Type::Float, Type::String) => {
                self.build_float_to_string_call(value.into_float_value())
            }

            (Type::Bool, Type::String) => self.build_bool_to_string_call(value.into_int_value()),

            (Type::String, Type::Int) => self.build_string_to_int_call(value.into_pointer_value()),

            (Type::String, Type::Float) => {
                self.build_string_to_float_call(value.into_pointer_value())
            }

            (Type::String, Type::Bool) => {
                self.build_string_to_bool_call(value.into_pointer_value())
            }

            (Type::Any, Type::Bool) => {
                // Convert Any to Bool using boxed_any_to_bool
                let boxed_any_to_bool_fn = self.module.get_function("boxed_any_to_bool")
                    .ok_or_else(|| "boxed_any_to_bool function not found".to_string())?;

                let call_site_value = self.builder.build_call(
                    boxed_any_to_bool_fn,
                    &[value.into()],
                    "any_to_bool_result"
                ).unwrap();

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to convert Any to Bool".to_string())?;

                Ok(result.into())
            }

            (Type::Int, Type::Any) => {
                // Convert Int to Any using boxed_any_from_int
                let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                    .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                let int_val = if value.is_pointer_value() {
                    // Load the integer value from the pointer
                    self.builder
                        .build_load(self.llvm_context.i64_type(), value.into_pointer_value(), "load_int")
                        .unwrap()
                        .into_int_value()
                } else {
                    value.into_int_value()
                };

                let call_site_value = self.builder.build_call(
                    boxed_any_from_int_fn,
                    &[int_val.into()],
                    "int_to_any_result"
                ).unwrap();

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to convert Int to Any".to_string())?;

                Ok(result)
            }

            (Type::Float, Type::Any) => {
                // Convert Float to Any using boxed_any_from_float
                let boxed_any_from_float_fn = self.module.get_function("boxed_any_from_float")
                    .ok_or_else(|| "boxed_any_from_float function not found".to_string())?;

                let float_val = if value.is_pointer_value() {
                    // Load the float value from the pointer
                    self.builder
                        .build_load(self.llvm_context.f64_type(), value.into_pointer_value(), "load_float")
                        .unwrap()
                        .into_float_value()
                } else {
                    value.into_float_value()
                };

                let call_site_value = self.builder.build_call(
                    boxed_any_from_float_fn,
                    &[float_val.into()],
                    "float_to_any_result"
                ).unwrap();

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to convert Float to Any".to_string())?;

                Ok(result)
            }

            (Type::Bool, Type::Any) => {
                // Convert Bool to Any using boxed_any_from_bool
                let boxed_any_from_bool_fn = self.module.get_function("boxed_any_from_bool")
                    .ok_or_else(|| "boxed_any_from_bool function not found".to_string())?;

                let bool_val = if value.is_pointer_value() {
                    // Load the bool value from the pointer
                    self.builder
                        .build_load(self.llvm_context.bool_type(), value.into_pointer_value(), "load_bool")
                        .unwrap()
                        .into_int_value()
                } else {
                    value.into_int_value()
                };

                let call_site_value = self.builder.build_call(
                    boxed_any_from_bool_fn,
                    &[bool_val.into()],
                    "bool_to_any_result"
                ).unwrap();

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to convert Bool to Any".to_string())?;

                Ok(result)
            }

            (Type::String, Type::Any) => {
                // Convert String to Any using boxed_any_from_string
                let boxed_any_from_string_fn = self.module.get_function("boxed_any_from_string")
                    .ok_or_else(|| "boxed_any_from_string function not found".to_string())?;

                if !value.is_pointer_value() {
                    return Err("Expected pointer value for string".to_string());
                }

                let string_ptr = value.into_pointer_value();

                let call_site_value = self.builder.build_call(
                    boxed_any_from_string_fn,
                    &[string_ptr.into()],
                    "string_to_any_result"
                ).unwrap();

                let result = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| "Failed to convert String to Any".to_string())?;

                Ok(result)
            }

            _ => Err(format!(
                "Unsupported type conversion from {:?} to {:?}",
                from_type, to_type
            )),
        }
    }

    /// Helper method to get the common type for binary operations
    pub fn get_common_type(&self, type1: &Type, type2: &Type) -> Result<Type, String> {
        if type1 == type2 {
            return Ok(type1.clone());
        }

        if let Type::Tuple(_) = type1 {
            return Ok(type1.clone());
        }
        if let Type::Tuple(_) = type2 {
            return Ok(type2.clone());
        }

        if type1.can_coerce_to(type2) {
            return Ok(type2.clone());
        }

        if type2.can_coerce_to(type1) {
            return Ok(type1.clone());
        }

        match (type1, type2) {
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
            (Type::Int, Type::Bool) | (Type::Bool, Type::Int) => Ok(Type::Int),
            (Type::Float, Type::Bool) | (Type::Bool, Type::Float) => Ok(Type::Float),

            (Type::List(_), Type::Int) => Ok(type1.clone()),
            (Type::Int, Type::List(_)) => Ok(type2.clone()),

            (Type::Tuple(_), Type::Int) => Ok(type1.clone()),
            (Type::Int, Type::Tuple(_)) => Ok(type2.clone()),

            _ => Err(format!("No common type for {:?} and {:?}", type1, type2)),
        }
    }

    fn build_int_to_string_call(
        &self,
        int_val: inkwell::values::IntValue<'ctx>,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let int_to_string_fn = self
            .module
            .get_function("int_to_string")
            .unwrap_or_else(|| {
                let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let fn_type = str_ptr_type.fn_type(&[self.llvm_context.i64_type().into()], false);
                self.module.add_function("int_to_string", fn_type, None)
            });

        let result = self
            .builder
            .build_call(int_to_string_fn, &[int_val.into()], "int_to_string_result")
            .unwrap();

        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call int_to_string function".to_string())
        }
    }

    fn build_float_to_string_call(
        &self,
        float_val: inkwell::values::FloatValue<'ctx>,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let float_to_string_fn = self
            .module
            .get_function("float_to_string")
            .unwrap_or_else(|| {
                let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let fn_type = str_ptr_type.fn_type(&[self.llvm_context.f64_type().into()], false);
                self.module.add_function("float_to_string", fn_type, None)
            });

        let result = self
            .builder
            .build_call(
                float_to_string_fn,
                &[float_val.into()],
                "float_to_string_result",
            )
            .unwrap();

        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call float_to_string function".to_string())
        }
    }

    /// Convert a value to a string
    pub fn convert_to_string(
        &self,
        value: inkwell::values::BasicValueEnum<'ctx>,
        value_type: &crate::compiler::types::Type,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        match value_type {
            crate::compiler::types::Type::String => {
                if value.is_pointer_value() {
                    Ok(value.into_pointer_value())
                } else {
                    Err("Expected pointer value for string".to_string())
                }
            },
            crate::compiler::types::Type::Int => {
                let int_to_string_fn = match self.module.get_function("int_to_string") {
                    Some(f) => f,
                    None => return Err("int_to_string function not found".to_string()),
                };

                // If value is a pointer, load it first to get the integer value
                let int_value = if value.is_pointer_value() {
                    let loaded = self.builder.build_load(
                        self.llvm_context.i64_type(),
                        value.into_pointer_value(),
                        "load_int"
                    ).unwrap();
                    loaded.into_int_value()
                } else {
                    value.into_int_value()
                };

                let call_site_value = self
                    .builder
                    .build_call(
                        int_to_string_fn,
                        &[int_value.into()],
                        "int_to_string_result",
                    )
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to convert integer to string".to_string())?;

                Ok(result.into_pointer_value())
            },
            crate::compiler::types::Type::Float => {
                let float_to_string_fn = match self.module.get_function("float_to_string") {
                    Some(f) => f,
                    None => return Err("float_to_string function not found".to_string()),
                };

                let call_site_value = self
                    .builder
                    .build_call(
                        float_to_string_fn,
                        &[value.into()],
                        "float_to_string_result",
                    )
                    .unwrap();

                let result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Failed to convert float to string".to_string())?;

                Ok(result.into_pointer_value())
            },
            crate::compiler::types::Type::Bool => {
                // Convert boolean to "True" or "False"
                let bool_val = value.into_int_value();
                let true_str = self.llvm_context.const_string("True".as_bytes(), true);
                let false_str = self.llvm_context.const_string("False".as_bytes(), true);

                let true_global = self.module.add_global(true_str.get_type(), None, "true_str");
                true_global.set_constant(true);
                true_global.set_initializer(&true_str);

                let false_global = self.module.add_global(false_str.get_type(), None, "false_str");
                false_global.set_constant(true);
                false_global.set_initializer(&false_str);

                let true_ptr = self
                    .builder
                    .build_pointer_cast(
                        true_global.as_pointer_value(),
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "true_ptr",
                    )
                    .unwrap();

                let false_ptr = self
                    .builder
                    .build_pointer_cast(
                        false_global.as_pointer_value(),
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "false_ptr",
                    )
                    .unwrap();

                let cond = self.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    bool_val,
                    self.llvm_context.bool_type().const_zero(),
                    "bool_cmp",
                ).unwrap();

                let result = self.builder.build_select(
                    cond,
                    true_ptr,
                    false_ptr,
                    "bool_str",
                ).unwrap();

                Ok(result.into_pointer_value())
            },
            crate::compiler::types::Type::None => {
                // Convert None to "None"
                let none_str = self.llvm_context.const_string("None".as_bytes(), true);
                let none_global = self.module.add_global(none_str.get_type(), None, "none_str");
                none_global.set_constant(true);
                none_global.set_initializer(&none_str);

                let none_ptr = self
                    .builder
                    .build_pointer_cast(
                        none_global.as_pointer_value(),
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "none_ptr",
                    )
                    .unwrap();

                Ok(none_ptr)
            },
            _ => {
                // For other types, use a placeholder string
                let placeholder = format!("<{:?}>", value_type);
                let placeholder_str = self.llvm_context.const_string(placeholder.as_bytes(), true);
                let placeholder_global = self.module.add_global(placeholder_str.get_type(), None, "placeholder_str");
                placeholder_global.set_constant(true);
                placeholder_global.set_initializer(&placeholder_str);

                let placeholder_ptr = self
                    .builder
                    .build_pointer_cast(
                        placeholder_global.as_pointer_value(),
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        "placeholder_ptr",
                    )
                    .unwrap();

                Ok(placeholder_ptr)
            }
        }
    }

    fn build_bool_to_string_call(
        &self,
        bool_val: inkwell::values::IntValue<'ctx>,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let bool_to_string_fn = self
            .module
            .get_function("bool_to_string")
            .unwrap_or_else(|| {
                let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let fn_type = str_ptr_type.fn_type(&[self.llvm_context.i64_type().into()], false);
                self.module.add_function("bool_to_string", fn_type, None)
            });

        let result = self
            .builder
            .build_call(
                bool_to_string_fn,
                &[bool_val.into()],
                "bool_to_string_result",
            )
            .unwrap();

        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call bool_to_string function".to_string())
        }
    }

    fn build_string_to_int_call(
        &self,
        string_ptr: inkwell::values::PointerValue<'ctx>,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let string_to_int_fn = self
            .module
            .get_function("string_to_int")
            .unwrap_or_else(|| {
                let i64_type = self.llvm_context.i64_type();
                let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let fn_type = i64_type.fn_type(&[str_ptr_type.into()], false);
                self.module.add_function("string_to_int", fn_type, None)
            });

        let result = self
            .builder
            .build_call(
                string_to_int_fn,
                &[string_ptr.into()],
                "string_to_int_result",
            )
            .unwrap();

        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call string_to_int function".to_string())
        }
    }

    fn build_string_to_float_call(
        &self,
        string_ptr: inkwell::values::PointerValue<'ctx>,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let string_to_float_fn = self
            .module
            .get_function("string_to_float")
            .unwrap_or_else(|| {
                let f64_type = self.llvm_context.f64_type();
                let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let fn_type = f64_type.fn_type(&[str_ptr_type.into()], false);
                self.module.add_function("string_to_float", fn_type, None)
            });

        let result = self
            .builder
            .build_call(
                string_to_float_fn,
                &[string_ptr.into()],
                "string_to_float_result",
            )
            .unwrap();

        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call string_to_float function".to_string())
        }
    }

    fn build_string_to_bool_call(
        &self,
        string_ptr: inkwell::values::PointerValue<'ctx>,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let string_to_bool_fn = self
            .module
            .get_function("string_to_bool")
            .unwrap_or_else(|| {
                let bool_type = self.llvm_context.bool_type();
                let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let fn_type = bool_type.fn_type(&[str_ptr_type.into()], false);
                self.module.add_function("string_to_bool", fn_type, None)
            });

        let result = self
            .builder
            .build_call(
                string_to_bool_fn,
                &[string_ptr.into()],
                "string_to_bool_result",
            )
            .unwrap();

        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call string_to_bool function".to_string())
        }
    }

    pub fn get_polymorphic_function(
        &self,
        name: &str,
        arg_type: &Type,
    ) -> Option<inkwell::values::FunctionValue<'ctx>> {
        if let Some(variants) = self.polymorphic_functions.get(name) {
            if let Some(&func) = variants.get(arg_type) {
                return Some(func);
            }

            for (type_key, &func) in variants.iter() {
                if arg_type.can_coerce_to(type_key) {
                    return Some(func);
                }
            }
        }
        None
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self, is_function: bool, is_loop: bool, is_class: bool) {
        self.scope_stack.push_scope(is_function, is_loop, is_class);
    }

    /// Pop the innermost scope from the stack
    pub fn pop_scope(&mut self) {
        self.scope_stack.pop_scope();
    }

    /// Add a variable to the current scope
    pub fn add_variable_to_scope(
        &mut self,
        name: String,
        ptr: inkwell::values::PointerValue<'ctx>,
        ty: Type,
    ) {
        self.scope_stack.add_variable(name, ptr, ty);
    }

    /// Declare a variable as global in the current scope
    pub fn declare_global(&mut self, name: String) {
        self.scope_stack.declare_global(name);
    }

    /// Declare a variable as nonlocal in the current scope
    pub fn declare_nonlocal(&mut self, name: String) {
        self.scope_stack.declare_nonlocal(name);
    }

    /// Declare a nested function
    pub fn declare_nested_function(
        &mut self,
        name: &str,
        params: &[ast::Parameter],
    ) -> Result<(), String> {
        let context = self.llvm_context;

        let mut param_types = Vec::new();

        for _ in params {
            param_types.push(context.i64_type().into());
        }

        self.create_closure_environment(name);

        let mut nonlocal_vars = Vec::new();

        if name.contains('.') {
            if let Some(current_scope) = self.scope_stack.current_scope() {
                nonlocal_vars = current_scope.nonlocal_vars.clone();
            }

            if let Some(env) = self.get_closure_environment(name) {
                for var_name in &env.nonlocal_params {
                    if !nonlocal_vars.contains(var_name) {
                        nonlocal_vars.push(var_name.clone());
                    }
                }
            }
        }

        println!(
            "Nonlocal variables for function {}: {:?}",
            name, nonlocal_vars
        );

        for (i, var_name) in nonlocal_vars.iter().enumerate() {
            param_types.push(context.i64_type().into());
            println!(
                "Adding nonlocal parameter {} ({}) to function {}",
                i, var_name, name
            );
        }

        println!(
            "Function {} has {} regular parameters and {} nonlocal parameters",
            name,
            params.len(),
            nonlocal_vars.len()
        );

        let env_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        param_types.push(env_ptr_type.into());

        let return_type = context.i64_type();
        let function_type = return_type.fn_type(&param_types, false);

        let function = self.module.add_function(name, function_type, None);

        self.functions.insert(name.to_string(), function);

        if !nonlocal_vars.is_empty() {
            if let Some(env) = self.get_closure_environment_mut(name) {
                env.nonlocal_params = nonlocal_vars;
            }
        }

        Ok(())
    }

    /// Compile a nested function body
    pub fn compile_nested_function_body(
        &mut self,
        name: &str,
        params: &[ast::Parameter],
        body: &[Box<ast::Stmt>],
    ) -> Result<(), String> {
        let context = self.llvm_context;

        let function = match self.functions.get(name) {
            Some(&f) => f,
            None => return Err(format!("Function {} not found", name)),
        };

        let basic_block = context.append_basic_block(function, "entry");

        let current_block = self.builder.get_insert_block();

        self.builder.position_at_end(basic_block);

        println!("Compiling nested function body for {}", name);
        println!(
            "Current scope stack size: {}",
            self.scope_stack.scopes.len()
        );

        self.push_scope(true, false, false);

        println!(
            "After pushing function scope, stack size: {}",
            self.scope_stack.scopes.len()
        );

        let mut local_vars = HashMap::new();

        for (i, param) in params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();

            let alloca = self
                .builder
                .build_alloca(context.i64_type(), &param.name)
                .unwrap();

            self.builder.build_store(alloca, param_value).unwrap();

            local_vars.insert(param.name.clone(), alloca);

            self.add_variable_to_scope(param.name.clone(), alloca, Type::Int);

            println!("Added parameter '{}' to function scope", param.name);

            self.register_variable(param.name.clone(), Type::Int);
        }

        let nonlocal_vars = if let Some(env) = self.get_closure_environment(name) {
            env.nonlocal_params.clone()
        } else {
            Vec::new()
        };

        let mut nonlocal_param_map = HashMap::new();
        for (i, var_name) in nonlocal_vars.iter().enumerate() {
            let param_value = function.get_nth_param((params.len() + i) as u32).unwrap();

            let unique_name = format!("__nonlocal_{}_{}", name.replace('.', "_"), var_name);

            let current_position = self.builder.get_insert_block().unwrap();

            let entry_block = function.get_first_basic_block().unwrap();
            if let Some(first_instr) = entry_block.get_first_instruction() {
                self.builder.position_before(&first_instr);
            } else {
                self.builder.position_at_end(entry_block);
            }

            let alloca = self
                .builder
                .build_alloca(context.i64_type(), &unique_name)
                .unwrap();

            self.builder.position_at_end(current_position);

            self.builder.build_store(alloca, param_value).unwrap();

            self.add_variable_to_scope(unique_name.clone(), alloca, Type::Int);

            if let Some(current_scope) = self.scope_stack.current_scope_mut() {
                current_scope.add_nonlocal_mapping(var_name.clone(), unique_name.clone());
            }

            nonlocal_param_map.insert(var_name.clone(), alloca);

            println!(
                "Added nonlocal parameter '{}' to function scope with unique name '{}'",
                var_name, unique_name
            );
        }

        let param_count = function.count_params();
        println!("Function {} has {} parameters", name, param_count);

        let expected_param_count = params.len() + nonlocal_vars.len() + 1;
        println!(
            "Function {} should have {} parameters: {} regular + {} nonlocal + 1 env ptr",
            name,
            expected_param_count,
            params.len(),
            nonlocal_vars.len()
        );

        let env_param = function
            .get_nth_param((params.len() + nonlocal_vars.len()) as u32)
            .unwrap();

        let env_alloca = self
            .builder
            .build_alloca(
                context.ptr_type(inkwell::AddressSpace::default()),
                "env_ptr",
            )
            .unwrap();

        self.builder.build_store(env_alloca, env_param).unwrap();

        self.set_current_environment(name.to_string());

        let mut additional_nonlocal_vars = Vec::new();
        for stmt in body {
            if let ast::Stmt::Nonlocal { names, .. } = stmt.as_ref() {
                for name in names {
                    if !nonlocal_vars.contains(name) {
                        additional_nonlocal_vars.push(name.clone());
                    }
                }
            }
        }

        if !nonlocal_vars.is_empty() {
            for var_name in &nonlocal_vars {
                self.scope_stack.declare_nonlocal(var_name.clone());

                let mut found_ptr = None;
                let mut found_type = None;

                let current_index = self.scope_stack.scopes.len() - 1;

                if current_index > 0 {
                    let parent_scope_index = current_index - 1;
                    if let Some(ptr) =
                        self.scope_stack.scopes[parent_scope_index].get_variable(var_name)
                    {
                        found_ptr = Some(*ptr);
                        found_type = self.scope_stack.scopes[parent_scope_index]
                            .get_type(var_name)
                            .cloned();
                        println!(
                            "Found nonlocal variable '{}' in immediate outer scope {}",
                            var_name, parent_scope_index
                        );
                    } else {
                        if self.scope_stack.scopes[parent_scope_index].is_nonlocal(var_name) {
                            if let Some(parent_unique_name) = self.scope_stack.scopes
                                [parent_scope_index]
                                .get_nonlocal_mapping(var_name)
                            {
                                if let Some(ptr) = self.scope_stack.scopes[parent_scope_index]
                                    .get_variable(parent_unique_name)
                                {
                                    found_ptr = Some(*ptr);
                                    found_type = self.scope_stack.scopes[parent_scope_index]
                                        .get_type(parent_unique_name)
                                        .cloned();
                                    println!("Found nonlocal variable '{}' using mapping '{}' in parent scope {}",
                                             var_name, parent_unique_name, parent_scope_index);
                                }
                            }
                        }
                    }
                }

                if found_ptr.is_none() {
                    for i in (0..current_index - 1).rev() {
                        if let Some(ptr) = self.scope_stack.scopes[i].get_variable(var_name) {
                            found_ptr = Some(*ptr);
                            found_type = self.scope_stack.scopes[i].get_type(var_name).cloned();
                            println!(
                                "Found nonlocal variable '{}' in outer scope {}",
                                var_name, i
                            );
                            break;
                        }
                    }
                }

                if let (Some(ptr), Some(var_type)) = (found_ptr, found_type) {
                    self.add_to_current_environment(var_name.clone(), ptr, var_type.clone());
                    println!(
                        "Added nonlocal variable '{}' to closure environment",
                        var_name
                    );

                    let function_name = name;
                    let unique_name = format!(
                        "__nonlocal_{}_{}",
                        function_name.replace('.', "_"),
                        var_name
                    );

                    self.scope_stack
                        .add_nonlocal_mapping(var_name.clone(), unique_name.clone());

                    let current_position = self.builder.get_insert_block().unwrap();

                    let entry_block = function.get_first_basic_block().unwrap();
                    if let Some(first_instr) = entry_block.get_first_instruction() {
                        self.builder.position_before(&first_instr);
                    } else {
                        self.builder.position_at_end(entry_block);
                    }

                    let llvm_type = self.get_llvm_type(&var_type);
                    let local_ptr = self.builder.build_alloca(llvm_type, &unique_name).unwrap();

                    self.builder.position_at_end(current_position);

                    let default_value: inkwell::values::BasicValueEnum<'ctx> = match var_type {
                        Type::Int => self.llvm_context.i64_type().const_int(0, false).into(),
                        Type::Float => self.llvm_context.f64_type().const_float(0.0).into(),
                        Type::Bool => self.llvm_context.bool_type().const_int(0, false).into(),
                        Type::String => self
                            .llvm_context
                            .ptr_type(inkwell::AddressSpace::default())
                            .const_null()
                            .into(),
                        _ => self.llvm_context.i64_type().const_int(0, false).into(),
                    };
                    self.builder.build_store(local_ptr, default_value).unwrap();
                    println!(
                        "Initialized nonlocal variable '{}' with default value",
                        unique_name
                    );

                    self.add_variable_to_scope(unique_name.clone(), local_ptr, var_type.clone());

                    local_vars.insert(unique_name.clone(), local_ptr);

                    self.register_variable(unique_name.clone(), var_type.clone());
                } else {
                    return Err(format!(
                        "Nonlocal variable '{}' not found in outer scopes",
                        var_name
                    ));
                }
            }

            let env_struct_ptr = self.allocate_closure_environment(name)?;

            self.builder
                .build_store(env_alloca, env_struct_ptr)
                .unwrap();

            for var_name in &nonlocal_vars {
                if let Some(current_scope) = self.scope_stack.current_scope() {
                    if let Some(unique_name) = current_scope.get_nonlocal_mapping(var_name) {
                        let env = self.get_closure_environment(name).unwrap();

                        if let Some(index) = env.get_index(var_name) {
                            let var_type = env.get_type(var_name).unwrap();

                            let struct_type = env.env_type.unwrap();

                            let _field_ptr = self
                                .builder
                                .build_struct_gep(
                                    struct_type,
                                    env_struct_ptr,
                                    index,
                                    &format!("env_{}_ptr", var_name),
                                )
                                .unwrap();

                            let default_value: inkwell::values::BasicValueEnum<'ctx> =
                                match var_type {
                                    Type::Int => self.llvm_context.i64_type().const_zero().into(),
                                    Type::Float => self.llvm_context.f64_type().const_zero().into(),
                                    Type::Bool => self.llvm_context.bool_type().const_zero().into(),
                                    Type::String => self
                                        .llvm_context
                                        .ptr_type(inkwell::AddressSpace::default())
                                        .const_null()
                                        .into(),
                                    Type::List(_) => self
                                        .llvm_context
                                        .ptr_type(inkwell::AddressSpace::default())
                                        .const_null()
                                        .into(),
                                    Type::Tuple(_) => self
                                        .llvm_context
                                        .ptr_type(inkwell::AddressSpace::default())
                                        .const_null()
                                        .into(),
                                    Type::Dict(_, _) => self
                                        .llvm_context
                                        .ptr_type(inkwell::AddressSpace::default())
                                        .const_null()
                                        .into(),
                                    Type::Set(_) => self
                                        .llvm_context
                                        .ptr_type(inkwell::AddressSpace::default())
                                        .const_null()
                                        .into(),
                                    _ => self
                                        .llvm_context
                                        .ptr_type(inkwell::AddressSpace::default())
                                        .const_null()
                                        .into(),
                                };

                            if let Some(local_ptr) = self.get_variable_ptr(unique_name) {
                                self.builder.build_store(local_ptr, default_value).unwrap();
                                println!(
                                    "Initialized nonlocal variable '{}' with default value",
                                    var_name
                                );
                            }
                        }
                    }
                }
            }
        }

        let old_function = self.current_function;
        let old_local_vars = std::mem::replace(&mut self.local_vars, local_vars);

        self.current_function = Some(function);

        for stmt in body {
            self.compile_stmt(stmt.as_ref())?;
        }

        if !self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_some()
        {
            let zero = context.i64_type().const_int(0, false);
            self.builder.build_return(Some(&zero)).unwrap();
        }

        self.current_function = old_function;
        self.local_vars = old_local_vars;

        self.current_environment = None;

        self.pop_scope();

        if let Some(block) = current_block {
            self.builder.position_at_end(block);
        }

        Ok(())
    }

    /// Create a new closure environment for a function
    pub fn create_closure_environment(&mut self, function_name: &str) {
        let env = ClosureEnvironment::new(function_name.to_string());
        self.closure_environments
            .insert(function_name.to_string(), env);
    }

    /// Allocate a closure environment on the heap
    pub fn allocate_closure_environment(
        &mut self,
        function_name: &str,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        if !self.closure_environments.contains_key(function_name) {
            return Err(format!(
                "Closure environment for function '{}' not found",
                function_name
            ));
        }

        let context = self.llvm_context;
        let is_empty = {
            let env = self.closure_environments.get_mut(function_name).unwrap();
            env.finalize(context);
            env.is_empty()
        };

        if is_empty {
            return Ok(self
                .llvm_context
                .ptr_type(inkwell::AddressSpace::default())
                .const_null());
        }

        let env = self.get_closure_environment(function_name).unwrap();

        let struct_type = match env.env_type {
            Some(ty) => ty,
            None => {
                return Err(format!(
                    "Struct type for environment of function '{}' not created",
                    function_name
                ))
            }
        };

        let malloc_fn = self.get_or_create_malloc_function();

        let size = struct_type.size_of().unwrap();

        let call = self
            .builder
            .build_call(malloc_fn, &[size.into()], "env_malloc")
            .unwrap();

        let env_ptr = call
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let env_struct_ptr = self
            .builder
            .build_pointer_cast(
                env_ptr,
                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                "env_struct_ptr",
            )
            .unwrap();

        let mut vars = Vec::new();
        {
            let env = self.get_closure_environment(function_name).unwrap();
            for (name, &index) in &env.var_indices {
                if let (Some(ptr), Some(ty)) = (env.get_variable(name), env.get_type(name)) {
                    vars.push((name.clone(), index, *ptr, ty.clone()));
                }
            }
        }

        vars.sort_by_key(|&(_, index, _, _)| index);

        for (var_name, index, _var_ptr, var_type) in vars {
            let field_ptr = self
                .builder
                .build_struct_gep(
                    struct_type,
                    env_struct_ptr,
                    index,
                    &format!("env_{}_ptr", var_name),
                )
                .unwrap();

            let default_value: inkwell::values::BasicValueEnum<'ctx> = match var_type {
                Type::Int => self.llvm_context.i64_type().const_zero().into(),
                Type::Float => self.llvm_context.f64_type().const_zero().into(),
                Type::Bool => self.llvm_context.bool_type().const_zero().into(),
                Type::String => self
                    .llvm_context
                    .ptr_type(inkwell::AddressSpace::default())
                    .const_null()
                    .into(),
                Type::List(_) => self
                    .llvm_context
                    .ptr_type(inkwell::AddressSpace::default())
                    .const_null()
                    .into(),
                Type::Tuple(_) => self
                    .llvm_context
                    .ptr_type(inkwell::AddressSpace::default())
                    .const_null()
                    .into(),
                Type::Dict(_, _) => self
                    .llvm_context
                    .ptr_type(inkwell::AddressSpace::default())
                    .const_null()
                    .into(),
                Type::Set(_) => self
                    .llvm_context
                    .ptr_type(inkwell::AddressSpace::default())
                    .const_null()
                    .into(),
                _ => self
                    .llvm_context
                    .ptr_type(inkwell::AddressSpace::default())
                    .const_null()
                    .into(),
            };

            self.builder.build_store(field_ptr, default_value).unwrap();
        }

        {
            let env = match self.get_closure_environment_mut(function_name) {
                Some(env) => env,
                None => {
                    return Err(format!(
                        "Closure environment for function '{}' not found",
                        function_name
                    ))
                }
            };
            env.env_ptr = Some(env_struct_ptr);
        }

        Ok(env_struct_ptr)
    }

    /// Add a variable to the current closure environment
    pub fn add_to_current_environment(
        &mut self,
        name: String,
        ptr: inkwell::values::PointerValue<'ctx>,
        ty: Type,
    ) {
        // Check if we're using BoxedAny values
        let use_boxed = self.use_boxed_values;

        if let Some(env_name) = self.current_environment.clone() {
            if let Some(env) = self.get_closure_environment_mut(&env_name) {
                // If we're using BoxedAny values, convert the type to Any
                if use_boxed {
                    env.add_variable(name, ptr, Type::Any);
                } else {
                    env.add_variable(name, ptr, ty);
                }
            }
        }
    }

    /// Get a closure environment by function name
    pub fn get_closure_environment(
        &self,
        function_name: &str,
    ) -> Option<&ClosureEnvironment<'ctx>> {
        self.closure_environments.get(function_name)
    }

    /// Get a mutable reference to a closure environment by function name
    pub fn get_closure_environment_mut(
        &mut self,
        function_name: &str,
    ) -> Option<&mut ClosureEnvironment<'ctx>> {
        self.closure_environments.get_mut(function_name)
    }

    /// Set the current closure environment
    pub fn set_current_environment(&mut self, function_name: String) {
        self.current_environment = Some(function_name);
    }

    /// Get the current closure environment (if any)
    pub fn current_environment(&self) -> Option<&ClosureEnvironment<'ctx>> {
        if let Some(name) = &self.current_environment {
            self.get_closure_environment(name)
        } else {
            None
        }
    }

    /// Get a mutable reference to the current closure environment (if any)
    pub fn current_environment_mut(&mut self) -> Option<&mut ClosureEnvironment<'ctx>> {
        if let Some(name) = self.current_environment.clone() {
            self.get_closure_environment_mut(&name)
        } else {
            None
        }
    }

    /// Get or create the malloc function
    fn get_or_create_malloc_function(&self) -> inkwell::values::FunctionValue<'ctx> {
        if let Some(malloc_fn) = self.module.get_function("malloc") {
            return malloc_fn;
        }

        let malloc_type = self
            .llvm_context
            .ptr_type(inkwell::AddressSpace::default())
            .fn_type(&[self.llvm_context.i64_type().into()], false);

        let malloc_fn = self.module.add_function("malloc", malloc_type, None);

        malloc_fn
    }

    /// Allocate a variable on the heap
    pub fn allocate_heap_variable(
        &mut self,
        name: &str,
        ty: &Type,
    ) -> inkwell::values::PointerValue<'ctx> {
        let _llvm_type = self.get_llvm_type(ty);

        let size_val = self.llvm_context.i64_type().const_int(8, false);

        let malloc_fn = self.get_or_create_malloc_function();
        let heap_ptr = self
            .builder
            .build_call(malloc_fn, &[size_val.into()], &format!("malloc_{}", name))
            .unwrap();
        let heap_ptr = heap_ptr
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
        let typed_ptr = self
            .builder
            .build_bit_cast(heap_ptr, ptr_type, &format!("{}_ptr", name))
            .unwrap();

        typed_ptr.into_pointer_value()
    }
}
