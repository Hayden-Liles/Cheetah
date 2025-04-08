#[cfg(test)]
mod ast_verification_tests {
    use cheetah::ast::{BoolOperator, CmpOperator, Expr, ExprContext, Module, NameConstant, Number, Operator, Stmt, UnaryOperator};
    use cheetah::lexer::Lexer;
    use cheetah::parser::{ParseError, Parser};
    use std::fmt;

    // Custom formatter for error types
    struct ErrorFormatter<'a>(pub &'a ParseError);

    impl<'a> fmt::Display for ErrorFormatter<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.0 {
                ParseError::UnexpectedToken { expected, found, line, column, suggestion: _ } => {
                    write!(f, "Unexpected token at line {}, column {}: expected '{}', found '{:?}'",
                           line, column, expected, found)
                },
                ParseError::InvalidSyntax { message, line, column, suggestion: _ } => {
                    write!(f, "Invalid syntax at line {}, column {}: {}",
                           line, column, message)
                },
                ParseError::EOF { expected, line, column, suggestion: _ } => {
                    write!(f, "Unexpected EOF at line {}, column {}: expected '{}'",
                           line, column, expected)
                },
            }
        }
    }

    fn parse_code(source: &str) -> Result<Module, Vec<ParseError>> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        if !lexer.get_errors().is_empty() {
            let parse_errors: Vec<ParseError> = lexer
                .get_errors()
                .iter()
                .map(|e| ParseError::InvalidSyntax {
                    message: e.message.clone(),
                    line: e.line,
                    column: e.column,
                    suggestion: None,
                })
                .collect();
            return Err(parse_errors);
        }

        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    // Helper function to assert parsing succeeds and print the AST for debugging
    fn assert_parses(source: &str) -> Module {
        match parse_code(source) {
            Ok(module) => module,
            Err(errors) => {
                println!("\n================================");
                println!("PARSING FAILED FOR CODE SNIPPET:");
                println!("================================");
                println!("{}", source);
                println!("\nERRORS:");

                for error in &errors {
                    println!("- {}", ErrorFormatter(error));
                }

                panic!("Parsing failed with {} errors", errors.len());
            },
        }
    }

    #[test]
    fn test_binary_operation_ast() {
        // Test addition
        let module = assert_parses("a + b");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { left, op, right, .. } = &**value {
                    // Verify left operand
                    if let Expr::Name { id, ctx, .. } = &**left {
                        assert_eq!(id, "a");
                        assert!(matches!(ctx, ExprContext::Load));
                    } else {
                        panic!("Expected left operand to be a name, got: {:?}", left);
                    }

                    // Verify operator
                    assert_eq!(*op, Operator::Add);

                    // Verify right operand
                    if let Expr::Name { id, ctx, .. } = &**right {
                        assert_eq!(id, "b");
                        assert!(matches!(ctx, ExprContext::Load));
                    } else {
                        panic!("Expected right operand to be a name, got: {:?}", right);
                    }
                } else {
                    panic!("Expected binary operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_all_binary_operators() {
        // Addition
        let module = assert_parses("a + b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::Add);
                }
            }
        }

        // Subtraction
        let module = assert_parses("a - b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::Sub);
                }
            }
        }

        // Multiplication
        let module = assert_parses("a * b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::Mult);
                }
            }
        }

        // Division
        let module = assert_parses("a / b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::Div);
                }
            }
        }

        // Modulo
        let module = assert_parses("a % b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::Mod);
                }
            }
        }

        // Power
        let module = assert_parses("a ** b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::Pow);
                }
            }
        }

        // Floor division
        let module = assert_parses("a // b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::FloorDiv);
                }
            }
        }

        // Bitwise and
        let module = assert_parses("a & b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::BitAnd);
                }
            }
        }

        // Bitwise or
        let module = assert_parses("a | b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::BitOr);
                }
            }
        }

        // Bitwise xor
        let module = assert_parses("a ^ b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::BitXor);
                }
            }
        }

        // Left shift
        let module = assert_parses("a << b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::LShift);
                }
            }
        }

        // Right shift
        let module = assert_parses("a >> b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::RShift);
                }
            }
        }

        // Matrix multiplication
        let module = assert_parses("a @ b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BinOp { op, .. } = &**value {
                    assert_eq!(*op, Operator::MatMult);
                }
            }
        }
    }

    #[test]
    fn test_unary_operations() {
        // Unary plus
        let module = assert_parses("+a");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::UnaryOp { op, operand, .. } = &**value {
                    assert_eq!(*op, UnaryOperator::UAdd);
                    if let Expr::Name { id, .. } = &**operand {
                        assert_eq!(id, "a");
                    } else {
                        panic!("Expected operand to be a name, got: {:?}", operand);
                    }
                } else {
                    panic!("Expected unary operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Unary minus
        let module = assert_parses("-a");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::UnaryOp { op, operand, .. } = &**value {
                    assert_eq!(*op, UnaryOperator::USub);
                    if let Expr::Name { id, .. } = &**operand {
                        assert_eq!(id, "a");
                    } else {
                        panic!("Expected operand to be a name, got: {:?}", operand);
                    }
                } else {
                    panic!("Expected unary operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Bitwise not
        let module = assert_parses("~a");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::UnaryOp { op, operand, .. } = &**value {
                    assert_eq!(*op, UnaryOperator::Invert);
                    if let Expr::Name { id, .. } = &**operand {
                        assert_eq!(id, "a");
                    } else {
                        panic!("Expected operand to be a name, got: {:?}", operand);
                    }
                } else {
                    panic!("Expected unary operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Logical not
        let module = assert_parses("not a");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::UnaryOp { op, operand, .. } = &**value {
                    assert_eq!(*op, UnaryOperator::Not);
                    if let Expr::Name { id, .. } = &**operand {
                        assert_eq!(id, "a");
                    } else {
                        panic!("Expected operand to be a name, got: {:?}", operand);
                    }
                } else {
                    panic!("Expected unary operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_boolean_operations() {
        // AND operation
        let module = assert_parses("a and b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BoolOp { op, values, .. } = &**value {
                    assert_eq!(*op, BoolOperator::And);
                    assert_eq!(values.len(), 2);

                    if let Expr::Name { id, .. } = &*values[0] {
                        assert_eq!(id, "a");
                    } else {
                        panic!("Expected first operand to be a name, got: {:?}", values[0]);
                    }

                    if let Expr::Name { id, .. } = &*values[1] {
                        assert_eq!(id, "b");
                    } else {
                        panic!("Expected second operand to be a name, got: {:?}", values[1]);
                    }
                } else {
                    panic!("Expected boolean operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // OR operation
        let module = assert_parses("a or b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BoolOp { op, values, .. } = &**value {
                    assert_eq!(*op, BoolOperator::Or);
                    assert_eq!(values.len(), 2);
                } else {
                    panic!("Expected boolean operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Multiple values in AND
        let module = assert_parses("a and b and c");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BoolOp { op, values, .. } = &**value {
                    assert_eq!(*op, BoolOperator::And);
                    assert_eq!(values.len(), 3);
                } else {
                    panic!("Expected boolean operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Multiple values in OR
        let module = assert_parses("a or b or c");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::BoolOp { op, values, .. } = &**value {
                    assert_eq!(*op, BoolOperator::Or);
                    assert_eq!(values.len(), 3);
                } else {
                    panic!("Expected boolean operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_comparison_operations() {
        // Equal to
        let module = assert_parses("a == b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { left, ops, comparators, .. } = &**value {
                    if let Expr::Name { id, .. } = &**left {
                        assert_eq!(id, "a");
                    } else {
                        panic!("Expected left operand to be a name, got: {:?}", left);
                    }

                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::Eq);

                    assert_eq!(comparators.len(), 1);
                    if let Expr::Name { id, .. } = &*comparators[0] {
                        assert_eq!(id, "b");
                    } else {
                        panic!("Expected right operand to be a name, got: {:?}", comparators[0]);
                    }
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Not equal to
        let module = assert_parses("a != b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { ops, .. } = &**value {
                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::NotEq);
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Less than
        let module = assert_parses("a < b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { ops, .. } = &**value {
                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::Lt);
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Less than or equal to
        let module = assert_parses("a <= b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { ops, .. } = &**value {
                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::LtE);
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Greater than
        let module = assert_parses("a > b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { ops, .. } = &**value {
                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::Gt);
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Greater than or equal to
        let module = assert_parses("a >= b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { ops, .. } = &**value {
                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::GtE);
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Is
        let module = assert_parses("a is b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { ops, .. } = &**value {
                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::Is);
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Is not
        let module = assert_parses("a is not b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { ops, .. } = &**value {
                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::IsNot);
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // In
        let module = assert_parses("a in b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { ops, .. } = &**value {
                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::In);
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Not in
        let module = assert_parses("a not in b");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { ops, .. } = &**value {
                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::NotIn);
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Chained comparison
        let module = assert_parses("a < b < c");
        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Compare { left, ops, comparators, .. } = &**value {
                    if let Expr::Name { id, .. } = &**left {
                        assert_eq!(id, "a");
                    } else {
                        panic!("Expected left operand to be a name, got: {:?}", left);
                    }

                    assert_eq!(ops.len(), 2);
                    assert_eq!(ops[0], CmpOperator::Lt);
                    assert_eq!(ops[1], CmpOperator::Lt);

                    assert_eq!(comparators.len(), 2);

                    if let Expr::Name { id, .. } = &*comparators[0] {
                        assert_eq!(id, "b");
                    } else {
                        panic!("Expected first comparator to be a name, got: {:?}", comparators[0]);
                    }

                    if let Expr::Name { id, .. } = &*comparators[1] {
                        assert_eq!(id, "c");
                    } else {
                        panic!("Expected second comparator to be a name, got: {:?}", comparators[1]);
                    }
                } else {
                    panic!("Expected comparison operation, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_if_statement() {
        // Simple if statement
        let module = assert_parses("if a > b:\n    c = d");

        if let Some(stmt) = module.body.first() {
            if let Stmt::If { test, body, orelse, .. } = &**stmt {
                // Verify condition
                if let Expr::Compare { left, ops, comparators, .. } = &**test {
                    if let Expr::Name { id, .. } = &**left {
                        assert_eq!(id, "a");
                    } else {
                        panic!("Expected left operand to be a name, got: {:?}", left);
                    }

                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0], CmpOperator::Gt);

                    assert_eq!(comparators.len(), 1);
                    if let Expr::Name { id, .. } = &*comparators[0] {
                        assert_eq!(id, "b");
                    } else {
                        panic!("Expected right operand to be a name, got: {:?}", comparators[0]);
                    }
                } else {
                    panic!("Expected comparison, got: {:?}", test);
                }

                // Verify body
                assert_eq!(body.len(), 1);
                if let Stmt::Assign { targets, value, .. } = &*body[0] {
                    assert_eq!(targets.len(), 1);
                    if let Expr::Name { id, .. } = &*targets[0] {
                        assert_eq!(id, "c");
                    } else {
                        panic!("Expected target to be a name, got: {:?}", targets[0]);
                    }

                    if let Expr::Name { id, .. } = &**value {
                        assert_eq!(id, "d");
                    } else {
                        panic!("Expected value to be a name, got: {:?}", value);
                    }
                } else {
                    panic!("Expected assignment statement, got: {:?}", body[0]);
                }

                // Verify no else clause
                assert!(orelse.is_empty());
            } else {
                panic!("Expected if statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // If-else statement
        let module = assert_parses("if a > b:\n    c = d\nelse:\n    e = f");

        if let Some(stmt) = module.body.first() {
            if let Stmt::If { test: _, body: _, orelse, .. } = &**stmt {
                // Verify else clause
                assert_eq!(orelse.len(), 1);
                if let Stmt::Assign { targets, value, .. } = &*orelse[0] {
                    assert_eq!(targets.len(), 1);
                    if let Expr::Name { id, .. } = &*targets[0] {
                        assert_eq!(id, "e");
                    } else {
                        panic!("Expected target to be a name, got: {:?}", targets[0]);
                    }

                    if let Expr::Name { id, .. } = &**value {
                        assert_eq!(id, "f");
                    } else {
                        panic!("Expected value to be a name, got: {:?}", value);
                    }
                } else {
                    panic!("Expected assignment statement, got: {:?}", orelse[0]);
                }
            } else {
                panic!("Expected if statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // If-elif-else statement
        let module = assert_parses("if a > b:\n    c = d\nelif a < b:\n    e = f\nelse:\n    g = h");

        if let Some(stmt) = module.body.first() {
            if let Stmt::If { test: _, body: _, orelse, .. } = &**stmt {
                // Verify elif clause (which is actually an if statement in the orelse list)
                assert_eq!(orelse.len(), 1);
                if let Stmt::If { test, body, orelse: elif_orelse, .. } = &*orelse[0] {
                    // Verify elif condition
                    if let Expr::Compare { left, ops, comparators, .. } = &**test {
                        if let Expr::Name { id, .. } = &**left {
                            assert_eq!(id, "a");
                        } else {
                            panic!("Expected left operand to be a name, got: {:?}", left);
                        }

                        assert_eq!(ops.len(), 1);
                        assert_eq!(ops[0], CmpOperator::Lt);

                        assert_eq!(comparators.len(), 1);
                        if let Expr::Name { id, .. } = &*comparators[0] {
                            assert_eq!(id, "b");
                        } else {
                            panic!("Expected right operand to be a name, got: {:?}", comparators[0]);
                        }
                    } else {
                        panic!("Expected comparison, got: {:?}", test);
                    }

                    // Verify elif body
                    assert_eq!(body.len(), 1);
                    if let Stmt::Assign { targets, value, .. } = &*body[0] {
                        assert_eq!(targets.len(), 1);
                        if let Expr::Name { id, .. } = &*targets[0] {
                            assert_eq!(id, "e");
                        } else {
                            panic!("Expected target to be a name, got: {:?}", targets[0]);
                        }

                        if let Expr::Name { id, .. } = &**value {
                            assert_eq!(id, "f");
                        } else {
                            panic!("Expected value to be a name, got: {:?}", value);
                        }
                    } else {
                        panic!("Expected assignment statement, got: {:?}", body[0]);
                    }

                    // Verify else clause
                    assert_eq!(elif_orelse.len(), 1);
                    if let Stmt::Assign { targets, value, .. } = &*elif_orelse[0] {
                        assert_eq!(targets.len(), 1);
                        if let Expr::Name { id, .. } = &*targets[0] {
                            assert_eq!(id, "g");
                        } else {
                            panic!("Expected target to be a name, got: {:?}", targets[0]);
                        }

                        if let Expr::Name { id, .. } = &**value {
                            assert_eq!(id, "h");
                        } else {
                            panic!("Expected value to be a name, got: {:?}", value);
                        }
                    } else {
                        panic!("Expected assignment statement, got: {:?}", elif_orelse[0]);
                    }
                } else {
                    panic!("Expected if statement for elif, got: {:?}", orelse[0]);
                }
            } else {
                panic!("Expected if statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_for_loop() {
        // Simple for loop
        let module = assert_parses("for i in range(10):\n    print(i)");

        if let Some(stmt) = module.body.first() {
            if let Stmt::For { target, iter, body, orelse, is_async, .. } = &**stmt {
                // Verify target
                if let Expr::Name { id, ctx, .. } = &**target {
                    assert_eq!(id, "i");
                    assert!(matches!(ctx, ExprContext::Store));
                } else {
                    panic!("Expected target to be a name, got: {:?}", target);
                }

                // Verify iterator
                if let Expr::Call { func, args, .. } = &**iter {
                    if let Expr::Name { id, .. } = &**func {
                        assert_eq!(id, "range");
                    } else {
                        panic!("Expected function name to be 'range', got: {:?}", func);
                    }

                    assert_eq!(args.len(), 1);
                    if let Expr::Num { value, .. } = &*args[0] {
                        assert_eq!(*value, Number::Integer(10));
                    } else {
                        panic!("Expected argument to be a number, got: {:?}", args[0]);
                    }
                } else {
                    panic!("Expected call expression, got: {:?}", iter);
                }

                // Verify body
                assert_eq!(body.len(), 1);
                if let Stmt::Expr { value, .. } = &*body[0] {
                    if let Expr::Call { func, args, .. } = &**value {
                        if let Expr::Name { id, .. } = &**func {
                            assert_eq!(id, "print");
                        } else {
                            panic!("Expected function name to be 'print', got: {:?}", func);
                        }

                        assert_eq!(args.len(), 1);
                        if let Expr::Name { id, .. } = &*args[0] {
                            assert_eq!(id, "i");
                        } else {
                            panic!("Expected argument to be a name, got: {:?}", args[0]);
                        }
                    } else {
                        panic!("Expected call expression, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", body[0]);
                }

                // Verify no else clause
                assert!(orelse.is_empty());

                // Verify not async
                assert!(!is_async);
            } else {
                panic!("Expected for statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // For loop with tuple unpacking
        let module = assert_parses("for k, v in items.items():\n    print(k, v)");

        if let Some(stmt) = module.body.first() {
            if let Stmt::For { target, .. } = &**stmt {
                // Verify target is a tuple
                if let Expr::Tuple { elts, ctx, .. } = &**target {
                    assert_eq!(elts.len(), 2);
                    assert!(matches!(ctx, ExprContext::Store));

                    if let Expr::Name { id, .. } = &*elts[0] {
                        assert_eq!(id, "k");
                    } else {
                        panic!("Expected first element to be a name, got: {:?}", elts[0]);
                    }

                    if let Expr::Name { id, .. } = &*elts[1] {
                        assert_eq!(id, "v");
                    } else {
                        panic!("Expected second element to be a name, got: {:?}", elts[1]);
                    }
                } else {
                    panic!("Expected target to be a tuple, got: {:?}", target);
                }
            } else {
                panic!("Expected for statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // For loop with else clause
        let module = assert_parses("for i in range(10):\n    print(i)\nelse:\n    print('Done')");

        if let Some(stmt) = module.body.first() {
            if let Stmt::For { orelse, .. } = &**stmt {
                // Verify else clause
                assert_eq!(orelse.len(), 1);
                if let Stmt::Expr { value, .. } = &*orelse[0] {
                    if let Expr::Call { func, args, .. } = &**value {
                        if let Expr::Name { id, .. } = &**func {
                            assert_eq!(id, "print");
                        } else {
                            panic!("Expected function name to be 'print', got: {:?}", func);
                        }

                        assert_eq!(args.len(), 1);
                        if let Expr::Str { value, .. } = &*args[0] {
                            assert_eq!(value, "Done");
                        } else {
                            panic!("Expected argument to be a string, got: {:?}", args[0]);
                        }
                    } else {
                        panic!("Expected call expression, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", orelse[0]);
                }
            } else {
                panic!("Expected for statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_while_loop() {
        // Simple while loop
        let module = assert_parses("while condition:\n    action()");

        if let Some(stmt) = module.body.first() {
            if let Stmt::While { test, body, orelse, .. } = &**stmt {
                // Verify condition
                if let Expr::Name { id, ctx, .. } = &**test {
                    assert_eq!(id, "condition");
                    assert!(matches!(ctx, ExprContext::Load));
                } else {
                    panic!("Expected condition to be a name, got: {:?}", test);
                }

                // Verify body
                assert_eq!(body.len(), 1);
                if let Stmt::Expr { value, .. } = &*body[0] {
                    if let Expr::Call { func, args, .. } = &**value {
                        if let Expr::Name { id, .. } = &**func {
                            assert_eq!(id, "action");
                        } else {
                            panic!("Expected function name to be 'action', got: {:?}", func);
                        }

                        assert_eq!(args.len(), 0);
                    } else {
                        panic!("Expected call expression, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", body[0]);
                }

                // Verify no else clause
                assert!(orelse.is_empty());
            } else {
                panic!("Expected while statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // While loop with else clause
        let module = assert_parses("while condition:\n    action()\nelse:\n    cleanup()");

        if let Some(stmt) = module.body.first() {
            if let Stmt::While { orelse, .. } = &**stmt {
                // Verify else clause
                assert_eq!(orelse.len(), 1);
                if let Stmt::Expr { value, .. } = &*orelse[0] {
                    if let Expr::Call { func, args, .. } = &**value {
                        if let Expr::Name { id, .. } = &**func {
                            assert_eq!(id, "cleanup");
                        } else {
                            panic!("Expected function name to be 'cleanup', got: {:?}", func);
                        }

                        assert_eq!(args.len(), 0);
                    } else {
                        panic!("Expected call expression, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", orelse[0]);
                }
            } else {
                panic!("Expected while statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_function_def() {
        // Simple function definition
        let module = assert_parses("def greet(name):\n    return 'Hello, ' + name");

        if let Some(stmt) = module.body.first() {
            if let Stmt::FunctionDef { name, params, body, returns, decorator_list, is_async, .. } = &**stmt {
                // Verify function name
                assert_eq!(name, "greet");

                // Verify parameters
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "name");
                assert!(params[0].default.is_none());

                // Verify body
                assert_eq!(body.len(), 1);
                if let Stmt::Return { value, .. } = &*body[0] {
                    assert!(value.is_some());
                    if let Some(return_value) = value {
                        if let Expr::BinOp { left, op, right, .. } = &**return_value {
                            if let Expr::Str { value, .. } = &**left {
                                assert_eq!(value, "Hello, ");
                            } else {
                                panic!("Expected left operand to be a string, got: {:?}", left);
                            }

                            assert_eq!(*op, Operator::Add);

                            if let Expr::Name { id, .. } = &**right {
                                assert_eq!(id, "name");
                            } else {
                                panic!("Expected right operand to be a name, got: {:?}", right);
                            }
                        } else {
                            panic!("Expected binary operation, got: {:?}", return_value);
                        }
                    }
                } else {
                    panic!("Expected return statement, got: {:?}", body[0]);
                }

                // Verify no return type annotation
                assert!(returns.is_none());

                // Verify no decorators
                assert!(decorator_list.is_empty());

                // Verify not async
                assert!(!is_async);
            } else {
                panic!("Expected function definition, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Function with default parameter
        let module = assert_parses("def greet(name, greeting='Hello'):\n    return greeting + ', ' + name");

        if let Some(stmt) = module.body.first() {
            if let Stmt::FunctionDef { params, .. } = &**stmt {
                // Verify parameters
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "name");
                assert!(params[0].default.is_none());
                assert_eq!(params[1].name, "greeting");
                assert!(params[1].default.is_some());

                // Verify default value
                if let Some(default) = &params[1].default {
                    if let Expr::Str { value, .. } = &**default {
                        assert_eq!(value, "Hello");
                    } else {
                        panic!("Expected default value to be a string, got: {:?}", default);
                    }
                }
            } else {
                panic!("Expected function definition, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Function with type annotations
        let module = assert_parses("def add(a: int, b: int) -> int:\n    return a + b");

        if let Some(stmt) = module.body.first() {
            if let Stmt::FunctionDef { params, returns, .. } = &**stmt {
                // Verify parameter type annotations
                assert_eq!(params.len(), 2);
                assert!(params[0].typ.is_some());
                assert!(params[1].typ.is_some());

                if let Some(typ) = &params[0].typ {
                    if let Expr::Name { id, .. } = &**typ {
                        assert_eq!(id, "int");
                    } else {
                        panic!("Expected type to be a name, got: {:?}", typ);
                    }
                }

                // Verify return type annotation
                assert!(returns.is_some());
                if let Some(ret_type) = returns {
                    if let Expr::Name { id, .. } = &**ret_type {
                        assert_eq!(id, "int");
                    } else {
                        panic!("Expected return type to be a name, got: {:?}", ret_type);
                    }
                }
            } else {
                panic!("Expected function definition, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Async function
        let module = assert_parses("async def fetch():\n    pass");

        if let Some(stmt) = module.body.first() {
            if let Stmt::FunctionDef { is_async, .. } = &**stmt {
                // Verify async
                assert!(*is_async);
            } else {
                panic!("Expected function definition, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Function with decorators
        let module = assert_parses("@decorator\ndef func():\n    pass");

        if let Some(stmt) = module.body.first() {
            if let Stmt::FunctionDef { decorator_list, .. } = &**stmt {
                // Verify decorators
                assert_eq!(decorator_list.len(), 1);
                if let Expr::Name { id, .. } = &*decorator_list[0] {
                    assert_eq!(id, "decorator");
                } else {
                    panic!("Expected decorator to be a name, got: {:?}", decorator_list[0]);
                }
            } else {
                panic!("Expected function definition, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_class_def() {
        // Simple class definition
        let module = assert_parses("class Point:\n    def __init__(self, x, y):\n        self.x = x\n        self.y = y");

        if let Some(stmt) = module.body.first() {
            if let Stmt::ClassDef { name, bases, keywords, body, decorator_list, .. } = &**stmt {
                // Verify class name
                assert_eq!(name, "Point");

                // Verify no base classes
                assert!(bases.is_empty());

                // Verify no keywords
                assert!(keywords.is_empty());

                // Verify body (should have one method: __init__)
                assert_eq!(body.len(), 1);
                if let Stmt::FunctionDef { name, params, body: method_body, .. } = &*body[0] {
                    assert_eq!(name, "__init__");

                    // Verify method parameters (self, x, y)
                    assert_eq!(params.len(), 3);
                    assert_eq!(params[0].name, "self");
                    assert_eq!(params[1].name, "x");
                    assert_eq!(params[2].name, "y");

                    // Verify method body (two assignments: self.x = x, self.y = y)
                    assert_eq!(method_body.len(), 2);

                    if let Stmt::Assign { targets, value, .. } = &*method_body[0] {
                        assert_eq!(targets.len(), 1);
                        if let Expr::Attribute { value: obj, attr, .. } = &*targets[0] {
                            if let Expr::Name { id, .. } = &**obj {
                                assert_eq!(id, "self");
                            } else {
                                panic!("Expected object to be 'self', got: {:?}", obj);
                            }

                            assert_eq!(attr, "x");
                        } else {
                            panic!("Expected target to be an attribute, got: {:?}", targets[0]);
                        }

                        if let Expr::Name { id, .. } = &**value {
                            assert_eq!(id, "x");
                        } else {
                            panic!("Expected value to be a name, got: {:?}", value);
                        }
                    } else {
                        panic!("Expected assignment statement, got: {:?}", method_body[0]);
                    }

                    if let Stmt::Assign { targets, value, .. } = &*method_body[1] {
                        assert_eq!(targets.len(), 1);
                        if let Expr::Attribute { value: obj, attr, .. } = &*targets[0] {
                            if let Expr::Name { id, .. } = &**obj {
                                assert_eq!(id, "self");
                            } else {
                                panic!("Expected object to be 'self', got: {:?}", obj);
                            }

                            assert_eq!(attr, "y");
                        } else {
                            panic!("Expected target to be an attribute, got: {:?}", targets[0]);
                        }

                        if let Expr::Name { id, .. } = &**value {
                            assert_eq!(id, "y");
                        } else {
                            panic!("Expected value to be a name, got: {:?}", value);
                        }
                    } else {
                        panic!("Expected assignment statement, got: {:?}", method_body[1]);
                    }
                } else {
                    panic!("Expected function definition, got: {:?}", body[0]);
                }

                // Verify no decorators
                assert!(decorator_list.is_empty());
            } else {
                panic!("Expected class definition, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Class with inheritance
        let module = assert_parses("class Rectangle(Shape):\n    pass");

        if let Some(stmt) = module.body.first() {
            if let Stmt::ClassDef { bases, .. } = &**stmt {
                // Verify base classes
                assert_eq!(bases.len(), 1);
                if let Expr::Name { id, .. } = &*bases[0] {
                    assert_eq!(id, "Shape");
                } else {
                    panic!("Expected base class to be a name, got: {:?}", bases[0]);
                }
            } else {
                panic!("Expected class definition, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Class with multiple inheritance
        let module = assert_parses("class Child(Mother, Father):\n    pass");

        if let Some(stmt) = module.body.first() {
            if let Stmt::ClassDef { bases, .. } = &**stmt {
                // Verify base classes
                assert_eq!(bases.len(), 2);
                if let Expr::Name { id, .. } = &*bases[0] {
                    assert_eq!(id, "Mother");
                } else {
                    panic!("Expected first base class to be a name, got: {:?}", bases[0]);
                }

                if let Expr::Name { id, .. } = &*bases[1] {
                    assert_eq!(id, "Father");
                } else {
                    panic!("Expected second base class to be a name, got: {:?}", bases[1]);
                }
            } else {
                panic!("Expected class definition, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Class with keyword arguments
        let module = assert_parses("class Meta(type, metaclass=ABCMeta):\n    pass");

        if let Some(stmt) = module.body.first() {
            if let Stmt::ClassDef { bases, keywords, .. } = &**stmt {
                // Verify base classes
                assert_eq!(bases.len(), 1);
                if let Expr::Name { id, .. } = &*bases[0] {
                    assert_eq!(id, "type");
                } else {
                    panic!("Expected base class to be a name, got: {:?}", bases[0]);
                }

                // Verify keywords
                assert_eq!(keywords.len(), 1);
                let (key, value) = &keywords[0];
                assert!(key.is_some());
                assert_eq!(key.as_ref().unwrap(), "metaclass");

                if let Expr::Name { id, .. } = &**value {
                    assert_eq!(id, "ABCMeta");
                } else {
                    panic!("Expected keyword value to be a name, got: {:?}", value);
                }
            } else {
                panic!("Expected class definition, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_import_statements() {
        // Simple import
        let module = assert_parses("import module");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Import { names, .. } = &**stmt {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0].name, "module");
                assert!(names[0].asname.is_none());
            } else {
                panic!("Expected import statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Import with alias
        let module = assert_parses("import module as mod");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Import { names, .. } = &**stmt {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0].name, "module");
                assert!(names[0].asname.is_some());
                assert_eq!(names[0].asname.as_ref().unwrap(), "mod");
            } else {
                panic!("Expected import statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Multiple imports
        let module = assert_parses("import module1, module2");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Import { names, .. } = &**stmt {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0].name, "module1");
                assert_eq!(names[1].name, "module2");
            } else {
                panic!("Expected import statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // From import
        let module = assert_parses("from module import item");

        if let Some(stmt) = module.body.first() {
            if let Stmt::ImportFrom { module, names, level, .. } = &**stmt {
                assert!(module.is_some());
                assert_eq!(module.as_ref().unwrap(), "module");

                assert_eq!(names.len(), 1);
                assert_eq!(names[0].name, "item");
                assert!(names[0].asname.is_none());

                assert_eq!(*level, 0);
            } else {
                panic!("Expected import from statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // From import with multiple items
        let module = assert_parses("from module import item1, item2");

        if let Some(stmt) = module.body.first() {
            if let Stmt::ImportFrom { names, .. } = &**stmt {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0].name, "item1");
                assert_eq!(names[1].name, "item2");
            } else {
                panic!("Expected import from statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // From import with aliases
        let module = assert_parses("from module import item1 as alias1, item2 as alias2");

        if let Some(stmt) = module.body.first() {
            if let Stmt::ImportFrom { names, .. } = &**stmt {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0].name, "item1");
                assert!(names[0].asname.is_some());
                assert_eq!(names[0].asname.as_ref().unwrap(), "alias1");

                assert_eq!(names[1].name, "item2");
                assert!(names[1].asname.is_some());
                assert_eq!(names[1].asname.as_ref().unwrap(), "alias2");
            } else {
                panic!("Expected import from statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // From import with relative imports
        let module = assert_parses("from ..module import item");

        if let Some(stmt) = module.body.first() {
            if let Stmt::ImportFrom { module, level, .. } = &**stmt {
                assert!(module.is_some());
                assert_eq!(module.as_ref().unwrap(), "module");

                assert_eq!(*level, 2);
            } else {
                panic!("Expected import from statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // From import with wildcard
        let module = assert_parses("from module import *");

        if let Some(stmt) = module.body.first() {
            if let Stmt::ImportFrom { names, .. } = &**stmt {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0].name, "*");
                assert!(names[0].asname.is_none());
            } else {
                panic!("Expected import from statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_try_except_statements() {
        // Simple try-except
        let module = assert_parses("try:\n    risky()\nexcept:\n    handle()");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Try { body, handlers, orelse, finalbody, .. } = &**stmt {
                // Verify try body
                assert_eq!(body.len(), 1);
                if let Stmt::Expr { value, .. } = &*body[0] {
                    if let Expr::Call { func, .. } = &**value {
                        if let Expr::Name { id, .. } = &**func {
                            assert_eq!(id, "risky");
                        } else {
                            panic!("Expected function name to be 'risky', got: {:?}", func);
                        }
                    } else {
                        panic!("Expected call expression, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", body[0]);
                }

                // Verify except handler
                assert_eq!(handlers.len(), 1);
                let handler = &handlers[0];
                assert!(handler.typ.is_none());
                assert!(handler.name.is_none());

                assert_eq!(handler.body.len(), 1);
                if let Stmt::Expr { value, .. } = &*handler.body[0] {
                    if let Expr::Call { func, .. } = &**value {
                        if let Expr::Name { id, .. } = &**func {
                            assert_eq!(id, "handle");
                        } else {
                            panic!("Expected function name to be 'handle', got: {:?}", func);
                        }
                    } else {
                        panic!("Expected call expression, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", handler.body[0]);
                }

                // Verify no else or finally
                assert!(orelse.is_empty());
                assert!(finalbody.is_empty());
            } else {
                panic!("Expected try statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Try-except with exception type
        let module = assert_parses("try:\n    risky()\nexcept Exception:\n    handle()");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Try { handlers, .. } = &**stmt {
                // Verify except handler
                assert_eq!(handlers.len(), 1);
                let handler = &handlers[0];
                assert!(handler.typ.is_some());
                assert!(handler.name.is_none());

                if let Some(typ) = &handler.typ {
                    if let Expr::Name { id, .. } = &**typ {
                        assert_eq!(id, "Exception");
                    } else {
                        panic!("Expected exception type to be a name, got: {:?}", typ);
                    }
                }
            } else {
                panic!("Expected try statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Try-except with exception type and alias
        let module = assert_parses("try:\n    risky()\nexcept Exception as e:\n    handle(e)");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Try { handlers, .. } = &**stmt {
                // Verify except handler
                assert_eq!(handlers.len(), 1);
                let handler = &handlers[0];
                assert!(handler.typ.is_some());
                assert!(handler.name.is_some());

                assert_eq!(handler.name.as_ref().unwrap(), "e");
            } else {
                panic!("Expected try statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Try-except-else
        let module = assert_parses("try:\n    risky()\nexcept:\n    handle()\nelse:\n    success()");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Try { orelse, .. } = &**stmt {
                // Verify else clause
                assert_eq!(orelse.len(), 1);
                if let Stmt::Expr { value, .. } = &*orelse[0] {
                    if let Expr::Call { func, .. } = &**value {
                        if let Expr::Name { id, .. } = &**func {
                            assert_eq!(id, "success");
                        } else {
                            panic!("Expected function name to be 'success', got: {:?}", func);
                        }
                    } else {
                        panic!("Expected call expression, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", orelse[0]);
                }
            } else {
                panic!("Expected try statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Try-except-finally
        let module = assert_parses("try:\n    risky()\nexcept:\n    handle()\nfinally:\n    cleanup()");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Try { finalbody, .. } = &**stmt {
                // Verify finally clause
                assert_eq!(finalbody.len(), 1);
                if let Stmt::Expr { value, .. } = &*finalbody[0] {
                    if let Expr::Call { func, .. } = &**value {
                        if let Expr::Name { id, .. } = &**func {
                            assert_eq!(id, "cleanup");
                        } else {
                            panic!("Expected function name to be 'cleanup', got: {:?}", func);
                        }
                    } else {
                        panic!("Expected call expression, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", finalbody[0]);
                }
            } else {
                panic!("Expected try statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Try-except-else-finally
        let module = assert_parses("try:\n    risky()\nexcept:\n    handle()\nelse:\n    success()\nfinally:\n    cleanup()");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Try { orelse, finalbody, .. } = &**stmt {
                // Verify both else and finally clauses are present
                assert_eq!(orelse.len(), 1);
                assert_eq!(finalbody.len(), 1);
            } else {
                panic!("Expected try statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Multiple except handlers
        let module = assert_parses("try:\n    risky()\nexcept ValueError:\n    handle_value()\nexcept TypeError:\n    handle_type()\nexcept:\n    handle_other()");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Try { handlers, .. } = &**stmt {
                // Verify multiple handlers
                assert_eq!(handlers.len(), 3);

                // First handler: ValueError
                assert!(handlers[0].typ.is_some());
                if let Some(typ) = &handlers[0].typ {
                    if let Expr::Name { id, .. } = &**typ {
                        assert_eq!(id, "ValueError");
                    } else {
                        panic!("Expected exception type to be 'ValueError', got: {:?}", typ);
                    }
                }

                // Second handler: TypeError
                assert!(handlers[1].typ.is_some());
                if let Some(typ) = &handlers[1].typ {
                    if let Expr::Name { id, .. } = &**typ {
                        assert_eq!(id, "TypeError");
                    } else {
                        panic!("Expected exception type to be 'TypeError', got: {:?}", typ);
                    }
                }

                // Third handler: catch-all
                assert!(handlers[2].typ.is_none());
            } else {
                panic!("Expected try statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }

    #[test]
    fn test_literals() {
        // Integer literal
        let module = assert_parses("42");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Num { value: num, .. } = &**value {
                    assert_eq!(*num, Number::Integer(42));
                } else {
                    panic!("Expected number, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Float literal
        let module = assert_parses("3.14");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Num { value: num, .. } = &**value {
                    match num {
                        Number::Float(f) => assert!((f - 3.14).abs() < f64::EPSILON),
                        _ => panic!("Expected float, got: {:?}", num),
                    }
                } else {
                    panic!("Expected number, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // String literal
        let module = assert_parses("\"hello\"");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Str { value: s, .. } = &**value {
                    assert_eq!(s, "hello");
                } else {
                    panic!("Expected string, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // True literal
        let module = assert_parses("True");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::NameConstant { value: constant, .. } = &**value {
                    assert_eq!(*constant, NameConstant::True);
                } else {
                    panic!("Expected name constant, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // False literal
        let module = assert_parses("False");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::NameConstant { value: constant, .. } = &**value {
                    assert_eq!(*constant, NameConstant::False);
                } else {
                    panic!("Expected name constant, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // None literal
        let module = assert_parses("None");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::NameConstant { value: constant, .. } = &**value {
                    assert_eq!(*constant, NameConstant::None);
                } else {
                    panic!("Expected name constant, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // List literal
        println!("\n==== TESTING SET LITERAL ====");
        let source = "{1, 2, 3}";
        println!("Source code: {}", source);

        let module = parse_code(source).unwrap_or_else(|errors| {
            println!("PARSING FAILED:");
            for error in &errors {
                println!("  - {}", ErrorFormatter(error));
            }
            panic!("Failed to parse set literal");
        });

        println!("Successfully parsed into module");

        if let Some(stmt) = module.body.first() {
            println!("Found first statement: {:?}", stmt);

            if let Stmt::Expr { value, .. } = &**stmt {
                println!("Statement is an expression: {:?}", value);

                if let Expr::Set { elts, .. } = &**value {
                    println!("Expression is a set with {} elements", elts.len());

                    // Dump each element
                    for (i, elt) in elts.iter().enumerate() {
                        println!("Element {}: {:?}", i, elt);

                        if let Expr::Num { value: num, .. } = &**elt {
                            println!("  - Number value: {:?}", num);
                        } else {
                            println!("  - Not a number: {:?}", elt);
                        }
                    }

                    // Try a different approach: collect all numbers without assuming order
                    let mut values = Vec::new();
                    for elt in elts.iter() {
                        if let Expr::Num { value: num, .. } = &**elt {
                            if let Number::Integer(i) = num {
                                println!("Adding integer: {}", i);
                                values.push(*i);
                            } else {
                                println!("Non-integer number: {:?}", num);
                                panic!("Expected integer, got: {:?}", num);
                            }
                        } else {
                            println!("Non-number element: {:?}", elt);
                            panic!("Expected number, got: {:?}", elt);
                        }
                    }

                    println!("Collected values: {:?}", values);
                    values.sort();
                    println!("Sorted values: {:?}", values);
                    println!("Expected values: [1, 2, 3]");

                    assert_eq!(values.len(), 3, "Expected 3 elements, got {}", values.len());
                    assert_eq!(values, vec![1, 2, 3], "Values don't match expected [1, 2, 3]");
                } else {
                    println!("Expression is not a set: {:?}", value);
                    panic!("Expected set, got: {:?}", value);
                }
            } else {
                println!("Statement is not an expression: {:?}", stmt);
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            println!("No statements in module");
            panic!("Expected at least one statement");
        }

        println!("Set literal test passed!");

        // Tuple literal
        let module = assert_parses("(1, 2, 3)");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Tuple { elts, ctx, .. } = &**value {
                    assert_eq!(elts.len(), 3);
                    assert!(matches!(ctx, ExprContext::Load));

                    for (i, elt) in elts.iter().enumerate() {
                        if let Expr::Num { value: num, .. } = &**elt {
                            assert_eq!(*num, Number::Integer(i as i64 + 1));
                        } else {
                            panic!("Expected number, got: {:?}", elt);
                        }
                    }
                } else {
                    panic!("Expected tuple, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Dict literal
        let module = assert_parses("{1: 'one', 2: 'two'}");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Dict { keys, values, .. } = &**value {
                    assert_eq!(keys.len(), 2);
                    assert_eq!(values.len(), 2);

                    // Check first key-value pair
                    assert!(keys[0].is_some());
                    if let Some(key) = &keys[0] {
                        if let Expr::Num { value: num, .. } = &**key {
                            assert_eq!(*num, Number::Integer(1));
                        } else {
                            panic!("Expected number for key, got: {:?}", key);
                        }
                    }

                    if let Expr::Str { value: s, .. } = &*values[0] {
                        assert_eq!(s, "one");
                    } else {
                        panic!("Expected string for value, got: {:?}", values[0]);
                    }

                    // Check second key-value pair
                    assert!(keys[1].is_some());
                    if let Some(key) = &keys[1] {
                        if let Expr::Num { value: num, .. } = &**key {
                            assert_eq!(*num, Number::Integer(2));
                        } else {
                            panic!("Expected number for key, got: {:?}", key);
                        }
                    }

                    if let Expr::Str { value: s, .. } = &*values[1] {
                        assert_eq!(s, "two");
                    } else {
                        panic!("Expected string for value, got: {:?}", values[1]);
                    }
                } else {
                    panic!("Expected dict, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }

        // Set literal
        let module = assert_parses("{1, 2, 3}");

        if let Some(stmt) = module.body.first() {
            if let Stmt::Expr { value, .. } = &**stmt {
                if let Expr::Set { elts, .. } = &**value {
                    assert_eq!(elts.len(), 3);

                    for (i, elt) in elts.iter().enumerate() {
                        if let Expr::Num { value: num, .. } = &**elt {
                            assert_eq!(*num, Number::Integer(i as i64 + 1));
                        } else {
                            panic!("Expected number, got: {:?}", elt);
                        }
                    }
                } else {
                    panic!("Expected set, got: {:?}", value);
                }
            } else {
                panic!("Expected expression statement, got: {:?}", stmt);
            }
        } else {
            panic!("Expected at least one statement");
        }
    }
}