use crate::lexer::{Token, TokenType};
use crate::ast::{
    Stmt, Expr, Module, ExprContext, BoolOperator, Operator, UnaryOperator, 
    CmpOperator, NameConstant, ExceptHandler, Comprehension,
    Alias, Parameter, Number
};

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken { expected: String, found: TokenType, line: usize, column: usize },
    InvalidSyntax { message: String, line: usize, column: usize },
    EOF { expected: String, line: usize, column: usize },
}

pub struct Parser {
    tokens: VecDeque<Token>,
    current: Option<Token>,
    last_token: Option<Token>,
    errors: Vec<ParseError>,
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
        }
    }

    pub fn parse(&mut self) -> Result<Module, Vec<ParseError>> {
        let mut body = Vec::new();
    
        while let Some(token) = &self.current {
            if matches!(token.token_type, TokenType::EOF) {
                break;
            }
            
            // Skip newlines before statements
            while matches!(self.current.as_ref().map(|t| &t.token_type), Some(&TokenType::Newline)) {
                self.advance();
            }
            
            // Check if we've reached EOF after skipping newlines
            if self.current.is_none() {
                break;
            }
            
            match self.parse_statement() {
                Ok(stmt) => body.push(Box::new(stmt)),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                },
            }
        }
    
        if self.errors.is_empty() {
            Ok(Module { body })
        } else {
            Err(self.errors.clone())
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let token = match &self.current {
            Some(token) => token,
            None => return Err(ParseError::EOF {
                expected: "statement".to_string(),
                line: 0,
                column: 0,
            }),
        };

        match &token.token_type {
            TokenType::Def => self.parse_function_def(),
            TokenType::Class => self.parse_class_def(),
            TokenType::Return => self.parse_return(),
            TokenType::Del => self.parse_delete(),
            TokenType::If => self.parse_if(),
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
            _ => self.parse_expr_statement(),
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
            decorator_list: Vec::new(), // No decorator support yet
            returns,
            line,
            column,
        })
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let mut params = Vec::new();
        
        if !self.check(TokenType::RightParen) {
            loop {
                let name = self.consume_identifier("parameter name")?;
                
                // Parse optional type annotation
                let typ = if self.match_token(TokenType::Colon) {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };
                
                // Parse default value
                let default = if self.match_token(TokenType::Assign) {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };
                
                params.push(Parameter { name, typ, default });
                
                if !self.match_token(TokenType::Comma) {
                    break;
                }
                
                // Handle trailing comma
                if self.check(TokenType::RightParen) {
                    break;
                }
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
            decorator_list: Vec::new(), // No decorator support yet
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
        
        Ok(Stmt::Return { value, line, column })
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
        
        Ok(Stmt::Delete { targets, line, column })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;
        
        // Consume 'if'
        self.advance();
        
        // Parse condition
        let test = Box::new(self.parse_expression()?);
        
        // Parse body
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
        
        Ok(Stmt::If { test, body, orelse, line, column })
    }
    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;
        
        // Consume 'for'
        self.advance();
        
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
        
        Ok(Stmt::For { target, iter, body, orelse, line, column })
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
        
        Ok(Stmt::While { test, body, orelse, line, column })
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
        
        Ok(Stmt::With { items, body, line, column })
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
        
        Ok(Stmt::Raise { exc, cause, line, column })
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
        
        Ok(Stmt::Assert { test, msg, line, column })
    }

    fn parse_import(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current.clone().unwrap();
        let line = token.line;
        let column = token.column;
        
        // Consume 'import'
        self.advance();
        
        // Parse import names
        let names = self.parse_import_names()?;
        
        // Consume newline
        self.consume_newline()?;
        
        Ok(Stmt::Import { names, line, column })
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
            vec![Alias { name: "*".to_string(), asname: None }]
        } else {
            self.parse_import_as_names()?
        };
        
        // Consume newline
        self.consume_newline()?;
        
        Ok(Stmt::ImportFrom { module, names, level, line, column })
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
            if (has_parens && self.check(TokenType::RightParen)) || 
               (!has_parens && (self.check_newline() || self.check(TokenType::EOF))) {
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
        
        Ok(Stmt::Global { names, line, column })
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
        
        Ok(Stmt::Nonlocal { names, line, column })
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
        let expr = self.parse_expression()?;
        let line = expr.get_line();
        let column = expr.get_column();
        
        // Check if it's an assignment
        if self.match_token(TokenType::Assign) {
            // Simple assignment
            let value = Box::new(self.parse_expression()?);
            self.consume_newline()?;
            
            Ok(Stmt::Assign {
                targets: vec![Box::new(expr)],
                value,
                line,
                column,
            })
        } else if self.is_augmented_assign() {
            // Augmented assignment like +=, -=, etc.
            let op = self.parse_augmented_assign_op();
            self.advance(); // Consume the operator
            let value = Box::new(self.parse_expression()?);
            self.consume_newline()?;
            
            Ok(Stmt::AugAssign {
                target: Box::new(expr),
                op,
                value,
                line,
                column,
            })
        } else if self.match_token(TokenType::Colon) {
            // Annotated assignment: x: int = 5
            let annotation = Box::new(self.parse_expression()?);
            
            let value = if self.match_token(TokenType::Assign) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };
            
            self.consume_newline()?;
            
            Ok(Stmt::AnnAssign {
                target: Box::new(expr),
                annotation,
                value,
                line,
                column,
            })
        } else {
            // Expression statement
            self.consume_newline()?;
            
            Ok(Stmt::Expr {
                value: Box::new(expr),
                line,
                column,
            })
        }
    }

    fn is_augmented_assign(&self) -> bool {
        match &self.current {
            Some(token) => matches!(
                token.token_type,
                TokenType::PlusAssign |
                TokenType::MinusAssign |
                TokenType::MulAssign |
                TokenType::DivAssign |
                TokenType::ModAssign |
                TokenType::PowAssign |
                TokenType::FloorDivAssign |
                TokenType::MatrixMulAssign |
                TokenType::BitwiseAndAssign |
                TokenType::BitwiseOrAssign |
                TokenType::BitwiseXorAssign |
                TokenType::ShiftLeftAssign |
                TokenType::ShiftRightAssign
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
                while !self.check(TokenType::Dedent) && !self.check(TokenType::EOF) {
                    if self.match_token(TokenType::Newline) {
                        continue;
                    }
                    let stmt = self.parse_statement()?;
                    statements.push(Box::new(stmt));
                    // Ensure we don’t parse beyond the current block
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
        self.parse_or_test()
    }

    fn parse_or_test(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_and_test()?;
        
        // Parse 'or' operator chains
        while self.match_token(TokenType::Or) {
            let right = self.parse_and_test()?;
            let line = expr.get_line();
            let column = expr.get_column();
            
            expr = Expr::BoolOp {
                op: BoolOperator::Or,
                values: vec![Box::new(expr), Box::new(right)],
                line,
                column,
            };
        }
        
        Ok(expr)
    }

    fn parse_and_test(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_not_test()?;
        
        // Parse 'and' operator chains
        while self.match_token(TokenType::And) {
            let right = self.parse_not_test()?;
            let line = expr.get_line();
            let column = expr.get_column();
            
            expr = Expr::BoolOp {
                op: BoolOperator::And,
                values: vec![Box::new(expr), Box::new(right)],
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
                    TokenType::Equal |
                    TokenType::NotEqual |
                    TokenType::LessThan |
                    TokenType::LessEqual |
                    TokenType::GreaterThan |
                    TokenType::GreaterEqual |
                    TokenType::Is |
                    TokenType::In |
                    TokenType::Not
                )
            },
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
            },
            TokenType::NotEqual => {
                self.advance(); // Consume '!='
                Ok(CmpOperator::NotEq)
            },
            TokenType::LessThan => {
                self.advance(); // Consume '<'
                Ok(CmpOperator::Lt)
            },
            TokenType::LessEqual => {
                self.advance(); // Consume '<='
                Ok(CmpOperator::LtE)
            },
            TokenType::GreaterThan => {
                self.advance(); // Consume '>'
                Ok(CmpOperator::Gt)
            },
            TokenType::GreaterEqual => {
                self.advance(); // Consume '>='
                Ok(CmpOperator::GtE)
            },
            TokenType::Is => {
                self.advance(); // Consume 'is'
                
                if self.match_token(TokenType::Not) {
                    // Handle 'is not'
                    Ok(CmpOperator::IsNot)
                } else {
                    Ok(CmpOperator::Is)
                }
            },
            TokenType::In => {
                self.advance(); // Consume 'in'
                Ok(CmpOperator::In)
            },
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
            },
            _ => {
                Err(ParseError::InvalidSyntax {
                    message: "Expected comparison operator".to_string(),
                    line,
                    column,
                })
            }
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_arithmetic()
    }

    fn parse_arithmetic(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_term()?;
        
        while self.match_token(TokenType::Plus) || self.match_token(TokenType::Minus) {
            let token = self.previous_token();
            let op = match token.token_type {
                TokenType::Plus => Operator::Add,
                TokenType::Minus => Operator::Sub,
                _ => return Err(ParseError::InvalidSyntax {
                    message: format!("Unexpected token in arithmetic: {:?}", token.token_type),
                    line: token.line,
                    column: token.column,
                }),
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
        
        while self.match_token(TokenType::Multiply) || 
              self.match_token(TokenType::Divide) || 
              self.match_token(TokenType::FloorDivide) || 
              self.match_token(TokenType::Modulo) || 
              self.match_token(TokenType::At) {
            
            let token = self.previous_token();
            let op = match token.token_type {
                TokenType::Multiply => Operator::Mult,
                TokenType::Divide => Operator::Div,
                TokenType::FloorDivide => Operator::FloorDiv,
                TokenType::Modulo => Operator::Mod,
                TokenType::At => Operator::MatMult,
                _ => return Err(ParseError::InvalidSyntax {
                    message: format!("Unexpected token in term: {:?}", token.token_type),
                    line: token.line,
                    column: token.column,
                }),
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
        // Parse unary operations +, -, ~
        if self.match_token(TokenType::Plus) || 
            self.match_token(TokenType::Minus) || 
            self.match_token(TokenType::BitwiseNot) {
            
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
            self.parse_power()
        }
    }

    fn parse_power(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_atom_expr()?;
        
        // Parse exponentiation
        if self.match_token(TokenType::Power) {
            let token = self.previous_token();
            let right = self.parse_factor()?;
            
            Ok(Expr::BinOp {
                left: Box::new(expr),
                op: Operator::Pow,
                right: Box::new(right),
                line: token.line,
                column: token.column,
            })
        } else {
            Ok(expr)
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
        
        loop {
            // Check for *args or **kwargs
            if self.match_token(TokenType::Multiply) {
                // *args
                let arg = Box::new(self.parse_expression()?);
                args.push(Box::new(Expr::Starred {
                    value: arg,
                    ctx: ExprContext::Load,
                    line: 0, // We'll fix this later
                    column: 0,
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
                    _ => return Err(ParseError::InvalidSyntax {
                        message: "Expected identifier in keyword argument".to_string(),
                        line: self.current.as_ref().unwrap().line,
                        column: self.current.as_ref().unwrap().column,
                    }),
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
        }
        
        Ok((args, keywords))
    }

    fn parse_slice(&mut self) -> Result<Expr, ParseError> {
        // Parse slices like a[start:stop:step]
        let start_expr = if !self.check(TokenType::Colon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        if self.match_token(TokenType::Colon) {
            // This is a slice
            let stop_expr = if !self.check(TokenType::Colon) && !self.check(TokenType::RightBracket) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            let step_expr = if self.match_token(TokenType::Colon) {
                if !self.check(TokenType::RightBracket) {
                    Some(self.parse_expression()?)
                } else {
                    None
                }
            } else {
                None
            };
            
            // Create a slice expression
            let line = self.current.as_ref().map_or(0, |t| t.line);
            let column = self.current.as_ref().map_or(0, |t| t.column);
            
            let slice = Expr::Dict {
                keys: vec![
                    Some(Box::new(Expr::Str { value: "start".to_string(), line, column })),
                    Some(Box::new(Expr::Str { value: "stop".to_string(), line, column })),
                    Some(Box::new(Expr::Str { value: "step".to_string(), line, column })),
                ],
                values: vec![
                    Box::new(match start_expr {
                        Some(expr) => expr,
                        None => Expr::NameConstant { value: NameConstant::None, line, column },
                    }),
                    Box::new(match stop_expr {
                        Some(expr) => expr,
                        None => Expr::NameConstant { value: NameConstant::None, line, column },
                    }),
                    Box::new(match step_expr {
                        Some(expr) => expr,
                        None => Expr::NameConstant { value: NameConstant::None, line, column },
                    }),
                ],
                line,
                column,
            };
            
            Ok(slice)
        } else {
            // Simple index, not a slice
            start_expr.ok_or_else(|| ParseError::InvalidSyntax {
                message: "Expected expression in subscription".to_string(),
                line: self.current.as_ref().map_or(0, |t| t.line),
                column: self.current.as_ref().map_or(0, |t| t.column),
            })
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let token = match &self.current {
            Some(t) => t.clone(),
            None => return Err(ParseError::EOF { expected: "expression".to_string(), line: 0, column: 0 }),
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
            TokenType::BinaryLiteral(value) => {
                self.advance();
                Ok(Expr::Num {
                    value: Number::Integer(*value),
                    line,
                    column,
                })
            },
            TokenType::OctalLiteral(value) => {
                self.advance();
                Ok(Expr::Num {
                    value: Number::Integer(*value),
                    line,
                    column,
                })
            },
            TokenType::HexLiteral(value) => {
                self.advance();
                Ok(Expr::Num {
                    value: Number::Integer(*value),
                    line,
                    column,
                })
            },
            
            // Handle string literals
            TokenType::StringLiteral(value) => {
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
            TokenType::FString(value) => {
                self.advance();
                // For simplicity, treat f-strings as regular strings for now
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
            
            // Handle parenthesized expressions and tuples
            TokenType::LeftParen => {
                self.advance(); // Consume '('
                
                if self.match_token(TokenType::RightParen) {
                    // Empty tuple
                    Ok(Expr::Tuple { elts: Vec::new(), ctx: ExprContext::Load, line, column })
                } else {
                    let expr = self.parse_expression()?;
                    
                    if self.match_token(TokenType::Comma) {
                        // This is a tuple
                        let mut elts = vec![Box::new(expr)];
                        if !self.check(TokenType::RightParen) {
                            elts.extend(self.parse_expr_list()?);
                        }
                        self.consume(TokenType::RightParen, ")")?;
                        Ok(Expr::Tuple { elts, ctx: ExprContext::Load, line, column })
                    } else {
                        // Simple parenthesized expression
                        self.consume(TokenType::RightParen, ")")?;
                        Ok(expr)
                    }
                }
            },
            
            // Handle lists and list comprehensions
            TokenType::LeftBracket => {
                self.advance(); // Consume '['
                
                if self.match_token(TokenType::RightBracket) {
                    // Empty list
                    Ok(Expr::List { elts: Vec::new(), ctx: ExprContext::Load, line, column })
                } else {
                    let first_expr = self.parse_expression()?;
                    
                    if self.match_token(TokenType::For) {
                        // List comprehension
                        // Use parse_atom_expr for the target to avoid consuming 'in'
                        let target = Box::new(self.parse_atom_expr()?);
                        self.consume(TokenType::In, "in")?;
                        let iter = Box::new(self.parse_expression()?);
                        
                        let mut ifs = Vec::new();
                        while self.match_token(TokenType::If) {
                            ifs.push(Box::new(self.parse_expression()?));
                        }
                        
                        let mut generators = vec![Comprehension { target, iter, ifs, is_async: false }];
                        
                        // Additional for loops in the comprehension
                        while self.match_token(TokenType::For) {
                            // Also use parse_atom_expr for additional comprehension targets
                            let target = Box::new(self.parse_atom_expr()?);
                            self.consume(TokenType::In, "in")?;
                            let iter = Box::new(self.parse_expression()?);
                            
                            let mut ifs = Vec::new();
                            while self.match_token(TokenType::If) {
                                ifs.push(Box::new(self.parse_expression()?));
                            }
                            
                            generators.push(Comprehension { target, iter, ifs, is_async: false });
                        }
                        
                        self.consume(TokenType::RightBracket, "]")?;
                        Ok(Expr::ListComp {
                            elt: Box::new(first_expr),
                            generators,
                            line,
                            column,
                        })
                    } else {
                        // List literal
                        let mut elts = vec![Box::new(first_expr)];
                        if self.match_token(TokenType::Comma) {
                            if !self.check(TokenType::RightBracket) {
                                elts.extend(self.parse_expr_list()?);
                            }
                        }
                        self.consume(TokenType::RightBracket, "]")?;
                        Ok(Expr::List { elts, ctx: ExprContext::Load, line, column })
                    }
                }
            },
            
            // Handle dictionaries and sets
            TokenType::LeftBrace => {
                self.advance(); // Consume '{'
                
                if self.match_token(TokenType::RightBrace) {
                    // Empty dict
                    Ok(Expr::Dict {
                        keys: Vec::new(),
                        values: Vec::new(),
                        line,
                        column,
                    })
                } else {
                    let key_expr = self.parse_expression()?;
                    
                    if self.match_token(TokenType::Colon) {
                        // This is a dict
                        let value_expr = self.parse_expression()?;
                        
                        if self.match_token(TokenType::For) {
                            // Dict comprehension
                            // Use parse_atom_expr for target
                            let target = Box::new(self.parse_atom_expr()?);
                            self.consume(TokenType::In, "in")?;
                            let iter = Box::new(self.parse_expression()?);
                            
                            let mut ifs = Vec::new();
                            while self.match_token(TokenType::If) {
                                ifs.push(Box::new(self.parse_expression()?));
                            }
                            
                            let mut generators = vec![Comprehension { target, iter, ifs, is_async: false }];
                            
                            // Additional for loops in the comprehension
                            while self.match_token(TokenType::For) {
                                // Use parse_atom_expr for additional targets
                                let target = Box::new(self.parse_atom_expr()?);
                                self.consume(TokenType::In, "in")?;
                                let iter = Box::new(self.parse_expression()?);
                                
                                let mut ifs = Vec::new();
                                while self.match_token(TokenType::If) {
                                    ifs.push(Box::new(self.parse_expression()?));
                                }
                                
                                generators.push(Comprehension { target, iter, ifs, is_async: false });
                            }
                            
                            self.consume(TokenType::RightBrace, "}")?;
                            Ok(Expr::DictComp {
                                key: Box::new(key_expr),
                                value: Box::new(value_expr),
                                generators,
                                line,
                                column,
                            })
                        } else {
                            // Dict literal
                            let mut keys = vec![Some(Box::new(key_expr))];
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
                            Ok(Expr::Dict { keys, values, line, column })
                        }
                    } else if self.match_token(TokenType::For) {
                        // Set comprehension
                        // Use parse_atom_expr for target
                        let target = Box::new(self.parse_atom_expr()?);
                        self.consume(TokenType::In, "in")?;
                        let iter = Box::new(self.parse_expression()?);
                        
                        let mut ifs = Vec::new();
                        while self.match_token(TokenType::If) {
                            ifs.push(Box::new(self.parse_expression()?));
                        }
                        
                        let mut generators = vec![Comprehension { target, iter, ifs, is_async: false }];
                        
                        // Additional for loops in the comprehension
                        while self.match_token(TokenType::For) {
                            // Use parse_atom_expr for additional targets
                            let target = Box::new(self.parse_atom_expr()?);
                            self.consume(TokenType::In, "in")?;
                            let iter = Box::new(self.parse_expression()?);
                            
                            let mut ifs = Vec::new();
                            while self.match_token(TokenType::If) {
                                ifs.push(Box::new(self.parse_expression()?));
                            }
                            
                            generators.push(Comprehension { target, iter, ifs, is_async: false });
                        }
                        
                        self.consume(TokenType::RightBrace, "}")?;
                        Ok(Expr::SetComp {
                            elt: Box::new(key_expr),
                            generators,
                            line,
                            column,
                        })
                    } else {
                        // Set literal
                        let mut elts = vec![Box::new(key_expr)];
                        
                        while self.match_token(TokenType::Comma) {
                            if self.check(TokenType::RightBrace) {
                                break;
                            }
                            
                            elts.push(Box::new(self.parse_expression()?));
                        }
                        
                        self.consume(TokenType::RightBrace, "}")?;
                        Ok(Expr::Set { elts, line, column })
                    }
                }
            },
            
            // Handle ellipsis
            TokenType::Ellipsis => {
                self.advance();
                Ok(Expr::Ellipsis { line, column })
            },
            
            // Handle other literals and special forms
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
            expressions.push(Box::new(self.parse_expression()?));
            
            if !self.match_token(TokenType::Comma) {
                break;
            }
            
            // Handle trailing comma
            if self.check(TokenType::RightParen) || 
                self.check(TokenType::RightBracket) || 
                self.check(TokenType::RightBrace) {
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
                },
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
                    Err(ParseError::UnexpectedToken {
                        expected: error_message.to_string(),
                        found: token.token_type.clone(),
                        line: token.line,
                        column: token.column,
                    })
                }
            },
            None => Err(ParseError::EOF {
                expected: error_message.to_string(),
                line: 0,
                column: 0,
            }),
        }
    }

    fn consume_newline(&mut self) -> Result<(), ParseError> {
        if !self.check_newline() && !self.check(TokenType::EOF) && !self.check(TokenType::SemiColon) {
            if let Some(token) = &self.current {
                return Err(ParseError::UnexpectedToken {
                    expected: "newline".to_string(),
                    found: token.token_type.clone(),
                    line: token.line,
                    column: token.column,
                });
            }
        }
        
        if self.match_token(TokenType::SemiColon) {
            // Optionally followed by newline
            if self.check_newline() {
                self.advance();
            }
        } else if self.check_newline() {
            self.advance();
        }
        
        Ok(())
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
            Some(token) => std::mem::discriminant(&token.token_type) == std::mem::discriminant(&expected_type),
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
        self.last_token.clone().expect("No previous token available")
    }

    fn synchronize(&mut self) {
        loop {
            // Clone or check token type before mutating self
            let should_break = match &self.current {
                Some(token) if matches!(token.token_type, TokenType::EOF) => true,
                Some(token) => {
                    let is_newline = matches!(token.token_type, TokenType::Newline);
                    self.advance();
                    is_newline
                },
                None => true,
            };
            
            if should_break {
                break;
            }
        }
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