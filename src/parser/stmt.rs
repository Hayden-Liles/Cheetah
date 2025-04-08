use crate::ast::{
    Alias, ExceptHandler, Expr, ExprContext, Parameter, Stmt, Operator
};
use crate::lexer::TokenType;
use crate::parser::{Parser, ParseError};
use crate::parser::helpers::TokenMatching;
use crate::parser::types::{ParserContext, GetLocation};
use crate::parser::expr::ExprParser;

/// Parser methods for statements
pub trait StmtParser {
    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a function definition
    fn parse_function_def(&mut self) -> Result<Stmt, ParseError>;

    /// Parse function parameters
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParseError>;

    /// Parse decorators
    fn parse_decorators(&mut self) -> Result<Vec<Box<Expr>>, ParseError>;

    /// Parse a class definition
    fn parse_class_def(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a class argument
    fn parse_class_argument(
        &mut self,
        bases: &mut Vec<Box<Expr>>,
        keywords: &mut Vec<(Option<String>, Box<Expr>)>,
    ) -> Result<(), ParseError>;

    /// Parse a return statement
    fn parse_return(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a delete statement
    fn parse_delete(&mut self) -> Result<Stmt, ParseError>;

    /// Parse an if statement
    fn parse_if(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a for statement
    fn parse_for(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a while statement
    fn parse_while(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a with statement
    fn parse_with(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a try statement
    fn parse_try(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a raise statement
    fn parse_raise(&mut self) -> Result<Stmt, ParseError>;

    /// Parse an assert statement
    fn parse_assert(&mut self) -> Result<Stmt, ParseError>;

    /// Parse an import statement
    fn parse_import(&mut self) -> Result<Stmt, ParseError>;

    /// Parse import names
    fn parse_import_names(&mut self) -> Result<Vec<Alias>, ParseError>;

    /// Parse an import-from statement
    fn parse_import_from(&mut self) -> Result<Stmt, ParseError>;

    /// Parse import-as names
    fn parse_import_as_names(&mut self) -> Result<Vec<Alias>, ParseError>;

    /// Parse a global statement
    fn parse_global(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a nonlocal statement
    fn parse_nonlocal(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a name list
    fn parse_name_list(&mut self) -> Result<Vec<String>, ParseError>;

    /// Parse a pass statement
    fn parse_pass(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a break statement
    fn parse_break(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a continue statement
    fn parse_continue(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a match statement
    fn parse_match(&mut self) -> Result<Stmt, ParseError>;

    /// Parse a match case
    fn parse_match_case(&mut self) -> Result<(Box<Expr>, Option<Box<Expr>>, Vec<Box<Stmt>>), ParseError>;

    /// Parse an expression statement
    fn parse_expr_statement(&mut self) -> Result<Stmt, ParseError>;

    /// Validate an assignment target
    fn validate_assignment_target(&self, expr: &Expr) -> Result<(), ParseError>;

    /// Check if token is an augmented assignment operator
    fn is_augmented_assign(&self) -> bool;

    /// Parse an augmented assignment operator
    fn parse_augmented_assign_op(&self) -> Operator;

    /// Parse a suite (indented block or single statement)
    fn parse_suite(&mut self) -> Result<Vec<Box<Stmt>>, ParseError>;

    /// Convert an expression to store context
    fn with_store_context(&self, expr: Expr) -> Result<Expr, ParseError>;
}

impl StmtParser for Parser {
    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let token_type;
        let line;
        let column;

        match &self.current {
            Some(token) => {
                token_type = token.token_type.clone();
                line = token.line;
                column = token.column;
            }
            None => {
                return Err(ParseError::EOF {
                    expected: "statement".to_string(),
                    line: 0,
                    column: 0,
                });
            }
        }

        if matches!(token_type, TokenType::SemiColon) {
            self.advance(); // Consume the semicolon
            return Ok(Stmt::Pass { line, column }); // Use Pass as an empty statement
        }

        // Check for invalid operators at start of statement
        if matches!(
            token_type,
            TokenType::Plus
                | TokenType::Minus
                | TokenType::Multiply
                | TokenType::Divide
                | TokenType::Modulo
                | TokenType::Power
        ) {
            if self.peek_matches(TokenType::EOF) || self.peek_matches(TokenType::Newline) {
                return Err(ParseError::InvalidSyntax {
                    message: "Incomplete expression".to_string(),
                    line,
                    column,
                });
            }
        }

        if matches!(token_type, TokenType::Try) {
            return self.parse_try();
        }

        if matches!(token_type, TokenType::Except) {
            return Err(ParseError::InvalidSyntax {
                message: "'except' statement outside of try block".to_string(),
                line,
                column,
            });
        }

        if matches!(
            token_type,
            TokenType::IntLiteral(_)
                | TokenType::FloatLiteral(_)
                | TokenType::StringLiteral(_)
                | TokenType::True
                | TokenType::False
                | TokenType::None
        ) {
            let expr = self.parse_expression()?;
            let expr_line = expr.get_line();
            let expr_column = expr.get_column();

            if self.match_token(TokenType::Assign) {
                return Err(ParseError::InvalidSyntax {
                    message: "Cannot assign to literal".to_string(),
                    line: expr_line,
                    column: expr_column,
                });
            }

            self.consume_newline()?;
            return Ok(Stmt::Expr {
                value: Box::new(expr),
                line: expr_line,
                column: expr_column,
            });
        }

        if matches!(token_type, TokenType::Yield) {
            let yield_expr = self.parse_yield_expr()?;
            let line = yield_expr.get_line();
            let column = yield_expr.get_column();

            self.consume_newline()?;

            return Ok(Stmt::Expr {
                value: Box::new(yield_expr),
                line,
                column,
            });
        }

        if matches!(token_type, TokenType::At) {
            let decorators = self.parse_decorators()?;

            let decorated_token_type = self.current.as_ref().map(|t| t.token_type.clone());

            match decorated_token_type {
                Some(TokenType::Def) => {
                    let mut func_def = self.parse_function_def()?;
                    if let Stmt::FunctionDef {
                        ref mut decorator_list,
                        ..
                    } = func_def
                    {
                        *decorator_list = decorators;
                    }
                    return Ok(func_def);
                }
                Some(TokenType::Class) => {
                    let mut class_def = self.parse_class_def()?;
                    if let Stmt::ClassDef {
                        ref mut decorator_list,
                        ..
                    } = class_def
                    {
                        *decorator_list = decorators;
                    }
                    return Ok(class_def);
                }
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected function or class definition after decorators".to_string(),
                        line,
                        column,
                    });
                }
            }
        }

        if matches!(token_type, TokenType::Async) {
            self.advance();

            let async_next_token_type = self.current.as_ref().map(|t| t.token_type.clone());

            match async_next_token_type {
                Some(TokenType::Def) => {
                    let mut func_def = self.parse_function_def()?;

                    if let Stmt::FunctionDef {
                        ref mut is_async, ..
                    } = func_def
                    {
                        *is_async = true;
                    }

                    return Ok(func_def);
                }
                Some(TokenType::For) => {
                    let mut for_stmt = self.parse_for()?;

                    if let Stmt::For {
                        ref mut is_async, ..
                    } = for_stmt
                    {
                        *is_async = true;
                    }

                    return Ok(for_stmt);
                }
                Some(TokenType::With) => {
                    let mut with_stmt = self.parse_with()?;

                    if let Stmt::With {
                        ref mut is_async, ..
                    } = with_stmt
                    {
                        *is_async = true;
                    }

                    return Ok(with_stmt);
                }
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected 'def', 'for', or 'with' after 'async'".to_string(),
                        line,
                        column,
                    });
                }
            }
        }

        match token_type {
            TokenType::Def => self.parse_function_def(),
            TokenType::Class => self.parse_class_def(),
            TokenType::Return => self.parse_return(),
            TokenType::Del => self.parse_delete(),
            TokenType::If => self.parse_if(),
            TokenType::For => self.parse_for(),
            TokenType::While => self.parse_while(),
            TokenType::With => self.parse_with(),
            TokenType::Raise => self.parse_raise(),
            TokenType::Assert => self.parse_assert(),
            TokenType::Import => self.parse_import(),
            TokenType::From => self.parse_import_from(),
            TokenType::Global => self.parse_global(),
            TokenType::Nonlocal => self.parse_nonlocal(),
            TokenType::Pass => self.parse_pass(),
            TokenType::Break => self.parse_break(),
            TokenType::Continue => self.parse_continue(),
            TokenType::Match => self.parse_match(),
            _ => self.parse_expr_statement()
        }
    }

    fn parse_function_def(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let name = self.consume_identifier("function name")?;

        self.consume(TokenType::LeftParen, "(")?;
        let params = self.parse_parameters()?;
        self.consume(TokenType::RightParen, ")")?;

        let returns = if self.match_token(TokenType::Arrow) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        let body = self.with_context(ParserContext::Function, |parser| {
            parser.consume(TokenType::Colon, ":")?;
            parser.parse_suite()
        })?;

        Ok(Stmt::FunctionDef {
            name,
            params,
            body,
            decorator_list: Vec::new(),
            returns,
            is_async: false,
            line,
            column,
        })
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let mut params = Vec::new();
        let mut has_kwarg = false;
        let mut has_vararg = false;
        let mut has_seen_default = false;
        let mut has_pos_only_separator = false;

        if self.check(TokenType::RightParen) {
            return Ok(params);
        }

        loop {
            // Handle trailing comma error
            if self.check(TokenType::RightParen) && params.len() > 0 {
                // We've seen a parameter and now we're at a closing paren
                // Check if the last token was a comma
                if let Some(last_token) = &self.last_token {
                    if matches!(last_token.token_type, TokenType::Comma) {
                        return Err(ParseError::InvalidSyntax {
                            message: "Trailing comma in parameter list".to_string(),
                            line: last_token.line,
                            column: last_token.column,
                        });
                    }
                }
            }

            if self.match_token(TokenType::Divide) {
                has_pos_only_separator = true;

                if !self.check(TokenType::Comma) && !self.check(TokenType::RightParen) {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected comma or closing parenthesis after '/'".to_string(),
                        line: self.current.as_ref().map_or(0, |t| t.line),
                        column: self.current.as_ref().map_or(0, |t| t.column),
                    });
                }

                if self.match_token(TokenType::Comma) {
                    if self.check(TokenType::RightParen) {
                        // This is the fix - report an error for trailing comma
                        return Err(ParseError::InvalidSyntax {
                            message: "Trailing comma in parameter list".to_string(),
                            line: self.last_token.as_ref().unwrap().line,
                            column: self.last_token.as_ref().unwrap().column,
                        });
                    }
                    continue;
                } else {
                    break;
                }
            }

            if has_kwarg {
                return Err(ParseError::InvalidSyntax {
                    message: "Parameter after **kwargs is not allowed".to_string(),
                    line: self.current.as_ref().map_or(0, |t| t.line),
                    column: self.current.as_ref().map_or(0, |t| t.column),
                });
            }

            if self.match_token(TokenType::Multiply) {
                if self.check(TokenType::Comma) || self.check(TokenType::RightParen) {
                    has_vararg = true;
                    if self.match_token(TokenType::Comma) {
                        if self.check(TokenType::RightParen) {
                            // This is the fix - report an error for trailing comma
                            return Err(ParseError::InvalidSyntax {
                                message: "Trailing comma in parameter list".to_string(),
                                line: self.last_token.as_ref().unwrap().line,
                                column: self.last_token.as_ref().unwrap().column,
                            });
                        }
                        continue;
                    }
                    break;
                }

                let name = self.consume_identifier("parameter name after *")?;

                let typ = if self.match_token(TokenType::Colon) {
                    Some(Box::new(self.parse_type_annotation(false)?))
                } else {
                    None
                };

                if self.check(TokenType::Assign) {
                    return Err(ParseError::InvalidSyntax {
                        message: "Variadic argument cannot have default value".to_string(),
                        line: self.current.as_ref().map_or(0, |t| t.line),
                        column: self.current.as_ref().map_or(0, |t| t.column),
                    });
                }

                has_vararg = true;

                params.push(Parameter {
                    name,
                    typ,
                    default: None,
                    is_vararg: true,
                    is_kwarg: false,
                });
            } else if self.match_token(TokenType::Power) {
                let name = self.consume_identifier("parameter name after **")?;

                let typ = if self.match_token(TokenType::Colon) {
                    Some(Box::new(self.parse_type_annotation(false)?))
                } else {
                    None
                };

                if self.check(TokenType::Assign) {
                    return Err(ParseError::InvalidSyntax {
                        message: "Keyword argument cannot have default value".to_string(),
                        line: self.current.as_ref().map_or(0, |t| t.line),
                        column: self.current.as_ref().map_or(0, |t| t.column),
                    });
                }

                has_kwarg = true;

                params.push(Parameter {
                    name,
                    typ,
                    default: None,
                    is_vararg: false,
                    is_kwarg: true,
                });
            } else if self.check_identifier() {
                let param_pos = (
                    self.current.as_ref().map_or(0, |t| t.line),
                    self.current.as_ref().map_or(0, |t| t.column),
                );
                let param_name = self.consume_identifier("parameter name")?;

                if self.check_identifier() {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected comma between parameters".to_string(),
                        line: param_pos.0,
                        column: param_pos.1 + param_name.len(),
                    });
                }

                let typ = if self.match_token(TokenType::Colon) {
                    Some(Box::new(self.parse_type_annotation(false)?))
                } else {
                    None
                };

                let default = if self.match_token(TokenType::Assign) {
                    has_seen_default = true;

                    let default_expr = self.parse_or_test()?;

                    Some(Box::new(default_expr))
                } else {
                    if has_seen_default && !has_kwarg && !has_vararg && !has_pos_only_separator {
                        println!(
                            "Warning: non-default parameter after default parameter at line {}, column {}",
                            param_pos.0, param_pos.1
                        );
                    }
                    None
                };

                params.push(Parameter {
                    name: param_name,
                    typ,
                    default,
                    is_vararg: false,
                    is_kwarg: false,
                });
            } else {
                let token = self.current.clone().unwrap_or_else(|| panic!("Expected token"));

                return Err(ParseError::InvalidSyntax {
                    message: "Expected parameter name, * or **".to_string(),
                    line: token.line,
                    column: token.column,
                });
            }

            if self.match_token(TokenType::Comma) {
                if self.check(TokenType::RightParen) {
                    // This is the key fix: return an error for trailing comma
                    return Err(ParseError::InvalidSyntax {
                        message: "Trailing comma in parameter list".to_string(),
                        line: self.last_token.as_ref().unwrap().line,
                        column: self.last_token.as_ref().unwrap().column,
                    });
                }
            } else {
                if !self.check(TokenType::RightParen) {
                    let token = self.current.clone().unwrap_or_else(|| panic!("Expected token"));

                    return Err(ParseError::InvalidSyntax {
                        message: "Expected comma or closing parenthesis".to_string(),
                        line: token.line,
                        column: token.column,
                    });
                }
                break;
            }
        }

        Ok(params)
    }

    fn parse_decorators(&mut self) -> Result<Vec<Box<Expr>>, ParseError> {
        let mut decorators = Vec::new();

        while self.match_token(TokenType::At) {
            let decorator_expr = self.parse_expression()?;

            match &decorator_expr {
                Expr::Name { .. } | Expr::Attribute { .. } | Expr::Call { .. } => {
                    decorators.push(Box::new(decorator_expr));
                }
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: "Invalid decorator expression".to_string(),
                        line: decorator_expr.get_line(),
                        column: decorator_expr.get_column(),
                    });
                }
            }

