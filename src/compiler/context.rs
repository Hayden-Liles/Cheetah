use std::collections::HashMap;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::BasicValueEnum;
use inkwell::basic_block::BasicBlock;
use crate::compiler::types::Type;
use crate::compiler::types::is_reference_type;
use crate::compiler::scope::ScopeStack;

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
}

impl<'ctx> CompilationContext<'ctx> {
    /// Create a new compilation context
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

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
        }
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
        // First check the scope stack
        if let Some(ty) = self.scope_stack.get_type(name) {
            return Some(ty);
        }

        // Then check the global type environment
        self.type_env.get(name)
    }

    /// Register a class type
    pub fn register_class(&mut self, name: String, fields: HashMap<String, Type>) {
        let ty = Type::Class {
            name: name.clone(),
            base_classes: vec![],
            methods: HashMap::new(),
            fields: fields.clone()
        };

        // Create the LLVM struct type for this class
        if let Type::Class { ref name, .. } = ty {
            let struct_type = ty.create_class_type(self.llvm_context, name, &fields);
            self.class_types.insert(name.clone(), struct_type);
        }

        self.type_env.insert(name, ty);
    }

    pub fn declare_variable(&mut self, name: String, init_value: BasicValueEnum<'ctx>,
                           value_type: &Type) -> Result<(), String> {
        // Allocate storage for the variable
        let ptr = self.allocate_variable(name.clone(), value_type);

        // Store the initial value
        self.builder.build_store(ptr, init_value).unwrap();

        // Add the variable to the current scope
        self.scope_stack.add_variable(name, ptr, value_type.clone());

        Ok(())
    }

    pub fn allocate_variable(&mut self, name: String, ty: &Type) -> inkwell::values::PointerValue<'ctx> {
        let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let entry_bb = current_function.get_first_basic_block().unwrap();

        // Position at the beginning of the function for allocations
        let current_position = self.builder.get_insert_block().unwrap();
        if let Some(first_instr) = entry_bb.get_first_instruction() {
            self.builder.position_before(&first_instr);
        } else {
            self.builder.position_at_end(entry_bb);
        }

        // Create the alloca instruction
        let llvm_type = self.get_llvm_type(ty);
        let ptr = self.builder.build_alloca(llvm_type, &name).unwrap();

        // Restore the original position
        self.builder.position_at_end(current_position);

        // Store the variable's storage location
        self.variables.insert(name.clone(), ptr);

        // Register the variable type if not already present
        if !self.type_env.contains_key(&name) {
            self.register_variable(name, ty.clone());
        }

        ptr
    }

    /// Get the storage location for a variable
    pub fn get_variable_ptr(&self, name: &str) -> Option<inkwell::values::PointerValue<'ctx>> {
        // First check the scope stack
        if let Some(ptr) = self.scope_stack.get_variable(name) {
            return Some(*ptr);
        }

        // For backward compatibility, check local variables
        if let Some(&ptr) = self.local_vars.get(name) {
            return Some(ptr);
        }

        // Then check global variables
        self.variables.get(name).copied()
    }

    /// Ensure a variable exists in the current scope or create it if it's a global variable
    pub fn ensure_variable(&mut self, name: &str) -> Option<inkwell::values::PointerValue<'ctx>> {
        // First try to get the variable from existing storage
        if let Some(ptr) = self.get_variable_ptr(name) {
            return Some(ptr);
        }

        // If not found, check if it's a global variable that needs to be created
        if self.type_env.contains_key(name) {
            // This is a global variable that exists in the type environment but not in the variables map
            // We need to allocate it
            let ty = self.type_env.get(name).unwrap().clone();
            let ptr = self.allocate_variable(name.to_string(), &ty);
            return Some(ptr);
        }

        None
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

    /// Convert a value from one type to another
    pub fn convert_type(&self, value: inkwell::values::BasicValueEnum<'ctx>,
                        from_type: &Type, to_type: &Type)
                        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        // If types are the same, no conversion needed
        if from_type == to_type {
            return Ok(value);
        }

        // Check if conversion is valid using existing can_coerce_to method
        if !from_type.can_coerce_to(to_type) {
            return Err(format!("Cannot convert from {:?} to {:?}", from_type, to_type));
        }

        // Handle different type conversions
        match (from_type, to_type) {
            // Bool to Int
            (Type::Bool, Type::Int) => {
                let bool_val = value.into_int_value();
                // A boolean in LLVM is i1, so we need to zero-extend it to i64
                let int_val = self.builder.build_int_z_extend(
                    bool_val,
                    self.llvm_context.i64_type(),
                    "bool_to_int"
                ).unwrap();
                Ok(int_val.into())
            },

            // Int to Bool
            (Type::Int, Type::Bool) => {
                let int_val = value.into_int_value();
                let zero = self.llvm_context.i64_type().const_zero();
                let bool_val = self.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    int_val,
                    zero,
                    "int_to_bool"
                ).unwrap();
                Ok(bool_val.into())
            },

            // Int to Float
            (Type::Int, Type::Float) => {
                let int_val = value.into_int_value();
                let float_val = self.builder.build_signed_int_to_float(
                    int_val,
                    self.llvm_context.f64_type(),
                    "int_to_float"
                ).unwrap();
                Ok(float_val.into())
            },

            // Float to Int (truncating conversion)
            (Type::Float, Type::Int) => {
                let float_val = value.into_float_value();
                let int_val = self.builder.build_float_to_signed_int(
                    float_val,
                    self.llvm_context.i64_type(),
                    "float_to_int"
                ).unwrap();
                Ok(int_val.into())
            },

            // Bool to Float
            (Type::Bool, Type::Float) => {
                // First convert bool to int
                let bool_val = value.into_int_value();
                let int_val = self.builder.build_int_z_extend(
                    bool_val,
                    self.llvm_context.i64_type(),
                    "bool_to_int"
                ).unwrap();

                // Then convert int to float
                let float_val = self.builder.build_signed_int_to_float(
                    int_val,
                    self.llvm_context.f64_type(),
                    "int_to_float"
                ).unwrap();
                Ok(float_val.into())
            },

            // Float to Bool
            (Type::Float, Type::Bool) => {
                let float_val = value.into_float_value();
                let zero = self.llvm_context.f64_type().const_float(0.0);
                let bool_val = self.builder.build_float_compare(
                    inkwell::FloatPredicate::ONE,
                    float_val,
                    zero,
                    "float_to_bool"
                ).unwrap();
                Ok(bool_val.into())
            },

            // None to any reference type (resulting in a null pointer)
            (Type::None, _) if is_reference_type(to_type) => {
                // Create a null pointer of the appropriate type
                let ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                let null_ptr = ptr_type.const_null();
                Ok(null_ptr.into())
            },

            // Numeric types to String (requires runtime support)
            (Type::Int, Type::String) => {
                // Call a runtime function to convert int to string
                self.build_int_to_string_call(value.into_int_value())
            },

            (Type::Float, Type::String) => {
                // Call a runtime function to convert float to string
                self.build_float_to_string_call(value.into_float_value())
            },

            (Type::Bool, Type::String) => {
                // Call a runtime function to convert bool to string
                self.build_bool_to_string_call(value.into_int_value())
            },

            // String to numeric types (requires runtime support)
            (Type::String, Type::Int) => {
                // Call a runtime function to parse string as int
                self.build_string_to_int_call(value.into_pointer_value())
            },

            (Type::String, Type::Float) => {
                // Call a runtime function to parse string as float
                self.build_string_to_float_call(value.into_pointer_value())
            },

            (Type::String, Type::Bool) => {
                // Call a runtime function to parse string as bool
                self.build_string_to_bool_call(value.into_pointer_value())
            },

            // Other cases (collections, complex types, etc.)
            _ => Err(format!("Unsupported type conversion from {:?} to {:?}", from_type, to_type)),
        }
    }

    /// Helper method to get the common type for binary operations
    pub fn get_common_type(&self, type1: &Type, type2: &Type) -> Result<Type, String> {
        if type1 == type2 {
            return Ok(type1.clone());
        }

        if type1.can_coerce_to(type2) {
            return Ok(type2.clone());
        }

        if type2.can_coerce_to(type1) {
            return Ok(type1.clone());
        }

        // Special cases for numeric types
        match (type1, type2) {
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
            (Type::Int, Type::Bool) | (Type::Bool, Type::Int) => Ok(Type::Int),
            (Type::Float, Type::Bool) | (Type::Bool, Type::Float) => Ok(Type::Float),
            // Add more special cases if needed
            _ => Err(format!("No common type for {:?} and {:?}", type1, type2)),
        }
    }

    // Placeholder methods for string conversions (to be implemented with runtime support)

    fn build_int_to_string_call(&self, int_val: inkwell::values::IntValue<'ctx>)
        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {

        // Get or create the int_to_string function
        let int_to_string_fn = self.module.get_function("int_to_string").unwrap_or_else(|| {
            // Define the function signature: int_to_string(int) -> string*
            let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[self.llvm_context.i64_type().into()], false);
            self.module.add_function("int_to_string", fn_type, None)
        });

        // Build the function call
        let result = self.builder.build_call(
            int_to_string_fn,
            &[int_val.into()],
            "int_to_string_result"
        ).unwrap();

        // Extract the return value (string pointer)
        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call int_to_string function".to_string())
        }
    }

    fn build_float_to_string_call(&self, float_val: inkwell::values::FloatValue<'ctx>)
        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {

        // Get or create the float_to_string function
        let float_to_string_fn = self.module.get_function("float_to_string").unwrap_or_else(|| {
            // Define the function signature: float_to_string(float) -> string*
            let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[self.llvm_context.f64_type().into()], false);
            self.module.add_function("float_to_string", fn_type, None)
        });

        // Build the function call
        let result = self.builder.build_call(
            float_to_string_fn,
            &[float_val.into()],
            "float_to_string_result"
        ).unwrap();

        // Extract the return value
        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call float_to_string function".to_string())
        }
    }

    fn build_bool_to_string_call(&self, bool_val: inkwell::values::IntValue<'ctx>)
    -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {

        // Get or create the bool_to_string function
        let bool_to_string_fn = self.module.get_function("bool_to_string").unwrap_or_else(|| {
            // Define the function signature: bool_to_string(i64) -> string*
            // We'll use i64 instead of bool to avoid type conversion issues
            let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = str_ptr_type.fn_type(&[self.llvm_context.i64_type().into()], false);
            self.module.add_function("bool_to_string", fn_type, None)
        });

        // Build the function call
        let result = self.builder.build_call(
            bool_to_string_fn,
            &[bool_val.into()],
            "bool_to_string_result"
        ).unwrap();

        // Extract the return value
        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call bool_to_string function".to_string())
        }
    }


    fn build_string_to_int_call(&self, string_ptr: inkwell::values::PointerValue<'ctx>)
        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {

        // Get or create the string_to_int function
        let string_to_int_fn = self.module.get_function("string_to_int").unwrap_or_else(|| {
            // Define the function signature: string_to_int(string*) -> int
            let i64_type = self.llvm_context.i64_type();
            let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = i64_type.fn_type(&[str_ptr_type.into()], false);
            self.module.add_function("string_to_int", fn_type, None)
        });

        // Build the function call
        let result = self.builder.build_call(
            string_to_int_fn,
            &[string_ptr.into()],
            "string_to_int_result"
        ).unwrap();

        // Extract the return value
        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call string_to_int function".to_string())
        }
    }

    fn build_string_to_float_call(&self, string_ptr: inkwell::values::PointerValue<'ctx>)
        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {

        // Get or create the string_to_float function
        let string_to_float_fn = self.module.get_function("string_to_float").unwrap_or_else(|| {
            // Define the function signature: string_to_float(string*) -> float
            let f64_type = self.llvm_context.f64_type();
            let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = f64_type.fn_type(&[str_ptr_type.into()], false);
            self.module.add_function("string_to_float", fn_type, None)
        });

        // Build the function call
        let result = self.builder.build_call(
            string_to_float_fn,
            &[string_ptr.into()],
            "string_to_float_result"
        ).unwrap();

        // Extract the return value
        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call string_to_float function".to_string())
        }
    }

    fn build_string_to_bool_call(&self, string_ptr: inkwell::values::PointerValue<'ctx>)
        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {

        // Get or create the string_to_bool function
        let string_to_bool_fn = self.module.get_function("string_to_bool").unwrap_or_else(|| {
            // Define the function signature: string_to_bool(string*) -> bool
            let bool_type = self.llvm_context.bool_type();
            let str_ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
            let fn_type = bool_type.fn_type(&[str_ptr_type.into()], false);
            self.module.add_function("string_to_bool", fn_type, None)
        });

        // Build the function call
        let result = self.builder.build_call(
            string_to_bool_fn,
            &[string_ptr.into()],
            "string_to_bool_result"
        ).unwrap();

        // Extract the return value
        if let Some(ret_val) = result.try_as_basic_value().left() {
            Ok(ret_val)
        } else {
            Err("Failed to call string_to_bool function".to_string())
        }
    }

    pub fn get_polymorphic_function(&self, name: &str, arg_type: &Type) -> Option<inkwell::values::FunctionValue<'ctx>> {
        if let Some(variants) = self.polymorphic_functions.get(name) {
            // Try to get exact match first
            if let Some(&func) = variants.get(arg_type) {
                return Some(func);
            }

            // If no exact match, try to find a compatible type
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
    pub fn add_variable_to_scope(&mut self, name: String, ptr: inkwell::values::PointerValue<'ctx>, ty: Type) {
        self.scope_stack.add_variable(name, ptr, ty);
    }
}