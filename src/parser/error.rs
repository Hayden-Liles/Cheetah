use crate::lexer::TokenType;
use colored::Colorize;
use std::fmt;

/// Formatter for parse errors with source context
pub struct ParseErrorFormatter<'a> {
    error: &'a ParseError,
    source: Option<&'a str>,
    colored: bool,
}

impl<'a> ParseErrorFormatter<'a> {
    /// Create a new error formatter
    pub fn new(error: &'a ParseError, source: Option<&'a str>, colored: bool) -> Self {
        Self {
            error,
            source,
            colored,
        }
    }

    /// Format the error with source context
    pub fn format(&self) -> String {
        let mut result = String::new();

        let error_msg = self.error.get_message();
        if self.colored {
            result.push_str(&error_msg.bright_red().to_string());
        } else {
            result.push_str(&error_msg);
        }
        result.push('\n');

        if let Some(source) = self.source {
            if let Some(context) = self.get_source_context(source) {
                result.push_str(&context);
            }
        }

        result
    }

    /// Get source context for the error
    fn get_source_context(&self, source: &str) -> Option<String> {
        let line = self.error.line();
        let column = self.error.column();

        if line == 0 {
            return None;
        }

        let lines: Vec<&str> = source.lines().collect();
        if line > lines.len() {
            return None;
        }

        let mut result = String::new();

        let start_line = if line > 2 { line - 2 } else { 1 };
        let end_line = std::cmp::min(line + 2, lines.len());

        let line_num_width = end_line.to_string().len();

        for i in start_line..=end_line {
            let line_content = lines[i - 1];

            let line_num = format!("{:>width$}", i, width = line_num_width);

            if i == line {
                if self.colored {
                    result.push_str(&format!(" {} | {}", line_num.bright_yellow(), line_content));
                } else {
                    result.push_str(&format!(" {} | {}", line_num, line_content));
                }
                result.push('\n');

                let spaces = " ".repeat(line_num_width + 3 + column);
                if self.colored {
                    result.push_str(&format!("{}{}", spaces, "^".bright_red()));
                } else {
                    result.push_str(&format!("{}{}", spaces, "^"));
                }
            } else {
                result.push_str(&format!(" {} | {}", line_num, line_content));
            }

            result.push('\n');
        }

        Some(result)
    }
}

impl<'a> fmt::Display for ParseErrorFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Errors that can occur during parsing
#[derive(Debug, Clone)]
pub enum ParseError {
    /// An unexpected token was encountered
    UnexpectedToken {
        expected: String,
        found: TokenType,
        line: usize,
        column: usize,
        suggestion: Option<String>,
    },

    /// Invalid syntax was detected
    InvalidSyntax {
        message: String,
        line: usize,
        column: usize,
        suggestion: Option<String>,
    },

    /// End of file was reached unexpectedly
    EOF {
        expected: String,
        line: usize,
        column: usize,
        suggestion: Option<String>,
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
            suggestion: None,
        }
    }

    /// Create a new unexpected token error with suggestion
    pub fn unexpected_token_with_suggestion(
        expected: &str,
        found: TokenType,
        line: usize,
        column: usize,
        suggestion: &str,
    ) -> Self {
        ParseError::UnexpectedToken {
            expected: expected.to_string(),
            found,
            line,
            column,
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Create a new invalid syntax error
    pub fn invalid_syntax(message: &str, line: usize, column: usize) -> Self {
        ParseError::InvalidSyntax {
            message: message.to_string(),
            line,
            column,
            suggestion: None,
        }
    }

    /// Create a new invalid syntax error with suggestion
    pub fn invalid_syntax_with_suggestion(
        message: &str,
        line: usize,
        column: usize,
        suggestion: &str,
    ) -> Self {
        ParseError::InvalidSyntax {
            message: message.to_string(),
            line,
            column,
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Create a new end of file error
    pub fn eof(expected: &str, line: usize, column: usize) -> Self {
        ParseError::EOF {
            expected: expected.to_string(),
            line,
            column,
            suggestion: None,
        }
    }

    /// Create a new end of file error with suggestion
    pub fn eof_with_suggestion(
        expected: &str,
        line: usize,
        column: usize,
        suggestion: &str,
    ) -> Self {
        ParseError::EOF {
            expected: expected.to_string(),
            line,
            column,
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Get a user-friendly error message
    pub fn get_message(&self) -> String {
        match self {
            ParseError::UnexpectedToken {
                expected,
                found,
                line,
                column,
                suggestion,
            } => {
                let mut msg = format!(
                    "Line {}, column {}: Expected {}, but found {:?}",
                    line, column, expected, found
                );
                if let Some(sug) = suggestion {
                    msg.push_str(&format!(". Suggestion: {}", sug));
                }
                msg
            }
            ParseError::InvalidSyntax {
                message,
                line,
                column,
                suggestion,
            } => {
                let mut msg = format!("Line {}, column {}: {}", line, column, message);
                if let Some(sug) = suggestion {
                    msg.push_str(&format!(". Suggestion: {}", sug));
                }
                msg
            }
            ParseError::EOF {
                expected,
                line,
                column,
                suggestion,
            } => {
                let mut msg = format!(
                    "Line {}, column {}: Unexpected end of file, expected {}",
                    line, column, expected
                );
                if let Some(sug) = suggestion {
                    msg.push_str(&format!(". Suggestion: {}", sug));
                }
                msg
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
    /// Build an unexpected token error with suggestion
    pub fn unexpected_token_with_suggestion(
        &self,
        expected: &str,
        found: TokenType,
        suggestion: &str,
    ) -> ParseError {
        ParseError::unexpected_token_with_suggestion(
            expected,
            found,
            self.line,
            self.column,
            suggestion,
        )
    }
    #[allow(dead_code)]
    /// Build an invalid syntax error
    pub fn invalid_syntax(&self, message: &str) -> ParseError {
        ParseError::invalid_syntax(message, self.line, self.column)
    }
    #[allow(dead_code)]
    /// Build an invalid syntax error with suggestion
    pub fn invalid_syntax_with_suggestion(&self, message: &str, suggestion: &str) -> ParseError {
        ParseError::invalid_syntax_with_suggestion(message, self.line, self.column, suggestion)
    }
    #[allow(dead_code)]
    /// Build an end of file error
    pub fn eof(&self, expected: &str) -> ParseError {
        ParseError::eof(expected, self.line, self.column)
    }
    #[allow(dead_code)]
    /// Build an end of file error with suggestion
    pub fn eof_with_suggestion(&self, expected: &str, suggestion: &str) -> ParseError {
        ParseError::eof_with_suggestion(expected, self.line, self.column, suggestion)
    }
}
