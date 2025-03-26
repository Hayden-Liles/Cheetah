use inkwell::context::Context;
use inkwell::passes::PassManager;
use inkwell::targets::{
    Target, TargetMachine, CodeModel, RelocMode, InitializationConfig,
    FileType,
};
use std::path::Path;
use std::fs::File;
use std::io::Write;
use crate::ast::Module;
use super::context::CompilationContext;
use super::statements::compile_stmt;
use super::error::CodegenError;
use super::builtins::register_builtins;

pub struct LLVMCompiler {
    context: Context,
    optimization_level: u32,
}

impl LLVMCompiler {
    pub fn new(optimization_level: u32) -> Self {
        // Initialize LLVM
        unsafe {
            Target::initialize_all(&InitializationConfig::default());
        }
        
        LLVMCompiler {
            context: Context::create(),
            optimization_level,
        }
    }
    
    /// Compile an AST into LLVM IR
    pub fn compile(&self, ast: &Module, module_name: &str) -> Result<String, CodegenError> {
        // Create a new compilation context
        let mut context = CompilationContext::new(&self.context, module_name);
        
        // Register built-in functions
        register_builtins(&mut context)?;
        
        // Compile each statement
        for stmt in &ast.body {
            compile_stmt(&mut context, stmt)?;
        }
        
        // Run optimization passes
        if self.optimization_level > 0 {
            let pass_manager = PassManager::create(());
            
            // Basic optimizations
            pass_manager.add_instruction_combining_pass();
            pass_manager.add_reassociate_pass();
            pass_manager.add_gvn_pass();
            pass_manager.add_cfg_simplification_pass();
            pass_manager.add_basic_alias_analysis_pass();
            pass_manager.add_promote_memory_to_register_pass();
            pass_manager.add_instruction_combining_pass();
            pass_manager.add_reassociate_pass();
            
            // More aggressive optimizations for higher levels
            if self.optimization_level >= 2 {
                pass_manager.add_tail_call_elimination_pass();
                pass_manager.add_function_inlining_pass();
                pass_manager.add_sccp_pass();
                pass_manager.add_dead_arg_elimination_pass();
            }
            
            pass_manager.run_on(&context.module);
        }
        
        // Return the LLVM IR as a string
        Ok(context.module.print_to_string().to_string())
    }
    
    /// Compile and save as object file
    pub fn compile_to_obj(&self, ast: &Module, module_name: &str, 
                         output_path: &Path) -> Result<(), CodegenError> {
        // Create a new compilation context
        let mut context = CompilationContext::new(&self.context, module_name);
        
        // Register built-in functions
        register_builtins(&mut context)?;
        
        // Compile each statement
        for stmt in &ast.body {
            compile_stmt(&mut context, stmt)?;
        }
        
        // Get the target machine for the host
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| CodegenError::internal_error(&e.to_string()))?;
        
        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                inkwell::OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| CodegenError::internal_error("Failed to create target machine"))?;
        
        // Write the object file
        target_machine
            .write_to_file(&context.module, FileType::Object, output_path)
            .map_err(|e| CodegenError::internal_error(&e.to_string()))?;
            
        Ok(())
    }
    
    /// Compile and run JIT
    pub fn compile_and_run(&self, ast: &Module, module_name: &str) -> Result<(), CodegenError> {
        use inkwell::execution_engine::ExecutionEngine;
        
        // Create a new compilation context
        let mut context = CompilationContext::new(&self.context, module_name);
        
        // Register built-in functions
        register_builtins(&mut context)?;
        
        // Compile each statement
        for stmt in &ast.body {
            compile_stmt(&mut context, stmt)?;
        }
        
        // Create execution engine
        let execution_engine = context.module
            .create_jit_execution_engine(inkwell::OptimizationLevel::Default)
            .map_err(|e| CodegenError::internal_error(&e.to_string()))?;
        
        // Look for main function
        let main_fn = context.module.get_function("main")
            .ok_or_else(|| CodegenError::function_error("No main function found"))?;
        
        // Run the main function
        unsafe {
            execution_engine.run_function(main_fn, &[])
                .map_err(|e| CodegenError::internal_error(&format!("JIT execution failed: {:?}", e)))?;
        }
        
        Ok(())
    }
}