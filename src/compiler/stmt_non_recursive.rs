// Non-recursive implementation of the statement compiler
// This implementation avoids deep recursion by using an explicit work stack

use crate::ast::{Stmt, Expr};
use crate::compiler::context::CompilationContext;
use crate::compiler::expr::{ExprCompiler, AssignmentCompiler, BinaryOpCompiler};
use crate::compiler::stmt::StmtCompiler;
use crate::compiler::types::Type;
use inkwell::values::BasicValueEnum;
use inkwell::types::BasicTypeEnum;
use std::collections::VecDeque;

// This trait is used to extend the CompilationContext with non-recursive statement compilation
pub trait StmtNonRecursive<'ctx> {
    fn compile_stmt_non_recursive(&mut self, stmt: &Stmt) -> Result<(), String>;

    // This is a helper method for the non-recursive implementation
    fn compile_stmt_fallback(&mut self, stmt: &Stmt) -> Result<(), String>;

    // Helper method to convert a value to a boolean for conditional branches
    fn convert_to_bool(&self, value: BasicValueEnum<'ctx>) -> inkwell::values::IntValue<'ctx>;
}

// Task for the work stack
enum StmtTask<'a, 'ctx> {
    // Execute a statement
    Execute(&'a Stmt),

    // Execute a block of statements
    ExecuteBlock {
        stmts: &'a [Box<Stmt>],
        index: usize,
    },



    // Process a for loop after the iterator is evaluated
    ProcessFor {
        target: &'a Expr,
        body: &'a [Box<Stmt>],
        orelse: &'a [Box<Stmt>],
        iter: &'a Expr,
    },

    // Process a while loop after the condition is evaluated
    ProcessWhile {
        test: &'a Expr,
        body: &'a [Box<Stmt>],
        orelse: &'a [Box<Stmt>],
    },

    // Removed unused variant ContinueLoop
    // This was previously used for loop optimization but is no longer needed

    // Process a try statement
    ProcessTry {
        body: &'a [Box<Stmt>],
        handlers: &'a [crate::ast::ExceptHandler],
        orelse: &'a [Box<Stmt>],
        finalbody: &'a [Box<Stmt>],
    },

    // Process a with statement
    ProcessWith {
        body: &'a [Box<Stmt>],
    },

    // Process an assignment after the value is evaluated
    ProcessAssign {
        targets: &'a [Box<Expr>],
        value_val: BasicValueEnum<'ctx>,
        value_type: Type,
    },

    // Process a return statement after the value is evaluated
    ProcessReturn {
        value_val: Option<BasicValueEnum<'ctx>>,
        value_type: Option<Type>,
    },

    // Process a function definition
    ProcessFunctionDef {
        name: String,
        params: &'a [crate::ast::Parameter],
        body: &'a [Box<Stmt>],
        is_nested: bool,
    },
}

impl<'ctx> StmtNonRecursive<'ctx> for CompilationContext<'ctx> {
    // Helper method to convert a value to a boolean for conditional branches
    fn convert_to_bool(&self, value: BasicValueEnum<'ctx>) -> inkwell::values::IntValue<'ctx> {
        match value {
            BasicValueEnum::IntValue(int_val) => {
                // If it's already a boolean (i1), use it directly
                if int_val.get_type().get_bit_width() == 1 {
                    return int_val;
                }

                // Otherwise, compare with zero
                let zero = int_val.get_type().const_zero();
                self.builder.build_int_compare(inkwell::IntPredicate::NE, int_val, zero, "bool_conv").unwrap()
            },
            BasicValueEnum::FloatValue(float_val) => {
                // Compare with zero
                let zero = float_val.get_type().const_float(0.0);
                self.builder.build_float_compare(inkwell::FloatPredicate::ONE, float_val, zero, "float_bool").unwrap()
            },
            _ => {
                // Default to true for other types
                self.llvm_context.bool_type().const_int(1, false)
            }
        }
    }
    // Non-recursive implementation of compile_stmt
    fn compile_stmt_non_recursive(&mut self, stmt: &Stmt) -> Result<(), String> {
        // Create work stack
        let mut work_stack: VecDeque<StmtTask> = VecDeque::new();

        // Start by executing the top-level statement
        work_stack.push_back(StmtTask::Execute(stmt));

        // Process tasks until the work stack is empty
        while let Some(task) = work_stack.pop_front() {
            match task {
                StmtTask::Execute(stmt) => {
                    match stmt {
                        // Compile an expression statement
                        Stmt::Expr { value, .. } => {
                            // Just compile the expression for its side effects
                            let _ = self.compile_expr(value)?;
                        },

                        // Compile an assignment statement
                        Stmt::Assign { targets, value, .. } => {
                            // Compile the right-hand side expression
                            let (val, val_type) = self.compile_expr(value)?;

                            // Add a task to process the assignment after the value is evaluated
                            work_stack.push_front(StmtTask::ProcessAssign {
                                targets,
                                value_val: val,
                                value_type: val_type,
                            });
                        },

                        // Compile an augmented assignment (e.g., x += 1)
                        Stmt::AugAssign { target, op, value, .. } => {
                            // Compile the target and value
                            let (target_val, target_type) = self.compile_expr(target)?;
                            let (value_val, value_type) = self.compile_expr(value)?;

                            // Perform the binary operation
                            let (result_val, result_type) = self.compile_binary_op(
                                target_val,
                                &target_type,
                                op.clone(),
                                value_val,
                                &value_type
                            )?;

                            // Assign the result back to the target
                            self.compile_assignment(target, result_val, &result_type)?;
                        },

                        // Compile an annotated assignment (e.g., x: int = 1)
                        Stmt::AnnAssign { target, value, .. } => {
                            if let Some(val_expr) = value {
                                // Compile the right-hand side expression
                                let (val, val_type) = self.compile_expr(val_expr)?;

                                // Assign to the target
                                self.compile_assignment(target, val, &val_type)?;
                            }
                        },

                        // Compile an if statement
                        Stmt::If { test, body, orelse, .. } => {
                            // Compile the test expression
                            let (test_val, _) = self.compile_expr(test)?;

                            // Convert to boolean using our helper method
                            let bool_val = self.convert_to_bool(test_val);

                            // Get the current function
                            let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();

                            // Create basic blocks for the then, else, and end parts
                            let then_block = self.llvm_context.append_basic_block(function, "then");
                            let else_block = self.llvm_context.append_basic_block(function, "else");
                            let end_block = self.llvm_context.append_basic_block(function, "endif");

                            // Branch based on the condition
                            self.builder.build_conditional_branch(bool_val, then_block, else_block).unwrap();

                            // Position at the then block
                            self.builder.position_at_end(then_block);

                            // Execute the then block
                            for stmt in body {
                                // Check if the current block already has a terminator
                                if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                    break;
                                }

                                // Execute the statement directly
                                // Non-recursive implementations are always used


                                if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                                    return Err(e);
                                }

                                // Non-recursive implementations are always used
                            }

                            // Check if the block already has a terminator (from break, continue, return)
                            if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                // If not, add a branch to the end block
                                self.builder.build_unconditional_branch(end_block).unwrap();
                            }

                            // Position at the else block
                            self.builder.position_at_end(else_block);

                            // Execute the else block
                            for stmt in orelse {
                                // Check if the current block already has a terminator
                                if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                    break;
                                }

                                // Execute the statement directly
                                // Non-recursive implementations are always used


                                if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                                    return Err(e);
                                }

                                // Non-recursive implementations are always used
                            }

                            // Check if the block already has a terminator (from break, continue, return)
                            if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                // If not, add a branch to the end block
                                self.builder.build_unconditional_branch(end_block).unwrap();
                            }

                            // Position at the end block for further code
                            self.builder.position_at_end(end_block);
                        },

