use crate::ast::{
    Alias, BoolOperator, CmpOperator, Comprehension, ExceptHandler, Expr, ExprContext, Module,
    NameConstant, Number, Operator, Parameter, Stmt, UnaryOperator
};
use crate::lexer::{Token, TokenType};

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: TokenType,
        line: usize,
        column: usize,
    },
    InvalidSyntax {
        message: String,
        line: usize,
        column: usize,
    },
    EOF {
        expected: String,
        line: usize,
        column: usize,
    },
}

pub struct Parser {
    tokens: VecDeque<Token>,
    current: Option<Token>,
    last_token: Option<Token>,
    errors: Vec<ParseError>,
    current_indent_level: usize,
    is_in_comprehension_context: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut tokens_deque = VecDeque::from(tokens);
        let current = tokens_deque.pop_front();

        Parser {
            tokens: tokens_deque,
            current,
            last_token: None,
            errors: Vec::new(),
            current_indent_level: 0,
            is_in_comprehension_context: false,
        }
    }

    pub fn parse(&mut self) -> Result<Module, Vec<ParseError>> {
        let mut body = Vec::new();
    
        while let Some(token) = &self.current {
            if matches!(token.token_type, TokenType::EOF) {
                break;
            }
    
            // Skip newlines before statements
            while matches!(
                self.current.as_ref().map(|t| &t.token_type),
                Some(&TokenType::Newline)
            ) {
                self.advance();
            }
    
            // Check if we've reached EOF after skipping newlines
            if self.current.is_none() {
                break;
            }
    
            match self.parse_statement() {
                Ok(stmt) => body.push(Box::new(stmt)),
                Err(e) => {
                    self.errors.push(e.clone());
                    // Return immediately on first error instead of trying to synchronize
                    return Err(vec![e]);
                }
            }
        }
    
        if self.errors.is_empty() {
            Ok(Module { body })
        } else {
            Err(self.errors.clone())
        }
    }

    fn synchronize(&mut self) {
        self.advance(); // Skip the current problematic token
    
        while let Some(token) = &self.current {
            // Stop at statement boundaries
            if matches!(token.token_type, 
                TokenType::SemiColon | 
                TokenType::Newline | 
                TokenType::RightBrace | 
                TokenType::Dedent | 
                TokenType::EOF) {
                return;
            }
            self.advance();
        }
    }

    fn check_identifier(&self) -> bool {
        matches!(self.current.as_ref().map(|t| &t.token_type), Some(TokenType::Identifier(_)))
    }

    fn with_comprehension_context<F, T>(&mut self, f: F) -> T
        where
            F: FnOnce(&mut Self) -> T,
        {
            // Save current state
            let old_context = self.is_in_comprehension_context;
            
            // Set comprehension context
            self.is_in_comprehension_context = true;
            
            // Execute the provided function
            let result = f(self);
            
            // Restore previous context
            self.is_in_comprehension_context = old_context;
            
            result
        }

    fn parse_decorators(&mut self) -> Result<Vec<Box<Expr>>, ParseError> {
        let mut decorators = Vec::new();

        while self.match_token(TokenType::At) {
            // Parse decorator expression
            let decorator = Box::new(self.parse_expression()?);
            decorators.push(decorator);

            // Each decorator should be followed by a newline
            self.consume_newline()?;
        }

        Ok(decorators)
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        // Clone token and line/column info before any mutable operations
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

        if matches!(token_type, TokenType::Plus | TokenType::Minus | TokenType::Multiply | 
            TokenType::Divide | TokenType::Modulo | TokenType::Power) {
            // If we're at the end of input or have a newline right after an operator, that's an error
            if self.peek_matches(TokenType::EOF) || self.peek_matches(TokenType::Newline) {
            return Err(ParseError::InvalidSyntax {
                message: "Incomplete expression".to_string(),
                line,
                column,
            });
            }
            }

            // Check for literal assignments (which are invalid in Python)
            if matches!(token_type, TokenType::IntLiteral(_) | TokenType::FloatLiteral(_) | 
                TokenType::StringLiteral(_) | TokenType::True | TokenType::False | TokenType::None) {
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
    
        // Check for yield statements
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
    
        // First, check for decorators
        if matches!(token_type, TokenType::At) {
            // Parse decorators for function or class
            let decorators = self.parse_decorators()?;
    
            // Next, we should have either a function or class definition
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
                        message: "Expected function or class definition after decorators"
                            .to_string(),
                        line,
                        column,
                    });
                }
            }
        }
    
        // Check for async keyword
        if matches!(token_type, TokenType::Async) {
            self.advance(); // Consume 'async'
    
            // Get the next token type after advancing
            let async_next_token_type = self.current.as_ref().map(|t| t.token_type.clone());
            
            // Next token should be a function definition or a with/for statement
            match async_next_token_type {
                Some(TokenType::Def) => {
                    // Parse as an async function
                    let mut func_def = self.parse_function_def()?;
    
                    // Set the async flag
                    if let Stmt::FunctionDef {
                        ref mut is_async, ..
                    } = func_def
                    {
                        *is_async = true;
                    }
    
                    return Ok(func_def);
                }
                Some(TokenType::For) => {
                    // Parse as an async for loop
                    let mut for_stmt = self.parse_for()?;
    
                    // Set the async flag
                    if let Stmt::For {
                        ref mut is_async, ..
                    } = for_stmt
                    {
                        *is_async = true;
                    }
    
                    return Ok(for_stmt);
                }
                Some(TokenType::With) => {
                    // Parse as an async with statement
                    let mut with_stmt = self.parse_with()?;
    
                    // Set the async flag
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
    
        // Original statement parsing for non-decorated, non-async statements
        match token_type {
            TokenType::Def => self.parse_function_def(),
            TokenType::Class => self.parse_class_def(),
            TokenType::Return => self.parse_return(),
            TokenType::Del => self.parse_delete(),
            TokenType::If => {
                // Check if there's a missing colon (for test_parse_error_cases)
                self.advance(); // Consume 'if'
                let test = Box::new(self.parse_expression()?);
                
                // Explicitly check for colon
                if !self.check(TokenType::Colon) {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected ':' after if condition".to_string(),
                        line: self.current.as_ref().map_or(line, |t| t.line),
                        column: self.current.as_ref().map_or(column + 2, |t| t.column),
                    });
                }
                
                // Continue normal parsing
                self.consume(TokenType::Colon, ":")?;
                let body = self.parse_suite()?;
                
                // Parse elif/else clauses
                let mut orelse = Vec::new();
                
                if self.check(TokenType::Elif) {
                    // Handle elif
                    let elif_stmt = self.parse_if()?;
                    orelse.push(Box::new(elif_stmt));
                } else if self.match_token(TokenType::Else) {
                    // Handle else
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
            },
            TokenType::For => self.parse_for(),
            TokenType::While => self.parse_while(),
            TokenType::With => self.parse_with(),
            TokenType::Try => self.parse_try(),
            TokenType::Raise => self.parse_raise(),
            TokenType::Assert => self.parse_assert(),
            TokenType::Import => self.parse_import(),
            TokenType::From => self.parse_import_from(),
            TokenType::Global => self.parse_global(),
            TokenType::Nonlocal => self.parse_nonlocal(),
            TokenType::Pass => self.parse_pass(),
            TokenType::Break => self.parse_break(),
            TokenType::Continue => self.parse_continue(),
            TokenType::Yield => {
                let expr = self.parse_yield_expr()?;
                let line = expr.get_line();
                let column = expr.get_column();
            
                self.consume_newline()?;
            
                Ok(Stmt::Expr {
                    value: Box::new(expr),
                    line,
                    column,
                })
            }
            _ => {
                // Add a special case for checking literal assignments
                if matches!(token_type, TokenType::IntLiteral(_) | TokenType::FloatLiteral(_)
                    | TokenType::StringLiteral(_) | TokenType::True | TokenType::False | TokenType::None) {
                    let expr = self.parse_expression()?;
                    let line = expr.get_line();
                    let column = expr.get_column();
                    
                    if self.match_token(TokenType::Assign) {
                        return Err(ParseError::InvalidSyntax {
                            message: "Cannot assign to literal".to_string(),
                            line,
                            column,
                        });
                    }
                    
                    self.consume_newline()?;
                    return Ok(Stmt::Expr {
                        value: Box::new(expr),
                        line,
                        column,
                    });
                }
                
                self.parse_expr_statement()
            }
        }
    }

    fn parse_function_def(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        // Consume 'def'
        self.advance();

        // Parse function name
        let name = self.consume_identifier("function name")?;

        // Parse parameters
        self.consume(TokenType::LeftParen, "(")?;
        let params = self.parse_parameters()?;
        self.consume(TokenType::RightParen, ")")?;

        // Parse optional return type annotation
        let returns = if self.match_token(TokenType::Arrow) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        // Parse function body
        self.consume(TokenType::Colon, ":")?;
        let body = self.parse_suite()?;

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
        
        // No parameters case
        if self.check(TokenType::RightParen) || self.check(TokenType::Colon) {
            return Ok(params);
        }
        
        // Parse parameters one by one
        loop {
            // Special handling for function with missing comma between parameters
            if self.check_identifier() {
                let prev_token_pos = (
                    self.current.as_ref().map_or(0, |t| t.line),
                    self.current.as_ref().map_or(0, |t| t.column)
                );
                
                // Regular parameter
                let param_name = self.consume_identifier("parameter name")?;
                
                // If we immediately see another identifier without a comma, that's an error!
                if self.check_identifier() {
                    // Make sure this error is propagated all the way up
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected comma between parameters".to_string(),
                        line: prev_token_pos.0,
                        column: prev_token_pos.1 + param_name.len(),
                    });
                }
                
                // Normal parameter processing...
                let typ = if self.match_token(TokenType::Colon) {
                    Some(Box::new(self.parse_type_annotation(false)?))
                } else {
                    None
                };
                
                let default = if self.match_token(TokenType::Assign) {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };
                
                params.push(Parameter { 
                    name: param_name, 
                    typ, 
                    default,
                    is_vararg: false,
                    is_kwarg: false,
                });
            }
            // Handle *args parameter
            else if self.match_token(TokenType::Multiply) {
                let name = self.consume_identifier("parameter name after *")?;
                
                // Parse optional type annotation
                let typ = if self.match_token(TokenType::Colon) {
                    Some(Box::new(self.parse_type_annotation(false)?))
                } else {
                    None
                };
                
                // Default value is not allowed for *args in Python, so raise an error if we see =
                if self.check(TokenType::Assign) {
                    return Err(ParseError::InvalidSyntax {
                        message: "Variadic argument cannot have default value".to_string(),
                        line: self.current.as_ref().map_or(0, |t| t.line),
                        column: self.current.as_ref().map_or(0, |t| t.column),
                    });
                }
                
                params.push(Parameter { 
                    name, 
                    typ, 
                    default: None,
                    is_vararg: true,
                    is_kwarg: false,
                });
            }
            // Handle **kwargs parameter
            else if self.match_token(TokenType::Power) {
                let name = self.consume_identifier("parameter name after **")?;
                
                // Parse optional type annotation
                let typ = if self.match_token(TokenType::Colon) {
                    Some(Box::new(self.parse_type_annotation(false)?))
                } else {
                    None
                };
                
                // Default value is not allowed for **kwargs in Python, so raise an error if we see =
                if self.check(TokenType::Assign) {
                    return Err(ParseError::InvalidSyntax {
                        message: "Keyword argument cannot have default value".to_string(),
                        line: self.current.as_ref().map_or(0, |t| t.line),
                        column: self.current.as_ref().map_or(0, |t| t.column),
                    });
                }
                
                params.push(Parameter { 
                    name, 
                    typ, 
                    default: None,
                    is_vararg: false,
                    is_kwarg: true,
                });
            }
            else {
                // Unexpected token at start of parameter
                let token = self.current.clone().unwrap();
                return Err(ParseError::UnexpectedToken {
                    expected: "parameter name or * or **".to_string(),
                    found: token.token_type,
                    line: token.line,
                    column: token.column,
                });
            }
            
            // Check for comma to continue or break
            if self.match_token(TokenType::Comma) {
                // After comma, we should have another parameter or a closing token
                if self.check(TokenType::RightParen) || self.check(TokenType::Colon) {
                    // Trailing comma is fine
                    break;
                }
                
                // If we see a comma followed by another comma, that's an error!
                if self.check(TokenType::Comma) {
                    let token = self.current.clone().unwrap();
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected parameter after comma".to_string(),
                        line: token.line,
                        column: token.column,
                    });
                }
                
                // Continue to parse the next parameter
                continue;
            } else {
                // No comma means we're at the end of the parameter list
                if !self.check(TokenType::RightParen) && !self.check(TokenType::Colon) {
                    let token = self.current.clone().unwrap();
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected comma or closing parenthesis/colon".to_string(),
                        line: token.line,
                        column: token.column,
                    });
                }
                break;
            }
        }
        
        Ok(params)
    }

    fn parse_class_def(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;
    
        // Consume 'class'
        self.advance();
    
        // Parse class name
        let name = self.consume_identifier("class name")?;
    
        // Parse optional bases
        let (bases, keywords_with_optional_names) = if self.match_token(TokenType::LeftParen) {
            // Check specifically for the case of just a comma inside the parentheses
            if self.check(TokenType::Comma) {
                let comma_token = self.current.clone().unwrap();
                return Err(ParseError::InvalidSyntax {
                    message: "Expected expression before comma".to_string(),
                    line: comma_token.line,
                    column: comma_token.column,
                });
            }
            
            let (b, k) = self.parse_arguments()?;
            self.consume(TokenType::RightParen, ")")?;
            (b, k)
        } else {
            (Vec::new(), Vec::new())
        };
    
        // Fix type mismatch by filtering out None keys
        let keywords: Vec<(String, Box<Expr>)> = keywords_with_optional_names
            .into_iter()
            .filter_map(|(k, v)| k.map(|key| (key, v)))
            .collect();
    
        // Parse class body
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

        // Consume 'return'
        self.advance();

        // Parse optional return value
        let value = if self.check_newline() || self.check(TokenType::EOF) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        // Consume newline
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

        // Consume 'del'
        self.advance();

        // Parse targets
        let targets = self.parse_expr_list()?;

        // Consume newline
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
    
        // Consume 'if'
        self.advance();
        
        // Check for missing condition
        if self.check(TokenType::Colon) {
            return Err(ParseError::InvalidSyntax {
                message: "Expected condition after 'if'".to_string(),
                line,
                column,
            });
        }
    
        // Parse condition
        let test = Box::new(self.parse_expression()?);
    
        // Strictly require a colon - generate error if not found
        if !self.check(TokenType::Colon) {
            return Err(ParseError::InvalidSyntax {
                message: "Expected ':' after if condition".to_string(),
                line: self.current.as_ref().map_or(line, |t| t.line),
                column: self.current.as_ref().map_or(column + 2, |t| t.column),
            });
        }
    
        // Now consume the colon
        self.advance(); // This replaces self.consume(TokenType::Colon, ":")
    
        // Parse the body
        let body = self.parse_suite()?;
    
        // Parse elif/else clauses
        let mut orelse = Vec::new();
    
        if self.check(TokenType::Elif) {
            // Handle elif
            let elif_stmt = self.parse_if()?;
            orelse.push(Box::new(elif_stmt));
        } else if self.match_token(TokenType::Else) {
            // Handle else
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
    
        // Consume 'for'
        self.advance();
        
        // Check for missing target
        if self.check(TokenType::In) {
            return Err(ParseError::InvalidSyntax {
                message: "Expected target after 'for'".to_string(),
                line,
                column,
            });
        }
    
        // Parse target - use parse_atom_expr to prevent "in" from being treated as a comparison operator
        let target = Box::new(self.parse_atom_expr()?);
    
        // Consume 'in'
        self.consume(TokenType::In, "in")?;
    
        // Parse iterable
        let iter = Box::new(self.parse_expression()?);
    
        // Parse body
        self.consume(TokenType::Colon, ":")?;
    
        // Handle both indented blocks and single-line suites
        let body = self.parse_suite()?;
    
        // Parse optional else clause
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

        // Consume 'while'
        self.advance();

        // Parse condition
        let test = Box::new(self.parse_expression()?);

        // Parse body
        self.consume(TokenType::Colon, ":")?;
        let body = self.parse_suite()?;

        // Parse optional else clause
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

        // Consume 'with'
        self.advance();

        // Parse with items
        let items = self.parse_with_items()?;

        // Parse body
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

    fn parse_with_items(&mut self) -> Result<Vec<(Box<Expr>, Option<Box<Expr>>)>, ParseError> {
        let mut items = Vec::new();

        loop {
            let context_expr = Box::new(self.parse_expression()?);

            let optional_vars = if self.match_token(TokenType::As) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            items.push((context_expr, optional_vars));

            if !self.match_token(TokenType::Comma) {
                break;
            }

            // Handle trailing comma
            if self.check(TokenType::Colon) {
                break;
            }
        }

        Ok(items)
    }

    fn parse_try(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        // Consume 'try'
        self.advance();

        // Parse try body
        self.consume(TokenType::Colon, ":")?;
        let body = self.parse_suite()?;

        // Parse except handlers
        let mut handlers = Vec::new();

        while self.match_token(TokenType::Except) {
            let h_line = self.previous_token().line;
            let h_column = self.previous_token().column;

            let typ = if !self.check(TokenType::Colon) {
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

        // Parse optional else clause
        let orelse = if self.match_token(TokenType::Else) {
            self.consume(TokenType::Colon, ":")?;
            self.parse_suite()?
        } else {
            Vec::new()
        };

        // Parse optional finally clause
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

        // Consume 'raise'
        self.advance();

        // Parse optional exception
        let exc = if self.check_newline() || self.check(TokenType::EOF) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        // Parse optional cause
        let cause = if self.match_token(TokenType::From) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        // Consume newline
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

        // Consume 'assert'
        self.advance();

        // Parse test expression
        let test = Box::new(self.parse_expression()?);

        // Parse optional message
        let msg = if self.match_token(TokenType::Comma) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        // Consume newline
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
    
        // Consume 'import'
        self.advance();
        
        // Check for empty import statement
        if self.check_newline() || self.check(TokenType::EOF) {
            return Err(ParseError::InvalidSyntax {
                message: "Expected module name after 'import'".to_string(),
                line,
                column: column + 6, // Position after "import"
            });
        }
    
        // Parse import names
        let names = self.parse_import_names()?;
    
        // Consume newline
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

            // Handle trailing comma
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

        // Consume 'from'
        self.advance();

        // Count leading dots for relative imports
        let mut level = 0;
        while self.match_token(TokenType::Dot) {
            level += 1;
        }

        // Parse optional module name
        let module = if self.check(TokenType::Import) {
            None
        } else {
            Some(self.consume_dotted_name("module name")?)
        };

        // Consume 'import'
        self.consume(TokenType::Import, "import")?;

        // Parse import names or star
        let names = if self.match_token(TokenType::Multiply) {
            vec![Alias {
                name: "*".to_string(),
                asname: None,
            }]
        } else {
            self.parse_import_as_names()?
        };

        // Consume newline
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

        // Check if we have parenthesized import names
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

            // Handle trailing comma
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

        // Consume 'global'
        self.advance();

        // Parse name list
        let names = self.parse_name_list()?;

        // Consume newline
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

        // Consume 'nonlocal'
        self.advance();

        // Parse name list
        let names = self.parse_name_list()?;

        // Consume newline
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

            // Handle trailing comma
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

        // Consume 'pass'
        self.advance();

        // Consume newline
        self.consume_newline()?;

        Ok(Stmt::Pass { line, column })
    }

    fn parse_break(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        // Consume 'break'
        self.advance();

        // Consume newline
        self.consume_newline()?;

        Ok(Stmt::Break { line, column })
    }

    fn parse_continue(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;

        // Consume 'continue'
        self.advance();

        // Consume newline
        self.consume_newline()?;

        Ok(Stmt::Continue { line, column })
    }

    fn parse_expr_statement(&mut self) -> Result<Stmt, ParseError> {
        // Support for a, *b, c = ... pattern
        if matches!(
            self.current.as_ref().map(|t| &t.token_type),
            Some(TokenType::Identifier(_))
        ) && self.peek_matches(TokenType::Comma) {
            let expr = self.parse_expression()?;
            let line = expr.get_line();
            let column = expr.get_column();
            
            // We're handling a tuple unpacking like "a, *b, c = ..."
            if self.match_token(TokenType::Comma) {
                let mut elts = vec![Box::new(expr)];
                
                // Check for starred expression after the comma
                while !self.check(TokenType::Assign) && !self.check_newline() && !self.check(TokenType::EOF) {
                    // Prevent consecutive commas
                    if self.check(TokenType::Comma) {
                        return Err(ParseError::InvalidSyntax {
                            message: "Expected expression after comma".to_string(),
                            line: self.current.as_ref().map_or(line, |t| t.line),
                            column: self.current.as_ref().map_or(column, |t| t.column),
                        });
                    }
                    
                    elts.push(Box::new(self.parse_expression()?));
                    
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
                
                // Expect assignment
                self.consume(TokenType::Assign, "=")?;
                let value = Box::new(self.parse_expression()?);
                self.consume_newline()?;
                
                return Ok(Stmt::Assign {
                    targets: vec![Box::new(tuple_expr)],
                    value,
                    line,
                    column,
                });
            }
        }
        
        // Original handling for standalone expressions
        let mut expr = self.parse_expression()?;
        let line = expr.get_line();
        let column = expr.get_column();
        
        // Handle cases starting with starred expression (*b, c = ...)
        if self.check(TokenType::Multiply) {
            let star_token = self.current.clone().unwrap();
            self.advance(); // Consume *
            
            // Parse the variable name after the star
            let var_expr = self.parse_atom_expr()?;
            let starred_expr = Expr::Starred {
                value: Box::new(var_expr),
                ctx: ExprContext::Store,
                line: star_token.line,
                column: star_token.column,
            };
            
            // Now handle as part of a tuple
            let mut elts = vec![Box::new(starred_expr)];
            
            // Must have a comma after *b in a, *b, c = ...
            if !self.match_token(TokenType::Comma) {
                return Err(ParseError::InvalidSyntax {
                    message: "Expected comma after starred expression in tuple unpacking".to_string(),
                    line: self.current.as_ref().map_or(0, |t| t.line),
                    column: self.current.as_ref().map_or(0, |t| t.column),
                });
            }
            
            // Parse remaining elements
            while !self.check(TokenType::Assign) && !self.check_newline() && !self.check(TokenType::EOF) {
                elts.push(Box::new(self.parse_expression()?));
                
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
            
            // Create tuple and process as assignment
            let tuple_expr = Expr::Tuple {
                elts,
                ctx: ExprContext::Store,
                line: star_token.line,
                column: star_token.column,
            };
            
            // Must be an assignment
            if !self.match_token(TokenType::Assign) {
                return Err(ParseError::InvalidSyntax {
                    message: "Expected '=' in tuple unpacking".to_string(),
                    line: self.current.as_ref().map_or(0, |t| t.line),
                    column: self.current.as_ref().map_or(0, |t| t.column),
                });
            }
            
            // Parse the value
            let value = Box::new(self.parse_expression()?);
            self.consume_newline()?;
            
            return Ok(Stmt::Assign {
                targets: vec![Box::new(tuple_expr)],
                value,
                line: star_token.line,
                column: star_token.column,
            });
        }
        
        // Handle tuple unpacking (e.g., a, b = 1, 2)
        if self.check(TokenType::Comma) {
            let mut elts = vec![Box::new(expr)];
            while self.match_token(TokenType::Comma) {
                if self.check(TokenType::Assign) || self.check_newline() || self.check(TokenType::EOF) {
                    break;
                }
                elts.push(Box::new(self.parse_expression()?));
            }
            expr = Expr::Tuple {
                elts,
                ctx: ExprContext::Store,
                line,
                column,
            };
        }
    
        // Handle different types of statements
        if self.match_token(TokenType::Assign) {
            // Validate assignment target
            self.validate_assignment_target(&expr)?;
    
            // Handle chained assignments (e.g., a = b = c)
            let value = if self.check_identifier() || self.check(TokenType::LeftParen) ||
                            self.check(TokenType::LeftBracket) || self.check(TokenType::LeftBrace) ||
                            self.check(TokenType::Yield) {
                
                // Parse the next expression (which could be the target of another assignment)
                let target_expr = self.parse_expression()?;
                
                // If we see another equals sign, this is a chained assignment
                if self.match_token(TokenType::Assign) {
                    // Get the rest of the assignment chain recursively by creating another statement
                    let stmt = self.parse_expr_statement()?;
                    
                    // Extract the value from the nested assignment
                    match stmt {
                        Stmt::Assign { value, .. } => value,
                        _ => {
                            return Err(ParseError::InvalidSyntax {
                                message: "Expected assignment in chained assignment".to_string(),
                                line: self.current.as_ref().map_or(0, |t| t.line),
                                column: self.current.as_ref().map_or(0, |t| t.column),
                            });
                        }
                    }
                } else {
                    // Not a chained assignment, just use the expression as the value
                    Box::new(target_expr)
                }
            } else {
                // Simple assignment
                Box::new(self.parse_expression()?)
            };
    
            self.consume_newline()?;
    
            // Create assignment statement
            Ok(Stmt::Assign {
                targets: vec![Box::new(expr)],
                value,
                line,
                column,
            })
        } else if self.is_augmented_assign() {
            // Handle augmented assignment (e.g., x += 1)
            
            // Validate assignment target for augmented assignment
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
    
            // Get the operator and consume the token
            let op = self.parse_augmented_assign_op();
            self.advance(); 
            
            // Parse the value
            let value = Box::new(self.parse_expression()?);
            
            self.consume_newline()?;
    
            // Create augmented assignment statement
            Ok(Stmt::AugAssign {
                target: Box::new(expr),
                op,
                value,
                line,
                column,
            })
        } else if self.match_token(TokenType::Colon) {
            // Handle annotated assignment (e.g., a: int = 5)
            
            // Validate annotated assignment target
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
    
            // Parse type annotation
            let annotation = Box::new(self.parse_type_annotation(false)?);
    
            // Parse optional value
            let value = if self.match_token(TokenType::Assign) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };
    
            self.consume_newline()?;
    
            // Create annotated assignment statement
            Ok(Stmt::AnnAssign {
                target: Box::new(expr),
                annotation,
                value,
                line,
                column,
            })
        } else {
            // Simple expression statement
            self.consume_newline()?;
    
            // Create expression statement
            Ok(Stmt::Expr {
                value: Box::new(expr),
                line,
                column,
            })
        }
    }

    fn validate_assignment_target(&self, expr: &Expr) -> Result<(), ParseError> {
        match expr {
            Expr::Name { .. } | Expr::Attribute { .. } | Expr::Subscript { .. } => {
                // These are valid targets
                Ok(())
            }
            Expr::List { elts, .. } | Expr::Tuple { elts, .. } => {
                // Nested lists/tuples are okay, check their elements
                for elt in elts {
                    self.validate_assignment_target(elt)?;
                }
                Ok(())
            }
            Expr::Starred { value, .. } => {
                // Check if the starred value is a valid target
                self.validate_assignment_target(value)
            }
            // For literals and other expressions that cannot be targets
            Expr::Num { line, column, .. } |
            Expr::Str { line, column, .. } |
            Expr::Bytes { line, column, .. } |
            Expr::NameConstant { line, column, .. } => {
                // Explicitly reject literals as assignment targets
                Err(ParseError::InvalidSyntax {
                    message: "Cannot assign to literal".to_string(),
                    line: *line,
                    column: *column,
                })
            }
            Expr::BoolOp { line, column, .. } |
            Expr::BinOp { line, column, .. } |
            Expr::UnaryOp { line, column, .. } => {
                // Explicitly reject operations as assignment targets
                Err(ParseError::InvalidSyntax {
                    message: "Cannot assign to expression".to_string(),
                    line: *line,
                    column: *column,
                })
            }
            _ => {
                // All other expressions are invalid targets
                Err(ParseError::InvalidSyntax {
                    message: "Invalid assignment target".to_string(),
                    line: expr.get_line(),
                    column: expr.get_column(),
                })
            }
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
                self.advance(); // Consume Indent
                let mut statements = Vec::new();
                
                // Track the expected indentation level
                let indent_level = self.current_indent_level;
                
                while !self.check(TokenType::Dedent) && !self.check(TokenType::EOF) {
                    if self.match_token(TokenType::Newline) {
                        continue;
                    }
                    
                    // Verify the indentation is consistent
                    if self.current_indent_level != indent_level {
                        return Err(ParseError::InvalidSyntax {
                            message: "Inconsistent indentation".to_string(),
                            line: self.current.as_ref().map_or(0, |t| t.line),
                            column: self.current.as_ref().map_or(0, |t| t.column),
                        });
                    }
                    
                    let stmt = self.parse_statement()?;
                    statements.push(Box::new(stmt));
                    
                    // Ensure we don't parse beyond the current block
                    if self.current.is_none() || self.check(TokenType::Dedent) {
                        break;
                    }
                }
                self.consume(TokenType::Dedent, "expected dedent at end of block")?;
                Ok(statements)
            } else {
                if !self.check_newline() && !self.check(TokenType::EOF) {
                    let stmt = Box::new(self.parse_statement()?);
                    Ok(vec![stmt])
                } else {
                    Err(ParseError::InvalidSyntax {
                        message: "Expected an indented block".to_string(),
                        line: self.current.as_ref().map_or(0, |t| t.line),
                        column: self.current.as_ref().map_or(0, |t| t.column),
                    })
                }
            }
        } else {
            let stmt = Box::new(self.parse_statement()?);
            Ok(vec![stmt])
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        // First parse the value (likely an or_test)
        let mut expr = self.parse_or_test()?;
        
        // Check for conditional expression (ternary operator)
        // Only treat "if" as a ternary operator if we're not in a comprehension
        if self.check(TokenType::If) && !self.is_in_comprehension_context {
            let line = expr.get_line();
            let column = expr.get_column();
            
            // Consume the 'if' token
            self.advance();
            
            // Parse the condition
            let test = Box::new(self.parse_or_test()?);
            
            // Expect 'else'
            self.consume(TokenType::Else, "else")?;
            
            // Parse the alternative value
            let orelse = Box::new(self.parse_expression()?);
            
            // Create the conditional expression
            expr = Expr::IfExp {
                test,
                body: Box::new(expr),
                orelse,
                line,
                column,
            };
        }
        
        Ok(expr)
    }

    fn parse_or_test(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_and_test()?;
    
        // Check if we have any 'or' operators
        if self.check(TokenType::Or) {
            let line = expr.get_line();
            let column = expr.get_column();
            
            // Start with the first expression we already parsed
            let mut values = vec![Box::new(expr)];
            
            // Parse all 'or' expressions into a flat list
            while self.match_token(TokenType::Or) {
                values.push(Box::new(self.parse_and_test()?));
            }
            
            // Create a single BoolOp with all values
            expr = Expr::BoolOp {
                op: BoolOperator::Or,
                values,
                line,
                column,
            };
        }
    
        Ok(expr)
    }

    fn parse_and_test(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_not_test()?;

        // Check if we have any 'and' operators
        if self.check(TokenType::And) {
            let line = expr.get_line();
            let column = expr.get_column();
            
            // Start with the first expression we already parsed
            let mut values = vec![Box::new(expr)];
            
            // Parse all 'and' expressions into a flat list
            while self.match_token(TokenType::And) {
                values.push(Box::new(self.parse_not_test()?));
            }
            
            // Create a single BoolOp with all values
            expr = Expr::BoolOp {
                op: BoolOperator::And,
                values,
                line,
                column,
            };
        }

        Ok(expr)
    }

    fn parse_not_test(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(TokenType::Not) {
            let token = self.previous_token();
            let operand = Box::new(self.parse_not_test()?);

            Ok(Expr::UnaryOp {
                op: UnaryOperator::Not,
                operand,
                line: token.line,
                column: token.column,
            })
        } else {
            self.parse_comparison()
        }
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_expr()?;

        // Parse comparison chains like a < b < c
        let mut ops = Vec::new();
        let mut comparators = Vec::new();

        while self.is_comparison_operator() {
            let op = self.parse_comparison_operator()?; // Note the ? to handle the Result
            let right = self.parse_expr()?;

            ops.push(op);
            comparators.push(Box::new(right));
        }

        if !ops.is_empty() {
            let line = expr.get_line();
            let column = expr.get_column();

            expr = Expr::Compare {
                left: Box::new(expr),
                ops,
                comparators,
                line,
                column,
            };
        }

        Ok(expr)
    }

    fn is_comparison_operator(&self) -> bool {
        match &self.current {
            Some(token) => {
                matches!(
                    token.token_type,
                    TokenType::Equal
                        | TokenType::NotEqual
                        | TokenType::LessThan
                        | TokenType::LessEqual
                        | TokenType::GreaterThan
                        | TokenType::GreaterEqual
                        | TokenType::Is
                        | TokenType::In
                        | TokenType::Not
                )
            }
            None => false,
        }
    }

    fn parse_comparison_operator(&mut self) -> Result<CmpOperator, ParseError> {
        // Clone token data before we mutate self
        let token_type = self.current.as_ref().unwrap().token_type.clone();
        let line = self.current.as_ref().unwrap().line;
        let column = self.current.as_ref().unwrap().column;

        match token_type {
            TokenType::Equal => {
                self.advance(); // Consume '=='
                Ok(CmpOperator::Eq)
            }
            TokenType::NotEqual => {
                self.advance(); // Consume '!='
                Ok(CmpOperator::NotEq)
            }
            TokenType::LessThan => {
                self.advance(); // Consume '<'
                Ok(CmpOperator::Lt)
            }
            TokenType::LessEqual => {
                self.advance(); // Consume '<='
                Ok(CmpOperator::LtE)
            }
            TokenType::GreaterThan => {
                self.advance(); // Consume '>'
                Ok(CmpOperator::Gt)
            }
            TokenType::GreaterEqual => {
                self.advance(); // Consume '>='
                Ok(CmpOperator::GtE)
            }
            TokenType::Is => {
                self.advance(); // Consume 'is'

                if self.match_token(TokenType::Not) {
                    // Handle 'is not'
                    Ok(CmpOperator::IsNot)
                } else {
                    Ok(CmpOperator::Is)
                }
            }
            TokenType::In => {
                self.advance(); // Consume 'in'
                Ok(CmpOperator::In)
            }
            TokenType::Not => {
                self.advance(); // Consume 'not'

                if self.match_token(TokenType::In) {
                    // Handle 'not in'
                    Ok(CmpOperator::NotIn)
                } else {
                    Err(ParseError::InvalidSyntax {
                        message: "Expected 'in' after 'not' in comparison".to_string(),
                        line,
                        column,
                    })
                }
            }
            _ => Err(ParseError::InvalidSyntax {
                message: "Expected comparison operator".to_string(),
                line,
                column,
            }),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_bitwise_or()
    }

    fn parse_bitwise_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_bitwise_xor()?;

        while self.match_token(TokenType::BitwiseOr) {
            let token = self.previous_token();
            let right = self.parse_bitwise_xor()?;
            
            expr = Expr::BinOp {
                left: Box::new(expr),
                op: Operator::BitOr,
                right: Box::new(right),
                line: token.line,
                column: token.column,
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_bitwise_and()?;

        while self.match_token(TokenType::BitwiseXor) {
            let token = self.previous_token();
            let right = self.parse_bitwise_and()?;
            
            expr = Expr::BinOp {
                left: Box::new(expr),
                op: Operator::BitXor,
                right: Box::new(right),
                line: token.line,
                column: token.column,
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_shift()?;

        while self.match_token(TokenType::BitwiseAnd) {
            let token = self.previous_token();
            let right = self.parse_shift()?;
            
            expr = Expr::BinOp {
                left: Box::new(expr),
                op: Operator::BitAnd,
                right: Box::new(right),
                line: token.line,
                column: token.column,
            };
        }

        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_arithmetic()?;

        while self.match_token(TokenType::ShiftLeft) || self.match_token(TokenType::ShiftRight) {
            let token = self.previous_token();
            let op = match token.token_type {
                TokenType::ShiftLeft => Operator::LShift,
                TokenType::ShiftRight => Operator::RShift,
                _ => unreachable!(),
            };
            
            let right = self.parse_arithmetic()?;
            
            expr = Expr::BinOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                line: token.line,
                column: token.column,
            };
        }

        Ok(expr)
    }

    fn parse_arithmetic(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_term()?;

        while self.match_token(TokenType::Plus) || self.match_token(TokenType::Minus) {
            let token = self.previous_token();
            let op = match token.token_type {
                TokenType::Plus => Operator::Add,
                TokenType::Minus => Operator::Sub,
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: format!("Unexpected token in arithmetic: {:?}", token.token_type),
                        line: token.line,
                        column: token.column,
                    });
                }
            };

            let right = self.parse_term()?;
            expr = Expr::BinOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                line: token.line,
                column: token.column,
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_factor()?;

        while self.match_token(TokenType::Multiply)
            || self.match_token(TokenType::Divide)
            || self.match_token(TokenType::FloorDivide)
            || self.match_token(TokenType::Modulo)
            || self.match_token(TokenType::At)
        {
            let token = self.previous_token();
            let op = match token.token_type {
                TokenType::Multiply => Operator::Mult,
                TokenType::Divide => Operator::Div,
                TokenType::FloorDivide => Operator::FloorDiv,
                TokenType::Modulo => Operator::Mod,
                TokenType::At => Operator::MatMult,
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: format!("Unexpected token in term: {:?}", token.token_type),
                        line: token.line,
                        column: token.column,
                    });
                }
            };

            let right = self.parse_factor()?;

            expr = Expr::BinOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                line: token.line,
                column: token.column,
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(TokenType::Plus)
            || self.match_token(TokenType::Minus)
            || self.match_token(TokenType::BitwiseNot)
        {
            // Unary operations
            let token = self.previous_token();
            let op = match token.token_type {
                TokenType::Plus => UnaryOperator::UAdd,
                TokenType::Minus => UnaryOperator::USub,
                TokenType::BitwiseNot => UnaryOperator::Invert,
                _ => unreachable!(),
            };

            let operand = Box::new(self.parse_factor()?);

            Ok(Expr::UnaryOp {
                op,
                operand,
                line: token.line,
                column: token.column,
            })
        } else {
            // Call parse_power instead of parse_await_expr
            self.parse_power()
        }
    }

    fn parse_power(&mut self) -> Result<Expr, ParseError> {
        // First, try to parse await expression
        let mut expr = self.parse_await_expr()?;
        
        // Then check for power operator
        if self.match_token(TokenType::Power) {
            let token = self.previous_token();
            
            // For right associativity, recursively call parse_power
            let right = self.parse_power()?;
            
            expr = Expr::BinOp {
                left: Box::new(expr),
                op: Operator::Pow,
                right: Box::new(right),
                line: token.line,
                column: token.column,
            };
        }
        
        Ok(expr)
    }

    fn parse_await_expr(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(TokenType::Await) {
            let token = self.previous_token();
            let line = token.line;
            let column = token.column;

            // Parse the expression being awaited
            let value = Box::new(self.parse_atom_expr()?);

            Ok(Expr::Await {
                value,
                line,
                column,
            })
        } else {
            self.parse_atom_expr()
        }
    }

    fn parse_yield_expr(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(TokenType::Yield) {
            let token = self.previous_token();
            let line = token.line;
            let column = token.column;
    
            if self.match_token(TokenType::From) {
                let value = Box::new(self.parse_expression()?);
                return Ok(Expr::YieldFrom { value, line, column });
            }
            
            let value = if self.check_newline() 
                        || self.check(TokenType::RightParen) 
                        || self.check(TokenType::Comma) 
                        || self.check(TokenType::Colon)
                        || self.check(TokenType::EOF)
                        || self.check(TokenType::Dedent) {
                None
            } else {
                Some(Box::new(self.parse_expression()?))
            };
            
            Ok(Expr::Yield { value, line, column })
        } else {
            Err(ParseError::InvalidSyntax {
                message: "Expected 'yield' keyword".to_string(),
                line: self.current.as_ref().map_or(0, |t| t.line),
                column: self.current.as_ref().map_or(0, |t| t.column),
            })
        }
    }

    fn parse_atom_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_atom()?;

        // Parse trailers (call, attribute, subscription)
        loop {
            if self.match_token(TokenType::LeftParen) {
                // Function call
                let line = expr.get_line();
                let column = expr.get_column();
                let (args, keywords) = self.parse_arguments()?;
                self.consume(TokenType::RightParen, ")")?;

                expr = Expr::Call {
                    func: Box::new(expr),
                    args,
                    keywords,
                    line,
                    column,
                };
            } else if self.match_token(TokenType::Dot) {
                // Attribute access
                let line = expr.get_line();
                let column = expr.get_column();
                let attr = self.consume_identifier("attribute name")?;

                expr = Expr::Attribute {
                    value: Box::new(expr),
                    attr,
                    ctx: ExprContext::Load,
                    line,
                    column,
                };
            } else if self.match_token(TokenType::LeftBracket) {
                // Subscription
                let line = expr.get_line();
                let column = expr.get_column();
                let slice = Box::new(self.parse_slice()?);
                self.consume(TokenType::RightBracket, "]")?;

                expr = Expr::Subscript {
                    value: Box::new(expr),
                    slice,
                    ctx: ExprContext::Load,
                    line,
                    column,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_arguments(&mut self) -> Result<(Vec<Box<Expr>>, Vec<(Option<String>, Box<Expr>)>), ParseError> {
        let mut args = Vec::new();
        let mut keywords = Vec::new();
        let mut saw_keyword = false;
    
        // Handle empty argument list
        if self.check(TokenType::RightParen) {
            return Ok((args, keywords));
        }
        
        // Handle the specific case of a lone comma in class bases list
        if self.check(TokenType::Comma) {
            // This is an error - an empty expression before comma
            let token = self.current.clone().unwrap();
            return Err(ParseError::InvalidSyntax {
                message: "Expected expression before comma".to_string(),
                line: token.line,
                column: token.column,
            });
        }
    
        loop {
            // Check for *args or **kwargs
            if self.match_token(TokenType::Multiply) {
                // *args
                let token = self.previous_token();
                let value = Box::new(self.parse_expression()?);
                
                args.push(Box::new(Expr::Starred {
                    value,
                    ctx: ExprContext::Load,
                    line: token.line,
                    column: token.column,
                }));
                saw_keyword = true; // After *args, only keyword args allowed
            } else if self.match_token(TokenType::Power) {
                // **kwargs
                let arg = Box::new(self.parse_expression()?);
                keywords.push((None, arg));
            } else if !saw_keyword && self.peek_matches(TokenType::Assign) {
                // Keyword argument
                let key = match self.current.as_ref().unwrap().token_type.clone() {
                    TokenType::Identifier(name) => name,
                    _ => {
                        return Err(ParseError::InvalidSyntax {
                            message: "Expected identifier in keyword argument".to_string(),
                            line: self.current.as_ref().unwrap().line,
                            column: self.current.as_ref().unwrap().column,
                        });
                    }
                };
    
                self.advance(); // Consume the identifier
                self.advance(); // Consume the =
    
                let value = Box::new(self.parse_expression()?);
                keywords.push((Some(key), value));
                saw_keyword = true;
            } else if !saw_keyword {
                // Positional argument
                args.push(Box::new(self.parse_expression()?));
            } else {
                return Err(ParseError::InvalidSyntax {
                    message: "Positional argument after keyword argument".to_string(),
                    line: self.current.as_ref().unwrap().line,
                    column: self.current.as_ref().unwrap().column,
                });
            }
    
            if !self.match_token(TokenType::Comma) {
                break;
            }
    
            // Handle trailing comma
            if self.check(TokenType::RightParen) {
                break;
            }
            
            // Check for consecutive commas - this is an error
            if self.check(TokenType::Comma) {
                let token = self.current.clone().unwrap();
                return Err(ParseError::InvalidSyntax {
                    message: "Expected expression between commas".to_string(), 
                    line: token.line,
                    column: token.column,
                });
            }
        }
    
        Ok((args, keywords))
    }

    fn parse_slice(&mut self) -> Result<Expr, ParseError> {
        let line = self.current.as_ref().map_or(0, |t| t.line);
        let column = self.current.as_ref().map_or(0, |t| t.column);

        // Check for ellipsis
        if self.match_token(TokenType::Ellipsis) {
            // Create an ellipsis expression
            let ellipsis_expr = Expr::Ellipsis { line, column };

            // Check for complex slice with comma
            if self.match_token(TokenType::Comma) {
                // This is a multi-dimensional slice like [..., 0]
                let mut indices = vec![Box::new(ellipsis_expr)];

                // Parse the remaining indices
                if !self.check(TokenType::RightBracket) {
                    indices.push(Box::new(self.parse_expression()?));

                    // Parse any additional indices
                    while self.match_token(TokenType::Comma) {
                        if self.check(TokenType::RightBracket) {
                            break;
                        }
                        indices.push(Box::new(self.parse_expression()?));
                    }
                }

                // Create a tuple for multi-dimensional slicing
                return Ok(Expr::Tuple {
                    elts: indices,
                    ctx: ExprContext::Load,
                    line,
                    column,
                });
            }

            // Simple ellipsis
            return Ok(ellipsis_expr);
        }

        // Original slice parsing - handle start:stop:step notation
        let start_expr = if !self.check(TokenType::Colon) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        if self.match_token(TokenType::Colon) {
            // This is a slice
            let stop_expr = if !self.check(TokenType::Colon)
                && !self.check(TokenType::RightBracket)
                && !self.check(TokenType::Comma)
            {
                Some(self.parse_expression()?)
            } else {
                None
            };

            let step_expr = if self.match_token(TokenType::Colon) {
                if !self.check(TokenType::RightBracket) && !self.check(TokenType::Comma) {
                    Some(self.parse_expression()?)
                } else {
                    None
                }
            } else {
                None
            };

            // Create a slice expression
            let slice = Expr::Dict {
                keys: vec![
                    Some(Box::new(Expr::Str {
                        value: "start".to_string(),
                        line,
                        column,
                    })),
                    Some(Box::new(Expr::Str {
                        value: "stop".to_string(),
                        line,
                        column,
                    })),
                    Some(Box::new(Expr::Str {
                        value: "step".to_string(),
                        line,
                        column,
                    })),
                ],
                values: vec![
                    Box::new(match start_expr {
                        Some(expr) => expr,
                        None => Expr::NameConstant {
                            value: NameConstant::None,
                            line,
                            column,
                        },
                    }),
                    Box::new(match stop_expr {
                        Some(expr) => expr,
                        None => Expr::NameConstant {
                            value: NameConstant::None,
                            line,
                            column,
                        },
                    }),
                    Box::new(match step_expr {
                        Some(expr) => expr,
                        None => Expr::NameConstant {
                            value: NameConstant::None,
                            line,
                            column,
                        },
                    }),
                ],
                line,
                column,
            };

            // Handle multi-dimensional slicing with comma
            if self.match_token(TokenType::Comma) {
                let mut indices = vec![Box::new(slice)];

                // Parse remaining indices
                if !self.check(TokenType::RightBracket) {
                    indices.push(Box::new(self.parse_expression()?));

                    while self.match_token(TokenType::Comma) {
                        if self.check(TokenType::RightBracket) {
                            break;
                        }
                        indices.push(Box::new(self.parse_expression()?));
                    }
                }

                return Ok(Expr::Tuple {
                    elts: indices,
                    ctx: ExprContext::Load,
                    line,
                    column,
                });
            }

            Ok(slice)
        } else if self.match_token(TokenType::Comma) {
            // Multi-index access like a[1, 2, 3]
            let mut indices = vec![Box::new(start_expr.unwrap())];

            if !self.check(TokenType::RightBracket) {
                indices.push(Box::new(self.parse_expression()?));

                while self.match_token(TokenType::Comma) {
                    if self.check(TokenType::RightBracket) {
                        break;
                    }
                    indices.push(Box::new(self.parse_expression()?));
                }
            }

            Ok(Expr::Tuple {
                elts: indices,
                ctx: ExprContext::Load,
                line,
                column,
            })
        } else {
            // Simple index access
            start_expr.ok_or_else(|| ParseError::InvalidSyntax {
                message: "Expected expression in subscription".to_string(),
                line,
                column,
            })
        }
    }

    fn parse_type_annotation(&mut self, _is_nested: bool) -> Result<Expr, ParseError> {
        // Parse the base type
        let mut expr = self.parse_atom_expr()?;

        // Check for generic parameters with brackets
        if self.match_token(TokenType::LeftBracket) {
            let line = expr.get_line();
            let column = expr.get_column();

            // Parse the generic parameters
            let mut params = Vec::new();

            if !self.check(TokenType::RightBracket) {
                // Parse first parameter (could be nested)
                params.push(Box::new(self.parse_type_annotation(true)?));

                // Parse additional parameters
                while self.match_token(TokenType::Comma) {
                    if self.check(TokenType::RightBracket) {
                        break;
                    }
                    params.push(Box::new(self.parse_type_annotation(true)?));
                }
            }

            self.consume(TokenType::RightBracket, "]")?;

            // Create subscript expression for generic type
            expr = Expr::Subscript {
                value: Box::new(expr),
                slice: Box::new(Expr::Tuple {
                    elts: params,
                    ctx: ExprContext::Load,
                    line,
                    column,
                }),
                ctx: ExprContext::Load,
                line,
                column,
            };
        }

        Ok(expr)
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let token = match &self.current {
            Some(t) => t.clone(),
            None => {
                return Err(ParseError::EOF {
                    expected: "expression".to_string(),
                    line: 0,
                    column: 0,
                });
            }
        };
        let line = token.line;
        let column = token.column;
        
        match &token.token_type {
            // Handle identifiers (variable names)
            TokenType::Identifier(name) => {
                self.advance();
                Ok(Expr::Name {
                    id: name.clone(),
                    ctx: ExprContext::Load,
                    line,
                    column,
                })
            },
            TokenType::Yield => {
                return self.parse_yield_expr();
            },
            TokenType::LeftParen => {
                self.advance(); // Consume '('
            
                if self.match_token(TokenType::RightParen) {
                    // Empty tuple case
                    if !self.is_in_comprehension_context {
                        // Empty parentheses (should fail for expressions but is valid for tuples)
                        return Err(ParseError::InvalidSyntax {
                            message: "Empty parentheses not allowed in expressions".to_string(),
                            line,
                            column,
                        });
                    }
                    // Otherwise valid for tuple
                    Ok(Expr::Tuple {
                        elts: Vec::new(),
                        ctx: ExprContext::Load,
                        line,
                        column,
                    })
                } else {
                    // Parse the first expression
                    let expr = self.parse_expression()?;
        
                    // Check if this is a generator expression (has 'for' after the expression)
                    if self.match_token(TokenType::For) {
                        // Store the initial expression as the element of the generator
                        let elt = expr;
                        
                        // Use with_comprehension_context to ensure we don't expect 'else' after 'if'
                        let generators = self.with_comprehension_context(|this| {
                            let mut generators = Vec::new();
                            
                            // Parse first generator
                            let target = Box::new(this.parse_atom_expr()?);
                            this.consume(TokenType::In, "in")?;
                            let iter = Box::new(this.parse_expression()?);
                            
                            let mut ifs = Vec::new();
                            while this.match_token(TokenType::If) {
                                // Use parse_or_test instead of parse_expression
                                ifs.push(Box::new(this.parse_or_test()?));
                            }
                            
                            generators.push(Comprehension {
                                target,
                                iter,
                                ifs,
                                is_async: false,
                            });
                            
                            // Handle additional 'for' clauses
                            while this.match_token(TokenType::For) {
                                let target = Box::new(this.parse_atom_expr()?);
                                this.consume(TokenType::In, "in")?;
                                let iter = Box::new(this.parse_expression()?);
                                
                                let mut ifs = Vec::new();
                                while this.match_token(TokenType::If) {
                                    ifs.push(Box::new(this.parse_or_test()?));
                                }
                                
                                generators.push(Comprehension {
                                    target,
                                    iter,
                                    ifs,
                                    is_async: false,
                                });
                            }
                            
                            Ok(generators)
                        })?;
        
                        self.consume(TokenType::RightParen, ")")?;
        
                        Ok(Expr::GeneratorExp {
                            elt: Box::new(elt),
                            generators,
                            line,
                            column,
                        })
                    } else if self.match_token(TokenType::Comma) {
                        // This is a tuple
                        let mut elts = vec![Box::new(expr)];
        
                        if !self.check(TokenType::RightParen) {
                            elts.extend(self.parse_expr_list()?);
                        }
        
                        self.consume(TokenType::RightParen, ")")?;
        
                        Ok(Expr::Tuple {
                            elts,
                            ctx: ExprContext::Load,
                            line,
                            column,
                        })
                    } else {
                        // Simple parenthesized expression
                        self.consume(TokenType::RightParen, ")")?;
                        Ok(expr)
                    }
                }
            }
            TokenType::LeftBrace => {
                self.advance(); // Consume '{'
                
                // Check for EOF or newline before proceeding
                if self.check(TokenType::EOF) || self.check_newline() {
                    return Err(ParseError::InvalidSyntax {
                        message: "Unclosed brace".to_string(),
                        line,
                        column,
                    });
                }
            
                if self.match_token(TokenType::RightBrace) {
                    // Empty dict
                    Ok(Expr::Dict {
                        keys: Vec::new(),
                        values: Vec::new(),
                        line,
                        column,
                    })
                } else {
                    let first_expr = self.parse_expression()?;
                    
                    if self.match_token(TokenType::Colon) {
                        let value_expr = self.parse_expression()?;
                        
                        if self.match_token(TokenType::For) {
                            let key = first_expr;
                            let value = value_expr;
                            let generators = self.with_comprehension_context(|this| {
                                let mut generators = Vec::new();
                                let target_expr = this.parse_atom_expr()?;
                                let target_line = target_expr.get_line();
                                let target_column = target_expr.get_column();
                                let target = if this.check(TokenType::Comma) {
                                    let mut elts = vec![Box::new(target_expr)];
                                    while this.match_token(TokenType::Comma) {
                                        if this.check(TokenType::In) {
                                            break;
                                        }
                                        elts.push(Box::new(this.parse_atom_expr()?));
                                    }
                                    Box::new(Expr::Tuple {
                                        elts,
                                        ctx: ExprContext::Store,
                                        line: target_line,
                                        column: target_column,
                                    })
                                } else {
                                    Box::new(target_expr)
                                };
                                this.consume(TokenType::In, "in")?;
                                let iter = Box::new(this.parse_expression()?);
                                let mut ifs = Vec::new();
                                while this.match_token(TokenType::If) {
                                    ifs.push(Box::new(this.parse_or_test()?));
                                }
                                generators.push(Comprehension { 
                                    target, 
                                    iter, 
                                    ifs, 
                                    is_async: false 
                                });
                                while this.match_token(TokenType::For) {
                                    let nested_target = Box::new(this.parse_atom_expr()?);
                                    this.consume(TokenType::In, "in")?;
                                    let nested_iter = Box::new(this.parse_expression()?);
                                    let mut nested_ifs = Vec::new();
                                    while this.match_token(TokenType::If) {
                                        nested_ifs.push(Box::new(this.parse_or_test()?));
                                    }
                                    generators.push(Comprehension { 
                                        target: nested_target, 
                                        iter: nested_iter, 
                                        ifs: nested_ifs, 
                                        is_async: false 
                                    });
                                }
                                Ok(generators)
                            })?;
                            self.consume(TokenType::RightBrace, "}")?;
                            Ok(Expr::DictComp {
                                key: Box::new(key),
                                value: Box::new(value),
                                generators,
                                line,
                                column,
                            })
                        } else {
                            let mut keys = vec![Some(Box::new(first_expr))];
                            let mut values = vec![Box::new(value_expr)];
                            while self.match_token(TokenType::Comma) {
                                if self.check(TokenType::RightBrace) {
                                    break;
                                }
                                let key = Box::new(self.parse_expression()?);
                                self.consume(TokenType::Colon, ":")?;
                                let value = Box::new(self.parse_expression()?);
                                keys.push(Some(key));
                                values.push(value);
                            }
                            self.consume(TokenType::RightBrace, "}")?;
                            Ok(Expr::Dict { 
                                keys, 
                                values, 
                                line, 
                                column 
                            })
                        }
                    } else {
                        if self.match_token(TokenType::For) {
                            let elt = first_expr;
                            let generators = self.with_comprehension_context(|this| {
                                let mut generators = Vec::new();
                                let target = Box::new(this.parse_atom_expr()?);
                                this.consume(TokenType::In, "in")?;
                                let iter = Box::new(this.parse_expression()?);
                                let mut ifs = Vec::new();
                                while this.match_token(TokenType::If) {
                                    ifs.push(Box::new(this.parse_or_test()?));
                                }
                                generators.push(Comprehension { 
                                    target, 
                                    iter, 
                                    ifs, 
                                    is_async: false 
                                });
                                while this.match_token(TokenType::For) {
                                    let target = Box::new(this.parse_atom_expr()?);
                                    this.consume(TokenType::In, "in")?;
                                    let iter = Box::new(this.parse_expression()?);
                                    let mut ifs = Vec::new();
                                    while this.match_token(TokenType::If) {
                                        ifs.push(Box::new(this.parse_or_test()?));
                                    }
                                    generators.push(Comprehension { 
                                        target, 
                                        iter, 
                                        ifs, 
                                        is_async: false 
                                    });
                                }
                                Ok(generators)
                            })?;
                            self.consume(TokenType::RightBrace, "}")?;
                            Ok(Expr::SetComp {
                                elt: Box::new(elt),
                                generators,
                                line,
                                column,
                            })
                        } else {
                            let mut elts = vec![Box::new(first_expr)];
                            while self.match_token(TokenType::Comma) {
                                if self.check(TokenType::RightBrace) {
                                    break;
                                }
                                elts.push(Box::new(self.parse_expression()?));
                            }
                            self.consume(TokenType::RightBrace, "}")?;
                            Ok(Expr::Set { 
                                elts, 
                                line, 
                                column 
                            })
                        }
                    }
                }
            },
            // Handle lists and list comprehensions
            TokenType::LeftBracket => {
                self.advance(); // Consume '['
                
                // Check for EOF or newline before proceeding
                if self.check(TokenType::EOF) || self.check_newline() {
                    return Err(ParseError::InvalidSyntax {
                        message: "Unclosed bracket".to_string(),
                        line,
                        column,
                    });
                }
            
                if self.match_token(TokenType::RightBracket) {
                    // Empty list
                    Ok(Expr::List {
                        elts: Vec::new(),
                        ctx: ExprContext::Load,
                        line,
                        column,
                    })
                } else {
                    // Parse the first expression
                    let first_expr = self.parse_expression()?;
            
                    if self.match_token(TokenType::For) {
                        // List comprehension
                        let elt = first_expr;
                        let generators = self.with_comprehension_context(|this| {
                            let mut generators = Vec::new();
                            let target = Box::new(this.parse_atom_expr()?);
                            this.consume(TokenType::In, "in")?;
                            let iter = Box::new(this.parse_expression()?);
                            let mut ifs = Vec::new();
                            while this.match_token(TokenType::If) {
                                ifs.push(Box::new(this.parse_or_test()?));
                            }
                            generators.push(Comprehension {
                                target,
                                iter,
                                ifs,
                                is_async: false,
                            });
                            while this.match_token(TokenType::For) {
                                let target = Box::new(this.parse_atom_expr()?);
                                this.consume(TokenType::In, "in")?;
                                let iter = Box::new(this.parse_expression()?);
                                let mut ifs = Vec::new();
                                while this.match_token(TokenType::If) {
                                    ifs.push(Box::new(this.parse_or_test()?));
                                }
                                generators.push(Comprehension {
                                    target,
                                    iter,
                                    ifs,
                                    is_async: false,
                                });
                            }
                            Ok(generators)
                        })?;
                        self.consume(TokenType::RightBracket, "]")?;
                        Ok(Expr::ListComp {
                            elt: Box::new(elt),
                            generators,
                            line,
                            column,
                        })
                    } else {
                        let mut elts = vec![Box::new(first_expr)];
                        if self.match_token(TokenType::Comma) {
                            if !self.check(TokenType::RightBracket) {
                                elts.extend(self.parse_expr_list()?);
                            }
                        }
                        self.consume(TokenType::RightBracket, "]")?;
                        Ok(Expr::List {
                            elts,
                            ctx: ExprContext::Load,
                            line,
                            column,
                        })
                    }
                }
            },            
            // Handle numeric literals
            TokenType::IntLiteral(value) => {
                self.advance();
                Ok(Expr::Num {
                    value: Number::Integer(*value),
                    line,
                    column,
                })
            },
            TokenType::FloatLiteral(value) => {
                self.advance();
                Ok(Expr::Num {
                    value: Number::Float(*value),
                    line,
                    column,
                })
            },
            // Handle other literals and special forms
            TokenType::Lambda => {
                self.advance(); // Consume 'lambda'
                let params = self.parse_parameters()?;
                self.consume(TokenType::Colon, ":")?;
                let body = Box::new(self.parse_expression()?);
                Ok(Expr::Lambda {
                    args: params,
                    body,
                    line,
                    column,
                })
            },
            TokenType::FString(value) => {
                self.advance();
                Ok(Expr::Str {
                    value: value.clone(),
                    line,
                    column,
                })
            },
            TokenType::RawString(value) => {
                self.advance();
                Ok(Expr::Str {
                    value: value.clone(),
                    line,
                    column,
                })
            },
            TokenType::BytesLiteral(bytes) => {
                self.advance();
                Ok(Expr::Bytes {
                    value: bytes.clone(),
                    line,
                    column,
                })
            },
            // Handle other literals 
            TokenType::StringLiteral(value) => {
                self.advance();
                Ok(Expr::Str {
                    value: value.clone(),
                    line,
                    column,
                })
            },
            // Handle constants
            TokenType::True => {
                self.advance();
                Ok(Expr::NameConstant {
                    value: NameConstant::True,
                    line,
                    column,
                })
            },
            TokenType::False => {
                self.advance();
                Ok(Expr::NameConstant {
                    value: NameConstant::False,
                    line,
                    column,
                })
            },
            TokenType::None => {
                self.advance();
                Ok(Expr::NameConstant {
                    value: NameConstant::None,
                    line,
                    column,
                })
            },
            // Handle ellipsis
            TokenType::Ellipsis => {
                self.advance();
                Ok(Expr::Ellipsis { line, column })
            },
            // Default case for unexpected tokens
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: token.token_type.clone(),
                line,
                column,
            }),
        }
    }

    fn parse_expr_list(&mut self) -> Result<Vec<Box<Expr>>, ParseError> {
        let mut expressions = Vec::new();
    
        loop {
            // Check for starred expressions (*a)
            if self.match_token(TokenType::Multiply) {
                let token = self.previous_token();
                let line = token.line;
                let column = token.column;
                
                // Parse the value after the star
                // Use parse_atom_expr instead of parse_expression to handle various contexts
                let value = Box::new(self.parse_atom_expr()?);
                
                expressions.push(Box::new(Expr::Starred {
                    value,
                    ctx: ExprContext::Load,
                    line,
                    column,
                }));
            } else {
                expressions.push(Box::new(self.parse_expression()?));
            }
    
            // Continue if there's a comma
            if !self.match_token(TokenType::Comma) {
                break;
            }
    
            // Handle trailing comma
            if self.check(TokenType::RightParen) || self.check(TokenType::RightBracket) 
                || self.check(TokenType::RightBrace) || self.check(TokenType::Assign)
                || self.check(TokenType::Colon) || self.check_newline() || self.check(TokenType::EOF) {
                break;
            }
        }
    
        Ok(expressions)
    }

    fn consume_identifier(&mut self, expected: &str) -> Result<String, ParseError> {
        match &self.current {
            Some(token) => match &token.token_type {
                TokenType::Identifier(name) => {
                    let result = name.clone();
                    self.advance();
                    Ok(result)
                }
                _ => Err(ParseError::UnexpectedToken {
                    expected: expected.to_string(),
                    found: token.token_type.clone(),
                    line: token.line,
                    column: token.column,
                }),
            },
            None => Err(ParseError::EOF {
                expected: expected.to_string(),
                line: 0,
                column: 0,
            }),
        }
    }

    fn consume_dotted_name(&mut self, expected: &str) -> Result<String, ParseError> {
        let mut name = self.consume_identifier(expected)?;

        while self.match_token(TokenType::Dot) {
            name.push('.');
            name.push_str(&self.consume_identifier("identifier after dot")?);
        }

        Ok(name)
    }

    fn consume(&mut self, expected_type: TokenType, error_message: &str) -> Result<Token, ParseError> {
        match &self.current {
            Some(token) => {
                if std::mem::discriminant(&token.token_type) == std::mem::discriminant(&expected_type) {
                    let result = token.clone();
                    self.advance();
                    Ok(result)
                } else {
                    // Create a more specific error message for unclosed delimiters
                    let message = match &expected_type {
                        TokenType::RightParen => "Unclosed parenthesis",
                        TokenType::RightBracket => "Unclosed bracket",
                        TokenType::RightBrace => "Unclosed brace",
                        _ => error_message,
                    };
                    
                    Err(ParseError::UnexpectedToken {
                        expected: format!("Expected {} but found {:?}", message, token.token_type),
                        found: token.token_type.clone(),
                        line: token.line,
                        column: token.column,
                    })
                }
            }
            None => Err(ParseError::EOF {
                expected: error_message.to_string(),
                line: self.last_token.as_ref().map_or(0, |t| t.line),
                column: self.last_token.as_ref().map_or(0, |t| t.column),
            }),
        }
    }

    fn consume_newline(&mut self) -> Result<(), ParseError> {
        // More flexible newline handling
        if self.match_token(TokenType::SemiColon) {
            // Semicolons can substitute for newlines
            if self.check_newline() {
                self.advance();
            }
            return Ok(());
        }
        
        if self.check_newline() {
            self.advance();
            return Ok(());
        }
        
        if self.check(TokenType::EOF) || self.check(TokenType::Dedent) {
            return Ok(());
        }
        return Ok(());
    }

    fn check_newline(&self) -> bool {
        match &self.current {
            Some(token) => matches!(token.token_type, TokenType::Newline),
            None => false,
        }
    }

    fn match_token(&mut self, expected_type: TokenType) -> bool {
        if self.check(expected_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, expected_type: TokenType) -> bool {
        match &self.current {
            Some(token) => {
                std::mem::discriminant(&token.token_type) == std::mem::discriminant(&expected_type)
            }
            None => false,
        }
    }

    // Fixed to avoid borrowing conflicts
    fn peek_matches(&self, expected_type: TokenType) -> bool {
        // Clone the first token from tokens queue if available
        if let Some(token) = self.tokens.front() {
            // Clone the token type to avoid borrowing issues
            let token_type = token.token_type.clone();
            std::mem::discriminant(&token_type) == std::mem::discriminant(&expected_type)
        } else {
            false
        }
    }

    fn advance(&mut self) -> Option<Token> {
        let current = self.current.take();
        if let Some(token) = &current {
            self.last_token = Some(token.clone());
        }
        self.current = self.tokens.pop_front();
        current
    }

    fn previous_token(&self) -> Token {
        self.last_token
            .clone()
            .expect("No previous token available")
    }
}

// Helper trait to get line and column from Expr
trait GetLocation {
    fn get_line(&self) -> usize;
    fn get_column(&self) -> usize;
}

impl GetLocation for Expr {
    fn get_line(&self) -> usize {
        match self {
            Expr::BoolOp { line, .. } => *line,
            Expr::BinOp { line, .. } => *line,
            Expr::UnaryOp { line, .. } => *line,
            Expr::Lambda { line, .. } => *line,
            Expr::IfExp { line, .. } => *line,
            Expr::Dict { line, .. } => *line,
            Expr::Set { line, .. } => *line,
            Expr::ListComp { line, .. } => *line,
            Expr::SetComp { line, .. } => *line,
            Expr::DictComp { line, .. } => *line,
            Expr::GeneratorExp { line, .. } => *line,
            Expr::Await { line, .. } => *line,
            Expr::Yield { line, .. } => *line,
            Expr::YieldFrom { line, .. } => *line,
            Expr::Compare { line, .. } => *line,
            Expr::Call { line, .. } => *line,
            Expr::Num { line, .. } => *line,
            Expr::Str { line, .. } => *line,
            Expr::FormattedValue { line, .. } => *line,
            Expr::JoinedStr { line, .. } => *line,
            Expr::Bytes { line, .. } => *line,
            Expr::NameConstant { line, .. } => *line,
            Expr::Ellipsis { line, .. } => *line,
            Expr::Constant { line, .. } => *line,
            Expr::Attribute { line, .. } => *line,
            Expr::Subscript { line, .. } => *line,
            Expr::Starred { line, .. } => *line,
            Expr::Name { line, .. } => *line,
            Expr::List { line, .. } => *line,
            Expr::Tuple { line, .. } => *line,
        }
    }

    fn get_column(&self) -> usize {
        match self {
            Expr::BoolOp { column, .. } => *column,
            Expr::BinOp { column, .. } => *column,
            Expr::UnaryOp { column, .. } => *column,
            Expr::Lambda { column, .. } => *column,
            Expr::IfExp { column, .. } => *column,
            Expr::Dict { column, .. } => *column,
            Expr::Set { column, .. } => *column,
            Expr::ListComp { column, .. } => *column,
            Expr::SetComp { column, .. } => *column,
            Expr::DictComp { column, .. } => *column,
            Expr::GeneratorExp { column, .. } => *column,
            Expr::Await { column, .. } => *column,
            Expr::Yield { column, .. } => *column,
            Expr::YieldFrom { column, .. } => *column,
            Expr::Compare { column, .. } => *column,
            Expr::Call { column, .. } => *column,
            Expr::Num { column, .. } => *column,
            Expr::Str { column, .. } => *column,
            Expr::FormattedValue { column, .. } => *column,
            Expr::JoinedStr { column, .. } => *column,
            Expr::Bytes { column, .. } => *column,
            Expr::NameConstant { column, .. } => *column,
            Expr::Ellipsis { column, .. } => *column,
            Expr::Constant { column, .. } => *column,
            Expr::Attribute { column, .. } => *column,
            Expr::Subscript { column, .. } => *column,
            Expr::Starred { column, .. } => *column,
            Expr::Name { column, .. } => *column,
            Expr::List { column, .. } => *column,
            Expr::Tuple { column, .. } => *column,
        }
    }
}