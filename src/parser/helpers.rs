use crate::lexer::{Token, TokenType};
use crate::parser::error::ParseError;
use crate::parser::Parser;

/// Common error messages
#[allow(dead_code)]
pub const ERR_EXPECTED_IDENTIFIER: &str = "Expected identifier";
#[allow(dead_code)]
pub const ERR_EXPECTED_COLON: &str = "Expected ':'";
#[allow(dead_code)]
pub const ERR_EXPECTED_PAREN: &str = "Expected '('";
pub const ERR_UNCLOSED_PAREN: &str = "Unclosed parenthesis";
pub const ERR_UNCLOSED_BRACKET: &str = "Unclosed bracket";
pub const ERR_UNCLOSED_BRACE: &str = "Unclosed brace";
#[allow(dead_code)]
pub const ERR_EXPECTED_EQUAL: &str = "Expected '='";
#[allow(dead_code)]
pub const ERR_EXPECTED_COMMA: &str = "Expected ','";
#[allow(dead_code)]
pub const ERR_EXPECTED_EXPRESSION: &str = "Expected expression";
#[allow(dead_code)]
pub const ERR_EXPECTED_DOT: &str = "Expected '.'";
#[allow(dead_code)]
pub const ERR_EXPECTED_NAME: &str = "Expected name";
#[allow(dead_code)]
pub const ERR_EXPECTED_NEWLINE: &str = "Expected newline";

/// Trait for token matching and consumption
pub trait TokenMatching {
    /// Check if the current token matches the expected type
    fn check(&self, expected_type: TokenType) -> bool;

    /// Check if the current token is one of several types
    #[allow(dead_code)]
    fn check_any(&self, types: &[TokenType]) -> bool;

    /// Match and consume a token if it's the expected type
    fn match_token(&mut self, expected_type: TokenType) -> bool;

    /// Peek at the next token
    fn peek_matches(&self, expected_type: TokenType) -> bool;

    /// Expect and consume a token of the given type, or return an error
    #[allow(dead_code)]
    fn expect(
        &mut self,
        expected_type: TokenType,
        error_message: &str,
    ) -> Result<Token, ParseError>;

    fn consume_attribute_name(&mut self, expected: &str) -> Result<String, ParseError>;

    fn get_keyword_name(&self, token_type: &TokenType) -> String;

    /// Consume a token of the given type, or return an error
    fn consume(
        &mut self,
        expected_type: TokenType,
        error_message: &str,
    ) -> Result<Token, ParseError>;

    /// Consume a newline token
    fn consume_newline(&mut self) -> Result<(), ParseError>;

    fn is_keyword_token(&self) -> bool;

    /// Check if the current token is a newline
    fn check_newline(&self) -> bool;

    /// Consume an identifier token
    fn consume_identifier(&mut self, expected: &str) -> Result<String, ParseError>;

    /// Consume a dotted name (like module.submodule)
    fn consume_dotted_name(&mut self, expected: &str) -> Result<String, ParseError>;

    /// Create a syntax error at the current position
    fn syntax_error<T>(&self, message: &str) -> Result<T, ParseError>;

    /// Create an EOF error
    fn unexpected_eof<T>(&self, expected: &str) -> Result<T, ParseError>;

    /// Token matching utility function
    fn token_matches(&self, a: &TokenType, b: &TokenType) -> bool;
}

impl TokenMatching for Parser {
    fn check(&self, expected_type: TokenType) -> bool {
        match &self.current {
            Some(token) => self.token_matches(&token.token_type, &expected_type),
            None => false,
        }
    }

    fn check_any(&self, types: &[TokenType]) -> bool {
        types.iter().any(|t| self.check(t.clone()))
    }