                        // Compile a for loop
                        Stmt::For { target, iter, body, orelse, .. } => {
                            // Compile the iterator expression
                            let (_iter_val, _iter_type) = self.compile_expr(iter)?;

                            // Add a task to process the for loop after the iterator is evaluated
                            work_stack.push_front(StmtTask::ProcessFor {
                                target,
                                body,
                                orelse,
                                iter,
                            });
                        },

                        // Compile a while loop
                        Stmt::While { test, body, orelse, .. } => {
                            // Add a task to process the while loop
                            work_stack.push_front(StmtTask::ProcessWhile {
                                test,
                                body,
                                orelse,
                            });
                        },

                        // Compile a return statement
                        Stmt::Return { value, .. } => {
                            if let Some(expr) = value {
                                // Compile the return value expression
                                let (ret_val, ret_type) = self.compile_expr(expr)?;

                                // Add a task to process the return statement after the value is evaluated
                                work_stack.push_front(StmtTask::ProcessReturn {
                                    value_val: Some(ret_val),
                                    value_type: Some(ret_type),
                                });
                            } else {
                                // Add a task to process a void return
                                work_stack.push_front(StmtTask::ProcessReturn {
                                    value_val: None,
                                    value_type: None,
                                });
                            }
                        },

                        // Compile a pass statement (no-op)
                        Stmt::Pass { .. } => {
                            // No operation needed
                        },

                        // Compile a with statement
                        Stmt::With { body, .. } => {
                            // Add a task to process the with statement
                            work_stack.push_front(StmtTask::ProcessWith {
                                body,
                            });
                        },

                        // Compile a try statement
                        Stmt::Try { body, handlers, orelse, finalbody, .. } => {
                            // Add a task to process the try statement
                            work_stack.push_front(StmtTask::ProcessTry {
                                body,
                                handlers,
                                orelse,
                                finalbody,
                            });
                        },

                        // Compile a break statement
                        Stmt::Break { .. } => {
                            // Get the current loop's break block
                            if let Some(break_block) = self.current_break_block() {
                                // Build a branch to the break block
                                self.builder.build_unconditional_branch(break_block).unwrap();
                            } else {
                                return Err("Break statement outside of loop".to_string());
                            }
                        },

                        // Compile a continue statement
                        Stmt::Continue { .. } => {
                            // Get the current loop's continue block
                            if let Some(continue_block) = self.current_continue_block() {
                                // Build a branch to the continue block
                                self.builder.build_unconditional_branch(continue_block).unwrap();
                            } else {
                                return Err("Continue statement outside of loop".to_string());
                            }
                        },

                        // Function definitions
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
                            let qualified_name = if let Some(parent) = &parent_function_name {
                                format!("{}.{}", parent, name)
                            } else {
                                name.clone()
                            };

