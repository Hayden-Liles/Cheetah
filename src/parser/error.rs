use crate::lexer::TokenType;
use std::fmt;

/// Errors that can occur during parsing
#[derive(Debug, Clone)]
pub enum ParseError {
    /// An unexpected token was encountered
    UnexpectedToken {
        expected: String,
        found: TokenType,
        line: usize,
        column: usize,
    },
    
    /// Invalid syntax was detected
    InvalidSyntax {
        message: String,
        line: usize,
        column: usize,
    },
    
    /// End of file was reached unexpectedly
    EOF {
        expected: String,
        line: usize,
        column: usize,
    },
}

impl ParseError {
    /// Get the line number where the error occurred
    pub fn line(&self) -> usize {
        match self {
            ParseError::UnexpectedToken { line, .. } => *line,
            ParseError::InvalidSyntax { line, .. } => *line,
            ParseError::EOF { line, .. } => *line,
        }
    }

    /// Get the column number where the error occurred
    pub fn column(&self) -> usize {
        match self {
            ParseError::UnexpectedToken { column, .. } => *column,
            ParseError::InvalidSyntax { column, .. } => *column,
            ParseError::EOF { column, .. } => *column,
        }
    }
    
    /// Create a new unexpected token error
    pub fn unexpected_token(expected: &str, found: TokenType, line: usize, column: usize) -> Self {
        ParseError::UnexpectedToken {
            expected: expected.to_string(),
            found,
            line,
            column,
        }
    }
    
    /// Create a new invalid syntax error
    pub fn invalid_syntax(message: &str, line: usize, column: usize) -> Self {
        ParseError::InvalidSyntax {
            message: message.to_string(),
            line,
            column,
        }
    }
    
    /// Create a new end of file error
    pub fn eof(expected: &str, line: usize, column: usize) -> Self {
        ParseError::EOF {
            expected: expected.to_string(),
            line,
            column,
        }
    }
    
    /// Get a user-friendly error message
    pub fn get_message(&self) -> String {
        match self {
            ParseError::UnexpectedToken { expected, found, line, column } => {
                format!("Line {}, column {}: Expected {}, but found {:?}", 
                    line, column, expected, found)
            }
            ParseError::InvalidSyntax { message, line, column } => {
                format!("Line {}, column {}: {}", line, column, message)
            }
            ParseError::EOF { expected, line, column } => {
                format!("Line {}, column {}: Unexpected end of file, expected {}", 
                    line, column, expected)
            }
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_message())
    }
}

impl std::error::Error for ParseError {}

/// Builder for parse errors
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParseErrorBuilder {
    line: usize,
    column: usize,
}

impl ParseErrorBuilder {
    #[allow(dead_code)]
    /// Create a new error builder with the given location
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
    #[allow(dead_code)]
    /// Build an unexpected token error
    pub fn unexpected_token(&self, expected: &str, found: TokenType) -> ParseError {
        ParseError::unexpected_token(expected, found, self.line, self.column)
    }
    #[allow(dead_code)]
    /// Build an invalid syntax error
    pub fn invalid_syntax(&self, message: &str) -> ParseError {
        ParseError::invalid_syntax(message, self.line, self.column)
    }
    #[allow(dead_code)]
    /// Build an end of file error
    pub fn eof(&self, expected: &str) -> ParseError {
        ParseError::eof(expected, self.line, self.column)
    }
}