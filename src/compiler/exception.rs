// exception.rs - Exception handling for the Cheetah compiler

use crate::ast::{Stmt, Expr, ExceptHandler};
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::ExprCompiler;
use crate::compiler::stmt::StmtCompiler;
use inkwell::values::{BasicValueEnum, PointerValue};

impl<'ctx> CompilationContext<'ctx> {
    /// Compile a try-except-else-finally statement
    pub fn compile_try_stmt(
        &mut self,
        body: &[Box<Stmt>],
        handlers: &[ExceptHandler],
        orelse: &[Box<Stmt>],
        finalbody: &[Box<Stmt>],
    ) -> Result<(), String> {
        // Get the current function
        let function = match self.current_function {
            Some(f) => f,
            None => return Err("Cannot use try statement outside of a function".to_string()),
        };

        // Create basic blocks for the try, except handlers, else, finally, and exit
        let try_block = self.llvm_context.append_basic_block(function, "try");

        // Create blocks for each except handler
        let mut except_blocks = Vec::with_capacity(handlers.len());
        for (i, _) in handlers.iter().enumerate() {
            let block = self.llvm_context.append_basic_block(function, &format!("except_{}", i));
            except_blocks.push(block);
        }

        let else_block = self.llvm_context.append_basic_block(function, "else");
        let finally_block = self.llvm_context.append_basic_block(function, "finally");
        let exit_block = self.llvm_context.append_basic_block(function, "exit");

        // Create a global variable to track if an exception was raised
        let exception_raised = self.create_exception_state();

        // Branch to the try block
        let _ = self.builder.build_unconditional_branch(try_block);

        // Compile the try block
        self.builder.position_at_end(try_block);

        // Reset exception state at the beginning of try block
        self.reset_exception_state(exception_raised);

        // Compile the body of the try block
        let mut has_terminator = false;
        for (i, stmt) in body.iter().enumerate() {
            // Check if the current block already has a terminator
            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                // If it does, create a new block to continue compilation
                let continue_block = self.llvm_context.append_basic_block(function, &format!("continue_try_{}", i));
                self.builder.position_at_end(continue_block);
            }

            self.compile_stmt(stmt.as_ref())?;

            // Check if the statement was a terminator (break, continue, return)
            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                has_terminator = true;
                break;
            }
        }

        // Only add a branch if we don't already have a terminator
        if !has_terminator {
            // If no exception was raised, branch to the else block
            let exception_value = self.load_exception_state(exception_raised);
            let _ = self.builder.build_conditional_branch(
                exception_value,
                except_blocks[0], // If exception raised, go to first except handler
                else_block,       // If no exception, go to else block
            );
        }

        // Compile the except handlers
        for (i, handler) in handlers.iter().enumerate() {
            self.builder.position_at_end(except_blocks[i]);

            // For now, we'll just use a catch-all handler for all exception types
            // This simplifies the implementation and avoids issues with basic block termination
            let matches = self.llvm_context.bool_type().const_int(1, false);

            // Create a block for the handler body
            let handler_body_block = self.llvm_context.append_basic_block(function, &format!("except_body_{}", i));

            // Create a block for the next handler (or finally if this is the last handler)
            let next_block = if i < handlers.len() - 1 {
                except_blocks[i + 1]
            } else {
                finally_block
            };

            // Branch based on whether this handler matches
            let _ = self.builder.build_conditional_branch(matches, handler_body_block, next_block);

            // Compile the handler body
            self.builder.position_at_end(handler_body_block);

            // Make sure all basic blocks have terminators
            // This is important for LLVM validation

            // Make sure we're positioned at the handler body block
            let _current_block = self.builder.get_insert_block().unwrap();

            // If the handler has a name, bind the exception to that name
            if let Some(name) = &handler.name {
                // Get the current exception
                let exception = self.get_current_exception();

                // Allocate space for the exception variable
                let exception_var = self.builder.build_alloca(
                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                    name,
                ).unwrap();

                // Store the exception in the variable
                let _ = self.builder.build_store(exception_var, exception);

                // Add the variable to the current scope
                self.scope_stack.add_variable(name.clone(), exception_var, crate::compiler::types::Type::Any);
            }

            // Compile the handler body
            let mut has_terminator = false;
            for stmt in &handler.body {
                // Check if the current block already has a terminator
                if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                    // If it does, create a new block to continue compilation
                    let continue_block = self.llvm_context.append_basic_block(function, &format!("continue_except_{}", i));
                    self.builder.position_at_end(continue_block);
                }

                self.compile_stmt(stmt.as_ref())?;

                // Check if the statement was a terminator (break, continue, return)
                if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                    has_terminator = true;
                    break;
                }
            }

            // Reset the exception state
            self.reset_exception_state(exception_raised);

            // Clear the current exception in the global state
            if let Some(clear_current_exception_fn) = self.module.get_function("clear_current_exception") {
                let _ = self.builder.build_call(
                    clear_current_exception_fn,
                    &[],
                    "clear_exception_result",
                );
            }

            // Make sure the block has a terminator
            if !has_terminator {
                // If we don't have a terminator, add a branch to the finally block
                let _ = self.builder.build_unconditional_branch(finally_block);
            } else {
                // If we have a terminator (like a return), create a new block to continue
                let continue_block = self.llvm_context.append_basic_block(function, &format!("continue_after_except_{}", i));
                self.builder.position_at_end(continue_block);
                let _ = self.builder.build_unconditional_branch(finally_block);
            }
        }

        // Compile the else block
        self.builder.position_at_end(else_block);

        // Compile the else body
        let mut has_terminator = false;
        for (i, stmt) in orelse.iter().enumerate() {
            // Check if the current block already has a terminator
            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                // If it does, create a new block to continue compilation
                let continue_block = self.llvm_context.append_basic_block(function, &format!("continue_else_{}", i));
                self.builder.position_at_end(continue_block);
            }

            self.compile_stmt(stmt.as_ref())?;

            // Check if the statement was a terminator (break, continue, return)
            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                has_terminator = true;
                break;
            }
        }

        // Make sure the block has a terminator
        if !has_terminator {
            // If we don't have a terminator, add a branch to the finally block
            let _ = self.builder.build_unconditional_branch(finally_block);
        } else {
            // If we have a terminator (like a return), create a new block to continue
            let continue_block = self.llvm_context.append_basic_block(function, "continue_after_else");
            self.builder.position_at_end(continue_block);
            let _ = self.builder.build_unconditional_branch(finally_block);
        }

        // Compile the finally block
        self.builder.position_at_end(finally_block);

        // Compile the finally body
        let mut has_terminator = false;
        for (i, stmt) in finalbody.iter().enumerate() {
            // Check if the current block already has a terminator
            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                // If it does, create a new block to continue compilation
                let continue_block = self.llvm_context.append_basic_block(function, &format!("continue_finally_{}", i));
                self.builder.position_at_end(continue_block);
            }

            self.compile_stmt(stmt.as_ref())?;

            // Check if the statement was a terminator (break, continue, return)
            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                has_terminator = true;
                break;
            }
        }

        // Make sure the block has a terminator
        if !has_terminator {
            // If we don't have a terminator, add a branch to the exit block
            let _ = self.builder.build_unconditional_branch(exit_block);
        } else {
            // If we have a terminator (like a return), create a new block to continue
            let continue_block = self.llvm_context.append_basic_block(function, "continue_after_finally");
            self.builder.position_at_end(continue_block);
            let _ = self.builder.build_unconditional_branch(exit_block);
        }

        // Position at the exit block for further code
        self.builder.position_at_end(exit_block);

        Ok(())
    }

    /// Compile a raise statement
    pub fn compile_raise_stmt(
        &mut self,
        exc: &Option<Box<Expr>>,
        cause: &Option<Box<Expr>>,
    ) -> Result<(), String> {
        // Get the exception_raise function
        let exception_raise_fn = match self.module.get_function("exception_raise") {
            Some(f) => f,
            None => return Err("exception_raise function not found".to_string()),
        };

        // If no exception is provided, re-raise the current exception
        let exception = if let Some(exc_expr) = exc {
            // Compile the exception expression
            let (exc_val, _) = self.compile_expr(exc_expr)?;

            // If it's not already an exception, create a new exception
            if !self.is_exception_type(exc_val) {
                // Convert to string if needed
                let exc_str = self.convert_to_string(exc_val)?;

                // Create a new exception with the string as the message
                self.create_exception("Exception", exc_str)
            } else {
                exc_val.into_pointer_value()
            }
        } else {
            // Re-raise the current exception
            self.get_current_exception()
        };

        // If a cause is provided, set it on the exception
        if let Some(cause_expr) = cause {
            // Compile the cause expression
            let (_cause_val, _) = self.compile_expr(cause_expr)?;

            // Set the cause on the exception
            // This would require additional runtime support
            // For now, we'll just ignore it
        }

        // Call exception_raise to raise the exception
        let _ = self.builder.build_call(
            exception_raise_fn,
            &[exception.into()],
            "raise_result",
        );

        // Set the exception state
        let exception_raised = self.create_exception_state();
        self.set_exception_state(exception_raised, true);

        // Set the current exception in the global state
        if let Some(set_current_exception_fn) = self.module.get_function("set_current_exception") {
            let _ = self.builder.build_call(
                set_current_exception_fn,
                &[exception.into()],
                "set_exception_result",
            );
        }

        Ok(())
    }

    /// Create a global variable to track if an exception was raised
    fn create_exception_state(&self) -> PointerValue<'ctx> {
        // Check if we already have an exception state variable
        if let Some(var) = self.module.get_global("__exception_raised") {
            return var.as_pointer_value();
        }

        // Create a new global variable
        let global = self.module.add_global(
            self.llvm_context.bool_type(),
            None,
            "__exception_raised",
        );

        // Initialize to false
        global.set_initializer(&self.llvm_context.bool_type().const_int(0, false));

        global.as_pointer_value()
    }

    /// Reset the exception state to false
    fn reset_exception_state(&self, exception_raised: PointerValue<'ctx>) {
        let false_val = self.llvm_context.bool_type().const_int(0, false);
        let _ = self.builder.build_store(exception_raised, false_val);
    }

    /// Set the exception state
    fn set_exception_state(&self, exception_raised: PointerValue<'ctx>, raised: bool) {
        let val = self.llvm_context.bool_type().const_int(if raised { 1 } else { 0 }, false);
        let _ = self.builder.build_store(exception_raised, val);
    }

    /// Load the exception state
    fn load_exception_state(&self, exception_raised: PointerValue<'ctx>) -> inkwell::values::IntValue<'ctx> {
        self.builder.build_load(self.llvm_context.bool_type(), exception_raised, "exception_raised")
            .expect("Failed to load exception state")
            .into_int_value()
    }

    /// Get the current exception
    fn get_current_exception(&self) -> PointerValue<'ctx> {
        // Get the current exception from the global state
        let get_current_exception_fn = match self.module.get_function("get_current_exception") {
            Some(f) => f,
            None => {
                // If the function doesn't exist, create a dummy exception
                return self.create_dummy_exception();
            }
        };

        // Call get_current_exception to get the current exception
        let call_site_value = self.builder.build_call(
            get_current_exception_fn,
            &[],
            "current_exception",
        ).unwrap();

        call_site_value.try_as_basic_value().left().unwrap().into_pointer_value()
    }

    /// Create a dummy exception for testing
    fn create_dummy_exception(&self) -> PointerValue<'ctx> {
        let exception_new_fn = self.module.get_function("exception_new").unwrap();

        // Create string constants for the type and message
        let type_str = self.create_string_constant("Exception");
        let msg_str = self.create_string_constant("Unknown exception");

        // Call exception_new to create a new exception
        let call_site_value = self.builder.build_call(
            exception_new_fn,
            &[type_str.into(), msg_str.into()],
            "new_exception",
        ).unwrap();

        call_site_value.try_as_basic_value().left().unwrap().into_pointer_value()
    }

    /// Check if a value is an exception type
    fn is_exception_type(&self, value: BasicValueEnum<'ctx>) -> bool {
        // In a real implementation, this would check if the value is an exception
        // For now, we'll just check if it's a pointer
        value.is_pointer_value()
    }

    /// Create a string constant
    fn create_string_constant(&self, s: &str) -> PointerValue<'ctx> {
        // Create a global string constant
        let string_val = self.builder.build_global_string_ptr(s, "str_const")
            .expect("Failed to create string constant");
        string_val.as_pointer_value()
    }

    /// Convert a value to a string
    fn convert_to_string(&self, _value: BasicValueEnum<'ctx>) -> Result<PointerValue<'ctx>, String> {
        // In a real implementation, this would convert the value to a string
        // For now, we'll just return a dummy string
        Ok(self.create_string_constant("dummy string"))
    }

    /// Create a new exception
    fn create_exception(&self, typ: &str, message: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let exception_new_fn = self.module.get_function("exception_new").unwrap();

        // Create a string constant for the type
        let type_str = self.create_string_constant(typ);

        // Call exception_new to create a new exception
        let call_site_value = self.builder.build_call(
            exception_new_fn,
            &[type_str.into(), message.into()],
            "new_exception",
        ).unwrap();

        call_site_value.try_as_basic_value().left().unwrap().into_pointer_value()
    }

}
