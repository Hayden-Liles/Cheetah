// In stmt.rs
use crate::ast::{Stmt, Expr};
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::{ExprCompiler, AssignmentCompiler, BinaryOpCompiler};
use crate::compiler::types::Type;

pub trait StmtCompiler<'ctx> {
    /// Compile a statement
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String>;

    /// Allocate a variable on the heap
    fn allocate_heap_variable(&mut self, name: &str, ty: &Type) -> inkwell::values::PointerValue<'ctx>;
}

impl<'ctx> StmtCompiler<'ctx> for CompilationContext<'ctx> {
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            // Compile an expression statement
            Stmt::Expr { value, .. } => {
                // Just compile the expression for its side effects
                let _ = self.compile_expr(value)?;
                Ok(())
            },

            // Compile an assignment statement
            Stmt::Assign { targets, value, .. } => {
                // Debug print
                println!("Compiling assignment statement");

                // Compile the right-hand side expression
                let (val, val_type) = self.compile_expr(value)?;

                // Debug print
                println!("Right-hand side value type: {:?}", val_type);

                // For each target on the left-hand side, assign the value
                for target in targets {
                    // Debug print
                    if let Expr::Name { id, .. } = target.as_ref() {
                        println!("Assigning to variable: {}", id);
                    }

                    self.compile_assignment(target, val, &val_type)?;
                }

                Ok(())
            },

            // Compile an augmented assignment (e.g., x += 1)
            Stmt::AugAssign { target, op, value, .. } => {
                // First get the current value of the target
                let (target_val, target_type) = self.compile_expr(target)?;

                // Then compile the value expression
                let (value_val, value_type) = self.compile_expr(value)?;

                // Perform the binary operation
                let (result_val, result_type) = self.compile_binary_op(
                    target_val, &target_type, op.clone(), value_val, &value_type
                )?;

                // Assign the result back to the target
                self.compile_assignment(target, result_val, &result_type)?;

                Ok(())
            },

            // Compile an annotated assignment (e.g., x: int = 1)
            Stmt::AnnAssign { target,  value, .. } => {
                // For now, we'll ignore the annotation and just handle the assignment
                // In a full implementation, you would check that the value type matches the annotation

                if let Some(val_expr) = value {
                    // Compile the right-hand side expression
                    let (val, val_type) = self.compile_expr(val_expr)?;

                    // Assign to the target
                    self.compile_assignment(target, val, &val_type)?;
                }

                Ok(())
            },

            // Compile an if statement
            Stmt::If { test, body, orelse, .. } => {
                // Get the current function
                let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();

                // Create basic blocks for then, else, and merge
                let then_block = self.llvm_context.append_basic_block(current_function, "then");
                let else_block = self.llvm_context.append_basic_block(current_function, "else");
                let merge_block = self.llvm_context.append_basic_block(current_function, "if.end");

                // Compile the test expression
                let (test_val, test_type) = self.compile_expr(test)?;

                // Convert to boolean if needed
                let cond_val = if test_type != Type::Bool {
                    self.convert_type(test_val, &test_type, &Type::Bool)?.into_int_value()
                } else {
                    test_val.into_int_value()
                };

                // Create the conditional branch
                self.builder.build_conditional_branch(cond_val, then_block, else_block).unwrap();

                // Compile the then block with its own scope
                self.builder.position_at_end(then_block);
                self.push_scope(false, false, false); // Create a new scope for the then block

                // Add nonlocal declarations to the new scope
                // First collect the nonlocal variables from the parent scope
                let nonlocal_vars = if self.scope_stack.scopes.len() >= 2 {
                    let parent_scope = &self.scope_stack.scopes[self.scope_stack.scopes.len() - 2];
                    parent_scope.nonlocal_vars.clone()
                } else {
                    Vec::new()
                };

                // Then declare them in the current scope
                for var_name in nonlocal_vars {
                    self.declare_nonlocal(var_name);
                }

                // Execute the then block
                let mut has_terminator = false;
                for stmt in body {
                    // Compile each statement in the then block
                    match self.compile_stmt(stmt.as_ref()) {
                        Ok(_) => {
                            // Check if the statement was a terminator (break, continue, return)
                            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                has_terminator = true;
                                break;
                            }
                        },
                        Err(e) => {
                            // Log the error but continue with the next statement
                            println!("Error compiling statement in if block: {}", e);
                        }
                    }
                }

                self.pop_scope(); // Pop the scope for the then block

                // Only add a branch to the merge block if we don't already have a terminator
                if !has_terminator && !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                    self.builder.build_unconditional_branch(merge_block).unwrap();
                }

                // Compile the else block with its own scope
                self.builder.position_at_end(else_block);
                self.push_scope(false, false, false); // Create a new scope for the else block

                // Add nonlocal declarations to the new scope
                // First collect the nonlocal variables from the parent scope
                let nonlocal_vars = if self.scope_stack.scopes.len() >= 2 {
                    let parent_scope = &self.scope_stack.scopes[self.scope_stack.scopes.len() - 2];
                    parent_scope.nonlocal_vars.clone()
                } else {
                    Vec::new()
                };

                // Then declare them in the current scope
                for var_name in nonlocal_vars {
                    self.declare_nonlocal(var_name);
                }

                // Execute the else block
                let mut has_terminator = false;
                for stmt in orelse {
                    // Compile each statement in the else block
                    match self.compile_stmt(stmt.as_ref()) {
                        Ok(_) => {
                            // Check if the statement was a terminator (break, continue, return)
                            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                has_terminator = true;
                                break;
                            }
                        },
                        Err(e) => {
                            // Log the error but continue with the next statement
                            println!("Error compiling statement in else block: {}", e);
                        }
                    }
                }

                self.pop_scope(); // Pop the scope for the else block

                // Only add a branch to the merge block if we don't already have a terminator
                if !has_terminator && !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                    self.builder.build_unconditional_branch(merge_block).unwrap();
                }

                // Continue at the merge block
                self.builder.position_at_end(merge_block);

                Ok(())
            },

            // Compile a while loop
            Stmt::While { test, body, orelse, .. } => {
                // Get the current function
                let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();

                // Create basic blocks for condition, loop body, else, and continue
                let cond_block = self.llvm_context.append_basic_block(current_function, "while.cond");
                let body_block = self.llvm_context.append_basic_block(current_function, "while.body");
                let else_block = self.llvm_context.append_basic_block(current_function, "while.else");
                let end_block = self.llvm_context.append_basic_block(current_function, "while.end");

                // Register this loop in the loop stack for break/continue
                self.push_loop(cond_block, end_block);

                // Branch to the condition block to start the loop
                self.builder.build_unconditional_branch(cond_block).unwrap();

                // Compile the condition
                self.builder.position_at_end(cond_block);
                let (test_val, test_type) = self.compile_expr(test)?;

                // Convert to boolean if needed
                let cond_val = if test_type != Type::Bool {
                    self.convert_type(test_val, &test_type, &Type::Bool)?.into_int_value()
                } else {
                    test_val.into_int_value()
                };

                // Create the conditional branch
                self.builder.build_conditional_branch(cond_val, body_block, else_block).unwrap();

                // Compile the loop body with its own scope
                self.builder.position_at_end(body_block);
                self.push_scope(false, true, false); // Create a new scope for the loop body (is_loop=true)

                // Add nonlocal declarations to the new scope
                // First collect the nonlocal variables from the parent scope
                let nonlocal_vars = if self.scope_stack.scopes.len() >= 2 {
                    let parent_scope = &self.scope_stack.scopes[self.scope_stack.scopes.len() - 2];
                    parent_scope.nonlocal_vars.clone()
                } else {
                    Vec::new()
                };

                // Then declare them in the current scope
                for var_name in nonlocal_vars {
                    self.declare_nonlocal(var_name);
                }

                for stmt in body {
                    self.compile_stmt(stmt.as_ref())?;
                }

                self.pop_scope(); // Pop the scope for the loop body

                if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                    self.builder.build_unconditional_branch(cond_block).unwrap();
                }

                // Compile the else block with its own scope (executed when the loop condition becomes false)
                self.builder.position_at_end(else_block);
                self.push_scope(false, false, false); // Create a new scope for the else block

                // Add nonlocal declarations to the new scope
                // First collect the nonlocal variables from the parent scope
                let nonlocal_vars = if self.scope_stack.scopes.len() >= 2 {
                    let parent_scope = &self.scope_stack.scopes[self.scope_stack.scopes.len() - 2];
                    parent_scope.nonlocal_vars.clone()
                } else {
                    Vec::new()
                };

                // Then declare them in the current scope
                for var_name in nonlocal_vars {
                    self.declare_nonlocal(var_name);
                }

                for stmt in orelse {
                    self.compile_stmt(stmt.as_ref())?;
                }

                self.pop_scope(); // Pop the scope for the else block

                if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                    self.builder.build_unconditional_branch(end_block).unwrap();
                }

                // Continue at the end block
                self.builder.position_at_end(end_block);

                // Pop this loop from the stack
                self.pop_loop();

                Ok(())
            },

            // Compile a for loop
            Stmt::For { target, iter, body, orelse, is_async, .. } => {
                // Get the current function
                let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();

                // Create basic blocks for loop initialization, condition, body, increment, else, and end
                let init_block = self.llvm_context.append_basic_block(current_function, "for.init");
                let cond_block = self.llvm_context.append_basic_block(current_function, "for.cond");
                let body_block = self.llvm_context.append_basic_block(current_function, "for.body");
                let increment_block = self.llvm_context.append_basic_block(current_function, "for.inc");
                let else_block = self.llvm_context.append_basic_block(current_function, "for.else");
                let end_block = self.llvm_context.append_basic_block(current_function, "for.end");

                // Register this loop in the loop stack for break/continue
                self.push_loop(increment_block, end_block);

                // Branch to the initialization block
                self.builder.build_unconditional_branch(init_block).unwrap();

                // Initialization block (get iterator)
                self.builder.position_at_end(init_block);

                // Compile the iterator expression
                let (iter_val, iter_type) = self.compile_expr(iter)?;

                // Create an index variable initialized to 0
                let i64_type = self.llvm_context.i64_type();
                let index_ptr = self.builder.build_alloca(i64_type, "for.index").unwrap();
                self.builder.build_store(index_ptr, i64_type.const_int(0, false)).unwrap();

                // Get the length of the iterable
                let len_val = match iter_type {
                    Type::List(_) => {
                        // For lists, get the length using list_len function
                        let list_len_fn = match self.module.get_function("list_len") {
                            Some(f) => f,
                            None => return Err("list_len function not found".to_string()),
                        };

                        let call_site_value = self.builder.build_call(
                            list_len_fn,
                            &[iter_val.into_pointer_value().into()],
                            "list_len_result"
                        ).unwrap();

                        call_site_value.try_as_basic_value().left()
                            .ok_or_else(|| "Failed to get list length".to_string())?
                    },
                    _ => {
                        // For other types (like range), use the value directly
                        iter_val
                    }
                };

                // Branch to the condition block
                self.builder.build_unconditional_branch(cond_block).unwrap();

                // Condition block (check if index < length)
                self.builder.position_at_end(cond_block);
                let index_val = self.builder.build_load(i64_type, index_ptr, "index").unwrap().into_int_value();
                let cond_val = self.builder.build_int_compare(
                    inkwell::IntPredicate::SLT,
                    index_val,
                    len_val.into_int_value(),
                    "loop.cond"
                ).unwrap();
                self.builder.build_conditional_branch(cond_val, body_block, else_block).unwrap();

                // Body block with its own scope
                self.builder.position_at_end(body_block);
                self.push_scope(false, true, false); // Create a new scope for the loop body (is_loop=true)

                // Assign the current index to the target variable
                if let Expr::Name { id, .. } = target.as_ref() {
                    // Always create a new variable in the loop scope
                    println!("Creating loop variable: {}", id);

                    match iter_type {
                        Type::List(elem_type) => {
                            // Get the element from the list
                            let list_ptr = iter_val.into_pointer_value();
                            let element_ptr = self.build_list_get_item(list_ptr, index_val)?;

                            // Allocate the loop variable based on the element type
                            let target_ptr = match *elem_type {
                                Type::Int => {
                                    let i64_type = self.llvm_context.i64_type();
                                    let ptr = self.builder.build_alloca(i64_type, id).unwrap();
                                    let element_val = self.builder.build_load(i64_type, element_ptr, "element").unwrap();
                                    self.builder.build_store(ptr, element_val).unwrap();
                                    ptr
                                },
                                Type::Float => {
                                    let f64_type = self.llvm_context.f64_type();
                                    let ptr = self.builder.build_alloca(f64_type, id).unwrap();
                                    let element_val = self.builder.build_load(f64_type, element_ptr, "element").unwrap();
                                    self.builder.build_store(ptr, element_val).unwrap();
                                    ptr
                                },
                                Type::Bool => {
                                    let bool_type = self.llvm_context.bool_type();
                                    let ptr = self.builder.build_alloca(bool_type, id).unwrap();
                                    let element_val = self.builder.build_load(bool_type, element_ptr, "element").unwrap();
                                    self.builder.build_store(ptr, element_val).unwrap();
                                    ptr
                                },
                                _ => {
                                    // For other types, use a generic pointer
                                    let ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                                    let ptr = self.builder.build_alloca(ptr_type, id).unwrap();
                                    let element_val = self.builder.build_load(ptr_type, element_ptr, "element").unwrap();
                                    self.builder.build_store(ptr, element_val).unwrap();
                                    ptr
                                }
                            };

                            // Register the variable in the current scope
                            self.scope_stack.add_variable(id.clone(), target_ptr, *elem_type.clone());
                        },
                        _ => {
                            // For other types (like range), use the index directly
                            let i64_type = self.llvm_context.i64_type();
                            let target_ptr = self.builder.build_alloca(i64_type, id).unwrap();
                            self.builder.build_store(target_ptr, index_val).unwrap();
                            self.scope_stack.add_variable(id.clone(), target_ptr, Type::Int);
                        }
                    }
                } else {
                    // For now, we only support simple variable targets
                    // In a full implementation, we would handle tuple unpacking and other complex targets
                    return Err("Only simple variable targets are supported for for loops".to_string());
                }

                // Execute the loop body
                for stmt in body {
                    // Compile each statement in the loop body
                    match self.compile_stmt(stmt.as_ref()) {
                        Ok(_) => {
                            // Check if the statement was a terminator (break, continue, return)
                            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                // If we have a terminator, stop processing the rest of the loop body
                                break;
                            }
                        },
                        Err(e) => {
                            // Log the error but continue with the next statement
                            println!("Error compiling statement in for loop: {}", e);
                        }
                    }
                }

                // Check if the block already has a terminator (from break, continue, return)
                if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                    // If not, add a branch to the increment block
                    self.builder.build_unconditional_branch(increment_block).unwrap();
                }

                self.pop_scope(); // Pop the scope for the loop body

                // Increment block
                self.builder.position_at_end(increment_block);
                let next_index = self.builder.build_int_add(
                    index_val,
                    i64_type.const_int(1, false),
                    "index.next"
                ).unwrap();
                self.builder.build_store(index_ptr, next_index).unwrap();
                self.builder.build_unconditional_branch(cond_block).unwrap();

                // Else block with its own scope
                self.builder.position_at_end(else_block);
                self.push_scope(false, false, false); // Create a new scope for the else block

                // Execute the else block
                for stmt in orelse {
                    // Compile each statement in the else block
                    match self.compile_stmt(stmt.as_ref()) {
                        Ok(_) => {
                            // Check if the statement was a terminator (break, continue, return)
                            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                // If we have a terminator, stop processing the rest of the else block
                                break;
                            }
                        },
                        Err(e) => {
                            // Log the error but continue with the next statement
                            println!("Error compiling statement in for loop else block: {}", e);
                        }
                    }
                }

                // Check if the block already has a terminator (from break, continue, return)
                if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                    // If not, add a branch to the end block
                    self.builder.build_unconditional_branch(end_block).unwrap();
                }

                self.pop_scope(); // Pop the scope for the else block

                // End block
                self.builder.position_at_end(end_block);

                // Pop this loop from the stack
                self.pop_loop();

                if *is_async {
                    Err("Async for loops not implemented yet".to_string())
                } else {
                    Ok(())
                }
            },

            // Compile a break statement
            Stmt::Break { .. } => {
                if let Some(break_block) = self.current_break_block() {
                    self.builder.build_unconditional_branch(break_block).unwrap();
                    Ok(())
                } else {
                    Err("Break statement outside of loop".to_string())
                }
            },

            // Compile a continue statement
            Stmt::Continue { .. } => {
                if let Some(continue_block) = self.current_continue_block() {
                    self.builder.build_unconditional_branch(continue_block).unwrap();
                    Ok(())
                } else {
                    Err("Continue statement outside of loop".to_string())
                }
            },

            // Compile a return statement
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    // Compile the return value expression
                    let (ret_val, ret_type) = self.compile_expr(expr)?;

                    // Check if we're returning a tuple from a function that expects an integer
                    // This is a common case in our current implementation where all functions return i64
                    if let Type::Tuple(_) = ret_type {
                        // Get the current function
                        if let Some(current_function) = self.current_function {
                            // Get the return type of the function
                            let return_type = current_function.get_type().get_return_type();

                            // If the function returns an integer but we're returning a tuple,
                            // we need to convert the tuple to a pointer and return that
                            if let Some(ret_type) = return_type {
                                if ret_type.is_int_type() {
                                    // Convert the tuple to a pointer and return that
                                    let ptr_val = if ret_val.is_pointer_value() {
                                        // Already a pointer, just return it
                                        ret_val
                                    } else {
                                        // Allocate memory for the tuple
                                        let tuple_ptr = self.builder.build_alloca(
                                            ret_val.get_type(),
                                            "tuple_return"
                                        ).unwrap();

                                        // Store the tuple in the allocated memory
                                        self.builder.build_store(tuple_ptr, ret_val).unwrap();

                                        // Return the pointer
                                        tuple_ptr.into()
                                    };

                                    // Convert the pointer to an integer
                                    let ptr_int = self.builder.build_ptr_to_int(
                                        ptr_val.into_pointer_value(),
                                        self.llvm_context.i64_type(),
                                        "ptr_to_int"
                                    ).unwrap();

                                    // Return the integer
                                    let return_val: inkwell::values::BasicValueEnum = ptr_int.into();
                                    self.builder.build_return(Some(&return_val)).unwrap();
                                    return Ok(());
                                }
                            }
                        }
                    }

                    // Get the function's return type
                    if let Some(current_function) = self.current_function {
                        let return_type = current_function.get_type().get_return_type();

                        if let Some(ret_type) = return_type {
                            // For functions that return pointers (lists, strings, dictionaries)
                            if ret_type.is_pointer_type() {
                                if ret_val.is_pointer_value() {
                                    // If it's already a pointer, return it directly
                                    self.builder.build_return(Some(&ret_val)).unwrap();
                                    return Ok(());
                                } else {
                                    // Convert the return value to a pointer if needed
                                    let ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                                    let ptr_val = self.builder.build_bit_cast(
                                        ret_val,
                                        ptr_type,
                                        "to_ptr"
                                    ).unwrap();
                                    self.builder.build_return(Some(&ptr_val)).unwrap();
                                    return Ok(());
                                }
                            } else if ret_type.is_int_type() && ret_val.is_pointer_value() {
                                // If the function returns an integer but we have a pointer value,
                                // convert the pointer to an integer
                                let int_type = self.llvm_context.i64_type();
                                let int_val = self.builder.build_ptr_to_int(
                                    ret_val.into_pointer_value(),
                                    int_type,
                                    "ptr_to_int"
                                ).unwrap();
                                self.builder.build_return(Some(&int_val)).unwrap();
                                return Ok(());
                            }
                        }
                    }

                    // Build the return instruction with the value
                    self.builder.build_return(Some(&ret_val)).unwrap();
                } else {
                    // Build a void return
                    self.builder.build_return(None).unwrap();
                }

                Ok(())
            },

            // Compile a pass statement (no-op)
            Stmt::Pass { .. } => {
                // No operation needed
                Ok(())
            },

            // Compile a with statement
            Stmt::With {  body, is_async, .. } => {
                // In a full implementation, we'd need to:
                // 1. Call __enter__ on each context manager
                // 2. Execute the body
                // 3. Call __exit__ in a finally block

                if *is_async {
                    Err("Async with statements not implemented yet".to_string())
                } else {
                    // For now, just execute the body without context management
                    for stmt in body {
                        self.compile_stmt(stmt.as_ref())?;
                    }
                    Ok(())
                }
            },

            // Compile a delete statement
            Stmt::Delete {  .. } => {
                // In Python, delete frees the variable name
                // In our IR, we don't need to do anything special except maybe
                // call destructors for complex types or set pointers to null

                // For now, this is a no-op
                Ok(())
            },

            // Compile an assert statement
            Stmt::Assert { test, msg, .. } => {
                // In a complete implementation, we'd:
                // 1. Evaluate the test expression
                // 2. If false, raise an AssertionError with the optional message

                // For now, just evaluate the test expression for side effects
                let _ = self.compile_expr(test)?;

                if let Some(msg_expr) = msg {
                    let _ = self.compile_expr(msg_expr)?;
                }

                // Assert implementation omitted - we'd need runtime support for exceptions
                Ok(())
            },

            // Compile global declarations
            Stmt::Global { names, .. } => {
                // Register each name as a global variable in the current scope
                for name in names {
                    self.declare_global(name.clone());
                }
                Ok(())
            },

            // Compile nonlocal declarations
            Stmt::Nonlocal { names, .. } => {
                // Register each name as a nonlocal variable in the current scope
                for name in names {
                    self.declare_nonlocal(name.clone());

                    // Check if we're in a nested function
                    if let Some(current_function) = self.current_function {
                        // Get the current function name
                        let fn_name = current_function.get_name().to_string_lossy().to_string();

                        // If this is a nested function (contains a dot in the name)
                        if fn_name.contains('.') {
                            // Just mark the variable as nonlocal in the current scope
                            // The actual handling is done in compile_nested_function_body
                            println!("Marked '{}' as nonlocal in nested function '{}'", name, fn_name);
                        }
                    }
                }
                Ok(())
            },

            // Compile try/except statement
            Stmt::Try { body,  orelse, finalbody, .. } => {
                // In a complete implementation, we'd need to:
                // 1. Set up exception handling blocks
                // 2. Execute the body with proper handlers
                // 3. Handle finally blocks

                // For now, just execute the body and ignore exception handling
                for stmt in body {
                    // Skip errors to avoid crashing
                    let _ = self.compile_stmt(stmt.as_ref());
                }

                // Execute the else block if we didn't skip due to exceptions
                // (which we can't detect yet)
                for stmt in orelse {
                    let _ = self.compile_stmt(stmt.as_ref());
                }

                // Always execute the finally block
                for stmt in finalbody {
                    let _ = self.compile_stmt(stmt.as_ref());
                }

                // Note that this is only partially implemented
                Ok(())
            },

            // Compile a raise statement
            Stmt::Raise { exc, cause, .. } => {
                // In a complete implementation, we'd:
                // 1. Evaluate the exception and cause expressions
                // 2. Call runtime functions to raise the exception

                // For now, just evaluate the expressions for side effects
                if let Some(exc_expr) = exc {
                    let _ = self.compile_expr(exc_expr)?;
                }

                if let Some(cause_expr) = cause {
                    let _ = self.compile_expr(cause_expr)?;
                }

                // Raise implementation omitted - we'd need runtime support for exceptions
                Err("Raise statement not fully implemented yet".to_string())
            },

            // Compile an import statement
            Stmt::Import { .. } | Stmt::ImportFrom { .. } => {
                // Imports require module management and runtime support
                // For now, just treat them as no-ops
                Ok(())
            },

            // Compile match statement (Python 3.10+)
            Stmt::Match { subject,  .. } => {
                // Pattern matching requires significant runtime support
                // For now, just evaluate the subject for side effects and report not implemented
                let _ = self.compile_expr(subject)?;

                Err("Match statements not implemented yet".to_string())
            },

            // Class definitions are handled separately in the Compiler
            Stmt::ClassDef { .. } => {
                Ok(()) // Classes are handled at module level
            },

            // Function definitions within other functions (nested functions)
            Stmt::FunctionDef { name, params, body, .. } => {
                // Get the current function name to create a qualified name for the nested function
                let parent_function_name = if let Some(current_function) = self.current_function {
                    // Get the function name from the LLVM function value
                    let fn_name = current_function.get_name().to_string_lossy().to_string();
                    Some(fn_name)
                } else {
                    None
                };

                // Create a qualified name for the nested function
                let qualified_name = if let Some(parent) = parent_function_name {
                    format!("{}.{}", parent, name)
                } else {
                    name.clone()
                };

                // Declare the nested function
                self.declare_nested_function(&qualified_name, params)?;

                // Compile the nested function body
                self.compile_nested_function_body(&qualified_name, params, body)?;

                Ok(())
            },
        }
    }

    fn allocate_heap_variable(&mut self, name: &str, ty: &Type) -> inkwell::values::PointerValue<'ctx> {
        // Delegate to the CompilationContext's allocate_heap_variable method
        self.allocate_heap_variable(name, ty)
    }
}