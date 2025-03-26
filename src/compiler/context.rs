use std::collections::HashMap;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use crate::compiler::types::Type;

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
}