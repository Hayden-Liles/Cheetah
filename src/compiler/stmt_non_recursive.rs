// Non-recursive implementation of the statement compiler
// This implementation avoids deep recursion by using an explicit work stack

use crate::ast::{Expr, Stmt};
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::{AssignmentCompiler, BinaryOpCompiler, ExprCompiler};
use crate::compiler::stmt::StmtCompiler;
use crate::compiler::types::Type;
use inkwell::values::BasicValueEnum;
use std::collections::VecDeque;

// This trait is used to extend the CompilationContext with non-recursive statement compilation
pub trait StmtNonRecursive<'ctx> {
    fn compile_stmt_non_recursive(&mut self, stmt: &Stmt) -> Result<(), String>;

    fn compile_stmt_fallback(&mut self, stmt: &Stmt) -> Result<(), String>;

    fn convert_to_bool(&self, value: BasicValueEnum<'ctx>) -> inkwell::values::IntValue<'ctx>;

    /// Detect if an expression is a range call and extract its parameters
    fn detect_range_call(&mut self, expr: &Expr) -> Result<Option<(inkwell::values::IntValue<'ctx>, inkwell::values::IntValue<'ctx>, inkwell::values::IntValue<'ctx>)>, String>;

    /// Generate an optimized LLVM loop for range iterables
    fn generate_optimized_range_loop(
        &mut self,
        target: &Expr,
        body: &[Box<Stmt>],
        orelse: &[Box<Stmt>],
        start_val: inkwell::values::IntValue<'ctx>,
        stop_val: inkwell::values::IntValue<'ctx>,
        step_val: inkwell::values::IntValue<'ctx>
    ) -> Result<(), String>;
}

// Task for the work stack
enum StmtTask<'a, 'ctx> {
    Execute(&'a Stmt),

    ExecuteBlock {
        stmts: &'a [Box<Stmt>],
        index: usize,
    },

    ProcessFor {
        target: &'a Expr,
        body: &'a [Box<Stmt>],
        orelse: &'a [Box<Stmt>],
        iter: &'a Expr,
    },

    ProcessWhile {
        test: &'a Expr,
        body: &'a [Box<Stmt>],
        orelse: &'a [Box<Stmt>],
    },

    ProcessTry {
        body: &'a [Box<Stmt>],
        handlers: &'a [crate::ast::ExceptHandler],
        orelse: &'a [Box<Stmt>],
        finalbody: &'a [Box<Stmt>],
    },

    ProcessWith {
        body: &'a [Box<Stmt>],
    },

    ProcessAssign {
        targets: &'a [Box<Expr>],
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
    },

    ProcessReturn {
        value_val: Option<BasicValueEnum<'ctx>>,
        value_type: Option<Type>,
    },

    ProcessFunctionDef {
        name: String,
        params: &'a [crate::ast::Parameter],
        body: &'a [Box<Stmt>],
        is_nested: bool,
    },
}

impl<'ctx> StmtNonRecursive<'ctx> for CompilationContext<'ctx> {
    /// Detect if an expression is a range call and extract its parameters
    fn detect_range_call(&mut self, expr: &Expr) -> Result<Option<(inkwell::values::IntValue<'ctx>, inkwell::values::IntValue<'ctx>, inkwell::values::IntValue<'ctx>)>, String> {
        if let Expr::Call { func, args, .. } = expr {
            if let Expr::Name { id, .. } = func.as_ref() {
                if id == "range" {
                    let i64_type = self.llvm_context.i64_type();

                    match args.len() {
                        1 => {
                            // range(stop)
                            let (stop_val, stop_type) = self.compile_expr(&args[0])?;
                            let start_val = i64_type.const_int(0, false); // start = 0
                            let step_val = i64_type.const_int(1, false);  // step = 1

                            let stop_val = if stop_type != Type::Int {
                                self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                            } else if stop_val.is_pointer_value() {
                                self.builder
                                    .build_load(i64_type, stop_val.into_pointer_value(), "range_stop")
                                    .unwrap()
                                    .into_int_value()
                            } else {
                                stop_val.into_int_value()
                            };

                            return Ok(Some((start_val, stop_val, step_val)));
                        },
                        2 => {
                            // range(start, stop)
                            let (start_val, start_type) = self.compile_expr(&args[0])?;
                            let (stop_val, stop_type) = self.compile_expr(&args[1])?;
                            let step_val = i64_type.const_int(1, false);  // step = 1

                            let start_val = if start_type != Type::Int {
                                self.convert_type(start_val, &start_type, &Type::Int)?.into_int_value()
                            } else if start_val.is_pointer_value() {
                                self.builder
                                    .build_load(i64_type, start_val.into_pointer_value(), "range_start")
                                    .unwrap()
                                    .into_int_value()
                            } else {
                                start_val.into_int_value()
                            };

                            let stop_val = if stop_type != Type::Int {
                                self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                            } else if stop_val.is_pointer_value() {
                                self.builder
                                    .build_load(i64_type, stop_val.into_pointer_value(), "range_stop")
                                    .unwrap()
                                    .into_int_value()
                            } else {
                                stop_val.into_int_value()
                            };

                            return Ok(Some((start_val, stop_val, step_val)));
                        },
                        3 => {
                            // range(start, stop, step)
                            let (start_val, start_type) = self.compile_expr(&args[0])?;
                            let (stop_val, stop_type) = self.compile_expr(&args[1])?;
                            let (step_val, step_type) = self.compile_expr(&args[2])?;

                            let start_val = if start_type != Type::Int {
                                self.convert_type(start_val, &start_type, &Type::Int)?.into_int_value()
                            } else if start_val.is_pointer_value() {
                                self.builder
                                    .build_load(i64_type, start_val.into_pointer_value(), "range_start")
                                    .unwrap()
                                    .into_int_value()
                            } else {
                                start_val.into_int_value()
                            };

                            let stop_val = if stop_type != Type::Int {
                                self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                            } else if stop_val.is_pointer_value() {
                                self.builder
                                    .build_load(i64_type, stop_val.into_pointer_value(), "range_stop")
                                    .unwrap()
                                    .into_int_value()
                            } else {
                                stop_val.into_int_value()
                            };

                            let step_val = if step_type != Type::Int {
                                self.convert_type(step_val, &step_type, &Type::Int)?.into_int_value()
                            } else if step_val.is_pointer_value() {
                                self.builder
                                    .build_load(i64_type, step_val.into_pointer_value(), "range_step")
                                    .unwrap()
                                    .into_int_value()
                            } else {
                                step_val.into_int_value()
                            };

                            return Ok(Some((start_val, stop_val, step_val)));
                        },
                        _ => {
                            return Err(format!("Invalid number of arguments for range: expected 1, 2, or 3, got {}", args.len()));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Generate an optimized LLVM loop for range iterables
    fn generate_optimized_range_loop(
        &mut self,
        target: &Expr,
        body: &[Box<Stmt>],
        orelse: &[Box<Stmt>],
        start_val: inkwell::values::IntValue<'ctx>,
        stop_val: inkwell::values::IntValue<'ctx>,
        step_val: inkwell::values::IntValue<'ctx>
    ) -> Result<(), String> {
        let current_function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        // Create the basic blocks for the loop
        let entry_block = self.llvm_context.append_basic_block(current_function, "range.entry");
        let cond_block = self.llvm_context.append_basic_block(current_function, "range.cond");
        let body_block = self.llvm_context.append_basic_block(current_function, "range.body");
        let inc_block = self.llvm_context.append_basic_block(current_function, "range.inc");
        let else_block = self.llvm_context.append_basic_block(current_function, "range.else");
        let exit_block = self.llvm_context.append_basic_block(current_function, "range.exit");

        // Register the loop for break/continue statements
        self.push_loop(inc_block, exit_block);

        // Branch to the entry block
        self.builder.build_unconditional_branch(entry_block).unwrap();

        // Entry block: initialize the loop variable
        self.builder.position_at_end(entry_block);
        let i64_type = self.llvm_context.i64_type();

        // Create the loop variable
        let var_ptr = if let Expr::Name { id, .. } = target {
            let ptr = self.builder.build_alloca(i64_type, id).unwrap();
            self.scope_stack.add_variable(id.clone(), ptr, Type::Int);
            ptr
        } else {
            return Err("Unsupported loop target".to_string());
        };

        // Store the initial value
        self.builder.build_store(var_ptr, start_val).unwrap();

        // Branch to the condition block
        self.builder.build_unconditional_branch(cond_block).unwrap();

        // Condition block: check if we should continue looping
        self.builder.position_at_end(cond_block);

        // Load the current value of the loop variable
        let current_val = self.builder
            .build_load(i64_type, var_ptr, "current")
            .unwrap()
            .into_int_value();

        // Determine the comparison predicate based on the step direction
        let step_positive = self.builder
            .build_int_compare(
                inkwell::IntPredicate::SGT,
                step_val,
                i64_type.const_int(0, true),
                "step_positive"
            )
            .unwrap();

        let cond_pos = self.builder
            .build_int_compare(
                inkwell::IntPredicate::SLT,
                current_val,
                stop_val,
                "cond_pos"
            )
            .unwrap();

        let cond_neg = self.builder
            .build_int_compare(
                inkwell::IntPredicate::SGT,
                current_val,
                stop_val,
                "cond_neg"
            )
            .unwrap();

        // Select the appropriate condition based on step direction
        let condition = self.builder
            .build_select(
                step_positive,
                cond_pos,
                cond_neg,
                "loop_condition"
            )
            .unwrap()
            .into_int_value();

        // Branch based on the condition
        self.builder
            .build_conditional_branch(condition, body_block, else_block)
            .unwrap();

        // Body block: execute the loop body
        self.builder.position_at_end(body_block);
        self.push_scope(false, true, false);

        // Execute the body statements
        for stmt in body {
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                break;
            }
            self.compile_stmt_non_recursive(stmt)?;
        }

        // If the block doesn't have a terminator, branch to the increment block
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(inc_block).unwrap();
        }

        self.pop_scope();

        // Increment block: update the loop variable
        self.builder.position_at_end(inc_block);

        // Load the current value
        let current_val = self.builder
            .build_load(i64_type, var_ptr, "current_inc")
            .unwrap()
            .into_int_value();

        // Add the step value
        let next_val = self.builder
            .build_int_add(current_val, step_val, "next")
            .unwrap();

        // Store the updated value
        self.builder.build_store(var_ptr, next_val).unwrap();

        // Branch back to the condition block
        self.builder.build_unconditional_branch(cond_block).unwrap();

        // Else block: execute the else clause if the loop condition is initially false
        self.builder.position_at_end(else_block);
        self.push_scope(false, false, false);

        if !orelse.is_empty() {
            for stmt in orelse {
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_some()
                {
                    break;
                }
                self.compile_stmt_non_recursive(stmt)?;
            }
        }

        // If the block doesn't have a terminator, branch to the exit block
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(exit_block).unwrap();
        }

        self.pop_scope();

        // Exit block: continue execution after the loop
        self.builder.position_at_end(exit_block);
        self.pop_loop();

        Ok(())
    }
    fn convert_to_bool(&self, value: BasicValueEnum<'ctx>) -> inkwell::values::IntValue<'ctx> {
        match value {
            BasicValueEnum::IntValue(int_val) => {
                if int_val.get_type().get_bit_width() == 1 {
                    return int_val;
                }

                let zero = int_val.get_type().const_zero();
                self.builder
                    .build_int_compare(inkwell::IntPredicate::NE, int_val, zero, "bool_conv")
                    .unwrap()
            }
            BasicValueEnum::FloatValue(float_val) => {
                let zero = float_val.get_type().const_float(0.0);
                self.builder
                    .build_float_compare(
                        inkwell::FloatPredicate::ONE,
                        float_val,
                        zero,
                        "float_bool",
                    )
                    .unwrap()
            }
            _ => self.llvm_context.bool_type().const_int(1, false),
        }
    }
    fn compile_stmt_non_recursive(&mut self, stmt: &Stmt) -> Result<(), String> {
        let mut work_stack: VecDeque<StmtTask> = VecDeque::new();

        work_stack.push_back(StmtTask::Execute(stmt));

        while let Some(task) = work_stack.pop_front() {
            match task {
                StmtTask::Execute(stmt) => match stmt {
                    Stmt::Expr { value, .. } => {
                        let _ = self.compile_expr(value)?;
                    }

                    Stmt::Assign { targets, value, .. } => {
                        let (val, val_type) = self.compile_expr(value)?;

                        work_stack.push_front(StmtTask::ProcessAssign {
                            targets,
                            value_val: val,
                            value_type: val_type,
                        });
                    }

                    Stmt::AugAssign {
                        target, op, value, ..
                    } => {
                        let (target_val, target_type) = self.compile_expr(target)?;
                        let (value_val, value_type) = self.compile_expr(value)?;

                        let (result_val, result_type) = self.compile_binary_op(
                            target_val,
                            &target_type,
                            op.clone(),
                            value_val,
                            &value_type,
                        )?;

                        self.compile_assignment(target, result_val, &result_type)?;
                    }

                    Stmt::AnnAssign { target, value, .. } => {
                        if let Some(val_expr) = value {
                            let (val, val_type) = self.compile_expr(val_expr)?;

                            self.compile_assignment(target, val, &val_type)?;
                        }
                    }

                    Stmt::If {
                        test, body, orelse, ..
                    } => {
                        let (test_val, _) = self.compile_expr(test)?;

                        let bool_val = self.convert_to_bool(test_val);

                        let function = self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_parent()
                            .unwrap();

                        let then_block = self.llvm_context.append_basic_block(function, "then");
                        let else_block = self.llvm_context.append_basic_block(function, "else");
                        let end_block = self.llvm_context.append_basic_block(function, "endif");

                        self.builder
                            .build_conditional_branch(bool_val, then_block, else_block)
                            .unwrap();

                        self.builder.position_at_end(then_block);

                        for stmt in body {
                            if self
                                .builder
                                .get_insert_block()
                                .unwrap()
                                .get_terminator()
                                .is_some()
                            {
                                break;
                            }

                            if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                                return Err(e);
                            }
                        }

                        if !self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            self.builder.build_unconditional_branch(end_block).unwrap();
                        }

                        self.builder.position_at_end(else_block);

                        for stmt in orelse {
                            if self
                                .builder
                                .get_insert_block()
                                .unwrap()
                                .get_terminator()
                                .is_some()
                            {
                                break;
                            }

                            if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                                return Err(e);
                            }
                        }

                        if !self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            self.builder.build_unconditional_branch(end_block).unwrap();
                        }

                        self.builder.position_at_end(end_block);
                    }

                    Stmt::For {
                        target,
                        iter,
                        body,
                        orelse,
                        ..
                    } => {
                        let (_iter_val, _iter_type) = self.compile_expr(iter)?;

                        work_stack.push_front(StmtTask::ProcessFor {
                            target,
                            body,
                            orelse,
                            iter,
                        });
                    }

                    Stmt::While {
                        test, body, orelse, ..
                    } => {
                        work_stack.push_front(StmtTask::ProcessWhile { test, body, orelse });
                    }

                    Stmt::Return { value, .. } => {
                        if let Some(expr) = value {
                            let (ret_val, ret_type) = self.compile_expr(expr)?;

                            work_stack.push_front(StmtTask::ProcessReturn {
                                value_val: Some(ret_val),
                                value_type: Some(ret_type),
                            });
                        } else {
                            work_stack.push_front(StmtTask::ProcessReturn {
                                value_val: None,
                                value_type: None,
                            });
                        }
                    }

                    Stmt::Pass { .. } => {}

                    Stmt::With { body, .. } => {
                        work_stack.push_front(StmtTask::ProcessWith { body });
                    }

                    Stmt::Try {
                        body,
                        handlers,
                        orelse,
                        finalbody,
                        ..
                    } => {
                        work_stack.push_front(StmtTask::ProcessTry {
                            body,
                            handlers,
                            orelse,
                            finalbody,
                        });
                    }

                    Stmt::Break { .. } => {
                        if let Some(break_block) = self.current_break_block() {
                            self.builder
                                .build_unconditional_branch(break_block)
                                .unwrap();
                        } else {
                            return Err("Break statement outside of loop".to_string());
                        }
                    }

                    Stmt::Continue { .. } => {
                        if let Some(continue_block) = self.current_continue_block() {
                            self.builder
                                .build_unconditional_branch(continue_block)
                                .unwrap();
                        } else {
                            return Err("Continue statement outside of loop".to_string());
                        }
                    }

                    Stmt::FunctionDef {
                        name, params, body, ..
                    } => {
                        let parent_function_name = if let Some(current_function) =
                            self.current_function
                        {
                            let fn_name = current_function.get_name().to_string_lossy().to_string();
                            Some(fn_name)
                        } else {
                            None
                        };

                        let qualified_name = if let Some(parent) = &parent_function_name {
                            format!("{}.{}", parent, name)
                        } else {
                            name.clone()
                        };

                        work_stack.push_front(StmtTask::ProcessFunctionDef {
                            name: qualified_name,
                            params,
                            body,
                            is_nested: parent_function_name.is_some(),
                        });
                    }

                    Stmt::Nonlocal { names, .. } => {
                        for name in names {
                            let mut found_in_outer_scope = false;

                            if self.scope_stack.scopes.len() >= 2 {
                                let parent_scope_index = self.scope_stack.scopes.len() - 2;
                                if let Some(_) =
                                    self.scope_stack.scopes[parent_scope_index].get_variable(&name)
                                {
                                    found_in_outer_scope = true;
                                    println!("Found variable '{}' in immediate outer scope {} for nonlocal declaration", name, parent_scope_index);
                                }
                            }

                            if !found_in_outer_scope && self.scope_stack.scopes.len() >= 3 {
                                for i in (0..self.scope_stack.scopes.len() - 2).rev() {
                                    if let Some(_) = self.scope_stack.scopes[i].get_variable(&name)
                                    {
                                        found_in_outer_scope = true;
                                        println!("Found variable '{}' in outer scope {} for nonlocal declaration", name, i);
                                        break;
                                    }
                                }
                            }

                            if found_in_outer_scope {
                                self.declare_nonlocal(name.clone());

                                if let Some(current_function) = self.current_function {
                                    let fn_name =
                                        current_function.get_name().to_string_lossy().to_string();

                                    let unique_name = format!(
                                        "__nonlocal_{}_{}",
                                        fn_name.replace('.', "_"),
                                        name
                                    );

                                    let mut found_ptr = None;
                                    let mut found_type = None;

                                    let current_index = self.scope_stack.scopes.len() - 1;

                                    if current_index > 0 {
                                        let parent_scope_index = current_index - 1;
                                        if let Some(ptr) = self.scope_stack.scopes
                                            [parent_scope_index]
                                            .get_variable(&name)
                                        {
                                            found_ptr = Some(*ptr);
                                            if let Some(ty) = self.scope_stack.scopes
                                                [parent_scope_index]
                                                .get_type(&name)
                                            {
                                                found_type = Some(ty.clone());
                                            }
                                        }
                                    }

                                    if found_ptr.is_none() && current_index > 1 {
                                        for i in (0..current_index - 1).rev() {
                                            if let Some(ptr) =
                                                self.scope_stack.scopes[i].get_variable(&name)
                                            {
                                                found_ptr = Some(*ptr);
                                                if let Some(ty) =
                                                    self.scope_stack.scopes[i].get_type(&name)
                                                {
                                                    found_type = Some(ty.clone());
                                                }
                                                break;
                                            }
                                        }
                                    }

                                    if let (Some(ptr), Some(var_type)) = (found_ptr, found_type) {
                                        self.add_to_current_environment(
                                            name.clone(),
                                            ptr,
                                            var_type.clone(),
                                        );
                                        println!("Added nonlocal variable '{}' to current closure environment", name);

                                        let current_position =
                                            self.builder.get_insert_block().unwrap();

                                        let entry_block =
                                            current_function.get_first_basic_block().unwrap();
                                        if let Some(first_instr) =
                                            entry_block.get_first_instruction()
                                        {
                                            self.builder.position_before(&first_instr);
                                        } else {
                                            self.builder.position_at_end(entry_block);
                                        }

                                        let local_ptr = self
                                            .builder
                                            .build_alloca(
                                                self.get_llvm_type(&var_type).into_int_type(),
                                                &unique_name,
                                            )
                                            .unwrap();

                                        self.builder.position_at_end(current_position);

                                        if let Some(current_scope) =
                                            self.scope_stack.current_scope_mut()
                                        {
                                            current_scope.add_variable(
                                                unique_name.clone(),
                                                local_ptr,
                                                var_type.clone(),
                                            );
                                            current_scope.add_nonlocal_mapping(
                                                name.clone(),
                                                unique_name.clone(),
                                            );
                                            println!("Created local variable for nonlocal variable '{}' with unique name '{}'", name, unique_name);
                                        }

                                        println!(
                                            "Marked '{}' as nonlocal in nested function '{}'",
                                            name, fn_name
                                        );
                                    }
                                }
                            } else {
                                self.declare_nonlocal(name.clone());
                            }
                        }
                    }

                    Stmt::Global { names, .. } => {
                        for name in names {
                            self.declare_global(name.clone());

                            if self.current_function.is_some() {
                                let var_exists_in_global =
                                    if let Some(global_scope) = self.scope_stack.global_scope() {
                                        global_scope.get_variable(&name).is_some()
                                    } else {
                                        false
                                    };

                                if !var_exists_in_global {
                                    let var_type = Type::Int;
                                    self.register_variable(name.clone(), var_type.clone());

                                    let global_var = self.module.add_global(
                                        self.get_llvm_type(&var_type).into_int_type(),
                                        None,
                                        &name,
                                    );

                                    global_var.set_initializer(
                                        &self.llvm_context.i64_type().const_zero(),
                                    );

                                    let ptr = global_var.as_pointer_value();

                                    if let Some(global_scope) = self.scope_stack.global_scope_mut()
                                    {
                                        global_scope.add_variable(
                                            name.clone(),
                                            ptr,
                                            var_type.clone(),
                                        );
                                    }

                                    self.variables.insert(name.clone(), ptr);

                                    self.type_env.insert(name.clone(), var_type.clone());
                                }
                            }
                        }
                    }

                    _ => {
                        self.compile_stmt_fallback(stmt)?;
                    }
                },

                StmtTask::ExecuteBlock { stmts, index } => {
                    if index < stmts.len() {
                        let stmt = &stmts[index];

                        work_stack.push_front(StmtTask::ExecuteBlock {
                            stmts,
                            index: index + 1,
                        });

                        work_stack.push_front(StmtTask::Execute(stmt.as_ref()));

                        if self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            work_stack.clear();
                        }
                    }
                }

                StmtTask::ProcessFor {
                    target,
                    body,
                    orelse,
                    iter,
                } => {
                    // Check if this is a range-based for loop that we can optimize
                    if let Ok(Some((start_val, stop_val, step_val))) = self.detect_range_call(iter) {
                        // This is a range-based for loop, use our optimized implementation
                        self.generate_optimized_range_loop(target, body, orelse, start_val, stop_val, step_val)?;
                    } else {
                        // This is a regular for loop, use the original implementation
                        let current_function = self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_parent()
                            .unwrap();

                        let init_block = self
                            .llvm_context
                            .append_basic_block(current_function, "for.init");
                        let cond_block = self
                            .llvm_context
                            .append_basic_block(current_function, "for.cond");
                        let body_block = self
                            .llvm_context
                            .append_basic_block(current_function, "for.body");
                        let increment_block = self
                            .llvm_context
                            .append_basic_block(current_function, "for.inc");
                        let else_block = self
                            .llvm_context
                            .append_basic_block(current_function, "for.else");
                        let end_block = self
                            .llvm_context
                            .append_basic_block(current_function, "for.end");

                        self.push_loop(increment_block, end_block);

                        self.builder.build_unconditional_branch(init_block).unwrap();

                        self.builder.position_at_end(init_block);
                        let i64_type = self.llvm_context.i64_type();

                        let index_ptr = self.builder.build_alloca(i64_type, "for.index").unwrap();
                        self.builder
                            .build_store(index_ptr, i64_type.const_int(0, false))
                            .unwrap();

                        let var_ptr = if let Expr::Name { id, .. } = target {
                            let ptr = self.builder.build_alloca(i64_type, id).unwrap();
                            self.scope_stack.add_variable(id.clone(), ptr, Type::Int);
                            ptr
                        } else {
                            return Err("Unsupported loop target".to_string());
                        };

                        let (iter_val, iter_type) = self.compile_expr(iter)?;

                        let len_val = match iter_type {
                            Type::List(_) => {
                                let list_len_fn = self
                                    .module
                                    .get_function("list_len")
                                    .ok_or("list_len function not found".to_string())?;
                                let call = self
                                    .builder
                                    .build_call(
                                        list_len_fn,
                                        &[iter_val.into_pointer_value().into()],
                                        "list_len_result",
                                    )
                                    .unwrap();
                                call.try_as_basic_value().left().unwrap()
                            }
                            Type::Int => {
                                if iter_val.is_pointer_value() {
                                    self.builder
                                        .build_load(
                                            i64_type,
                                            iter_val.into_pointer_value(),
                                            "range_len",
                                        )
                                        .unwrap()
                                } else {
                                    iter_val
                                }
                            }
                            _ => iter_val,
                        };

                        self.builder.build_unconditional_branch(cond_block).unwrap();

                        self.builder.position_at_end(cond_block);
                        let index_val = self
                            .builder
                            .build_load(i64_type, index_ptr, "index")
                            .unwrap()
                            .into_int_value();
                        let cond = self
                            .builder
                            .build_int_compare(
                                inkwell::IntPredicate::SLT,
                                index_val,
                                len_val.into_int_value(),
                                "loop.cond",
                            )
                            .unwrap();
                        self.builder
                            .build_conditional_branch(cond, body_block, else_block)
                            .unwrap();

                        self.builder.position_at_end(body_block);
                        self.push_scope(false, true, false);

                        self.builder.build_store(var_ptr, index_val).unwrap();

                        for stmt in body {
                            if self
                                .builder
                                .get_insert_block()
                                .unwrap()
                                .get_terminator()
                                .is_some()
                            {
                                break;
                            }
                            self.compile_stmt_non_recursive(stmt)?;
                        }

                        if self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_none()
                        {
                            self.builder
                                .build_unconditional_branch(increment_block)
                                .unwrap();
                        }
                        self.pop_scope();

                        self.builder.position_at_end(increment_block);
                        let prev_index = self
                            .builder
                            .build_load(i64_type, index_ptr, "index")
                            .unwrap()
                            .into_int_value();
                        let next_index = self
                            .builder
                            .build_int_add(prev_index, i64_type.const_int(1, false), "next_index")
                            .unwrap();
                        self.builder.build_store(index_ptr, next_index).unwrap();
                        self.builder.build_unconditional_branch(cond_block).unwrap();

                        self.builder.position_at_end(else_block);
                        self.push_scope(false, false, false);
                        if !orelse.is_empty() {
                            for stmt in orelse {
                                if self
                                    .builder
                                    .get_insert_block()
                                    .unwrap()
                                    .get_terminator()
                                    .is_some()
                                {
                                    break;
                                }
                                self.compile_stmt_non_recursive(stmt)?;
                            }
                        }
                        if self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_none()
                        {
                            self.builder.build_unconditional_branch(end_block).unwrap();
                        }
                        self.pop_scope();

                        self.builder.position_at_end(end_block);
                        self.pop_loop();
                    }
                }

                StmtTask::ProcessWhile { test, body, orelse } => {
                    let context = self.llvm_context;

                    let function = match self.builder.get_insert_block() {
                        Some(block) => block.get_parent().unwrap(),
                        None => match self.current_function {
                            Some(f) => f,
                            None => match self.module.get_function("main") {
                                Some(f) => f,
                                None => return Err("No main function found".to_string()),
                            },
                        },
                    };

                    let cond_block = context.append_basic_block(function, "while.cond");
                    let body_block = context.append_basic_block(function, "while.body");
                    let else_block = context.append_basic_block(function, "while.else");
                    let end_block = context.append_basic_block(function, "while.end");

                    self.builder.build_unconditional_branch(cond_block).unwrap();

                    self.builder.position_at_end(cond_block);

                    let (test_val, _) = self.compile_expr(test)?;

                    let cond_val = self.convert_to_bool(test_val);

                    self.builder
                        .build_conditional_branch(cond_val, body_block, else_block)
                        .unwrap();

                    self.builder.position_at_end(body_block);

                    self.push_scope(false, true, false);

                    let _old_break_block = self.current_break_block();
                    let _old_continue_block = self.current_continue_block();

                    self.push_loop(cond_block, end_block);

                    for stmt in body {
                        if self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            break;
                        }

                        if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                            return Err(e);
                        }
                    }

                    if !self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_some()
                    {
                        self.builder.build_unconditional_branch(cond_block).unwrap();
                    }

                    self.pop_loop();

                    self.pop_scope();

                    self.builder.position_at_end(else_block);

                    if !orelse.is_empty() {
                        for stmt in orelse {
                            if self
                                .builder
                                .get_insert_block()
                                .unwrap()
                                .get_terminator()
                                .is_some()
                            {
                                break;
                            }

                            if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                                return Err(e);
                            }
                        }
                    }

                    if !self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_some()
                    {
                        self.builder.build_unconditional_branch(end_block).unwrap();
                    }

                    self.builder.position_at_end(end_block);
                }

                StmtTask::ProcessTry {
                    body,
                    handlers,
                    orelse,
                    finalbody,
                } => {
                    self.ensure_block_has_terminator();

                    let function = match self.current_function {
                        Some(f) => f,
                        None => {
                            return Err("Cannot use try statement outside of a function".to_string())
                        }
                    };

                    let try_block = self.llvm_context.append_basic_block(function, "try");

                    let mut except_blocks = Vec::new();
                    for i in 0..handlers.len() {
                        except_blocks.push(
                            self.llvm_context
                                .append_basic_block(function, &format!("except_{}", i)),
                        );
                    }

                    if except_blocks.is_empty() {
                        except_blocks.push(
                            self.llvm_context
                                .append_basic_block(function, "except_default"),
                        );
                    }

                    let else_block = self.llvm_context.append_basic_block(function, "else");
                    let finally_block = self.llvm_context.append_basic_block(function, "finally");
                    let exit_block = self.llvm_context.append_basic_block(function, "exit");

                    let exception_raised = self.create_exception_state();

                    self.builder.build_unconditional_branch(try_block).unwrap();

                    self.builder.position_at_end(try_block);

                    self.reset_exception_state(exception_raised);

                    for stmt in body {
                        if self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            break;
                        }

                        if let Err(e) = self.compile_stmt(stmt) {
                            return Err(e);
                        }
                    }

                    if !self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_some()
                    {
                        let exception_value = self.load_exception_state(exception_raised);
                        self.builder
                            .build_conditional_branch(exception_value, except_blocks[0], else_block)
                            .unwrap();
                    }

                    for (i, handler) in handlers.iter().enumerate() {
                        self.builder.position_at_end(except_blocks[i]);

                        let matches = self.llvm_context.bool_type().const_int(1, false);

                        let handler_body_block = self
                            .llvm_context
                            .append_basic_block(function, &format!("except_body_{}", i));

                        let next_block = if i < handlers.len() - 1 {
                            except_blocks[i + 1]
                        } else {
                            finally_block
                        };

                        self.builder
                            .build_conditional_branch(matches, handler_body_block, next_block)
                            .unwrap();

                        self.builder.position_at_end(handler_body_block);

                        if let Some(name) = &handler.name {
                            let exception = self.get_current_exception();

                            let exception_ptr = self
                                .builder
                                .build_alloca(
                                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                    name,
                                )
                                .unwrap();

                            self.builder.build_store(exception_ptr, exception).unwrap();

                            self.add_variable_to_scope(name.clone(), exception_ptr, Type::Any);
                        }

                        for stmt in &handler.body {
                            if self
                                .builder
                                .get_insert_block()
                                .unwrap()
                                .get_terminator()
                                .is_some()
                            {
                                break;
                            }

                            if let Err(e) = self.compile_stmt(stmt) {
                                return Err(e);
                            }
                        }

                        self.reset_exception_state(exception_raised);

                        if let Some(clear_current_exception_fn) =
                            self.module.get_function("clear_current_exception")
                        {
                            self.builder
                                .build_call(
                                    clear_current_exception_fn,
                                    &[],
                                    "clear_exception_result",
                                )
                                .unwrap();
                        }

                        if !self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            self.builder
                                .build_unconditional_branch(finally_block)
                                .unwrap();
                        }
                    }

                    self.builder.position_at_end(else_block);

                    for stmt in orelse {
                        if self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            break;
                        }

                        if let Err(e) = self.compile_stmt(stmt) {
                            return Err(e);
                        }
                    }

                    if !self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_some()
                    {
                        self.builder
                            .build_unconditional_branch(finally_block)
                            .unwrap();
                    }

                    self.builder.position_at_end(finally_block);

                    self.push_scope(false, false, false);

                    for stmt in finalbody {
                        if self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            break;
                        }

                        if let Err(e) = self.compile_stmt(stmt) {
                            return Err(e);
                        }
                    }

                    if !self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_some()
                    {
                        self.builder.build_unconditional_branch(exit_block).unwrap();
                    }

                    self.pop_scope();

                    self.builder.position_at_end(exit_block);
                }

                StmtTask::ProcessWith { body } => {
                    if !body.is_empty() {
                        work_stack.push_front(StmtTask::ExecuteBlock {
                            stmts: body,
                            index: 0,
                        });
                    }
                }

                StmtTask::ProcessAssign {
                    targets,
                    value_val,
                    value_type,
                } => {
                    for target in targets {
                        self.compile_assignment(target, value_val, &value_type)?;
                    }
                }

                StmtTask::ProcessReturn {
                    value_val,
                    value_type,
                } => {
                    if let Some(ret_val) = value_val {
                        if let Some(current_function) = self.current_function {
                            let return_type = current_function.get_type().get_return_type();

                            if let Some(ret_type) = return_type {
                                if ret_type.is_int_type() && ret_val.is_pointer_value() {
                                    let loaded_val = self
                                        .builder
                                        .build_load(
                                            ret_type.into_int_type(),
                                            ret_val.into_pointer_value(),
                                            "load_return",
                                        )
                                        .unwrap();

                                    self.builder.build_return(Some(&loaded_val)).unwrap();
                                    return Ok(());
                                }
                            }

                            if let Some(ret_type) = value_type {
                                if let Type::Tuple(_) = ret_type {
                                    if return_type.is_some() && return_type.unwrap().is_int_type() {
                                        self.builder.build_return(Some(&ret_val)).unwrap();
                                        return Ok(());
                                    }
                                }
                            }
                        }

                        self.builder.build_return(Some(&ret_val)).unwrap();
                    } else {
                        self.builder.build_return(None).unwrap();
                    }
                }

                StmtTask::ProcessFunctionDef {
                    name,
                    params,
                    body,
                    is_nested,
                } => {
                    if is_nested {
                        self.declare_nested_function(&name, params)?;
                    } else {
                        self.declare_nested_function(&name, params)?;
                    }

                    if is_nested {
                        let result = self.compile_nested_function_body(&name, params, body);

                        if let Err(e) = result {
                            return Err(e);
                        }
                    } else {
                        let result = self.compile_nested_function_body(&name, params, body);

                        if let Err(e) = result {
                            return Err(e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn compile_stmt_fallback(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::While {
                test, body, orelse, ..
            } => {
                let context = self.llvm_context;

                let function = match self.builder.get_insert_block() {
                    Some(block) => block.get_parent().unwrap(),
                    None => match self.current_function {
                        Some(f) => f,
                        None => match self.module.get_function("main") {
                            Some(f) => f,
                            None => return Err("No main function found".to_string()),
                        },
                    },
                };

                let cond_block = context.append_basic_block(function, "while.cond");
                let body_block = context.append_basic_block(function, "while.body");
                let else_block = context.append_basic_block(function, "while.else");
                let end_block = context.append_basic_block(function, "while.end");

                self.builder.build_unconditional_branch(cond_block).unwrap();

                self.builder.position_at_end(cond_block);

                let (test_val, _) = self.compile_expr(test)?;

                let cond_val = match test_val {
                    BasicValueEnum::IntValue(int_val) => {
                        if int_val.get_type().get_bit_width() == 1 {
                            int_val
                        } else {
                            let zero = int_val.get_type().const_zero();
                            self.builder
                                .build_int_compare(
                                    inkwell::IntPredicate::NE,
                                    int_val,
                                    zero,
                                    "bool_conv",
                                )
                                .unwrap()
                        }
                    }
                    BasicValueEnum::FloatValue(float_val) => {
                        let zero = float_val.get_type().const_float(0.0);
                        self.builder
                            .build_float_compare(
                                inkwell::FloatPredicate::ONE,
                                float_val,
                                zero,
                                "float_bool",
                            )
                            .unwrap()
                    }
                    _ => self.llvm_context.bool_type().const_int(1, false),
                };

                self.builder
                    .build_conditional_branch(cond_val, body_block, else_block)
                    .unwrap();

                self.builder.position_at_end(body_block);

                self.push_scope(false, true, false);

                self.push_loop(cond_block, end_block);

                for stmt in body {
                    if self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_some()
                    {
                        break;
                    }

                    if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                        return Err(e);
                    }
                }

                if !self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_some()
                {
                    self.builder.build_unconditional_branch(cond_block).unwrap();
                }

                self.pop_loop();

                self.pop_scope();

                self.builder.position_at_end(else_block);

                if !orelse.is_empty() {
                    for stmt in orelse {
                        if self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            break;
                        }

                        if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                            return Err(e);
                        }
                    }
                }

                if !self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_some()
                {
                    self.builder.build_unconditional_branch(end_block).unwrap();
                }

                self.builder.position_at_end(end_block);

                Ok(())
            }
            _ => Err(format!(
                "Statement type not supported in fallback implementation: {:?}",
                stmt
            )),
        }
    }
}
