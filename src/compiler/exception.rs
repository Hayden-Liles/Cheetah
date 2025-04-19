// exception.rs - Exception handling for the Cheetah compiler

use crate::ast::{ExceptHandler, Expr, Stmt};
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
        self.ensure_block_has_terminator();

        let function = match self.current_function {
            Some(f) => f,
            None => return Err("Cannot use try statement outside of a function".to_string()),
        };

        let try_block = self.llvm_context.append_basic_block(function, "try");

        let mut except_blocks = Vec::with_capacity(handlers.len());
        for (i, _) in handlers.iter().enumerate() {
            let block = self
                .llvm_context
                .append_basic_block(function, &format!("except_{}", i));
            except_blocks.push(block);
        }

        let else_block = self.llvm_context.append_basic_block(function, "else");
        let finally_block = self.llvm_context.append_basic_block(function, "finally");
        let exit_block = self.llvm_context.append_basic_block(function, "exit");

        let exception_raised = self.create_exception_state();

        self.ensure_block_has_terminator();

        let _ = self.builder.build_unconditional_branch(try_block);

        self.builder.position_at_end(try_block);

        self.ensure_block_has_terminator();

        self.reset_exception_state(exception_raised);

        self.ensure_block_has_terminator();

        let mut has_terminator = false;
        for (i, stmt) in body.iter().enumerate() {
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                let continue_block = self
                    .llvm_context
                    .append_basic_block(function, &format!("continue_try_{}", i));
                self.builder.position_at_end(continue_block);

                self.ensure_block_has_terminator();
            }

            self.compile_stmt(stmt.as_ref())?;

            self.ensure_block_has_terminator();

            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                has_terminator = true;
                break;
            }
        }

        if !has_terminator {
            self.ensure_block_has_terminator();

            let exception_value = self.load_exception_state(exception_raised);
            let _ = self.builder.build_conditional_branch(
                exception_value,
                except_blocks[0],
                else_block,
            );
        }

        for (i, handler) in handlers.iter().enumerate() {
            self.builder.position_at_end(except_blocks[i]);

            self.ensure_block_has_terminator();

            let matches = self.llvm_context.bool_type().const_int(1, false);

            let handler_body_block = self
                .llvm_context
                .append_basic_block(function, &format!("except_body_{}", i));

            let next_block = if i < handlers.len() - 1 {
                except_blocks[i + 1]
            } else {
                finally_block
            };

            self.ensure_block_has_terminator();

            let _ = self
                .builder
                .build_conditional_branch(matches, handler_body_block, next_block);

            self.builder.position_at_end(handler_body_block);

            self.ensure_block_has_terminator();

            let _current_block = self.builder.get_insert_block().unwrap();

            self.ensure_block_has_terminator();

            if let Some(name) = &handler.name {
                self.ensure_block_has_terminator();

                let exception = self.get_current_exception();

                self.ensure_block_has_terminator();

                let exception_var = self
                    .builder
                    .build_alloca(
                        self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                        name,
                    )
                    .unwrap();

                self.ensure_block_has_terminator();

                let _ = self.builder.build_store(exception_var, exception);

                self.ensure_block_has_terminator();

                self.scope_stack.add_variable(
                    name.clone(),
                    exception_var,
                    crate::compiler::types::Type::Any,
                );
            }

            self.ensure_block_has_terminator();

            let mut has_terminator = false;
            for stmt in &handler.body {
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_some()
                {
                    let continue_block = self
                        .llvm_context
                        .append_basic_block(function, &format!("continue_except_{}", i));
                    self.builder.position_at_end(continue_block);

                    self.ensure_block_has_terminator();
                }

                self.compile_stmt(stmt.as_ref())?;

                self.ensure_block_has_terminator();

                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_some()
                {
                    has_terminator = true;
                    break;
                }
            }

            self.ensure_block_has_terminator();

            self.reset_exception_state(exception_raised);

            self.ensure_block_has_terminator();

            if let Some(clear_current_exception_fn) =
                self.module.get_function("clear_current_exception")
            {
                let _ = self.builder.build_call(
                    clear_current_exception_fn,
                    &[],
                    "clear_exception_result",
                );

                self.ensure_block_has_terminator();
            }

            if !has_terminator {
                self.ensure_block_has_terminator();

                let _ = self.builder.build_unconditional_branch(finally_block);
            } else {
                let continue_block = self
                    .llvm_context
                    .append_basic_block(function, &format!("continue_after_except_{}", i));
                self.builder.position_at_end(continue_block);

                self.ensure_block_has_terminator();

                let _ = self.builder.build_unconditional_branch(finally_block);
            }
        }

        self.builder.position_at_end(else_block);

        self.ensure_block_has_terminator();

        let mut has_terminator = false;
        for (i, stmt) in orelse.iter().enumerate() {
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                let continue_block = self
                    .llvm_context
                    .append_basic_block(function, &format!("continue_else_{}", i));
                self.builder.position_at_end(continue_block);

                self.ensure_block_has_terminator();
            }

            self.compile_stmt(stmt.as_ref())?;

            self.ensure_block_has_terminator();

            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                has_terminator = true;
                break;
            }
        }

        if !has_terminator {
            self.ensure_block_has_terminator();

            let _ = self.builder.build_unconditional_branch(finally_block);
        } else {
            let continue_block = self
                .llvm_context
                .append_basic_block(function, "continue_after_else");
            self.builder.position_at_end(continue_block);

            self.ensure_block_has_terminator();

            let _ = self.builder.build_unconditional_branch(finally_block);
        }

        self.builder.position_at_end(finally_block);

        self.ensure_block_has_terminator();

        let mut has_terminator = false;
        for (i, stmt) in finalbody.iter().enumerate() {
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                let continue_block = self
                    .llvm_context
                    .append_basic_block(function, &format!("continue_finally_{}", i));
                self.builder.position_at_end(continue_block);

                self.ensure_block_has_terminator();
            }

            self.compile_stmt(stmt.as_ref())?;

            self.ensure_block_has_terminator();

            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                has_terminator = true;
                break;
            }
        }

        if !has_terminator {
            self.ensure_block_has_terminator();

            let _ = self.builder.build_unconditional_branch(exit_block);
        } else {
            let continue_block = self
                .llvm_context
                .append_basic_block(function, "continue_after_finally");
            self.builder.position_at_end(continue_block);

            self.ensure_block_has_terminator();

            let _ = self.builder.build_unconditional_branch(exit_block);
        }

        self.builder.position_at_end(exit_block);

        self.ensure_block_has_terminator();

        Ok(())
    }

    /// Compile a raise statement
    pub fn compile_raise_stmt(
        &mut self,
        exc: &Option<Box<Expr>>,
        cause: &Option<Box<Expr>>,
    ) -> Result<(), String> {
        let exception_raise_fn = match self.module.get_function("exception_raise") {
            Some(f) => f,
            None => return Err("exception_raise function not found".to_string()),
        };

        let exception = if let Some(exc_expr) = exc {
            let (exc_val, _) = self.compile_expr(exc_expr)?;

            if !self.is_exception_type(exc_val) {
                let exc_str = self.convert_to_string(exc_val)?;

                self.create_exception("Exception", exc_str)
            } else {
                exc_val.into_pointer_value()
            }
        } else {
            self.get_current_exception()
        };

        if let Some(cause_expr) = cause {
            let (_cause_val, _) = self.compile_expr(cause_expr)?;
        }

        let _ = self
            .builder
            .build_call(exception_raise_fn, &[exception.into()], "raise_result");

        let exception_raised = self.create_exception_state();
        self.set_exception_state(exception_raised, true);

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
    pub fn create_exception_state(&self) -> PointerValue<'ctx> {
        if let Some(var) = self.module.get_global("__exception_raised") {
            return var.as_pointer_value();
        }

        let global =
            self.module
                .add_global(self.llvm_context.bool_type(), None, "__exception_raised");

        global.set_initializer(&self.llvm_context.bool_type().const_int(0, false));

        global.as_pointer_value()
    }

    /// Reset the exception state to false
    pub fn reset_exception_state(&self, exception_raised: PointerValue<'ctx>) {
        let false_val = self.llvm_context.bool_type().const_int(0, false);
        let _ = self.builder.build_store(exception_raised, false_val);
    }

    /// Set the exception state
    fn set_exception_state(&self, exception_raised: PointerValue<'ctx>, raised: bool) {
        let val = self
            .llvm_context
            .bool_type()
            .const_int(if raised { 1 } else { 0 }, false);
        let _ = self.builder.build_store(exception_raised, val);
    }

    /// Load the exception state
    pub fn load_exception_state(
        &self,
        exception_raised: PointerValue<'ctx>,
    ) -> inkwell::values::IntValue<'ctx> {
        self.builder
            .build_load(
                self.llvm_context.bool_type(),
                exception_raised,
                "exception_raised",
            )
            .expect("Failed to load exception state")
            .into_int_value()
    }

    /// Get the current exception
    pub fn get_current_exception(&self) -> PointerValue<'ctx> {
        let get_current_exception_fn = match self.module.get_function("get_current_exception") {
            Some(f) => f,
            None => {
                return self.create_dummy_exception();
            }
        };

        let call_site_value = self
            .builder
            .build_call(get_current_exception_fn, &[], "current_exception")
            .unwrap();

        call_site_value
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value()
    }

    /// Create a dummy exception for testing
    fn create_dummy_exception(&self) -> PointerValue<'ctx> {
        let exception_new_fn = self.module.get_function("exception_new").unwrap();

        let type_str = self.create_string_constant("Exception");
        let msg_str = self.create_string_constant("Unknown exception");

        let call_site_value = self
            .builder
            .build_call(
                exception_new_fn,
                &[type_str.into(), msg_str.into()],
                "new_exception",
            )
            .unwrap();

        call_site_value
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value()
    }

    /// Check if a value is an exception type
    fn is_exception_type(&self, value: BasicValueEnum<'ctx>) -> bool {
        value.is_pointer_value()
    }

    /// Create a string constant
    fn create_string_constant(&self, s: &str) -> PointerValue<'ctx> {
        let string_val = self
            .builder
            .build_global_string_ptr(s, "str_const")
            .expect("Failed to create string constant");
        string_val.as_pointer_value()
    }

    /// Convert a value to a string
    fn convert_to_string(
        &self,
        _value: BasicValueEnum<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        Ok(self.create_string_constant("dummy string"))
    }

    /// Create a new exception
    fn create_exception(&self, typ: &str, message: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let exception_new_fn = self.module.get_function("exception_new").unwrap();

        let type_str = self.create_string_constant(typ);

        let call_site_value = self
            .builder
            .build_call(
                exception_new_fn,
                &[type_str.into(), message.into()],
                "new_exception",
            )
            .unwrap();

        call_site_value
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value()
    }
}