            self.consume_newline()?;
        }

        Ok(decorators)
    }

    fn parse_class_argument(
        &mut self,
        bases: &mut Vec<Box<Expr>>,
        keywords: &mut Vec<(Option<String>, Box<Expr>)>,
    ) -> Result<(), ParseError> {
        if self.current.is_none() {
            return Err(ParseError::EOF {
                expected: "class argument".to_string(),
                line: self.last_token.as_ref().map_or(0, |t| t.line),
                column: self.last_token.as_ref().map_or(0, |t| t.column),
            });
        }

        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        match &token.token_type {
            TokenType::Multiply => {
                self.advance();

                if let Some(id_token) = &self.current {
                    if let TokenType::Identifier(name) = &id_token.token_type {
                        let args_name = name.clone();
                        self.advance();

                        bases.push(Box::new(Expr::Starred {
                            value: Box::new(Expr::Name {
                                id: args_name,
                                ctx: ExprContext::Load,
                                line,
                                column: column + 1,
                            }),
                            ctx: ExprContext::Load,
                            line,
                            column,
                        }));

                        return Ok(());
                    }
                }

                return Err(ParseError::InvalidSyntax {
                    message: "Expected identifier after *".to_string(),
                    line,
                    column,
                });
            }
            TokenType::Power => {
                self.advance();

                if let Some(id_token) = &self.current {
                    if let TokenType::Identifier(name) = &id_token.token_type {
                        let kwargs_name = name.clone();
                        self.advance();

                        keywords.push((
                            None,
                            Box::new(Expr::Name {
                                id: kwargs_name,
                                ctx: ExprContext::Load,
                                line,
                                column: column + 2,
                            }),
                        ));

                        return Ok(());
                    }
                }

                return Err(ParseError::InvalidSyntax {
                    message: "Expected identifier after **".to_string(),
                    line,
                    column,
                });
            }

            TokenType::Comma => {
                return Err(ParseError::UnexpectedToken {
                    expected: "expression".to_string(),
                    found: TokenType::Comma,
                    line,
                    column,
                });
            }

            TokenType::Identifier(name) => {
                let id_name = name.clone();
                self.advance();

                if let Some(token) = &self.current {
                    if matches!(token.token_type, TokenType::Assign) {
                        self.advance();

                        let value = self.parse_or_test()?;
                        keywords.push((Some(id_name), Box::new(value)));
                        return Ok(());
                    } else if matches!(token.token_type, TokenType::LeftParen) {
                        self.advance();

                        let args = Vec::new();
                        let kw_args = Vec::new();

                        if let Some(token) = &self.current {
                            if !matches!(token.token_type, TokenType::RightParen) {
                                // This method would typically be implemented elsewhere
                                // but for simplicity we'll just close the parentheses here
                                self.consume(TokenType::RightParen, ")")?;
                            } else {
                                self.advance();
                            }
                        }

                        bases.push(Box::new(Expr::Call {
                            func: Box::new(Expr::Name {
                                id: id_name,
                                ctx: ExprContext::Load,
                                line,
                                column,
                            }),
                            args,
                            keywords: kw_args,
                            line,
                            column,
                        }));
                        return Ok(());
                    }
                }

                bases.push(Box::new(Expr::Name {
                    id: id_name,
                    ctx: ExprContext::Load,
                    line,
                    column,
                }));
                return Ok(());
            }

            _ => {
                let expr = self.parse_atom_expr()?;
                bases.push(Box::new(expr));
                return Ok(());
            }
        }
    }

    fn parse_class_def(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let name = self.consume_identifier("class name")?;

        let (bases, keywords) = if self.match_token(TokenType::LeftParen) {
            let mut bases = Vec::new();
            let mut keywords = Vec::new();

            if !self.check(TokenType::RightParen) {
                self.parse_class_argument(&mut bases, &mut keywords)?;

                // Check for missing comma between base classes
                if !self.check(TokenType::Comma) && !self.check(TokenType::RightParen) {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected comma between base classes".to_string(),
                        line: self.current.as_ref().unwrap().line,
                        column: self.current.as_ref().unwrap().column,
                    });
                }

                while self.match_token(TokenType::Comma) {
                    if self.check(TokenType::RightParen) {
                        break;
                    }

                    self.parse_class_argument(&mut bases, &mut keywords)?;
                }
            }

            self.consume(TokenType::RightParen, ")")?;
            (bases, keywords)
        } else {
            (Vec::new(), Vec::new())
        };

        self.consume(TokenType::Colon, ":")?;

        let body = self.parse_suite()?;

        Ok(Stmt::ClassDef {
            name,
            bases,
            keywords,
            body,
            decorator_list: Vec::new(),
            line,
            column,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        if !self.is_in_context(ParserContext::Function) {
            return Err(ParseError::InvalidSyntax {
                message: "Return statement outside of function".to_string(),
                line,
                column,
            });
        }

        self.advance();

        let value = if self.check_newline() || self.check(TokenType::EOF) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        self.consume_newline()?;

        Ok(Stmt::Return {
            value,
            line,
            column,
        })
    }

    fn parse_delete(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let targets = self.parse_expr_list()?;

        self.consume_newline()?;

        Ok(Stmt::Delete {
            targets,
            line,
            column,
        })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        if self.check(TokenType::Colon) {
            return Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: TokenType::Colon,
                line: self.current.as_ref().unwrap().line,
                column: self.current.as_ref().unwrap().column,
            });
        }

        // Special case: detect assignment in condition
        if self.check_identifier() && self.peek_matches(TokenType::Assign) {
            let id_token = self.current.clone().unwrap();
            self.advance(); // consume the identifier
            self.advance(); // consume the '=' token

            // Return the error immediately
            return Err(ParseError::InvalidSyntax {
                message: "Cannot use assignment in a condition".to_string(),
                line: id_token.line,
                column: id_token.column,
            });
        }

        let test = Box::new(self.parse_expression()?);

        if !self.check(TokenType::Colon) {
            return Err(ParseError::InvalidSyntax {
                message: "Expected ':' after if condition".to_string(),
                line: self.current.as_ref().map_or(line, |t| t.line),
                column: self.current.as_ref().map_or(column + 2, |t| t.column),
            });
        }

        self.advance();

        let body = self.parse_suite()?;

        let mut orelse = Vec::new();

        if self.check(TokenType::Elif) {
            let elif_stmt = self.parse_if()?;
            orelse.push(Box::new(elif_stmt));
        } else if self.check(TokenType::Else) {
            self.advance(); // Consume the 'else' token
            self.consume(TokenType::Colon, ":")?;
            orelse = self.parse_suite()?;
        }

        Ok(Stmt::If {
            test,
            body,
            orelse,
            line,
            column,
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        if self.check(TokenType::In) {
            return Err(ParseError::InvalidSyntax {
                message: "Expected target after 'for'".to_string(),
                line,
                column,
            });
        }

        let target = self.with_context(ParserContext::Loop, |parser| {
            // Parse the target of the for loop
            if parser.check_identifier() && parser.peek_matches(TokenType::Comma) {
                let id_line = parser.current.as_ref().unwrap().line;
                let id_column = parser.current.as_ref().unwrap().column;

                let first_name = parser.consume_identifier("identifier")?;
                let first_expr = Expr::Name {
                    id: first_name,
                    ctx: ExprContext::Store,
                    line: id_line,
                    column: id_column,
                };

                parser.advance();

                let mut elts = vec![Box::new(first_expr)];

                while !parser.check(TokenType::In) {
                    if parser.check_identifier() {
                        let next_line = parser.current.as_ref().unwrap().line;
                        let next_column = parser.current.as_ref().unwrap().column;
                        let next_name = parser.consume_identifier("identifier")?;

                        elts.push(Box::new(Expr::Name {
                            id: next_name,
                            ctx: ExprContext::Store,
                            line: next_line,
                            column: next_column,
                        }));
                    } else if parser.check(TokenType::LeftParen) {
                        let nested_expr = parser.parse_atom_expr()?;
                        let nested_expr_with_store = parser.with_store_context(nested_expr)?;
                        elts.push(Box::new(nested_expr_with_store));
                    } else {
                        return Err(ParseError::InvalidSyntax {
                            message: "Expected identifier or tuple in for loop target".to_string(),
                            line: parser.current.as_ref().unwrap().line,
                            column: parser.current.as_ref().unwrap().column,
                        });
                    }

                    if !parser.match_token(TokenType::Comma) {
                        break;
                    }
                }

                Ok(Box::new(Expr::Tuple {
                    elts,
                    ctx: ExprContext::Store,
                    line: id_line,
                    column: id_column,
                }))
            } else {
                let expr = parser.parse_atom_expr()?;
                Ok(Box::new(parser.with_store_context(expr)?))
            }
        })?;

        self.consume(TokenType::In, "in")?;

        let iter = Box::new(self.parse_expression()?);

        self.consume(TokenType::Colon, ":")?;

        let body = self.with_context(ParserContext::Loop, |parser| {
            parser.parse_suite()
        })?;

        let orelse = if self.match_token(TokenType::Else) {
            self.consume(TokenType::Colon, ":")?;
            self.parse_suite()?
        } else {
            Vec::new()
        };

        Ok(Stmt::For {
            target,
            iter,
            body,
            orelse,
            is_async: false,
            line,
            column,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let test = Box::new(self.parse_expression()?);

        self.consume(TokenType::Colon, ":")?;

        let body = self.with_context(ParserContext::Loop, |parser| {
            parser.parse_suite()
        })?;

        let orelse = if self.match_token(TokenType::Else) {
            self.consume(TokenType::Colon, ":")?;
            self.parse_suite()?
        } else {
            Vec::new()
        };

        Ok(Stmt::While {
            test,
            body,
            orelse,
            line,
            column,
        })
    }

    fn parse_with(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let mut items = Vec::new();

        loop {
            let context_expr = Box::new(self.parse_expression()?);

            let optional_vars = if self.match_token(TokenType::As) {
                let expr = self.parse_atom_expr()?;
                Some(Box::new(self.with_store_context(expr)?))
            } else {
                None
            };

            items.push((context_expr, optional_vars));

            if !self.match_token(TokenType::Comma) {
                break;
            }

            if self.check(TokenType::Colon) {
                return Err(ParseError::InvalidSyntax {
                    message: "Expected context manager after comma".to_string(),
                    line: self.current.as_ref().unwrap().line,
                    column: self.current.as_ref().unwrap().column,
                });
            }
        }

        self.consume(TokenType::Colon, ":")?;

        let body = self.parse_suite()?;

        Ok(Stmt::With {
            items,
            body,
            is_async: false,
            line,
            column,
        })
    }

    fn parse_try(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        self.consume(TokenType::Colon, ":")?;
        let body = self.parse_suite()?;

        let mut handlers = Vec::new();

        while self.check(TokenType::Except) {
            self.advance();

            let h_line = self.previous_token().line;
            let h_column = self.previous_token().column;

            let typ = if !self.check(TokenType::Colon) && !self.check(TokenType::As) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            let name = if self.match_token(TokenType::As) {
                Some(self.consume_identifier("exception name")?)
            } else {
                None
            };

            self.consume(TokenType::Colon, ":")?;
            let except_body = self.parse_suite()?;

            handlers.push(ExceptHandler {
                typ,
                name,
                body: except_body,
                line: h_line,
                column: h_column,
            });
        }

        let orelse = if self.match_token(TokenType::Else) {
            self.consume(TokenType::Colon, ":")?;
            self.parse_suite()?
        } else {
            Vec::new()
        };

        let finalbody = if self.match_token(TokenType::Finally) {
            self.consume(TokenType::Colon, ":")?;
            self.parse_suite()?
        } else {
            Vec::new()
        };

        Ok(Stmt::Try {
            body,
            handlers,
            orelse,
            finalbody,
            line,
            column,
        })
    }

    fn parse_raise(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let exc = if self.check_newline() || self.check(TokenType::EOF) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        let cause = if self.match_token(TokenType::From) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.consume_newline()?;

        Ok(Stmt::Raise {
            exc,
            cause,
            line,
            column,
        })
    }

    fn parse_assert(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let test = Box::new(self.parse_expression()?);

        let msg = if self.match_token(TokenType::Comma) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.consume_newline()?;

        Ok(Stmt::Assert {
            test,
            msg,
            line,
            column,
        })
    }

    fn parse_import(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        if self.check_newline() || self.check(TokenType::EOF) || self.check(TokenType::SemiColon) {
            return Err(ParseError::InvalidSyntax {
                message: "Expected module name after 'import'".to_string(),
                line,
                column: column + 6,
            });
        }

        let names = self.parse_import_names()?;

        if names.is_empty() {
            return Err(ParseError::InvalidSyntax {
                message: "Expected module name after 'import'".to_string(),
                line,
                column: column + 6,
            });
        }

        self.consume_newline()?;

        Ok(Stmt::Import {
            names,
            line,
            column,
        })
    }

    fn parse_import_names(&mut self) -> Result<Vec<Alias>, ParseError> {
        let mut names = Vec::new();

        loop {
            let name = self.consume_dotted_name("module name")?;

            let asname = if self.match_token(TokenType::As) {
                Some(self.consume_identifier("import alias")?)
            } else {
                None
            };

            names.push(Alias { name, asname });

            if !self.match_token(TokenType::Comma) {
                break;
            }

            if self.check_newline() || self.check(TokenType::EOF) {
                break;
            }
        }

        Ok(names)
    }

    fn parse_import_from(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let mut level = 0;
        while self.match_token(TokenType::Dot) {
            level += 1;
        }

        if self.check(TokenType::Import) && level == 0 {
            return Err(ParseError::InvalidSyntax {
                message: "Expected module name after 'from'".to_string(),
                line: self.current.as_ref().unwrap().line,
                column: self.current.as_ref().unwrap().column,
            });
        }

        let module = if self.check(TokenType::Import) {
            None
        } else {
            Some(self.consume_dotted_name("module name")?)
        };

        self.consume(TokenType::Import, "import")?;

        let names = if self.match_token(TokenType::Multiply) {
            vec![Alias {
                name: "*".to_string(),
                asname: None,
            }]
        } else if self.check_newline()
            || self.check(TokenType::EOF)
            || self.check(TokenType::SemiColon)
        {
            return Err(ParseError::InvalidSyntax {
                message: "Expected import item after 'import'".to_string(),
                line: self.current.as_ref().unwrap().line,
                column: self.current.as_ref().unwrap().column,
            });
        } else {
            self.parse_import_as_names()?
        };

        self.consume_newline()?;

        Ok(Stmt::ImportFrom {
            module,
            names,
            level,
            line,
            column,
        })
    }

    fn parse_import_as_names(&mut self) -> Result<Vec<Alias>, ParseError> {
        let mut names = Vec::new();

        let has_parens = self.match_token(TokenType::LeftParen);

        loop {
            let name = self.consume_identifier("import name")?;

            let asname = if self.match_token(TokenType::As) {
                Some(self.consume_identifier("import alias")?)
            } else {
                None
            };

            names.push(Alias { name, asname });

            if !self.match_token(TokenType::Comma) {
                break;
            }

            if (has_parens && self.check(TokenType::RightParen))
                || (!has_parens && (self.check_newline() || self.check(TokenType::EOF)))
            {
                break;
            }
        }

        if has_parens {
            self.consume(TokenType::RightParen, ")")?;
        }

        Ok(names)
    }

    fn parse_global(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let names = self.parse_name_list()?;

        self.consume_newline()?;

        Ok(Stmt::Global {
            names,
            line,
            column,
        })
    }

    fn parse_nonlocal(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let names = self.parse_name_list()?;

        self.consume_newline()?;

        Ok(Stmt::Nonlocal {
            names,
            line,
            column,
        })
    }

    fn parse_name_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut names = Vec::new();

        loop {
            names.push(self.consume_identifier("name")?);

            if !self.match_token(TokenType::Comma) {
                break;
            }

            if self.check_newline() || self.check(TokenType::EOF) {
                break;
            }
        }

        Ok(names)
    }

    fn parse_pass(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        self.consume_newline()?;

        Ok(Stmt::Pass { line, column })
    }

    fn parse_break(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        if !self.is_in_context(ParserContext::Loop) {
            return Err(ParseError::InvalidSyntax {
                message: "'break' outside loop".to_string(),
                line,
                column,
            });
        }

        self.advance();

        self.consume_newline()?;

        Ok(Stmt::Break { line, column })
    }

    fn parse_continue(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        if !self.is_in_context(ParserContext::Loop) {
            return Err(ParseError::InvalidSyntax {
                message: "'continue' outside loop".to_string(),
                line,
                column,
            });
        }

        self.advance();

        self.consume_newline()?;

        Ok(Stmt::Continue { line, column })
    }

    fn parse_match(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        self.advance();

        let subject = Box::new(self.parse_expression()?);

        self.consume(TokenType::Colon, ":")?;

        let mut cases = Vec::new();

        self.consume_newline()?;

        if !self.match_token(TokenType::Indent) {
            return Err(ParseError::InvalidSyntax {
                message: "Expected indented block after 'match' statement".to_string(),
                line,
                column,
            });
        }

        while self.match_token(TokenType::Case) {
            let (pattern, guard, body) = self.with_context(ParserContext::Match, |parser| {
                parser.parse_match_case()
            })?;
            cases.push((pattern, guard, body));
        }

        self.consume(TokenType::Dedent, "expected dedent after case block")?;

        Ok(Stmt::Match {
            subject,
            cases,
            line,
            column,
        })
    }

    fn parse_match_case(&mut self) -> Result<(Box<Expr>, Option<Box<Expr>>, Vec<Box<Stmt>>), ParseError> {
        let pattern = Box::new(self.parse_expression()?);

        let guard = if self.match_token(TokenType::If) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.consume(TokenType::Colon, ":")?;

        let body = self.with_context(ParserContext::Function, |parser| {
            parser.parse_suite()
        })?;

        Ok((pattern, guard, body))
    }

    fn parse_expr_statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(TokenType::Multiply) {
            let star_token = self.previous_token();
            let star_line = star_token.line;
            let star_column = star_token.column;

            let name = self.consume_identifier("identifier after *")?;

            let starred_expr = Expr::Starred {
                value: Box::new(Expr::Name {
                    id: name,
                    ctx: ExprContext::Store,
                    line: star_line,
                    column: star_column + 1,
                }),
                ctx: ExprContext::Store,
                line: star_line,
                column: star_column,
            };

            if self.match_token(TokenType::Comma) {
                let tuple_expr = Expr::Tuple {
                    elts: vec![Box::new(starred_expr)],
                    ctx: ExprContext::Store,
                    line: star_line,
                    column: star_column,
                };

                self.consume(TokenType::Assign, "=")?;
                let value = Box::new(self.parse_expression()?);
                self.consume_newline()?;

                return Ok(Stmt::Assign {
                    targets: vec![Box::new(tuple_expr)],
                    value,
                    line: star_line,
                    column: star_column,
                });
            }

            self.consume(TokenType::Assign, "=")?;
            let value = Box::new(self.parse_expression()?);
            self.consume_newline()?;

            return Ok(Stmt::Assign {
                targets: vec![Box::new(starred_expr)],
                value,
                line: star_line,
                column: star_column,
            });
        }

        if self.check_identifier() && self.peek_matches(TokenType::Comma) {
            let line = self.current.as_ref().unwrap().line;
            let column = self.current.as_ref().unwrap().column;

            let ident = self.consume_identifier("identifier")?;
            let mut elts = vec![Box::new(Expr::Name {
                id: ident,
                ctx: ExprContext::Store,
                line,
                column,
            })];

            self.advance();

            while !self.check(TokenType::Assign)
                && !self.check_newline()
                && !self.check(TokenType::EOF)
            {
                if self.check(TokenType::Comma) {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected expression after comma".to_string(),
                        line: self.current.as_ref().map_or(line, |t| t.line),
                        column: self.current.as_ref().map_or(column, |t| t.column),
                    });
                }

                if self.match_token(TokenType::Multiply) {
                    if self.check_identifier() {
                        let star_line = self.current.as_ref().unwrap().line;
                        let star_column = self.current.as_ref().unwrap().column;
                        let star_name = self.consume_identifier("identifier after *")?;

                        elts.push(Box::new(Expr::Starred {
                            value: Box::new(Expr::Name {
                                id: star_name,
                                ctx: ExprContext::Store,
                                line: star_line,
                                column: star_column,
                            }),
                            ctx: ExprContext::Store,
                            line: star_line,
                            column: star_column - 1,
                        }));
                    } else {
                        return Err(ParseError::InvalidSyntax {
                            message: "Expected identifier after *".to_string(),
                            line: self.current.as_ref().map_or(line, |t| t.line),
                            column: self.current.as_ref().map_or(column, |t| t.column),
                        });
                    }
                } else if self.check_identifier() {
                    let item_line = self.current.as_ref().unwrap().line;
                    let item_column = self.current.as_ref().unwrap().column;
                    let item_ident = self.consume_identifier("identifier")?;

                    elts.push(Box::new(Expr::Name {
                        id: item_ident,
                        ctx: ExprContext::Store,
                        line: item_line,
                        column: item_column,
                    }));
                } else {
                    elts.push(Box::new(self.parse_atom_expr()?));
                }

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }

            let tuple_expr = Expr::Tuple {
                elts,
                ctx: ExprContext::Store,
                line,
                column,
            };

            self.consume(TokenType::Assign, "=")?;

            // Parse the right-hand side and handle chained assignments
            let mut targets = vec![Box::new(tuple_expr)];
            let mut current_expr = self.parse_expression()?;

            // Process chained assignments
            while self.match_token(TokenType::Assign) {
                self.validate_assignment_target(&current_expr)?;
                targets.push(Box::new(current_expr));
                current_expr = self.parse_expression()?;
            }

            self.consume_newline()?;

            return Ok(Stmt::Assign {
                targets,
                value: Box::new(current_expr),
                line,
                column,
            });
        }

        let expr = self.parse_expression()?;
        let line = expr.get_line();
        let column = expr.get_column();

        if self.match_token(TokenType::Assign) {
            self.validate_assignment_target(&expr)?;

            let mut targets = vec![Box::new(expr)];

            // Parse the right-hand side
            let mut current_expr = self.parse_expression()?;

            // Keep collecting targets until we stop seeing '=' signs
            while self.match_token(TokenType::Assign) {
                self.validate_assignment_target(&current_expr)?;
                targets.push(Box::new(current_expr));
                current_expr = self.parse_expression()?;
            }

            self.consume_newline()?;

            return Ok(Stmt::Assign {
                targets,
                value: Box::new(current_expr),
                line,
                column,
            });
        } else if self.is_augmented_assign() {
            match &expr {
                Expr::Name { .. } | Expr::Attribute { .. } | Expr::Subscript { .. } => {}
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: "Invalid augmented assignment target".to_string(),
                        line,
                        column,
                    });
                }
            }

            let op = self.parse_augmented_assign_op();
            self.advance();

            let value = Box::new(self.parse_expression()?);
            self.consume_newline()?;

            return Ok(Stmt::AugAssign {
                target: Box::new(expr),
                op,
                value,
                line,
                column,
            });
        } else if self.match_token(TokenType::Colon) {
            match &expr {
                Expr::Name { .. } | Expr::Attribute { .. } | Expr::Subscript { .. } => {}
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: "Invalid annotated assignment target".to_string(),
                        line,
                        column,
                    });
                }
            }

            let annotation = Box::new(self.parse_type_annotation(false)?);

            let value = if self.match_token(TokenType::Assign) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            self.consume_newline()?;

            return Ok(Stmt::AnnAssign {
                target: Box::new(expr),
                annotation,
                value,
                line,
                column,
            });
        } else {
            self.consume_newline()?;

            return Ok(Stmt::Expr {
                value: Box::new(expr),
                line,
                column,
            });
        }
    }

    fn validate_assignment_target(&self, expr: &Expr) -> Result<(), ParseError> {
        match expr {
            Expr::Name { .. } | Expr::Attribute { .. } | Expr::Subscript { .. } => Ok(()),
            Expr::List { elts, .. } | Expr::Tuple { elts, .. } => {
                for elt in elts {
                    self.validate_assignment_target(elt)?;
                }
                Ok(())
            }
            Expr::Starred { value, .. } => self.validate_assignment_target(value),
            Expr::Num { line, column, .. }
            | Expr::Str { line, column, .. }
            | Expr::Bytes { line, column, .. }
            | Expr::NameConstant { line, column, .. } => Err(ParseError::InvalidSyntax {
                message: "Cannot assign to literal".to_string(),
                line: *line,
                column: *column,
            }),
            Expr::BoolOp { line, column, .. }
            | Expr::BinOp { line, column, .. }
            | Expr::UnaryOp { line, column, .. } => Err(ParseError::InvalidSyntax {
                message: "Cannot assign to expression".to_string(),
                line: *line,
                column: *column,
            }),
            Expr::Call { line, column, .. } => Err(ParseError::InvalidSyntax {
                message: "Cannot assign to function call".to_string(),
                line: *line,
                column: *column,
            }),
            _ => Err(ParseError::InvalidSyntax {
                message: "Invalid assignment target".to_string(),
                line: expr.get_line(),
                column: expr.get_column(),
            }),
        }
    }

    fn is_augmented_assign(&self) -> bool {
        match &self.current {
            Some(token) => matches!(
                token.token_type,
                TokenType::PlusAssign
                    | TokenType::MinusAssign
                    | TokenType::MulAssign
                    | TokenType::DivAssign
                    | TokenType::ModAssign
                    | TokenType::PowAssign
                    | TokenType::FloorDivAssign
                    | TokenType::MatrixMulAssign
                    | TokenType::BitwiseAndAssign
                    | TokenType::BitwiseOrAssign
                    | TokenType::BitwiseXorAssign
                    | TokenType::ShiftLeftAssign
                    | TokenType::ShiftRightAssign
            ),
            None => false,
        }
    }

    fn parse_augmented_assign_op(&self) -> Operator {
        match &self.current {
            Some(token) => match token.token_type {
                TokenType::PlusAssign => Operator::Add,
                TokenType::MinusAssign => Operator::Sub,
                TokenType::MulAssign => Operator::Mult,
                TokenType::DivAssign => Operator::Div,
                TokenType::ModAssign => Operator::Mod,
                TokenType::PowAssign => Operator::Pow,
                TokenType::FloorDivAssign => Operator::FloorDiv,
                TokenType::MatrixMulAssign => Operator::MatMult,
                TokenType::BitwiseAndAssign => Operator::BitAnd,
                TokenType::BitwiseOrAssign => Operator::BitOr,
                TokenType::BitwiseXorAssign => Operator::BitXor,
                TokenType::ShiftLeftAssign => Operator::LShift,
                TokenType::ShiftRightAssign => Operator::RShift,
                _ => panic!("Not an augmented assign operator"),
            },
            None => panic!("Unexpected EOF"),
        }
    }

    fn parse_suite(&mut self) -> Result<Vec<Box<Stmt>>, ParseError> {
        if self.match_token(TokenType::Newline) {
            if self.check(TokenType::Indent) {
                self.advance();

                let indent_level = self.current_indent_level;
                let mut statements = Vec::new();

                while !self.check(TokenType::Dedent) && !self.check(TokenType::EOF) {
                    if self.match_token(TokenType::Newline) {
                        continue;
                    }

                    if self.current_indent_level != indent_level {
                        let current_token = self
                            .current
                            .as_ref()
                            .unwrap_or_else(|| panic!("Expected token at this position"));

                        return Err(ParseError::InvalidSyntax {
                            message: format!(
                                "Inconsistent indentation: expected level {} but got {}",
                                indent_level, self.current_indent_level
                            ),
                            line: current_token.line,
                            column: current_token.column,
                        });
                    }

                    let stmt = self.parse_statement()?;
                    statements.push(Box::new(stmt));

                    if self.current.is_none() || self.check(TokenType::Dedent) {
                        break;
                    }
                }

                if !self.check(TokenType::Dedent) && !self.check(TokenType::EOF) {
                    let current_token = self
                        .current
                        .as_ref()
                        .unwrap_or_else(|| panic!("Expected token at this position"));

                    return Err(ParseError::InvalidSyntax {
                        message: "Expected dedent at end of block".to_string(),
                        line: current_token.line,
                        column: current_token.column,
                    });
                }

                if !self.check(TokenType::EOF) {
                    self.consume(TokenType::Dedent, "expected dedent at end of block")?;
                }

                Ok(statements)
            } else {
                let current_token = self
                    .current
                    .as_ref()
                    .unwrap_or_else(|| panic!("Expected token at this position"));

                Err(ParseError::InvalidSyntax {
                    message: "Expected an indented block".to_string(),
                    line: current_token.line,
                    column: current_token.column,
                })
            }
        } else {
            let stmt = Box::new(self.parse_statement()?);
            Ok(vec![stmt])
        }
    }

    fn with_store_context(&self, expr: Expr) -> Result<Expr, ParseError> {
        match expr {
            Expr::Name {
                id,
                ctx: _,
                line,
                column,
            } => Ok(Expr::Name {
                id,
                ctx: ExprContext::Store,
                line,
                column,
            }),
            Expr::Tuple {
                elts,
                ctx: _,
                line,
                column,
            } => {
                let mut new_elts = Vec::new();
                for elt in elts {
                    new_elts.push(Box::new(self.with_store_context(*elt)?));
                }
                Ok(Expr::Tuple {
                    elts: new_elts,
                    ctx: ExprContext::Store,
                    line,
                    column,
                })
            }
            Expr::List {
                elts,
                ctx: _,
                line,
                column,
            } => {
                let mut new_elts = Vec::new();
                for elt in elts {
                    new_elts.push(Box::new(self.with_store_context(*elt)?));
                }
                Ok(Expr::List {
                    elts: new_elts,
                    ctx: ExprContext::Store,
                    line,
                    column,
                })
            }
            Expr::Starred {
                value,
                ctx: _,
                line,
                column,
            } => Ok(Expr::Starred {
                value: Box::new(self.with_store_context(*value)?),
                ctx: ExprContext::Store,
                line,
                column,
            }),
            Expr::Subscript {
                value,
                slice,
                ctx: _,
                line,
                column,
            } => Ok(Expr::Subscript {
                value,
                slice,
                ctx: ExprContext::Store,
                line,
                column,
            }),
            Expr::Attribute {
                value,
                attr,
                ctx: _,
                line,
                column,
            } => Ok(Expr::Attribute {
                value,
                attr,
                ctx: ExprContext::Store,
                line,
                column,
            }),
            _ => Err(ParseError::InvalidSyntax {
                message: "Invalid target for assignment".to_string(),
                line: expr.get_line(),
                column: expr.get_column(),
            }),
        }
    }
}