                            // Add a task to process the function definition
                            work_stack.push_front(StmtTask::ProcessFunctionDef {
                                name: qualified_name,
                                params,
                                body,
                                is_nested: parent_function_name.is_some(),
                            });
                        },

                        // Nonlocal declarations
                        Stmt::Nonlocal { names, .. } => {
                            // Register each name as a nonlocal variable in the current scope
                            for name in names {
                                // For nonlocal declarations, we check if the variable exists in any outer scope
                                // This is for debugging purposes only, as we'll proceed with the declaration regardless

                                // First check the immediate outer scope (this handles shadowing correctly)
                                if self.scope_stack.scopes.len() >= 2 {
                                    let parent_scope_index = self.scope_stack.scopes.len() - 2;
                                    if let Some(_) = self.scope_stack.scopes[parent_scope_index].get_variable(&name) {
                                        println!("Found variable '{}' in immediate outer scope {} for nonlocal declaration", name, parent_scope_index);
                                    } else if self.scope_stack.scopes.len() >= 3 {
                                        // If not found in the immediate outer scope, look in all outer scopes
                                        // This is needed for cases where the variable is defined in a scope that's not the immediate parent
                                        for i in (0..self.scope_stack.scopes.len() - 2).rev() {
                                            if let Some(_) = self.scope_stack.scopes[i].get_variable(&name) {
                                                println!("Found variable '{}' in outer scope {} for nonlocal declaration", name, i);
                                                break;
                                            }
                                        }
                                    }
                                }

                                // Always proceed with nonlocal declaration, even if not found in outer scope
                                // This allows for cases where the variable is defined later or in a parent function
                                // that's not directly in the scope stack
                                // Proceed with nonlocal declaration
                                self.declare_nonlocal(name.clone());

                                // If we're in a nested function, we need to create a local variable for the nonlocal variable
                                if let Some(current_function) = self.current_function {
                                    // Get the current function name
                                    let fn_name = current_function.get_name().to_string_lossy().to_string();

                                    // Create a unique name for the nonlocal variable that includes the function name
                                    let unique_name = format!("__nonlocal_{}_{}", fn_name.replace('.', "_"), name);

                                    // Find the variable in the outer scope
                                    let mut found_ptr = None;
                                    let mut found_type = None;

                                    // Get the current scope index
                                    let current_index = self.scope_stack.scopes.len() - 1;

                                    // First check the immediate outer scope
                                    if current_index > 0 {
                                        let parent_scope_index = current_index - 1;
                                        if let Some(ptr) = self.scope_stack.scopes[parent_scope_index].get_variable(&name) {
                                            found_ptr = Some(*ptr);
                                            if let Some(ty) = self.scope_stack.scopes[parent_scope_index].get_type(&name) {
                                                found_type = Some(ty.clone());
                                            }
                                        }
                                    }

                                    // If not found in the immediate outer scope, look in all outer scopes
                                    if found_ptr.is_none() && current_index > 1 {
                                        for i in (0..current_index-1).rev() {
                                            if let Some(ptr) = self.scope_stack.scopes[i].get_variable(&name) {
                                                found_ptr = Some(*ptr);
                                                if let Some(ty) = self.scope_stack.scopes[i].get_type(&name) {
                                                    found_type = Some(ty.clone());
                                                }
                                                break;
                                            }
                                        }
                                    }

                                    // If we found the variable in an outer scope, create a local variable for it
                                    if let (Some(ptr), Some(var_type)) = (found_ptr, found_type) {
                                        // Add the variable to the closure environment
                                        self.add_to_current_environment(name.clone(), ptr, var_type.clone());
                                        println!("Added nonlocal variable '{}' to current closure environment", name);

                                        // Create a local variable for the nonlocal variable at the beginning of the function
                                        // Save current position
                                        let current_position = self.builder.get_insert_block().unwrap();

                                        // Move to the beginning of the entry block
                                        let entry_block = current_function.get_first_basic_block().unwrap();
                                        if let Some(first_instr) = entry_block.get_first_instruction() {
                                            self.builder.position_before(&first_instr);
                                        } else {
                                            self.builder.position_at_end(entry_block);
                                        }

                                        // Create the alloca at the beginning of the function
                                        let local_ptr = self.builder.build_alloca(
                                            self.get_llvm_type(&var_type).into_int_type(),
                                            &unique_name
                                        ).unwrap();

                                        // Restore position
                                        self.builder.position_at_end(current_position);

                                        // Add the variable to the current scope with the unique name
                                        if let Some(current_scope) = self.scope_stack.current_scope_mut() {
                                            current_scope.add_variable(unique_name.clone(), local_ptr, var_type.clone());
                                            current_scope.add_nonlocal_mapping(name.clone(), unique_name.clone());
                                            println!("Created local variable for nonlocal variable '{}' with unique name '{}'", name, unique_name);
                                        }

                                        // Mark the variable as nonlocal in the current scope
                                        println!("Marked '{}' as nonlocal in nested function '{}'", name, fn_name);
                                    }
                                }
                                // No else clause needed since we always declare the variable as nonlocal
                            }
                        },

                        // Compile a global statement
                        Stmt::Global { names, .. } => {
                            // Register each name as a global variable in the current scope
                            for name in names {
                                self.declare_global(name.clone());

                                // Check if we're in a function
                                if self.current_function.is_some() {
                                    // If we're in a function, we need to make sure the global variable exists
                                    // in the global scope and is accessible from the function

                                    // First, check if the variable exists in the global scope
                                    let var_exists_in_global = if let Some(global_scope) = self.scope_stack.global_scope() {
                                        global_scope.get_variable(&name).is_some()
                                    } else {
                                        false
                                    };

                                    if !var_exists_in_global {
                                        // If the global variable doesn't exist yet, create it
                                        // First, register the variable with a default type (Int)
                                        let var_type = Type::Int;
                                        self.register_variable(name.clone(), var_type.clone());

                                        // Create a global variable
                                        let global_var = self.module.add_global(
                                            self.get_llvm_type(&var_type).into_int_type(),
                                            None,
                                            &name
                                        );

                                        // Initialize with zero
                                        global_var.set_initializer(&self.llvm_context.i64_type().const_zero());

                                        // Get a pointer to the global variable
                                        let ptr = global_var.as_pointer_value();

                                        // Store the variable's storage location in the global scope
                                        if let Some(global_scope) = self.scope_stack.global_scope_mut() {
                                            global_scope.add_variable(name.clone(), ptr, var_type.clone());
                                        }

                                        // Also add to the variables map for backward compatibility
                                        self.variables.insert(name.clone(), ptr);

                                        // Also add to the type environment for backward compatibility
                                        self.type_env.insert(name.clone(), var_type.clone());
                                    }
                                }
                            }
                        },

                        // For other statement types, fall back to the original recursive implementation
                        _ => {
                            self.compile_stmt_fallback(stmt)?;
                        }
                    }
                },

                StmtTask::ExecuteBlock { stmts, index } => {
                    if index < stmts.len() {
                        // Get the current statement
                        let stmt = &stmts[index];

                        // Add a task to continue with the rest of the block after this statement
                        work_stack.push_front(StmtTask::ExecuteBlock {
                            stmts,
                            index: index + 1,
                        });

                        // Add a task to execute the current statement
                        work_stack.push_front(StmtTask::Execute(stmt.as_ref()));

                        // Check if the block already has a terminator (from break, continue, return)
                        if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                            // If it does, skip the rest of the block
                            work_stack.clear();
                        }
                    }
                },



                StmtTask::ProcessFor { target, body, orelse, iter } => {
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

                    // Initialize the index variable to 0
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
                        Type::Int => {
                            // For integers (like range), use the value directly
                            // If it's a pointer, load it first
                            if iter_val.is_pointer_value() {
                                self.builder.build_load(
                                    self.llvm_context.i64_type(),
                                    iter_val.into_pointer_value(),
                                    "range_len"
                                ).unwrap()
                            } else {
                                iter_val
                            }
                        },
                        _ => {
                            // For other types, use the value directly
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

                    // Load the current index value
                    let index_val = self.builder.build_load(i64_type, index_ptr, "index").unwrap().into_int_value();

                    // Assign the current index to the target variable
                    if let Expr::Name { id, .. } = target {
                        // Always create a new variable in the loop scope
                        println!("Creating loop variable: {}", id);

                        match iter_type {
                            Type::List(elem_type) => {
                                // Get the element from the list
                                let _list_ptr = iter_val.into_pointer_value();

                                // Create a new variable for the loop index
                                let var_ptr = self.builder.build_alloca(i64_type, id.as_str()).unwrap();

                                // Store the index value in the variable
                                self.builder.build_store(var_ptr, index_val).unwrap();

                                // If the element type is a tuple, extract the element type if all elements are the same
                                let element_type = match &*elem_type {
                                    Type::Tuple(tuple_element_types) => {
                                        if !tuple_element_types.is_empty() && tuple_element_types.iter().all(|t| t == &tuple_element_types[0]) {
                                            // All tuple elements have the same type, use that type
                                            tuple_element_types[0].clone()
                                        } else {
                                            // Keep the original type
                                            *elem_type.clone()
                                        }
                                    },
                                    _ => *elem_type.clone()
                                };

                                // Add the variable to the current scope
                                self.add_variable_to_scope(id.clone(), var_ptr, element_type);
                            },
                            _ => {
                                // For other types, just use the index directly
                                let var_ptr = self.builder.build_alloca(i64_type, id.as_str()).unwrap();
                                self.builder.build_store(var_ptr, index_val).unwrap();
                                self.add_variable_to_scope(id.clone(), var_ptr, Type::Int);
                            }
                        }

                        // Execute the loop body
                        for stmt in body {
                            // Check if the current block already has a terminator
                            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                break;
                            }

                            // Execute the statement directly
                            if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                                return Err(e);
                            }
                        }

                        // Check if the block already has a terminator (from break, continue, return)
                        if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                            // If not, add a branch to the increment block
                            self.builder.build_unconditional_branch(increment_block).unwrap();
                        }
                    } else {
                        // For complex targets, use the fallback implementation
                        // Non-recursive implementations are always used


                        // Create a temporary For statement to pass to the fallback
                        let for_stmt = Stmt::For {
                            target: Box::new(target.clone()),
                            iter: Box::new(iter.clone()),
                            body: body.to_vec(),
                            orelse: orelse.to_vec(),
                            is_async: false,
                            line: 0,
                            column: 0
                        };

                        // Call the fallback implementation
                        let result = self.compile_stmt_fallback(&for_stmt);

                        // Non-recursive implementations are always used

                        if let Err(e) = result {
                            return Err(e);
                        }

                        // Return early since we've already compiled the for loop
                        return Ok(());
                    }

                    // After executing the body, increment the index and branch back to the condition
                    self.builder.position_at_end(increment_block);
                    let index_val = self.builder.build_load(i64_type, index_ptr, "index").unwrap().into_int_value();
                    let next_index = self.builder.build_int_add(
                        index_val,
                        i64_type.const_int(1, false),
                        "next_index"
                    ).unwrap();
                    self.builder.build_store(index_ptr, next_index).unwrap();
                    self.builder.build_unconditional_branch(cond_block).unwrap();

                    // Else block
                    self.builder.position_at_end(else_block);
                    self.push_scope(false, false, false); // Create a new scope for the else block

                    // Execute the else block if it exists
                    if !orelse.is_empty() {
                        for stmt in orelse {
                            // Check if the current block already has a terminator
                            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                break;
                            }

                            // Execute the statement directly
                            // Non-recursive implementations are always used


                            if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                                return Err(e);
                            }

                            // Non-recursive implementations are always used
                        }
                    }

                    // Check if the block already has a terminator (from break, continue, return)
                    if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                        // If not, add a branch to the end block
                        self.builder.build_unconditional_branch(end_block).unwrap();
                    }

                    // Pop the scope for the else block
                    self.pop_scope();

                    // End block
                    self.builder.position_at_end(end_block);

                    // Pop this loop from the stack
                    self.pop_loop();

                    // Pop the scope for the loop body
                    self.pop_scope();
                },

                StmtTask::ProcessWhile { test, body, orelse } => {
                    // Get the LLVM context
                    let context = self.llvm_context;

                    // Get the current function
                    let function = match self.builder.get_insert_block() {
                        Some(block) => block.get_parent().unwrap(),
                        None => {
                            // Fallback to current_function if no current block
                            match self.current_function {
                                Some(f) => f,
                                None => {
                                    // We're in the main module, get the main function
                                    match self.module.get_function("main") {
                                        Some(f) => f,
                                        None => return Err("No main function found".to_string()),
                                    }
                                },
                            }
                        },
                    };

                    // Create basic blocks for the loop
                    let cond_block = context.append_basic_block(function, "while.cond");
                    let body_block = context.append_basic_block(function, "while.body");
                    let else_block = context.append_basic_block(function, "while.else");
                    let end_block = context.append_basic_block(function, "while.end");

                    // Branch to the condition block
                    self.builder.build_unconditional_branch(cond_block).unwrap();

                    // Position at the condition block
                    self.builder.position_at_end(cond_block);

                    // Compile the test expression
                    let (test_val, _) = self.compile_expr(test)?;

                    // Convert to boolean using our helper method
                    let cond_val = self.convert_to_bool(test_val);

                    // Branch based on the condition
                    self.builder.build_conditional_branch(cond_val, body_block, else_block).unwrap();

                    // Position at the body block
                    self.builder.position_at_end(body_block);

                    // Create a new scope for the loop body
                    self.push_scope(false, true, false); // Create a new scope for the loop body (is_loop=true)

                    // Save the current loop context
                    let _old_break_block = self.current_break_block();
                    let _old_continue_block = self.current_continue_block();

                    // Set the current loop context
                    self.push_loop(cond_block, end_block);

                    // Execute the loop body
                    for stmt in body {
                        // Check if the current block already has a terminator
                        if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                            break;
                        }

                        // Execute the statement directly
                        if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                            return Err(e);
                        }
                    }

                    // Check if the block already has a terminator (from break, continue, return)
                    if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                        // If not, add a branch back to the condition block
                        self.builder.build_unconditional_branch(cond_block).unwrap();
                    }

                    // Restore the loop context
                    self.pop_loop();

                    // Pop the scope for the loop body
                    self.pop_scope();

                    // Position at the else block
                    self.builder.position_at_end(else_block);

                    // Execute the else block if it exists
                    if !orelse.is_empty() {
                        for stmt in orelse {
                            // Check if the current block already has a terminator
                            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                break;
                            }

                            // Execute the statement directly
                            // Non-recursive implementations are always used


                            if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                                return Err(e);
                            }

                            // Non-recursive implementations are always used
                        }
                    }

                    // Check if the block already has a terminator (from break, continue, return)
                    if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                        // If not, add a branch to the end block
                        self.builder.build_unconditional_branch(end_block).unwrap();
                    }

                    // Position at the end block for further code
                    self.builder.position_at_end(end_block);
                },

                StmtTask::ProcessTry { body, handlers, orelse, finalbody } => {
                    // Ensure the current block has a terminator before creating new blocks
                    self.ensure_block_has_terminator();

                    // Get the current function
                    let function = match self.current_function {
                        Some(f) => f,
                        None => return Err("Cannot use try statement outside of a function".to_string()),
                    };

                    // Create basic blocks for try, except handlers, else, finally, and exit
                    let try_block = self.llvm_context.append_basic_block(function, "try");

                    // Create blocks for each except handler
                    let mut except_blocks = Vec::new();
                    for i in 0..handlers.len() {
                        except_blocks.push(self.llvm_context.append_basic_block(function, &format!("except_{}", i)));
                    }

                    // If there are no handlers, add a default one
                    if except_blocks.is_empty() {
                        except_blocks.push(self.llvm_context.append_basic_block(function, "except_default"));
                    }

                    let else_block = self.llvm_context.append_basic_block(function, "else");
                    let finally_block = self.llvm_context.append_basic_block(function, "finally");
                    let exit_block = self.llvm_context.append_basic_block(function, "exit");

                    // Create a global variable to track if an exception was raised
                    let exception_raised = self.create_exception_state();

                    // Branch to the try block
                    self.builder.build_unconditional_branch(try_block).unwrap();

                    // Compile the try block
                    self.builder.position_at_end(try_block);

                    // We don't need a separate scope for the try block
                    // Variables defined in the try block should be accessible outside

                    // Reset exception state at the beginning of try block
                    self.reset_exception_state(exception_raised);

                    // Compile the body of the try block
                    for stmt in body {
                        // Check if the current block already has a terminator
                        if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                            break;
                        }

                        // Execute the statement directly
                        // Non-recursive implementations are always used


                        if let Err(e) = self.compile_stmt(stmt) {
                            return Err(e);
                        }

                        // Non-recursive implementations are always used
                    }

                    // Only add a branch if we don't already have a terminator
                    if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                        // If no exception was raised, branch to the else block
                        let exception_value = self.load_exception_state(exception_raised);
                        self.builder.build_conditional_branch(
                            exception_value,
                            except_blocks[0], // If exception raised, go to first except handler
                            else_block,       // If no exception, go to else block
                        ).unwrap();
                    }

                    // Compile the except handlers
                    for (i, handler) in handlers.iter().enumerate() {
                        self.builder.position_at_end(except_blocks[i]);

                        // For now, we'll just use a catch-all handler for all exception types
                        let matches = self.llvm_context.bool_type().const_int(1, false);

                        // Create a block for the handler body
                        let handler_body_block = self.llvm_context.append_basic_block(function, &format!("except_body_{}", i));

                        // Create a block for the next handler (or finally if this is the last handler)
                        let next_block = if i < handlers.len() - 1 {
                            except_blocks[i + 1]
                        } else {
                            finally_block
                        };

                        // Branch to the handler body if the exception matches, otherwise to the next handler
                        self.builder.build_conditional_branch(
                            matches,
                            handler_body_block,
                            next_block,
                        ).unwrap();

                        // Compile the handler body
                        self.builder.position_at_end(handler_body_block);

                        // We don't need a separate scope for the handler body
                        // Variables defined in the handler body should be accessible outside

                        // If the handler has a name, bind the exception to that name
                        if let Some(name) = &handler.name {
                            // Get the current exception
                            let exception = self.get_current_exception();

                            // Create a variable for the exception
                            let exception_ptr = self.builder.build_alloca(
                                self.llvm_context.ptr_type(inkwell::AddressSpace::default()),
                                name,
                            ).unwrap();

                            // Store the exception in the variable
                            self.builder.build_store(exception_ptr, exception).unwrap();

                            // Add the variable to the current scope
                            self.add_variable_to_scope(
                                name.clone(),
                                exception_ptr,
                                Type::Any, // For now, use Any as the type
                            );
                        }

                        // Compile the handler body
                        for stmt in &handler.body {
                            // Check if the current block already has a terminator
                            if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                                break;
                            }

                            // Execute the statement directly
                            // Non-recursive implementations are always used


                            if let Err(e) = self.compile_stmt(stmt) {
                                return Err(e);
                            }

                            // Non-recursive implementations are always used
                        }

                        // Reset the exception state
                        self.reset_exception_state(exception_raised);

                        // Clear the current exception in the global state
                        if let Some(clear_current_exception_fn) = self.module.get_function("clear_current_exception") {
                            self.builder.build_call(
                                clear_current_exception_fn,
                                &[],
                                "clear_exception_result",
                            ).unwrap();
                        }

                        // Branch to the finally block
                        if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                            self.builder.build_unconditional_branch(finally_block).unwrap();
                        }

                        // We didn't create a separate scope for the handler body
                    }

                    // Compile the else block
                    self.builder.position_at_end(else_block);

                    // We don't need a separate scope for the else block
                    // Variables defined in the else block should be accessible outside

                    // Compile the else body
                    for stmt in orelse {
                        // Check if the current block already has a terminator
                        if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                            break;
                        }

                        // Execute the statement directly
                        // Non-recursive implementations are always used


                        if let Err(e) = self.compile_stmt(stmt) {
                            return Err(e);
                        }

                        // Non-recursive implementations are always used
                    }

                    // Branch to the finally block
                    if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                        self.builder.build_unconditional_branch(finally_block).unwrap();
                    }

                    // We didn't create a separate scope for the else block

                    // Compile the finally block
                    self.builder.position_at_end(finally_block);

                    // Create a new scope for the finally block
                    self.push_scope(false, false, false);

                    // Compile the finally body
                    for stmt in finalbody {
                        // Check if the current block already has a terminator
                        if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                            break;
                        }

                        // Execute the statement directly
                        if let Err(e) = self.compile_stmt(stmt) {
                            return Err(e);
                        }
                    }

                    // Branch to the exit block
                    if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                        self.builder.build_unconditional_branch(exit_block).unwrap();
                    }

                    // Pop the scope for the finally block
                    self.pop_scope();

                    // We didn't create a separate scope for the try block

                    // Position at the exit block for further code
                    self.builder.position_at_end(exit_block);
                },

                StmtTask::ProcessWith { body } => {
                    // Execute the body without context management
                    if !body.is_empty() {
                        work_stack.push_front(StmtTask::ExecuteBlock {
                            stmts: body,
                            index: 0,
                        });
                    }
                },

                StmtTask::ProcessAssign { targets, value_val, value_type } => {
                    // For each target on the left-hand side, assign the value
                    for target in targets {
                        self.compile_assignment(target, value_val, &value_type)?;
                    }
                },

                StmtTask::ProcessReturn { value_val, value_type } => {
                    if let Some(ret_val) = value_val {
                        // Get the current function
                        if let Some(current_function) = self.current_function {
                            // Get the return type of the function
                            let return_type = current_function.get_type().get_return_type();

                            // Check if we need to handle type conversion
                            if let Some(ret_type) = return_type {
                                // Check if we're returning a pointer but the function returns an integer
                                if ret_type.is_int_type() && ret_val.is_pointer_value() {
                                    // Load the value from the pointer
                                    let loaded_val = self.builder.build_load(
                                        ret_type.into_int_type(),
                                        ret_val.into_pointer_value(),
                                        "load_return"
                                    ).unwrap();

                                    // Build the return instruction with the loaded value
                                    self.builder.build_return(Some(&loaded_val)).unwrap();
                                    return Ok(());
                                }
                            }

                            // Check if we're returning a tuple from a function that expects an integer
                            if let Some(ret_type) = value_type {
                                if let Type::Tuple(element_types) = ret_type {
                                    // If the function returns an integer but we're returning a tuple,
                                    // extract the first element of the tuple
                                    if return_type.is_some() {
                                        // Get the function return type
                                        let func_return_type = return_type.unwrap();

                                        // If the return value is a pointer to a tuple
                                        if ret_val.is_pointer_value() {
                                            // If the function expects an integer return
                                            if func_return_type.is_int_type() {
                                                // Get the tuple struct type
                                                let llvm_types: Vec<BasicTypeEnum> = element_types
                                                    .iter()
                                                    .map(|ty| self.get_llvm_type(ty))
                                                    .collect();

                                                let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

                                                // Get a pointer to the first element
                                                let element_ptr = self.builder.build_struct_gep(
                                                    tuple_struct,
                                                    ret_val.into_pointer_value(),
                                                    0,
                                                    "tuple_element_0"
                                                ).unwrap();

                                                // Load the first element
                                                let element_val = self.builder.build_load(
                                                    self.get_llvm_type(&element_types[0]),
                                                    element_ptr,
                                                    "load_tuple_element_0"
                                                ).unwrap();

                                                // Return the first element
                                                self.builder.build_return(Some(&element_val)).unwrap();
                                                return Ok(());
                                            } else if func_return_type.is_pointer_type() {
                                                // If the function expects a pointer return, return the pointer directly
                                                self.builder.build_return(Some(&ret_val)).unwrap();
                                                return Ok(());
                                            } else {
                                                // For other return types, try to extract the first element
                                                let llvm_types: Vec<BasicTypeEnum> = element_types
                                                    .iter()
                                                    .map(|ty| self.get_llvm_type(ty))
                                                    .collect();

                                                let tuple_struct = self.llvm_context.struct_type(&llvm_types, false);

                                                // Get a pointer to the first element
                                                let element_ptr = self.builder.build_struct_gep(
                                                    tuple_struct,
                                                    ret_val.into_pointer_value(),
                                                    0,
                                                    "tuple_element_0"
                                                ).unwrap();

                                                // Load the first element
                                                let element_val = self.builder.build_load(
                                                    self.get_llvm_type(&element_types[0]),
                                                    element_ptr,
                                                    "load_tuple_element_0"
                                                ).unwrap();

                                                // Return the first element
                                                self.builder.build_return(Some(&element_val)).unwrap();
                                                return Ok(());
                                            }
                                        } else {
                                            // If the return value is not a pointer
                                            if func_return_type.is_int_type() {
                                                // If the function expects an integer return, return the value directly
                                                self.builder.build_return(Some(&ret_val)).unwrap();
                                                return Ok(());
                                            } else if func_return_type.is_pointer_type() {
                                                // If the function expects a pointer return, convert the value to a pointer
                                                let ptr_type = self.llvm_context.ptr_type(inkwell::AddressSpace::default());
                                                let ptr_val = self.builder.build_int_to_ptr(
                                                    ret_val.into_int_value(),
                                                    ptr_type,
                                                    "int_to_ptr"
                                                ).unwrap();

                                                // Return the pointer
                                                self.builder.build_return(Some(&ptr_val)).unwrap();
                                                return Ok(());
                                            } else {
                                                // For other return types, return the value directly
                                                self.builder.build_return(Some(&ret_val)).unwrap();
                                                return Ok(());
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Build the return instruction with the value
                        self.builder.build_return(Some(&ret_val)).unwrap();
                    } else {
                        // Build a void return
                        self.builder.build_return(None).unwrap();
                    }
                },

                StmtTask::ProcessFunctionDef { name, params, body, is_nested } => {
                    // Declare the function
                    if is_nested {
                        // For nested functions, we need to create a closure environment
                        self.declare_nested_function(&name, params)?;
                    } else {
                        // For top-level functions, we can use the regular function declaration
                        // We'll use the nested function declaration for now
                        self.declare_nested_function(&name, params)?;
                    }

                    // Compile the function body
                    if is_nested {
                        // Use a non-recursive approach for nested function bodies
                        // Compile the nested function body
                        let result = self.compile_nested_function_body(&name, params, body);

                        // Return any error
                        if let Err(e) = result {
                            return Err(e);
                        }
                    } else {
                        // For top-level functions, we can use the nested function body compilation
                        // Compile the function body using the nested function body compilation
                        let result = self.compile_nested_function_body(&name, params, body);

                        // Return any error
                        if let Err(e) = result {
                            return Err(e);
                        }
                    }
                },
                // Removed ContinueLoop variant case

                // All task types are now handled above
                // No need for a catch-all pattern
            }
        }

        Ok(())
    }

    fn compile_stmt_fallback(&mut self, stmt: &Stmt) -> Result<(), String> {
        // This is a fallback for complex statements that aren't fully implemented in the non-recursive version
        // We'll implement a simplified version here to avoid circular references
        match stmt {
            Stmt::While { test, body, orelse, .. } => {
                // Get the LLVM context
                let context = self.llvm_context;

                // Get the current function
                let function = match self.builder.get_insert_block() {
                    Some(block) => block.get_parent().unwrap(),
                    None => {
                        // Fallback to current_function if no current block
                        match self.current_function {
                            Some(f) => f,
                            None => {
                                // We're in the main module, get the main function
                                match self.module.get_function("main") {
                                    Some(f) => f,
                                    None => return Err("No main function found".to_string()),
                                }
                            },
                        }
                    },
                };

                // Create basic blocks for the loop
                let cond_block = context.append_basic_block(function, "while.cond");
                let body_block = context.append_basic_block(function, "while.body");
                let else_block = context.append_basic_block(function, "while.else");
                let end_block = context.append_basic_block(function, "while.end");

                // Branch to the condition block
                self.builder.build_unconditional_branch(cond_block).unwrap();

                // Position at the condition block
                self.builder.position_at_end(cond_block);

                // Compile the test expression
                let (test_val, _) = self.compile_expr(test)?;

                // Convert to boolean
                let cond_val = match test_val {
                    BasicValueEnum::IntValue(int_val) => {
                        // If it's already a boolean (i1), use it directly
                        if int_val.get_type().get_bit_width() == 1 {
                            int_val
                        } else {
                            // Otherwise, compare with zero
                            let zero = int_val.get_type().const_zero();
                            self.builder.build_int_compare(inkwell::IntPredicate::NE, int_val, zero, "bool_conv").unwrap()
                        }
                    },
                    BasicValueEnum::FloatValue(float_val) => {
                        // Compare with zero
                        let zero = float_val.get_type().const_float(0.0);
                        self.builder.build_float_compare(inkwell::FloatPredicate::ONE, float_val, zero, "float_bool").unwrap()
                    },
                    _ => {
                        // Default to true for other types
                        self.llvm_context.bool_type().const_int(1, false)
                    }
                };

                // Branch based on the condition
                self.builder.build_conditional_branch(cond_val, body_block, else_block).unwrap();

                // Position at the body block
                self.builder.position_at_end(body_block);

                // Create a new scope for the loop body
                self.push_scope(false, true, false); // Create a new scope for the loop body (is_loop=true)

                // Set the current loop context
                self.push_loop(cond_block, end_block);

                // Execute the loop body
                for stmt in body {
                    // Check if the current block already has a terminator
                    if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                        break;
                    }

                    // Execute the statement directly
                    if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                        return Err(e);
                    }
                }

                // Check if the block already has a terminator (from break, continue, return)
                if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                    // If not, add a branch back to the condition block
                    self.builder.build_unconditional_branch(cond_block).unwrap();
                }

                // Restore the loop context
                self.pop_loop();

                // Pop the scope for the loop body
                self.pop_scope();

                // Position at the else block
                self.builder.position_at_end(else_block);

                // Execute the else block if it exists
                if !orelse.is_empty() {
                    for stmt in orelse {
                        // Check if the current block already has a terminator
                        if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                            break;
                        }

                        // Execute the statement directly
                        if let Err(e) = self.compile_stmt_non_recursive(stmt.as_ref()) {
                            return Err(e);
                        }
                    }
                }

                // Check if the block already has a terminator (from break, continue, return)
                if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                    // If not, add a branch to the end block
                    self.builder.build_unconditional_branch(end_block).unwrap();
                }

                // Position at the end block for further code
                self.builder.position_at_end(end_block);

                Ok(())
            },
            _ => {
                // For other statements, we'll just return an error
                Err(format!("Statement type not supported in fallback implementation: {:?}", stmt))
            }
        }
    }
}
