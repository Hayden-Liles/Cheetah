// Non-recursive implementation of the statement compiler
// This implementation avoids deep recursion by using an explicit work stack

use crate::ast::{Expr, Stmt};
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::{AssignmentCompiler, BinaryOpCompiler, ExprCompiler};
use crate::compiler::stmt::StmtCompiler;
use crate::compiler::types::Type;
use inkwell::values::BasicValueEnum;
use std::collections::{HashMap, VecDeque};

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

                    // Get the boxed_any_to_int function for converting BoxedAny pointers to integers
                    let boxed_any_to_int_fn = self.module.get_function("boxed_any_to_int")
                        .ok_or_else(|| "boxed_any_to_int function not found".to_string())?;

                    match args.len() {
                        1 => {
                            // range(stop)
                            let (stop_val, stop_type) = self.compile_expr(&args[0])?;
                            let start_val = i64_type.const_int(0, false); // start = 0
                            let step_val = i64_type.const_int(1, false);  // step = 1

                            let stop_val = if stop_type != Type::Int {
                                self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                            } else if stop_val.is_pointer_value() {
                                // Check if it's a BoxedAny pointer
                                if stop_type == Type::Any {
                                    // Convert BoxedAny to int
                                    let call_site_value = self.builder.build_call(
                                        boxed_any_to_int_fn,
                                        &[stop_val.into()],
                                        "boxed_to_int"
                                    ).unwrap();

                                    call_site_value.try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert BoxedAny to int".to_string())?
                                        .into_int_value()
                                } else {
                                    // Regular pointer, load the value
                                    self.builder
                                        .build_load(i64_type, stop_val.into_pointer_value(), "range_stop")
                                        .unwrap()
                                        .into_int_value()
                                }
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

                            // Check if we should use boxed_range_2 instead
                            if start_type == Type::Any || stop_type == Type::Any {
                                // Convert the values to BoxedAny pointers if needed
                                let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                                    .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                                let boxed_start_val = if start_type == Type::Any {
                                    start_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_start_val = if start_type != Type::Int {
                                        self.convert_type(start_val, &start_type, &Type::Int)?.into_int_value()
                                    } else if start_val.is_pointer_value() {
                                        self.builder
                                            .build_load(i64_type, start_val.into_pointer_value(), "range_start")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        start_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_start_val.into()],
                                        "boxed_start"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                let boxed_stop_val = if stop_type == Type::Any {
                                    stop_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_stop_val = if stop_type != Type::Int {
                                        self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                                    } else if stop_val.is_pointer_value() {
                                        self.builder
                                            .build_load(i64_type, stop_val.into_pointer_value(), "range_stop")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        stop_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_stop_val.into()],
                                        "boxed_stop"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                // Use boxed_range_2 instead of range_2
                                let boxed_range_2_fn = match self.module.get_function("boxed_range_2") {
                                    Some(f) => f,
                                    None => {
                                        return Err("boxed_range_2 function not found".to_string())
                                    }
                                };

                                let call_site_value = self.builder.build_call(
                                    boxed_range_2_fn,
                                    &[boxed_start_val.into(), boxed_stop_val.into()],
                                    "boxed_range_2_result"
                                ).unwrap();

                                let range_val = call_site_value
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or_else(|| "Failed to get range value".to_string())?
                                    .into_int_value();

                                return Ok(Some((i64_type.const_int(0, false), range_val, i64_type.const_int(1, false))));
                            }

                            // Regular case with integer values
                            let start_val = if start_type != Type::Int {
                                self.convert_type(start_val, &start_type, &Type::Int)?.into_int_value()
                            } else if start_val.is_pointer_value() {
                                // Check if it's a BoxedAny pointer
                                if start_type == Type::Any {
                                    // Convert BoxedAny to int
                                    let call_site_value = self.builder.build_call(
                                        boxed_any_to_int_fn,
                                        &[start_val.into()],
                                        "boxed_to_int"
                                    ).unwrap();

                                    call_site_value.try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert BoxedAny to int".to_string())?
                                        .into_int_value()
                                } else {
                                    // Regular pointer, load the value
                                    self.builder
                                        .build_load(i64_type, start_val.into_pointer_value(), "range_start")
                                        .unwrap()
                                        .into_int_value()
                                }
                            } else {
                                start_val.into_int_value()
                            };

                            let stop_val = if stop_type != Type::Int {
                                self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                            } else if stop_val.is_pointer_value() {
                                // Check if it's a BoxedAny pointer
                                if stop_type == Type::Any {
                                    // Convert BoxedAny to int
                                    let call_site_value = self.builder.build_call(
                                        boxed_any_to_int_fn,
                                        &[stop_val.into()],
                                        "boxed_to_int"
                                    ).unwrap();

                                    call_site_value.try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert BoxedAny to int".to_string())?
                                        .into_int_value()
                                } else {
                                    // Regular pointer, load the value
                                    self.builder
                                        .build_load(i64_type, stop_val.into_pointer_value(), "range_stop")
                                        .unwrap()
                                        .into_int_value()
                                }
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

                            // Check if we should use boxed_range_3 instead
                            if start_type == Type::Any || stop_type == Type::Any || step_type == Type::Any {
                                // Convert the values to BoxedAny pointers if needed
                                let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                                    .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                                let boxed_start_val = if start_type == Type::Any {
                                    start_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_start_val = if start_type != Type::Int {
                                        self.convert_type(start_val, &start_type, &Type::Int)?.into_int_value()
                                    } else if start_val.is_pointer_value() {
                                        self.builder
                                            .build_load(i64_type, start_val.into_pointer_value(), "range_start")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        start_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_start_val.into()],
                                        "boxed_start"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                let boxed_stop_val = if stop_type == Type::Any {
                                    stop_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_stop_val = if stop_type != Type::Int {
                                        self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                                    } else if stop_val.is_pointer_value() {
                                        self.builder
                                            .build_load(i64_type, stop_val.into_pointer_value(), "range_stop")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        stop_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_stop_val.into()],
                                        "boxed_stop"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                let boxed_step_val = if step_type == Type::Any {
                                    step_val
                                } else {
                                    // Convert to BoxedAny
                                    let int_step_val = if step_type != Type::Int {
                                        self.convert_type(step_val, &step_type, &Type::Int)?.into_int_value()
                                    } else if step_val.is_pointer_value() {
                                        self.builder
                                            .build_load(i64_type, step_val.into_pointer_value(), "range_step")
                                            .unwrap()
                                            .into_int_value()
                                    } else {
                                        step_val.into_int_value()
                                    };

                                    self.builder.build_call(
                                        boxed_any_from_int_fn,
                                        &[int_step_val.into()],
                                        "boxed_step"
                                    ).unwrap().try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert int to BoxedAny".to_string())?
                                };

                                // Use boxed_range_3 instead of range_3
                                let boxed_range_3_fn = match self.module.get_function("boxed_range_3") {
                                    Some(f) => f,
                                    None => {
                                        return Err("boxed_range_3 function not found".to_string())
                                    }
                                };

                                let call_site_value = self.builder.build_call(
                                    boxed_range_3_fn,
                                    &[boxed_start_val.into(), boxed_stop_val.into(), boxed_step_val.into()],
                                    "boxed_range_3_result"
                                ).unwrap();

                                let range_val = call_site_value
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or_else(|| "Failed to get range value".to_string())?
                                    .into_int_value();

                                // We return a dummy start and step value since the range_val already contains the full range
                                return Ok(Some((i64_type.const_int(0, false), range_val, i64_type.const_int(1, false))));
                            }

                            // Regular case with integer values
                            let start_val = if start_type != Type::Int {
                                self.convert_type(start_val, &start_type, &Type::Int)?.into_int_value()
                            } else if start_val.is_pointer_value() {
                                // Check if it's a BoxedAny pointer
                                if start_type == Type::Any {
                                    // Convert BoxedAny to int
                                    let call_site_value = self.builder.build_call(
                                        boxed_any_to_int_fn,
                                        &[start_val.into()],
                                        "boxed_to_int"
                                    ).unwrap();

                                    call_site_value.try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert BoxedAny to int".to_string())?
                                        .into_int_value()
                                } else {
                                    // Regular pointer, load the value
                                    self.builder
                                        .build_load(i64_type, start_val.into_pointer_value(), "range_start")
                                        .unwrap()
                                        .into_int_value()
                                }
                            } else {
                                start_val.into_int_value()
                            };

                            let stop_val = if stop_type != Type::Int {
                                self.convert_type(stop_val, &stop_type, &Type::Int)?.into_int_value()
                            } else if stop_val.is_pointer_value() {
                                // Check if it's a BoxedAny pointer
                                if stop_type == Type::Any {
                                    // Convert BoxedAny to int
                                    let call_site_value = self.builder.build_call(
                                        boxed_any_to_int_fn,
                                        &[stop_val.into()],
                                        "boxed_to_int"
                                    ).unwrap();

                                    call_site_value.try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert BoxedAny to int".to_string())?
                                        .into_int_value()
                                } else {
                                    // Regular pointer, load the value
                                    self.builder
                                        .build_load(i64_type, stop_val.into_pointer_value(), "range_stop")
                                        .unwrap()
                                        .into_int_value()
                                }
                            } else {
                                stop_val.into_int_value()
                            };

                            let step_val = if step_type != Type::Int {
                                self.convert_type(step_val, &step_type, &Type::Int)?.into_int_value()
                            } else if step_val.is_pointer_value() {
                                // Check if it's a BoxedAny pointer
                                if step_type == Type::Any {
                                    // Convert BoxedAny to int
                                    let call_site_value = self.builder.build_call(
                                        boxed_any_to_int_fn,
                                        &[step_val.into()],
                                        "boxed_to_int"
                                    ).unwrap();

                                    call_site_value.try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to convert BoxedAny to int".to_string())?
                                        .into_int_value()
                                } else {
                                    // Regular pointer, load the value
                                    self.builder
                                        .build_load(i64_type, step_val.into_pointer_value(), "range_step")
                                        .unwrap()
                                        .into_int_value()
                                }
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
            // If we're using BoxedAny values, we need to create a BoxedAny pointer
            if self.use_boxed_values {
                // Create a pointer to store the integer value
                let ptr = self.builder.build_alloca(i64_type, &format!("{}_raw", id)).unwrap();

                // Store the initial value in the raw pointer
                self.builder.build_store(ptr, start_val).unwrap();

                // Get the boxed_any_from_int function
                let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                    .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                // Call boxed_any_from_int to create a BoxedAny value
                let call_site_value = self.builder.build_call(
                    boxed_any_from_int_fn,
                    &[start_val.into()],
                    &format!("box_{}", id)
                ).unwrap();

                let boxed_val = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| format!("Failed to create BoxedAny for {}", id))?;

                // Create a pointer to store the BoxedAny pointer
                let boxed_ptr = self.builder.build_alloca(
                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                    id
                ).unwrap();

                // Store the BoxedAny pointer
                self.builder.build_store(boxed_ptr, boxed_val).unwrap();

                // Add the variable to the scope stack with type Any
                self.scope_stack.add_variable(id.clone(), boxed_ptr, Type::Any);

                // Return the raw pointer for the loop logic
                ptr
            } else {
                // Regular case, just create an integer pointer
                let ptr = self.builder.build_alloca(i64_type, id).unwrap();
                self.scope_stack.add_variable(id.clone(), ptr, Type::Int);
                ptr
            }
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

        // Store the updated value in the raw pointer
        self.builder.build_store(var_ptr, next_val).unwrap();

        // If we're using BoxedAny values, we need to update the BoxedAny value as well
        if self.use_boxed_values {
            if let Expr::Name { id, .. } = target {
                // Get the boxed_any_from_int function
                let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                    .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                // Call boxed_any_from_int to create a new BoxedAny value
                let call_site_value = self.builder.build_call(
                    boxed_any_from_int_fn,
                    &[next_val.into()],
                    &format!("box_{}_inc", id)
                ).unwrap();

                let boxed_val = call_site_value.try_as_basic_value().left()
                    .ok_or_else(|| format!("Failed to create BoxedAny for {}", id))?;

                // Get the BoxedAny pointer from the scope stack
                if let Some(boxed_ptr) = self.scope_stack.get_variable(id) {
                    // Store the new BoxedAny pointer
                    self.builder.build_store(*boxed_ptr, boxed_val).unwrap();
                }
            }
        }

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

                        // Push a new scope for the then block
                        self.push_scope(false, false, false);

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

                        // Pop the scope for the then block
                        self.pop_scope();

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

                        // Push a new scope for the else block
                        self.push_scope(false, false, false);

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

                        // Pop the scope for the else block
                        self.pop_scope();

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

                                        // If we're using BoxedAny values, use a pointer type for the local variable
                                        let local_ptr = if self.use_boxed_values {
                                            self.builder
                                                .build_alloca(
                                                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                                    &unique_name,
                                                )
                                                .unwrap()
                                        } else {
                                            self.builder
                                                .build_alloca(
                                                    self.get_llvm_type(&var_type).into_int_type(),
                                                    &unique_name,
                                                )
                                                .unwrap()
                                        };

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
                                    let var_type = if self.use_boxed_values {
                                        Type::Any
                                    } else {
                                        Type::Int
                                    };

                                    self.register_variable(name.clone(), var_type.clone());

                                    let global_var = if self.use_boxed_values {
                                        // For BoxedAny, we use a pointer type
                                        self.module.add_global(
                                            self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                            None,
                                            &name,
                                        )
                                    } else {
                                        // For regular values, we use the original type
                                        self.module.add_global(
                                            self.get_llvm_type(&var_type).into_int_type(),
                                            None,
                                            &name,
                                        )
                                    };

                                    if self.use_boxed_values {
                                        // Initialize with null pointer (None)
                                        global_var.set_initializer(
                                            &self.llvm_context.ptr_type(inkwell::AddressSpace::default()).const_null(),
                                        );
                                    } else {
                                        // Initialize with zero
                                        global_var.set_initializer(
                                            &self.llvm_context.i64_type().const_zero(),
                                        );
                                    }

                                    let ptr = global_var.as_pointer_value();

                                    if let Some(global_scope) = self.scope_stack.global_scope_mut() {
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
                            // If we're using BoxedAny values, we need to create a BoxedAny pointer
                            if self.use_boxed_values {
                                // Create a pointer to store the integer value
                                let ptr = self.builder.build_alloca(i64_type, &format!("{}_raw", id)).unwrap();

                                // Create a pointer to store the BoxedAny pointer
                                let boxed_ptr = self.builder.build_alloca(
                                    self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                    id
                                ).unwrap();

                                // Add the variable to the scope stack with type Any
                                self.scope_stack.add_variable(id.clone(), boxed_ptr, Type::Any);

                                // Return the raw pointer for the loop logic
                                ptr
                            } else {
                                // Regular case, just create an integer pointer
                                let ptr = self.builder.build_alloca(i64_type, id).unwrap();
                                self.scope_stack.add_variable(id.clone(), ptr, Type::Int);
                                ptr
                            }
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

                        // Store the index value in the raw pointer
                        self.builder.build_store(var_ptr, index_val).unwrap();

                        // If we're using BoxedAny values, we need to update the BoxedAny value as well
                        if self.use_boxed_values {
                            if let Expr::Name { id, .. } = target {
                                // Get the boxed_any_from_int function
                                let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                                    .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                                // Call boxed_any_from_int to create a BoxedAny value
                                let call_site_value = self.builder.build_call(
                                    boxed_any_from_int_fn,
                                    &[index_val.into()],
                                    &format!("box_{}", id)
                                ).unwrap();

                                let boxed_val = call_site_value.try_as_basic_value().left()
                                    .ok_or_else(|| format!("Failed to create BoxedAny for {}", id))?;

                                // Get the BoxedAny pointer from the scope stack
                                if let Some(boxed_ptr) = self.scope_stack.get_variable(id) {
                                    // Store the BoxedAny pointer
                                    self.builder.build_store(*boxed_ptr, boxed_val).unwrap();
                                }
                            }
                        }

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
                                // Check if the function returns a pointer (BoxedAny)
                                if ret_type.is_pointer_type() {
                                    // If the return value is already a pointer, return it directly
                                    if ret_val.is_pointer_value() {
                                        self.builder.build_return(Some(&ret_val)).unwrap();
                                        return Ok(());
                                    } else {
                                        // Convert the value to a BoxedAny pointer
                                        let boxed_val = match value_type {
                                            Some(Type::Int) => {
                                                let boxed_any_from_int_fn = self.module.get_function("boxed_any_from_int")
                                                    .ok_or_else(|| "boxed_any_from_int function not found".to_string())?;

                                                let call_site_value = self.builder.build_call(
                                                    boxed_any_from_int_fn,
                                                    &[ret_val.into()],
                                                    "boxed_return"
                                                ).unwrap();

                                                call_site_value.try_as_basic_value().left()
                                                    .ok_or_else(|| "Failed to create BoxedAny from int".to_string())?
                                            },
                                            Some(Type::Float) => {
                                                let boxed_any_from_float_fn = self.module.get_function("boxed_any_from_float")
                                                    .ok_or_else(|| "boxed_any_from_float function not found".to_string())?;

                                                let call_site_value = self.builder.build_call(
                                                    boxed_any_from_float_fn,
                                                    &[ret_val.into()],
                                                    "boxed_return"
                                                ).unwrap();

                                                call_site_value.try_as_basic_value().left()
                                                    .ok_or_else(|| "Failed to create BoxedAny from float".to_string())?
                                            },
                                            Some(Type::Bool) => {
                                                let boxed_any_from_bool_fn = self.module.get_function("boxed_any_from_bool")
                                                    .ok_or_else(|| "boxed_any_from_bool function not found".to_string())?;

                                                let call_site_value = self.builder.build_call(
                                                    boxed_any_from_bool_fn,
                                                    &[ret_val.into()],
                                                    "boxed_return"
                                                ).unwrap();

                                                call_site_value.try_as_basic_value().left()
                                                    .ok_or_else(|| "Failed to create BoxedAny from bool".to_string())?
                                            },
                                            _ => {
                                                // For other types, return None
                                                let boxed_any_none_fn = self.module.get_function("boxed_any_none")
                                                    .ok_or_else(|| "boxed_any_none function not found".to_string())?;

                                                let call_site_value = self.builder.build_call(
                                                    boxed_any_none_fn,
                                                    &[],
                                                    "none_return"
                                                ).unwrap();

                                                call_site_value.try_as_basic_value().left()
                                                    .ok_or_else(|| "Failed to create None value for return".to_string())?
                                            }
                                        };

                                        self.builder.build_return(Some(&boxed_val)).unwrap();
                                        return Ok(());
                                    }
                                } else if ret_type.is_int_type() && ret_val.is_pointer_value() {
                                    // If the function returns an integer but we have a pointer,
                                    // try to load the integer value from the pointer
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

                        // Default case: just return the value as is
                        self.builder.build_return(Some(&ret_val)).unwrap();
                    } else {
                        // No return value, return None
                        if let Some(current_function) = self.current_function {
                            let return_type = current_function.get_type().get_return_type();

                            if let Some(ret_type) = return_type {
                                if ret_type.is_pointer_type() {
                                    // If the function returns a pointer, return None as a BoxedAny
                                    let boxed_any_none_fn = self.module.get_function("boxed_any_none")
                                        .ok_or_else(|| "boxed_any_none function not found".to_string())?;

                                    let call_site_value = self.builder.build_call(
                                        boxed_any_none_fn,
                                        &[],
                                        "none_return"
                                    ).unwrap();

                                    let none_val = call_site_value.try_as_basic_value().left()
                                        .ok_or_else(|| "Failed to create None value for return".to_string())?;

                                    self.builder.build_return(Some(&none_val)).unwrap();
                                    return Ok(());
                                }
                            }
                        }

                        // Default case: return void
                        self.builder.build_return(None).unwrap();
                    }
                }

                StmtTask::ProcessFunctionDef {
                    name,
                    params,
                    body,
                    is_nested,
                } => {
                    // Create a custom function declaration for the non-recursive implementation
                    // that uses BoxedAny pointers for parameters and return value
                    let context = self.llvm_context;

                    // Create parameter types - all parameters are BoxedAny pointers
                    let mut param_types = Vec::new();
                    for _ in params {
                        param_types.push(context.ptr_type(inkwell::AddressSpace::default()).into());
                    }

                    // Add environment pointer for nested functions
                    if is_nested {
                        let env_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
                        param_types.push(env_ptr_type.into());
                    }

                    // Return type is also a BoxedAny pointer
                    let return_type = context.ptr_type(inkwell::AddressSpace::default());
                    let function_type = return_type.fn_type(&param_types, false);

                    // Add the function to the module
                    let function = self.module.add_function(&name, function_type, None);
                    self.functions.insert(name.clone(), function);

                    // If this is a nested function, register it with the parent function name
                    if is_nested {
                        println!("Registered nested function: {}", name);
                    }

                    // Create the function body
                    let basic_block = context.append_basic_block(function, "entry");
                    let current_block = self.builder.get_insert_block();
                    self.builder.position_at_end(basic_block);

                    // Push a new scope for the function
                    self.push_scope(true, false, false);

                    // Store the parameters in local variables
                    let mut local_vars = HashMap::new();
                    for (i, param) in params.iter().enumerate() {
                        let param_value = function.get_nth_param(i as u32).unwrap();

                        // Allocate space for the parameter
                        let alloca = self.builder
                            .build_alloca(
                                context.ptr_type(inkwell::AddressSpace::default()),
                                &param.name
                            )
                            .unwrap();

                        // Store the parameter value
                        self.builder.build_store(alloca, param_value).unwrap();

                        // Add the parameter to the scope
                        local_vars.insert(param.name.clone(), alloca);
                        self.add_variable_to_scope(param.name.clone(), alloca, Type::Any);
                        self.register_variable(param.name.clone(), Type::Any);
                    }

                    // Handle environment pointer for nested functions
                    if is_nested {
                        let env_param = function.get_nth_param(params.len() as u32).unwrap();
                        let env_alloca = self.builder
                            .build_alloca(
                                context.ptr_type(inkwell::AddressSpace::default()),
                                "env_ptr"
                            )
                            .unwrap();

                        self.builder.build_store(env_alloca, env_param).unwrap();
                    }

                    // Save the current function and local variables
                    let old_function = self.current_function;
                    let old_local_vars = std::mem::replace(&mut self.local_vars, local_vars);

                    // Set the current function
                    self.current_function = Some(function);

                    // Compile the function body
                    for stmt in body {
                        self.compile_stmt_non_recursive(stmt.as_ref())?;
                    }

                    // Add a default return if the function doesn't have one
                    if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                        // Return None as the default value
                        let boxed_any_none_fn = self.module.get_function("boxed_any_none")
                            .ok_or_else(|| "boxed_any_none function not found".to_string())?;

                        let call_site_value = self.builder.build_call(
                            boxed_any_none_fn,
                            &[],
                            "default_return"
                        ).unwrap();

                        let none_val = call_site_value.try_as_basic_value().left()
                            .ok_or_else(|| "Failed to create None value for default return".to_string())?;

                        self.builder.build_return(Some(&none_val)).unwrap();
                    }

                    // Restore the previous function and local variables
                    self.current_function = old_function;
                    self.local_vars = old_local_vars;

                    // Restore the previous insertion point
                    if let Some(block) = current_block {
                        self.builder.position_at_end(block);
                    }

                    // Pop the function scope
                    self.pop_scope();
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
