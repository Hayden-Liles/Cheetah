use std::collections::HashMap;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::BasicValueEnum;
use crate::compiler::types::Type;
use crate::compiler::types::is_reference_type;

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
        let ptr = self.allocate_variable(name, value_type);
        
        // Store the initial value
        self.builder.build_store(ptr, init_value).unwrap();
        
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
        self.variables.get(name).copied()
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
        let _ = int_val;
        // In a complete implementation, this would call a runtime function
        // that converts an integer to a string
        Err("Int to String conversion requires runtime support (not yet implemented)".to_string())
    }
    
    fn build_float_to_string_call(&self, float_val: inkwell::values::FloatValue<'ctx>) 
        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let _ = float_val;
        // In a complete implementation, this would call a runtime function
        // that converts a float to a string
        Err("Float to String conversion requires runtime support (not yet implemented)".to_string())
    }
    
    fn build_bool_to_string_call(&self, bool_val: inkwell::values::IntValue<'ctx>) 
        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let _ = bool_val;
        // In a complete implementation, this would call a runtime function
        // that converts a boolean to a string
        Err("Bool to String conversion requires runtime support (not yet implemented)".to_string())
    }
    
    fn build_string_to_int_call(&self, string_ptr: inkwell::values::PointerValue<'ctx>) 
        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let _ = string_ptr;
        // In a complete implementation, this would call a runtime function
        // that parses a string as an integer
        Err("String to Int conversion requires runtime support (not yet implemented)".to_string())
    }
    
    fn build_string_to_float_call(&self, string_ptr: inkwell::values::PointerValue<'ctx>) 
        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let _ = string_ptr;
        // In a complete implementation, this would call a runtime function
        // that parses a string as a float
        Err("String to Float conversion requires runtime support (not yet implemented)".to_string())
    }
    
    fn build_string_to_bool_call(&self, string_ptr: inkwell::values::PointerValue<'ctx>) 
        -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        let _ = string_ptr;
        // In a complete implementation, this would call a runtime function
        // that parses a string as a boolean
        Err("String to Bool conversion requires runtime support (not yet implemented)".to_string())
    }
}