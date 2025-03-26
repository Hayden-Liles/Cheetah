use std::collections::HashMap;
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, PointerValue, BasicValueEnum};
use inkwell::types::{BasicTypeEnum, StructType};
use crate::symtable::{Scope, Symbol};
use super::error::CodegenError;

/// A variable entry in the compilation symbol table
pub struct VariableInfo {
    pub ptr: PointerValue<'static>,
    pub ty: BasicTypeEnum<'static>,
    pub is_mutable: bool,
}

/// A function entry in the compilation symbol table
pub struct FunctionInfo {
    pub function: FunctionValue<'static>,
    pub return_type: Option<BasicTypeEnum<'static>>,
}

/// A type entry in the compilation symbol table
pub struct TypeInfo {
    pub llvm_type: BasicTypeEnum<'static>,
    pub struct_type: Option<StructType<'static>>,
    pub methods: HashMap<String, FunctionInfo>,
}

/// The compilation context that holds all necessary information
/// during code generation
pub struct CompilationContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    
    // Symbol tables
    pub variables: HashMap<String, VariableInfo>,
    pub functions: HashMap<String, FunctionInfo>,
    pub types: HashMap<String, TypeInfo>,
    
    // Current function being compiled
    pub current_function: Option<FunctionValue<'ctx>>,
    
    // Block stack for handling branches, loops, etc.
    pub continue_block_stack: Vec<inkwell::basic_block::BasicBlock<'ctx>>,
    pub break_block_stack: Vec<inkwell::basic_block::BasicBlock<'ctx>>,
    
    // Source scope information
    pub source_scope: Option<Scope>,
}

impl<'ctx> CompilationContext<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        CompilationContext {
            context,
            module,
            builder,
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            current_function: None,
            continue_block_stack: Vec::new(),
            break_block_stack: Vec::new(),
            source_scope: None,
        }
    }
    
    // Look up a variable in the current context
    pub fn lookup_variable(&self, name: &str) -> Result<&VariableInfo, CodegenError> {
        self.variables.get(name)
            .ok_or_else(|| CodegenError::undefined_symbol(name))
    }
    
    // Look up a function in the current context
    pub fn lookup_function(&self, name: &str) -> Result<&FunctionInfo, CodegenError> {
        self.functions.get(name)
            .ok_or_else(|| CodegenError::undefined_symbol(name))
    }
    
    // Register a new variable
    pub fn register_variable(&mut self, name: &str, var_info: VariableInfo) {
        self.variables.insert(name.to_string(), var_info);
    }
    
    // Register a new function
    pub fn register_function(&mut self, name: &str, func_info: FunctionInfo) {
        self.functions.insert(name.to_string(), func_info);
    }
    
    // Get the current function
    pub fn current_function(&self) -> Result<FunctionValue<'ctx>, CodegenError> {
        self.current_function.ok_or_else(|| 
            CodegenError::internal_error("No current function in context")
        )
    }
    
    // Enter a new block with break/continue points
    pub fn enter_loop(&mut self, continue_block: inkwell::basic_block::BasicBlock<'ctx>,
                     break_block: inkwell::basic_block::BasicBlock<'ctx>) {
        self.continue_block_stack.push(continue_block);
        self.break_block_stack.push(break_block);
    }
    
    // Exit the current loop
    pub fn exit_loop(&mut self) {
        self.continue_block_stack.pop();
        self.break_block_stack.pop();
    }
    
    // Get the current continue block
    pub fn continue_block(&self) -> Result<inkwell::basic_block::BasicBlock<'ctx>, CodegenError> {
        self.continue_block_stack.last()
            .copied()
            .ok_or_else(|| CodegenError::internal_error("No continue block available"))
    }
    
    // Get the current break block
    pub fn break_block(&self) -> Result<inkwell::basic_block::BasicBlock<'ctx>, CodegenError> {
        self.break_block_stack.last()
            .copied()
            .ok_or_else(|| CodegenError::internal_error("No break block available"))
    }
}