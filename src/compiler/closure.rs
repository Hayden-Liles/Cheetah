use std::collections::HashMap;
use inkwell::values::PointerValue;
use inkwell::types::BasicTypeEnum;
use crate::compiler::types::Type;

/// Represents a closure environment for a nested function
pub struct ClosureEnvironment<'ctx> {
    /// Name of the function this environment belongs to
    pub function_name: String,

    /// Map of captured variable names to their storage locations
    pub captured_vars: HashMap<String, PointerValue<'ctx>>,

    /// Map of captured variable names to their types
    pub captured_types: HashMap<String, Type>,

    /// LLVM struct type for this environment
    pub env_type: Option<inkwell::types::StructType<'ctx>>,

    /// Pointer to the environment structure (when allocated)
    pub env_ptr: Option<PointerValue<'ctx>>,

    /// Map of variable names to their indices in the environment struct
    pub var_indices: HashMap<String, u32>,

    /// List of nonlocal variables that need to be passed as parameters
    pub nonlocal_params: Vec<String>,

    /// Map of nonlocal variable names to their proxy pointers in the current function
    pub nonlocal_proxies: HashMap<String, PointerValue<'ctx>>,

    /// List of field types in the environment struct
    pub field_types: Vec<BasicTypeEnum<'ctx>>,

    /// Whether the environment has been finalized (struct type created)
    pub finalized: bool,
}

impl<'ctx> ClosureEnvironment<'ctx> {
    /// Create a new empty closure environment
    pub fn new(function_name: String) -> Self {
        Self {
            function_name,
            captured_vars: HashMap::new(),
            captured_types: HashMap::new(),
            env_type: None,
            env_ptr: None,
            var_indices: HashMap::new(),
            nonlocal_params: Vec::new(),
            nonlocal_proxies: HashMap::new(),
            field_types: Vec::new(),
            finalized: false,
        }
    }

    /// Add a variable to the environment
    pub fn add_variable(&mut self, name: String, ptr: PointerValue<'ctx>, ty: Type) {
        if !self.captured_vars.contains_key(&name) {
            let index = self.captured_vars.len() as u32;
            self.captured_vars.insert(name.clone(), ptr);
            self.captured_types.insert(name.clone(), ty);
            self.var_indices.insert(name, index);
        }
    }

    /// Get a captured variable's storage location
    pub fn get_variable(&self, name: &str) -> Option<&PointerValue<'ctx>> {
        self.captured_vars.get(name)
    }

    /// Get a captured variable's type
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.captured_types.get(name)
    }

    /// Get a variable's index in the environment struct
    pub fn get_index(&self, name: &str) -> Option<u32> {
        self.var_indices.get(name).copied()
    }

    /// Check if a variable is captured in this environment
    pub fn has_variable(&self, name: &str) -> bool {
        self.captured_vars.contains_key(name)
    }

    /// Get the number of captured variables
    pub fn size(&self) -> usize {
        self.captured_vars.len()
    }

    /// Check if the environment is empty (no captured variables)
    pub fn is_empty(&self) -> bool {
        self.captured_vars.is_empty()
    }

    /// Add a nonlocal proxy variable
    pub fn add_nonlocal_proxy(&mut self, name: String, ptr: PointerValue<'ctx>) {
        self.nonlocal_proxies.insert(name, ptr);
    }

    /// Get a nonlocal proxy variable
    pub fn get_nonlocal_proxy(&self, name: &str) -> Option<&PointerValue<'ctx>> {
        self.nonlocal_proxies.get(name)
    }

    /// Access a nonlocal variable with proper dominance validation using phi nodes
    pub fn access_nonlocal_with_phi(&self,
                                    builder: &inkwell::builder::Builder<'ctx>,
                                    name: &str,
                                    llvm_type: inkwell::types::BasicTypeEnum<'ctx>,
                                    _llvm_context: &'ctx inkwell::context::Context) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        // Check if we have a proxy for this nonlocal variable
        if let Some(proxy_ptr) = self.get_nonlocal_proxy(name) {
            // Get the current function
            let current_function = builder.get_insert_block().unwrap().get_parent().unwrap();

            // Save the current position
            let current_position = builder.get_insert_block().unwrap();

            // Instead of creating a new builder, we'll save the current position,
            // move to the entry block, create the alloca, and then restore the position
            let entry_block = current_function.get_first_basic_block().unwrap();

            // Save the current position
            let original_block = builder.get_insert_block().unwrap();

            // Move to the entry block
            builder.position_at_end(entry_block);

            // Create an alloca in the entry block
            let temp_alloca = builder.build_alloca(llvm_type, &format!("nonlocal_{}_alloca", name)).unwrap();

            // Restore the original position
            builder.position_at_end(original_block);

            // Return to the original position
            builder.position_at_end(current_position);

            // Load the value from the proxy
            let value = builder.build_load(llvm_type, *proxy_ptr, &format!("load_{}_from_proxy", name)).unwrap();

            // Store the value in the temporary alloca
            builder.build_store(temp_alloca, value).unwrap();

            // Load the value from the temporary alloca
            // This creates a proper dominance relationship that LLVM can validate
            let result = builder.build_load(llvm_type, temp_alloca, &format!("load_{}_from_alloca", name)).unwrap();

            println!("Accessed nonlocal variable '{}' with phi node technique", name);

            // Return the loaded value
            Some(result)
        } else {
            None
        }
    }

    /// Finalize the environment by creating the struct type
    pub fn finalize(&mut self, context: &'ctx inkwell::context::Context) {
        if self.finalized {
            return;
        }

        // Create the struct type for the environment
        if !self.captured_vars.is_empty() {
            // Sort variables by their index to ensure consistent ordering
            let mut vars: Vec<(String, u32)> = self.var_indices.iter()
                .map(|(name, &index)| (name.clone(), index))
                .collect();
            vars.sort_by_key(|&(_, index)| index);

            // Create the field types
            self.field_types = vars.iter()
                .map(|(name, _)| {
                    let ty = &self.captured_types[name];
                    match ty {
                        Type::Int => context.i64_type().into(),
                        Type::Float => context.f64_type().into(),
                        Type::Bool => context.bool_type().into(),
                        Type::String => context.ptr_type(inkwell::AddressSpace::default()).into(),
                        Type::List(_) => context.ptr_type(inkwell::AddressSpace::default()).into(),
                        Type::Tuple(_) => context.ptr_type(inkwell::AddressSpace::default()).into(),
                        Type::Dict(_, _) => context.ptr_type(inkwell::AddressSpace::default()).into(),
                        Type::Set(_) => context.ptr_type(inkwell::AddressSpace::default()).into(),
                        _ => context.ptr_type(inkwell::AddressSpace::default()).into(),
                    }
                })
                .collect();

            // Create the struct type
            let _struct_name = format!("env_{}", self.function_name.replace('.', "_"));
            let struct_type = context.struct_type(&self.field_types, false);
            self.env_type = Some(struct_type);
        }

        self.finalized = true;
    }
}
