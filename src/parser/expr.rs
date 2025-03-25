use crate::ast::{
    BoolOperator, CmpOperator, Comprehension, Expr, ExprContext, NameConstant,
    Number, Operator, UnaryOperator,
};
use crate::lexer::TokenType;
use crate::parser::{Parser, ParseError};
use crate::parser::helpers::TokenMatching;
use crate::parser::types::{ParserContext, GetLocation};
use crate::parser::stmt::StmtParser;

/// Parser methods for expressions
pub trait ExprParser {
    /// Parse an expression
    fn parse_expression(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse an 'or' test
    fn parse_or_test(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse an 'and' test
    fn parse_and_test(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a 'not' test
    fn parse_not_test(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a comparison
    fn parse_comparison(&mut self) -> Result<Expr, ParseError>;
    
    /// Check if the current token is a comparison operator
    fn is_comparison_operator(&self) -> bool;  // Added this method to the trait
    
    /// Parse a comparison operator
    fn parse_comparison_operator(&mut self) -> Result<CmpOperator, ParseError>;
    
    /// Parse an expression (e-context)
    fn parse_expr(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a bitwise OR expression
    fn parse_bitwise_or(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a bitwise XOR expression
    fn parse_bitwise_xor(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a bitwise AND expression
    fn parse_bitwise_and(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a shift expression
    fn parse_shift(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse an arithmetic expression
    fn parse_arithmetic(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a term
    fn parse_term(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a factor
    fn parse_factor(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a power expression
    fn parse_power(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse an await expression
    fn parse_await_expr(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a yield expression
    fn parse_yield_expr(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse an atom expression
    fn parse_atom_expr(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a slice
    fn parse_slice(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a type annotation
    fn parse_type_annotation(&mut self, is_nested: bool) -> Result<Expr, ParseError>;

    /// Parse lambda parameters
    fn parse_lambda_parameters(&mut self) -> Result<Vec<crate::ast::Parameter>, ParseError>;
    
    /// Parse an atom
    fn parse_atom(&mut self) -> Result<Expr, ParseError>;
    
    /// Parse a list of expressions
    fn parse_expr_list(&mut self) -> Result<Vec<Box<Expr>>, ParseError>;
    
    /// Parse a dictionary literal
    fn parse_dict_literal(&mut self, line: usize, column: usize) -> Result<Expr, ParseError>;

    /// Parse function arguments 
    fn parse_more_arguments(&mut self) -> Result<(Vec<Box<Expr>>, Vec<(Option<String>, Box<Expr>)>), ParseError>;
}

impl ExprParser for Parser {
    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        // Check for starred expressions first before standard expression parsing
        if self.check(TokenType::Multiply) {
            let star_token = self.current.clone().unwrap();
            self.advance(); // Consume the * token
            
            // Parse the expression after the star
            let value = Box::new(self.parse_atom_expr()?);
            
            // Create a Starred expression
            let expr = Expr::Starred {
                value,
                ctx: ExprContext::Load,
                line: star_token.line,
                column: star_token.column,
            };
            
            // Handle potential tuple creation with comma
            if self.match_token(TokenType::Comma) {
                let line = expr.get_line();
                let column = expr.get_column();
                
                let mut elts = vec![Box::new(expr)];
                
                while !self.check_newline()
                    && !self.check(TokenType::EOF)
                    && !self.check(TokenType::RightParen)
                    && !self.check(TokenType::RightBracket)
                {
                    if self.check(TokenType::Comma) {
                        return Err(ParseError::InvalidSyntax {
                            message: "Expected expression after comma".to_string(),
                            line: self.current.as_ref().map_or(line, |t| t.line),
                            column: self.current.as_ref().map_or(column, |t| t.column),
                        });
                    }
                    
                    elts.push(Box::new(self.parse_or_test()?));
                    
                    if !self.match_token(TokenType::Comma) {
                        break;
                    }
                }
                
                return Ok(Expr::Tuple {
                    elts,
                    ctx: ExprContext::Load,
                    line,
                    column,
                });
            }
            
            return Ok(expr);
        }
        
        // Original parse_expression code follows
        let mut expr = self.parse_or_test()?;
    
        if self.check(TokenType::If)
            && !self.is_in_context(ParserContext::Comprehension)
            && !self.is_in_context(ParserContext::Match)
        {
            let line = expr.get_line();
            let column = expr.get_column();
    
            self.advance();
    
            let test = Box::new(self.parse_or_test()?);
    
            self.consume(TokenType::Else, "else")?;
    
            let orelse = Box::new(self.parse_expression()?);
    
            expr = Expr::IfExp {
                test,
                body: Box::new(expr),
                orelse,
                line,
                column,
            };
        } else if self.match_token(TokenType::Comma) {
            let line = expr.get_line();
            let column = expr.get_column();
    
            let mut elts = vec![Box::new(expr)];
    
            // Continue parsing the tuple until we reach a delimiter token
            while !self.check_newline()
                && !self.check(TokenType::EOF)
                && !self.check(TokenType::RightParen)
                && !self.check(TokenType::RightBracket)
            {
                if self.check(TokenType::Comma) {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected expression after comma".to_string(),
                        line: self.current.as_ref().map_or(line, |t| t.line),
                        column: self.current.as_ref().map_or(column, |t| t.column),
                    });
                }
    
                elts.push(Box::new(self.parse_or_test()?));
    
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
    
            expr = Expr::Tuple {
                elts,
                ctx: ExprContext::Load,
                line,
                column,
            };
        }
    
        Ok(expr)
    }
    
    fn parse_or_test(&mut self) -> Result<Expr, ParseError> {
        // Special handling for starred expressions
        if self.check(TokenType::Multiply) {
            let star_token = self.current.clone().unwrap();
            self.advance(); // Consume the * token
            
            // Parse the expression after the star
            let value = Box::new(self.parse_or_test()?);
            
            // Create a Starred expression
            return Ok(Expr::Starred {
                value,
                ctx: ExprContext::Load,
                line: star_token.line,
                column: star_token.column,
            });
        }
    
        // Original code follows
        let mut expr = self.parse_and_test()?;
    
        if self.check(TokenType::Walrus) {
            let line = expr.get_line();
            let column = expr.get_column();
    
            match &expr {
                Expr::Name { .. } => {
                    self.advance();
    
                    let value = Box::new(self.parse_or_test()?);
    
                    expr = Expr::NamedExpr {
                        target: Box::new(expr),
                        value,
                        line,
                        column,
                    };
                }
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: "Invalid target for walrus operator".to_string(),
                        line,
                        column,
                    });
                }
            }
        }
    
        if self.check(TokenType::Or) {
            let line = expr.get_line();
            let column = expr.get_column();
    
            let mut values = vec![Box::new(expr)];
    
            while self.match_token(TokenType::Or) {
                values.push(Box::new(self.parse_and_test()?));
            }
    
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

        if self.check(TokenType::And) {
            let line = expr.get_line();
            let column = expr.get_column();

            let mut values = vec![Box::new(expr)];

            while self.match_token(TokenType::And) {
                values.push(Box::new(self.parse_not_test()?));
            }

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

        let mut ops = Vec::new();
        let mut comparators = Vec::new();

        while self.is_comparison_operator() {
            let op = self.parse_comparison_operator()?;
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
    
    /// Check if the current token is a comparison operator
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
        let token_type = self.current.as_ref().unwrap().token_type.clone();
        let line = self.current.as_ref().unwrap().line;
        let column = self.current.as_ref().unwrap().column;

        match token_type {
            TokenType::Equal => {
                self.advance();
                Ok(CmpOperator::Eq)
            }
            TokenType::NotEqual => {
                self.advance();
                Ok(CmpOperator::NotEq)
            }
            TokenType::LessThan => {
                self.advance();
                Ok(CmpOperator::Lt)
            }
            TokenType::LessEqual => {
                self.advance();
                Ok(CmpOperator::LtE)
            }
            TokenType::GreaterThan => {
                self.advance();
                Ok(CmpOperator::Gt)
            }
            TokenType::GreaterEqual => {
                self.advance();
                Ok(CmpOperator::GtE)
            }
            TokenType::Is => {
                self.advance();

                if self.match_token(TokenType::Not) {
                    Ok(CmpOperator::IsNot)
                } else {
                    Ok(CmpOperator::Is)
                }
            }
            TokenType::In => {
                self.advance();
                Ok(CmpOperator::In)
            }
            TokenType::Not => {
                self.advance();

                if self.match_token(TokenType::In) {
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

            if (token.token_type == TokenType::Plus && self.check(TokenType::Plus))
                || (token.token_type == TokenType::Minus && self.check(TokenType::Minus))
            {
                return Err(ParseError::InvalidSyntax {
                    message: "Invalid syntax: consecutive operators".to_string(),
                    line: token.line,
                    column: token.column,
                });
            }

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

            if self.check(TokenType::Multiply)
                || self.check(TokenType::Divide)
                || self.check(TokenType::FloorDivide)
                || self.check(TokenType::Modulo)
                || self.check(TokenType::Plus)
                || self.check(TokenType::Minus)
                || self.check(TokenType::At)
            {
                return Err(ParseError::InvalidSyntax {
                    message: "Invalid syntax: consecutive operators".to_string(),
                    line: token.line,
                    column: token.column + token.lexeme.len(),
                });
            }

            if self.check(TokenType::EOF) || self.check_newline() {
                return Err(ParseError::InvalidSyntax {
                    message: "Incomplete expression".to_string(),
                    line: token.line,
                    column: token.column + 1,
                });
            }

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
        let mut expr = self.parse_await_expr()?;

        if self.match_token(TokenType::Power) {
            let token = self.previous_token();

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

            if !self.is_in_context(ParserContext::Function) {
                return Err(ParseError::InvalidSyntax {
                    message: "Yield statement outside of function".to_string(),
                    line,
                    column,
                });
            }

            if self.match_token(TokenType::From) {
                let value = Box::new(self.parse_expression()?);
                return Ok(Expr::YieldFrom {
                    value,
                    line,
                    column,
                });
            }

            let value = if self.check_newline()
                || self.check(TokenType::RightParen)
                || self.check(TokenType::Comma)
                || self.check(TokenType::Colon)
                || self.check(TokenType::EOF)
                || self.check(TokenType::Dedent)
            {
                None
            } else {
                Some(Box::new(self.parse_expression()?))
            };

            Ok(Expr::Yield {
                value,
                line,
                column,
            })
        } else {
            Err(ParseError::InvalidSyntax {
                message: "Expected 'yield' keyword".to_string(),
                line: self.current.as_ref().map_or(0, |t| t.line),
                column: self.current.as_ref().map_or(0, |t| t.column),
            })
        }
    }
    
    fn parse_more_arguments(
        &mut self,
    ) -> Result<(Vec<Box<Expr>>, Vec<(Option<String>, Box<Expr>)>), ParseError> {
        let mut args = Vec::new();
        let mut keywords = Vec::new();
        let mut saw_keyword = false;

        loop {
            if self.match_token(TokenType::Multiply) {
                let token = self.previous_token();
                let value = Box::new(self.parse_or_test()?);

                args.push(Box::new(Expr::Starred {
                    value,
                    ctx: ExprContext::Load,
                    line: token.line,
                    column: token.column,
                }));
                saw_keyword = true;
            } else if self.match_token(TokenType::Power) {
                let arg = Box::new(self.parse_or_test()?);
                keywords.push((None, arg));
                saw_keyword = true;
            } else if self.check_identifier() {
                let id_token = self.current.clone().unwrap();
                let id_line = id_token.line;
                let id_column = id_token.column;
                let id_name = match &id_token.token_type {
                    TokenType::Identifier(name) => name.clone(),
                    _ => unreachable!(),
                };

                self.advance();

                let is_keyword = self.check(TokenType::Assign);

                if is_keyword {
                    self.advance();

                    let value = Box::new(self.parse_or_test()?);
                    keywords.push((Some(id_name), value));
                    saw_keyword = true;
                } else if !saw_keyword {
                    let name_expr = Expr::Name {
                        id: id_name,
                        ctx: ExprContext::Load,
                        line: id_line,
                        column: id_column,
                    };

                    args.push(Box::new(name_expr));
                } else {
                    return Err(ParseError::InvalidSyntax {
                        message: "Positional argument after keyword argument".to_string(),
                        line: id_line,
                        column: id_column,
                    });
                }
            } else if !saw_keyword {
                args.push(Box::new(self.parse_or_test()?));
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

            if self.check(TokenType::RightParen) {
                break;
            }

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
    
    fn parse_atom_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_atom()?;

        loop {
            if self.match_token(TokenType::LeftParen) {
                let line = expr.get_line();
                let column = expr.get_column();

                if self.match_token(TokenType::RightParen) {
                    expr = Expr::Call {
                        func: Box::new(expr),
                        args: Vec::new(),
                        keywords: Vec::new(),
                        line,
                        column,
                    };
                    continue;
                }

                if self.match_token(TokenType::Multiply) {
                    let star_token = self.previous_token();
                    let value = Box::new(self.parse_or_test()?);

                    let mut args = vec![Box::new(Expr::Starred {
                        value,
                        ctx: ExprContext::Load,
                        line: star_token.line,
                        column: star_token.column,
                    })];
                    let mut keywords = Vec::new();

                    if self.match_token(TokenType::Comma) {
                        if !self.check(TokenType::RightParen) {
                            let (more_args, kw_args) = self.parse_more_arguments()?;
                            args.extend(more_args);
                            keywords.extend(kw_args);
                        }
                    }

                    self.consume(TokenType::RightParen, ")")?;

                    expr = Expr::Call {
                        func: Box::new(expr),
                        args,
                        keywords,
                        line,
                        column,
                    };
                } else if self.match_token(TokenType::Power) {
                    let _star_token = self.previous_token();
                    let value = Box::new(self.parse_or_test()?);

                    let args = Vec::new();
                    let mut keywords = vec![(None, value)];

                    if self.match_token(TokenType::Comma) {
                        if !self.check(TokenType::RightParen) {
                            let (_more_args, kw_args) = self.parse_more_arguments()?;
                            keywords.extend(kw_args);
                        }
                    }

                    self.consume(TokenType::RightParen, ")")?;

                    expr = Expr::Call {
                        func: Box::new(expr),
                        args,
                        keywords,
                        line,
                        column,
                    };
                } else {
                    let first_arg = self.parse_or_test()?;

                    if self.check(TokenType::Assign) && matches!(&first_arg, Expr::Name { .. }) {
                        if let Expr::Name { id, .. } = first_arg {
                            self.advance();
                            let value = Box::new(self.parse_or_test()?);

                            let mut args = Vec::new();
                            let mut keywords = vec![(Some(id), value)];

                            if self.match_token(TokenType::Comma) {
                                if !self.check(TokenType::RightParen) {
                                    let (more_args, kw_args) = self.parse_more_arguments()?;
                                    args.extend(more_args);
                                    keywords.extend(kw_args);
                                }
                            }

                            self.consume(TokenType::RightParen, ")")?;

                            expr = Expr::Call {
                                func: Box::new(expr),
                                args,
                                keywords,
                                line,
                                column,
                            };
                        } else {
                            unreachable!("We already checked this is a Name expression");
                        }
                    } else {
                        let mut args = vec![Box::new(first_arg)];
                        let mut keywords = Vec::new();

                        if self.match_token(TokenType::Comma) {
                            if !self.check(TokenType::RightParen) {
                                let (more_args, kw_args) = self.parse_more_arguments()?;
                                args.extend(more_args);
                                keywords = kw_args;
                            }
                        }

                        self.consume(TokenType::RightParen, ")")?;

                        expr = Expr::Call {
                            func: Box::new(expr),
                            args,
                            keywords,
                            line,
                            column,
                        };
                    }
                }
            } else if self.match_token(TokenType::Dot) {
                let line = expr.get_line();
                let column = expr.get_column();
                let attr = self.consume_attribute_name("attribute name")?;

                expr = Expr::Attribute {
                    value: Box::new(expr),
                    attr,
                    ctx: ExprContext::Load,
                    line,
                    column,
                };
            } else if self.match_token(TokenType::LeftBracket) {
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
    
    fn parse_slice(&mut self) -> Result<Expr, ParseError> {
        let line = self.current.as_ref().map_or(0, |t| t.line);
        let column = self.current.as_ref().map_or(0, |t| t.column);

        if self.match_token(TokenType::Ellipsis) {
            let ellipsis_expr = Expr::Ellipsis { line, column };

            if self.match_token(TokenType::Comma) {
                let mut indices = vec![Box::new(ellipsis_expr)];

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

            return Ok(ellipsis_expr);
        }

        let start_expr = if !self.check(TokenType::Colon) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        if self.match_token(TokenType::Colon) {
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

            if self.match_token(TokenType::Comma) {
                let mut indices = vec![Box::new(slice)];

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
            start_expr.ok_or_else(|| ParseError::InvalidSyntax {
                message: "Expected expression in subscription".to_string(),
                line,
                column,
            })
        }
    }
    
    fn parse_type_annotation(&mut self, _is_nested: bool) -> Result<Expr, ParseError> {
        let mut expr = self.parse_atom_expr()?;

        if self.match_token(TokenType::LeftBracket) {
            let line = expr.get_line();
            let column = expr.get_column();

            let mut params = Vec::new();

            if !self.check(TokenType::RightBracket) {
                params.push(Box::new(self.parse_type_annotation(true)?));

                while self.match_token(TokenType::Comma) {
                    if self.check(TokenType::RightBracket) {
                        break;
                    }
                    params.push(Box::new(self.parse_type_annotation(true)?));
                }
            }

            self.consume(TokenType::RightBracket, "]")?;

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

    fn parse_lambda_parameters(&mut self) -> Result<Vec<crate::ast::Parameter>, ParseError> {
        let mut params = Vec::new();
        let mut has_seen_default = false;
        let mut has_vararg = false;
        let mut has_kwarg = false;

        if self.check(TokenType::Colon) {
            return Ok(params);
        }

        if self.match_token(TokenType::Multiply) {
            let name = self.consume_identifier("parameter name after *")?;
            params.push(crate::ast::Parameter {
                name,
                typ: None,
                default: None,
                is_vararg: true,
                is_kwarg: false,
            });
            has_vararg = true;
        } else if self.match_token(TokenType::Power) {
            let name = self.consume_identifier("parameter name after **")?;
            params.push(crate::ast::Parameter {
                name,
                typ: None,
                default: None,
                is_vararg: false,
                is_kwarg: true,
            });
            has_kwarg = true;
        } else if self.check_identifier() {
            let param_name = self.consume_identifier("parameter name")?;

            let default = if self.match_token(TokenType::Assign) {
                has_seen_default = true;
                Some(Box::new(self.parse_or_test()?))
            } else {
                None
            };

            params.push(crate::ast::Parameter {
                name: param_name,
                typ: None,
                default,
                is_vararg: false,
                is_kwarg: false,
            });
        } else {
            return Err(ParseError::InvalidSyntax {
                message: "Expected parameter name".to_string(),
                line: self.current.as_ref().map_or(0, |t| t.line),
                column: self.current.as_ref().map_or(0, |t| t.column),
            });
        }

        while self.match_token(TokenType::Comma) {
            if self.check(TokenType::Colon) {
                return Err(ParseError::InvalidSyntax {
                    message: "Expected parameter after comma".to_string(),
                    line: self.current.as_ref().map_or(0, |t| t.line),
                    column: self.current.as_ref().map_or(0, |t| t.column),
                });
            }

            if self.match_token(TokenType::Multiply) {
                let name = self.consume_identifier("parameter name after *")?;
                params.push(crate::ast::Parameter {
                    name,
                    typ: None,
                    default: None,
                    is_vararg: true,
                    is_kwarg: false,
                });
                has_vararg = true;
            } else if self.match_token(TokenType::Power) {
                let name = self.consume_identifier("parameter name after **")?;
                params.push(crate::ast::Parameter {
                    name,
                    typ: None,
                    default: None,
                    is_vararg: false,
                    is_kwarg: true,
                });
                has_kwarg = true;
            } else if self.check_identifier() {
                let param_pos = (
                    self.current.as_ref().map_or(0, |t| t.line),
                    self.current.as_ref().map_or(0, |t| t.column),
                );
                let param_name = self.consume_identifier("parameter name")?;

                let default = if self.match_token(TokenType::Assign) {
                    has_seen_default = true;
                    Some(Box::new(self.parse_or_test()?))
                } else {
                    if has_seen_default && !has_vararg && !has_kwarg {
                        println!(
                            "Warning: non-default parameter after default parameter at line {}, column {}",
                            param_pos.0, param_pos.1
                        );
                    }
                    None
                };

                params.push(crate::ast::Parameter {
                    name: param_name,
                    typ: None,
                    default,
                    is_vararg: false,
                    is_kwarg: false,
                });
            } else {
                return Err(ParseError::InvalidSyntax {
                    message: "Expected parameter name".to_string(),
                    line: self.current.as_ref().map_or(0, |t| t.line),
                    column: self.current.as_ref().map_or(0, |t| t.column),
                });
            }
        }

        Ok(params)
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
            TokenType::Identifier(name) => {
                self.advance();
                Ok(Expr::Name {
                    id: name.clone(),
                    ctx: ExprContext::Load,
                    line,
                    column,
                })
            }
            TokenType::Yield => {
                return self.parse_yield_expr();
            }
            TokenType::LeftParen => {
                self.advance();
    
                if self.match_token(TokenType::RightParen) {
                    if !self.is_in_context(ParserContext::Comprehension) {
                        return Ok(Expr::Tuple {
                            elts: Vec::new(),
                            ctx: ExprContext::Load,
                            line,
                            column,
                        });
                    }
                    Ok(Expr::Tuple {
                        elts: Vec::new(),
                        ctx: ExprContext::Load,
                        line,
                        column,
                    })
                } else {
                    let expr = self.parse_expression()?;
    
                    if self.match_token(TokenType::For) {
                        let elt = expr;
    
                        return self.with_context(ParserContext::Comprehension, |this| {
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
    
                            while this.match_token(TokenType::For)
                                || (this.check(TokenType::Async)
                                    && this.peek_matches(TokenType::For))
                            {
                                let is_async = if this.check(TokenType::Async) {
                                    this.advance();
                                    this.consume(TokenType::For, "for")?;
                                    true
                                } else {
                                    false
                                };
    
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
                                    is_async,
                                });
                            }
    
                            this.consume(TokenType::RightParen, ")")?;
    
                            Ok(Expr::GeneratorExp {
                                elt: Box::new(elt),
                                generators,
                                line,
                                column,
                            })
                        });
                    } else if self.check(TokenType::Async) && self.peek_matches(TokenType::For) {
                        let elt = expr;
    
                        return self.with_context(ParserContext::Comprehension, |this| {
                            let mut generators = Vec::new();
    
                            this.advance();
                            this.consume(TokenType::For, "for")?;
    
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
                                is_async: true,
                            });
    
                            while this.match_token(TokenType::For)
                                || (this.check(TokenType::Async)
                                    && this.peek_matches(TokenType::For))
                            {
                                let is_async = if this.check(TokenType::Async) {
                                    this.advance();
                                    this.consume(TokenType::For, "for")?;
                                    true
                                } else {
                                    false
                                };
    
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
                                    is_async,
                                });
                            }
    
                            this.consume(TokenType::RightParen, ")")?;
    
                            Ok(Expr::GeneratorExp {
                                elt: Box::new(elt),
                                generators,
                                line,
                                column,
                            })
                        });
                    } else if self.match_token(TokenType::Comma) {
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
                        self.consume(TokenType::RightParen, ")")?;
                        Ok(expr)
                    }
                }
            }
            TokenType::LeftBracket => {
                self.advance();
            
                if self.check(TokenType::EOF) || self.check_newline() {
                    return Err(ParseError::InvalidSyntax {
                        message: "Unclosed bracket".to_string(),
                        line,
                        column,
                    });
                }
            
                if self.match_token(TokenType::RightBracket) {
                    Ok(Expr::List {
                        elts: Vec::new(),
                        ctx: ExprContext::Load,
                        line,
                        column,
                    })
                } else {
                    // Special handling for lists starting with *
                    if self.check(TokenType::Multiply) {
                        self.advance(); // Consume the * token
                        let star_token = self.previous_token();
                        
                        // Parse the value after the * token
                        let value = Box::new(self.parse_atom_expr()?);
                        
                        // Create a Starred expression
                        let starred_expr = Expr::Starred {
                            value,
                            ctx: if self.is_in_context(ParserContext::Match) {
                                ExprContext::Store
                            } else {
                                ExprContext::Load
                            },
                            line: star_token.line,
                            column: star_token.column,
                        };
                        
                        // Start building the list with our starred element
                        let mut elts = vec![Box::new(starred_expr)];
                        
                        // Handle any remaining list elements
                        if self.match_token(TokenType::Comma) {
                            if !self.check(TokenType::RightBracket) {
                                elts.extend(self.parse_expr_list()?);
                            }
                        }
                        
                        self.consume(TokenType::RightBracket, "]")?;
                        
                        Ok(Expr::List {
                            elts,
                            ctx: if self.is_in_context(ParserContext::Match) {
                                ExprContext::Store
                            } else {
                                ExprContext::Load
                            },
                            line,
                            column,
                        })
                    } else {
                        let first_expr = self.parse_expression()?;
                
                        if self.match_token(TokenType::For) {
                            return self.with_context(ParserContext::Comprehension, |this| {
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
                
                                while this.match_token(TokenType::For)
                                    || (this.check(TokenType::Async)
                                        && this.peek_matches(TokenType::For))
                                {
                                    let is_async = if this.check(TokenType::Async) {
                                        this.advance();
                                        this.consume(TokenType::For, "for")?;
                                        true
                                    } else {
                                        false
                                    };
                
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
                                        is_async,
                                    });
                                }
                
                                this.consume(TokenType::RightBracket, "]")?;
                
                                Ok(Expr::ListComp {
                                    elt: Box::new(first_expr),
                                    generators,
                                    line,
                                    column,
                                })
                            });
                        } else if self.check(TokenType::Async) && self.peek_matches(TokenType::For) {
                            return self.with_context(ParserContext::Comprehension, |this| {
                                let mut generators = Vec::new();
                
                                this.advance();
                                this.consume(TokenType::For, "for")?;
                
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
                                    is_async: true,
                                });
                
                                while this.match_token(TokenType::For)
                                    || (this.check(TokenType::Async)
                                        && this.peek_matches(TokenType::For))
                                {
                                    let is_async = if this.check(TokenType::Async) {
                                        this.advance();
                                        this.consume(TokenType::For, "for")?;
                                        true
                                    } else {
                                        false
                                    };
                
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
                                        is_async,
                                    });
                                }
                
                                this.consume(TokenType::RightBracket, "]")?;
                
                                Ok(Expr::ListComp {
                                    elt: Box::new(first_expr),
                                    generators,
                                    line,
                                    column,
                                })
                            });
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
                }
            }
            TokenType::LeftBrace => {
                self.advance();
    
                if self.check(TokenType::EOF) || self.check_newline() {
                    return Err(ParseError::InvalidSyntax {
                        message: "Unclosed brace".to_string(),
                        line,
                        column,
                    });
                }
    
                self.parse_dict_literal(line, column)
            }
            TokenType::IntLiteral(value) => {
                self.advance();
                Ok(Expr::Num {
                    value: Number::Integer(*value),
                    line,
                    column,
                })
            }
            TokenType::FloatLiteral(value) => {
                self.advance();
                Ok(Expr::Num {
                    value: Number::Float(*value),
                    line,
                    column,
                })
            }
            TokenType::Lambda => {
                self.advance();
                let line_start = line;
                let column_start = column;
    
                let params = self.parse_lambda_parameters()?;
    
                self.consume(TokenType::Colon, ":")?;
    
                let body = Box::new(self.parse_expression()?);
    
                Ok(Expr::Lambda {
                    args: params,
                    body,
                    line: line_start,
                    column: column_start,
                })
            }
            TokenType::FString(value) => {
                self.advance();
                Ok(Expr::Str {
                    value: value.clone(),
                    line,
                    column,
                })
            }
            TokenType::RawString(value) => {
                self.advance();
                Ok(Expr::Str {
                    value: value.clone(),
                    line,
                    column,
                })
            }
            TokenType::BytesLiteral(bytes) => {
                self.advance();
                Ok(Expr::Bytes {
                    value: bytes.clone(),
                    line,
                    column,
                })
            }
            TokenType::StringLiteral(value) => {
                self.advance();
                Ok(Expr::Str {
                    value: value.clone(),
                    line,
                    column,
                })
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::NameConstant {
                    value: NameConstant::True,
                    line,
                    column,
                })
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::NameConstant {
                    value: NameConstant::False,
                    line,
                    column,
                })
            }
            TokenType::None => {
                self.advance();
                Ok(Expr::NameConstant {
                    value: NameConstant::None,
                    line,
                    column,
                })
            }
            TokenType::Ellipsis => {
                self.advance();
                Ok(Expr::Ellipsis { line, column })
            }
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
    
        if self.check(TokenType::RightParen)
            || self.check(TokenType::RightBracket)
            || self.check(TokenType::RightBrace)
            || self.check(TokenType::Assign)
            || self.check_newline()
            || self.check(TokenType::EOF)
        {
            return Ok(expressions);
        }
    
        // Add handling for starred expressions here
        if self.match_token(TokenType::Multiply) {
            let token = self.previous_token();
            let line = token.line;
            let column = token.column;
    
            // Parse the expression after the star
            let value = Box::new(self.parse_or_test()?);
    
            expressions.push(Box::new(Expr::Starred {
                value,
                ctx: ExprContext::Load,
                line,
                column,
            }));
        } else {
            expressions.push(Box::new(self.parse_expression()?));
        }
    
        while self.match_token(TokenType::Comma) {
            if self.check(TokenType::RightParen)
                || self.check(TokenType::RightBracket)
                || self.check(TokenType::RightBrace)
                || self.check(TokenType::Assign)
                || self.check_newline()
                || self.check(TokenType::EOF)
            {
                break;
            }
    
            if self.check(TokenType::Comma) {
                return Err(ParseError::InvalidSyntax {
                    message: "Expected expression after comma".to_string(),
                    line: self.current.as_ref().map_or(0, |t| t.line),
                    column: self.current.as_ref().map_or(0, |t| t.column),
                });
            }
    
            // Add handling for starred expressions here
            if self.match_token(TokenType::Multiply) {
                let token = self.previous_token();
                let line = token.line;
                let column = token.column;
    
                // Parse the expression after the star
                let value = Box::new(self.parse_or_test()?);
    
                expressions.push(Box::new(Expr::Starred {
                    value,
                    ctx: ExprContext::Load,
                    line,
                    column,
                }));
            } else {
                expressions.push(Box::new(self.parse_expression()?));
            }
        }
    
        Ok(expressions)
    }
    
    fn parse_dict_literal(&mut self, line: usize, column: usize) -> Result<Expr, ParseError> {
        if self.match_token(TokenType::RightBrace) {
            return Ok(Expr::Dict {
                keys: Vec::new(),
                values: Vec::new(),
                line,
                column,
            });
        }

        let mut keys = Vec::new();
        let mut values = Vec::new();

        if self.match_token(TokenType::Power) {
            let value = Box::new(self.parse_or_test()?);
            keys.push(None);
            values.push(value);
        } else {
            let first_expr = self.parse_or_test()?;

            if self.match_token(TokenType::Colon) {
                keys.push(Some(Box::new(first_expr)));
                let first_value = Box::new(self.parse_or_test()?);
                values.push(first_value);

                if self.match_token(TokenType::For) {
                    return self.with_context(ParserContext::Comprehension, |this| {
                        let key = keys[0].as_ref().unwrap().clone();
                        let value = values[0].clone();

                        let mut generators = Vec::new();

                        let target = if this.check_identifier() && this.peek_matches(TokenType::Comma) {
                            // Handle tuple target (e.g., "k, v")
                            let id_line = this.current.as_ref().unwrap().line;
                            let id_column = this.current.as_ref().unwrap().column;
                        
                            let first_name = this.consume_identifier("identifier")?;
                            let first_expr = Expr::Name {
                                id: first_name,
                                ctx: ExprContext::Store,
                                line: id_line,
                                column: id_column,
                            };
                        
                            this.advance(); // Move past the comma
                        
                            let mut elts = vec![Box::new(first_expr)];
                        
                            while !this.check(TokenType::In) {
                                if this.check(TokenType::Comma) {
                                    return Err(ParseError::InvalidSyntax {
                                        message: "Expected identifier after comma".to_string(),
                                        line: this.current.as_ref().unwrap().line,
                                        column: this.current.as_ref().unwrap().column,
                                    });
                                }
                        
                                if this.check_identifier() {
                                    let next_line = this.current.as_ref().unwrap().line;
                                    let next_column = this.current.as_ref().unwrap().column;
                                    let next_name = this.consume_identifier("identifier")?;
                        
                                    elts.push(Box::new(Expr::Name {
                                        id: next_name,
                                        ctx: ExprContext::Store,
                                        line: next_line,
                                        column: next_column,
                                    }));
                                } else {
                                    return Err(ParseError::InvalidSyntax {
                                        message: "Expected identifier in for loop target".to_string(),
                                        line: this.current.as_ref().unwrap().line,
                                        column: this.current.as_ref().unwrap().column,
                                    });
                                }
                        
                                if !this.match_token(TokenType::Comma) {
                                    break;
                                }
                            }
                        
                            Box::new(Expr::Tuple {
                                elts,
                                ctx: ExprContext::Store,
                                line: id_line,
                                column: id_column,
                            })
                        } else {
                            // Handle single identifier target
                            let expr = this.parse_atom_expr()?;
                            Box::new(this.with_store_context(expr)?)
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
                            is_async: false,
                        });

                        while this.match_token(TokenType::For)
                            || (this.check(TokenType::Async) && this.peek_matches(TokenType::For))
                        {
                            let is_async = if this.check(TokenType::Async) {
                                this.advance();
                                this.consume(TokenType::For, "for")?;
                                true
                            } else {
                                false
                            };

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
                                is_async,
                            });
                        }

                        this.consume(TokenType::RightBrace, "}")?;

                        Ok(Expr::DictComp {
                            key,
                            value,
                            generators,
                            line,
                            column,
                        })
                    });
                } else if self.check(TokenType::Async) && self.peek_matches(TokenType::For) {
                    return self.with_context(ParserContext::Comprehension, |this| {
                        let key = keys[0].as_ref().unwrap().clone();
                        let value = values[0].clone();

                        let mut generators = Vec::new();

                        this.advance();
                        this.consume(TokenType::For, "for")?;

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
                            is_async: true,
                        });

                        while this.match_token(TokenType::For)
                            || (this.check(TokenType::Async) && this.peek_matches(TokenType::For))
                        {
                            let is_async = if this.check(TokenType::Async) {
                                this.advance();
                                this.consume(TokenType::For, "for")?;
                                true
                            } else {
                                false
                            };

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
                                is_async,
                            });
                        }

                        this.consume(TokenType::RightBrace, "}")?;

                        Ok(Expr::DictComp {
                            key,
                            value,
                            generators,
                            line,
                            column,
                        })
                    });
                }
            } else if self.match_token(TokenType::For) {
                return self.with_context(ParserContext::Comprehension, |this| {
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

                    while this.match_token(TokenType::For)
                        || (this.check(TokenType::Async) && this.peek_matches(TokenType::For))
                    {
                        let is_async = if this.check(TokenType::Async) {
                            this.advance();
                            this.consume(TokenType::For, "for")?;
                            true
                        } else {
                            false
                        };

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
                            is_async,
                        });
                    }

                    this.consume(TokenType::RightBrace, "}")?;

                    Ok(Expr::SetComp {
                        elt: Box::new(first_expr),
                        generators,
                        line,
                        column,
                    })
                });
            } else if self.check(TokenType::Async) && self.peek_matches(TokenType::For) {
                return self.with_context(ParserContext::Comprehension, |this| {
                    let mut generators = Vec::new();

                    this.advance();
                    this.consume(TokenType::For, "for")?;

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
                        is_async: true,
                    });

                    while this.match_token(TokenType::For)
                        || (this.check(TokenType::Async) && this.peek_matches(TokenType::For))
                    {
                        let is_async = if this.check(TokenType::Async) {
                            this.advance();
                            this.consume(TokenType::For, "for")?;
                            true
                        } else {
                            false
                        };

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
                            is_async,
                        });
                    }

                    this.consume(TokenType::RightBrace, "}")?;

                    Ok(Expr::SetComp {
                        elt: Box::new(first_expr),
                        generators,
                        line,
                        column,
                    })
                });
            } else {
                let mut elts = vec![Box::new(first_expr)];

                while self.match_token(TokenType::Comma) {
                    if self.check(TokenType::RightBrace) {
                        break;
                    }

                    if self.match_token(TokenType::Power) {
                        let value = Box::new(self.parse_or_test()?);
                        keys.push(None);
                        values.push(value);

                        while self.match_token(TokenType::Comma) {
                            if self.check(TokenType::RightBrace) {
                                break;
                            }

                            if self.match_token(TokenType::Power) {
                                let value = Box::new(self.parse_or_test()?);
                                keys.push(None);
                                values.push(value);
                            } else {
                                let key = self.parse_or_test()?;

                                if !self.match_token(TokenType::Colon) {
                                    return Err(ParseError::InvalidSyntax {
                                        message: "Expected ':' after dictionary key".to_string(),
                                        line: self.current.as_ref().map_or(line, |t| t.line),
                                        column: self.current.as_ref().map_or(column, |t| t.column),
                                    });
                                }

                                let value = self.parse_or_test()?;
                                keys.push(Some(Box::new(key)));
                                values.push(Box::new(value));
                            }
                        }

                        self.consume(TokenType::RightBrace, "}")?;

                        return Ok(Expr::Dict {
                            keys,
                            values,
                            line,
                            column,
                        });
                    } else {
                        elts.push(Box::new(self.parse_or_test()?));
                    }
                }

                self.consume(TokenType::RightBrace, "}")?;

                return Ok(Expr::Set { elts, line, column });
            }
        }

        while self.match_token(TokenType::Comma) {
            if self.check(TokenType::RightBrace) {
                break;
            }

            if self.match_token(TokenType::Power) {
                let value = Box::new(self.parse_or_test()?);
                keys.push(None);
                values.push(value);
            } else {
                let key = self.parse_or_test()?;

                self.consume(TokenType::Colon, ":")?;

                let value = self.parse_or_test()?;
                keys.push(Some(Box::new(key)));
                values.push(Box::new(value));
            }
        }

        self.consume(TokenType::RightBrace, "}")?;

        Ok(Expr::Dict {
            keys,
            values,
            line,
            column,
        })
    }
}