    fn match_token(&mut self, expected_type: TokenType) -> bool {
        if self.check(expected_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek_matches(&self, expected_type: TokenType) -> bool {
        if let Some(token) = self.tokens.front() {
            self.token_matches(&token.token_type, &expected_type)
        } else {
            false
        }
    }

    fn expect(
        &mut self,
        expected_type: TokenType,
        error_message: &str,
    ) -> Result<Token, ParseError> {
        if self.check(expected_type) {
            Ok(self.advance().unwrap())
        } else {
            self.syntax_error(error_message)
        }
    }

    fn get_keyword_name(&self, token_type: &TokenType) -> String {
        match token_type {
            TokenType::And => "and".to_string(),
            TokenType::As => "as".to_string(),
            TokenType::Assert => "assert".to_string(),
            TokenType::Async => "async".to_string(),
            TokenType::Await => "await".to_string(),
            TokenType::Break => "break".to_string(),
            TokenType::Class => "class".to_string(),
            TokenType::Continue => "continue".to_string(),
            TokenType::Def => "def".to_string(),
            TokenType::Del => "del".to_string(),
            TokenType::Elif => "elif".to_string(),
            TokenType::Else => "else".to_string(),
            TokenType::Except => "except".to_string(),
            TokenType::Finally => "finally".to_string(),
            TokenType::For => "for".to_string(),
            TokenType::From => "from".to_string(),
            TokenType::Global => "global".to_string(),
            TokenType::If => "if".to_string(),
            TokenType::Import => "import".to_string(),
            TokenType::In => "in".to_string(),
            TokenType::Is => "is".to_string(),
            TokenType::Lambda => "lambda".to_string(),
            TokenType::None => "None".to_string(),
            TokenType::Nonlocal => "nonlocal".to_string(),
            TokenType::Not => "not".to_string(),
            TokenType::Or => "or".to_string(),
            TokenType::Pass => "pass".to_string(),
            TokenType::Raise => "raise".to_string(),
            TokenType::Return => "return".to_string(),
            TokenType::Try => "try".to_string(),
            TokenType::While => "while".to_string(),
            TokenType::With => "with".to_string(),
            TokenType::Yield => "yield".to_string(),
            TokenType::Match => "match".to_string(),
            TokenType::Case => "case".to_string(),
            TokenType::True => "True".to_string(),
            TokenType::False => "False".to_string(),
            _ => "unknown_keyword".to_string(),
        }
    }

    fn consume_attribute_name(&mut self, expected: &str) -> Result<String, ParseError> {
        match &self.current {
            Some(token) => match &token.token_type {
                TokenType::Identifier(name) => {
                    let result = name.clone();
                    self.advance();
                    Ok(result)
                }
                _ if self.is_keyword_token() => {
                    let keyword_name = self.get_keyword_name(&token.token_type);
                    self.advance();
                    Ok(keyword_name)
                }
                _ => Err(ParseError::unexpected_token(
                    expected,
                    token.token_type.clone(),
                    token.line,
                    token.column,
                )),
            },
            None => Err(ParseError::eof(
                expected,
                self.last_position().0,
                self.last_position().1,
            )),
        }
    }

    fn consume(
        &mut self,
        expected_type: TokenType,
        error_message: &str,
    ) -> Result<Token, ParseError> {
        match &self.current {
            Some(token) => {
                if matches!(expected_type, TokenType::RightParen)
                    && matches!(token.token_type, TokenType::Assign)
                    && self
                        .last_token
                        .as_ref()
                        .map_or(false, |t| matches!(t.token_type, TokenType::Identifier(_)))
                {
                    return Ok(token.clone());
                }

                if self.token_matches(&token.token_type, &expected_type) {
                    let result = token.clone();
                    self.advance();
                    Ok(result)
                } else {
                    let expected_str = match &expected_type {
                        TokenType::RightParen => ERR_UNCLOSED_PAREN,
                        TokenType::RightBracket => ERR_UNCLOSED_BRACKET,
                        TokenType::RightBrace => ERR_UNCLOSED_BRACE,
                        _ => error_message,
                    };

                    Err(ParseError::unexpected_token(
                        expected_str,
                        token.token_type.clone(),
                        token.line,
                        token.column,
                    ))
                }
            }
            None => self.unexpected_eof(error_message),
        }
    }

    fn is_keyword_token(&self) -> bool {
        match &self.current {
            Some(token) => matches!(
                token.token_type,
                TokenType::And
                    | TokenType::As
                    | TokenType::Assert
                    | TokenType::Async
                    | TokenType::Await
                    | TokenType::Break
                    | TokenType::Class
                    | TokenType::Continue
                    | TokenType::Def
                    | TokenType::Del
                    | TokenType::Elif
                    | TokenType::Else
                    | TokenType::Except
                    | TokenType::Finally
                    | TokenType::For
                    | TokenType::From
                    | TokenType::Global
                    | TokenType::If
                    | TokenType::Import
                    | TokenType::In
                    | TokenType::Is
                    | TokenType::Lambda
                    | TokenType::None
                    | TokenType::Nonlocal
                    | TokenType::Not
                    | TokenType::Or
                    | TokenType::Pass
                    | TokenType::Raise
                    | TokenType::Return
                    | TokenType::Try
                    | TokenType::While
                    | TokenType::With
                    | TokenType::Yield
                    | TokenType::Match
                    | TokenType::Case
            ),
            None => false,
        }
    }

    fn consume_newline(&mut self) -> Result<(), ParseError> {
        // Treat semicolons as statement terminators (like newlines)
        if self.match_token(TokenType::SemiColon) {
            // Skip any extra semicolons
            while self.match_token(TokenType::SemiColon) {}
            // Skip any actual newline tokens afterwards
            while self.match_token(TokenType::Newline) {}
            return Ok(());
        }

        // Otherwise we still require a real newline, EOF or dedent
        if !self.check_newline() && !self.check(TokenType::EOF) && !self.check(TokenType::Dedent) {
            if let Some(token) = &self.current {
                // If we're closing a bracket/paren, report that
                match token.token_type {
                    TokenType::RightParen => {
                        return Err(ParseError::unexpected_token(
                            "newline",
                            TokenType::RightParen,
                            token.line,
                            token.column,
                        ));
                    }
                    TokenType::RightBracket => {
                        return Err(ParseError::unexpected_token(
                            "newline",
                            TokenType::RightBracket,
                            token.line,
                            token.column,
                        ));
                    }
                    TokenType::RightBrace => {
                        return Err(ParseError::unexpected_token(
                            "newline",
                            TokenType::RightBrace,
                            token.line,
                            token.column,
                        ));
                    }
                    _ => {}
                }
            }
            return Err(ParseError::InvalidSyntax {
                message: "Expected newline after statement".to_string(),
                line: self.current.as_ref().map_or(0, |t| t.line),
                column: self.current.as_ref().map_or(0, |t| t.column),
                suggestion: None,
            });
        }

        // Finally, skip any trailing newlines
        while self.match_token(TokenType::Newline) {}
        Ok(())
    }

    fn check_newline(&self) -> bool {
        match &self.current {
            Some(token) => matches!(token.token_type, TokenType::Newline),
            None => false,
        }
    }

    fn consume_identifier(&mut self, expected: &str) -> Result<String, ParseError> {
        match &self.current {
            Some(token) => match &token.token_type {
                TokenType::Identifier(name) => {
                    let result = name.clone();
                    self.advance();
                    Ok(result)
                }
                _ => Err(ParseError::unexpected_token(
                    expected,
                    token.token_type.clone(),
                    token.line,
                    token.column,
                )),
            },
            None => Err(ParseError::eof(
                expected,
                self.last_position().0,
                self.last_position().1,
            )),
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

    fn syntax_error<T>(&self, message: &str) -> Result<T, ParseError> {
        let (line, column) = self.current_position();

        Err(ParseError::InvalidSyntax {
            message: message.to_string(),
            line,
            column,
            suggestion: None,
        })
    }

    fn unexpected_eof<T>(&self, expected: &str) -> Result<T, ParseError> {
        let (line, column) = self.last_position();

        Err(ParseError::EOF {
            expected: expected.to_string(),
            line,
            column,
            suggestion: None,
        })
    }

    fn token_matches(&self, actual: &TokenType, expected: &TokenType) -> bool {
        match (actual, expected) {
            (TokenType::Identifier(_), TokenType::Identifier(_)) => true,
            (TokenType::IntLiteral(_), TokenType::IntLiteral(_)) => true,
            (TokenType::FloatLiteral(_), TokenType::FloatLiteral(_)) => true,
            (TokenType::StringLiteral(_), TokenType::StringLiteral(_)) => true,
            (TokenType::FString(_), TokenType::FString(_)) => true,
            (TokenType::RawString(_), TokenType::RawString(_)) => true,
            (TokenType::BytesLiteral(_), TokenType::BytesLiteral(_)) => true,

            _ => std::mem::discriminant(actual) == std::mem::discriminant(expected),
        }
    }
}

/// Helper functions for AST construction
pub trait AstBuilder {
    /// Create a new identifier expression
    #[allow(dead_code)]
    fn create_identifier(&self, name: &str, line: usize, column: usize) -> crate::ast::Expr;

    /// Create a new string literal expression
    #[allow(dead_code)]
    fn create_string(&self, value: &str, line: usize, column: usize) -> crate::ast::Expr;

    /// Create a new integer literal expression
    #[allow(dead_code)]
    fn create_integer(&self, value: i64, line: usize, column: usize) -> crate::ast::Expr;

    /// Create a new float literal expression
    #[allow(dead_code)]
    fn create_float(&self, value: f64, line: usize, column: usize) -> crate::ast::Expr;

    /// Create a new boolean literal expression
    #[allow(dead_code)]
    fn create_boolean(&self, value: bool, line: usize, column: usize) -> crate::ast::Expr;

    /// Create a new None literal expression
    #[allow(dead_code)]
    fn create_none(&self, line: usize, column: usize) -> crate::ast::Expr;
}

impl AstBuilder for Parser {
    fn create_identifier(&self, name: &str, line: usize, column: usize) -> crate::ast::Expr {
        crate::ast::Expr::Name {
            id: name.to_string(),
            ctx: crate::ast::ExprContext::Load,
            line,
            column,
        }
    }

    fn create_string(&self, value: &str, line: usize, column: usize) -> crate::ast::Expr {
        crate::ast::Expr::Str {
            value: value.to_string(),
            line,
            column,
        }
    }

    fn create_integer(&self, value: i64, line: usize, column: usize) -> crate::ast::Expr {
        crate::ast::Expr::Num {
            value: crate::ast::Number::Integer(value),
            line,
            column,
        }
    }

    fn create_float(&self, value: f64, line: usize, column: usize) -> crate::ast::Expr {
        crate::ast::Expr::Num {
            value: crate::ast::Number::Float(value),
            line,
            column,
        }
    }

    fn create_boolean(&self, value: bool, line: usize, column: usize) -> crate::ast::Expr {
        crate::ast::Expr::NameConstant {
            value: if value {
                crate::ast::NameConstant::True
            } else {
                crate::ast::NameConstant::False
            },
            line,
            column,
        }
    }

    fn create_none(&self, line: usize, column: usize) -> crate::ast::Expr {
        crate::ast::Expr::NameConstant {
            value: crate::ast::NameConstant::None,
            line,
            column,
        }
    }
}
