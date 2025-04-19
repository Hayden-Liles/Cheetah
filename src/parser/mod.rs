mod error;
mod expr;
mod helpers;
mod stmt;
mod types;

pub use error::{ParseError, ParseErrorFormatter};
use helpers::TokenMatching;
use stmt::StmtParser;
use types::ParserContext;

use crate::ast::Module;
use crate::lexer::{Token, TokenType};

use std::collections::VecDeque;

/// Parser for Python source code
///
/// This parser implements a recursive descent parser for Python syntax,
/// producing an AST (Abstract Syntax Tree) conforming to Python's ast module.
pub struct Parser {
    /// Queue of tokens to be processed
    tokens: VecDeque<Token>,

    /// Current token being processed
    current: Option<Token>,

    /// Last token that was processed
    last_token: Option<Token>,

    /// Errors encountered during parsing
    errors: Vec<ParseError>,

    /// Current indentation level
    current_indent_level: usize,

    /// Stack of parser contexts
    context_stack: Vec<ParserContext>,
}

impl Parser {
    /// Creates a new parser with the given tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut tokens_deque = VecDeque::from(tokens);
        let current = tokens_deque.pop_front();

        Parser {
            tokens: tokens_deque,
            current,
            last_token: None,
            errors: Vec::new(),
            current_indent_level: 0,
            context_stack: vec![ParserContext::Normal],
        }
    }

    /// Parses the entire input and returns a module
    pub fn parse(&mut self) -> Result<Module, Vec<ParseError>> {
        let mut body = Vec::new();

        while let Some(token) = &self.current {
            if matches!(token.token_type, TokenType::EOF) {
                break;
            }

            while self.match_token(TokenType::Newline) {}

            if self.current.is_none() {
                break;
            }

            match self.parse_statement() {
                Ok(stmt) => body.push(Box::new(stmt)),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                    if self.current.is_none()
                        || matches!(self.current.as_ref().unwrap().token_type, TokenType::EOF)
                    {
                        break;
                    }
                }
            }
        }

        if self.errors.is_empty() {
            Ok(Module { body })
        } else {
            Err(self.errors.clone())
        }
    }

    /// Push a context onto the stack
    pub fn push_context(&mut self, context: ParserContext) {
        self.context_stack.push(context);
    }

    /// Pop a context from the stack
    pub fn pop_context(&mut self) -> Option<ParserContext> {
        if self.context_stack.len() > 1 {
            self.context_stack.pop()
        } else {
            None
        }
    }

    /// Check if any context in the stack matches the given context
    pub fn is_in_context(&self, context: ParserContext) -> bool {
        self.context_stack.contains(&context)
    }

    /// Execute a function with a temporary context
    pub fn with_context<F, T>(&mut self, context: ParserContext, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.push_context(context);
        let result = f(self);
        self.pop_context();
        result
    }

    /// Get the current token position or a default if no token exists
    pub fn current_position(&self) -> (usize, usize) {
        match &self.current {
            Some(token) => (token.line, token.column),
            None => self.last_position(),
        }
    }

    /// Get the position of the last token or (0,0) if no last token
    pub fn last_position(&self) -> (usize, usize) {
        match &self.last_token {
            Some(token) => (token.line, token.column),
            None => (0, 0),
        }
    }

    /// Advance to the next token, returning the current one
    pub fn advance(&mut self) -> Option<Token> {
        let current = self.current.take();
        if let Some(token) = &current {
            self.last_token = Some(token.clone());
        }
        self.current = self.tokens.pop_front();

        if let Some(token) = &self.current {
            match token.token_type {
                TokenType::Indent => {
                    self.current_indent_level += 1;
                }
                TokenType::Dedent => {
                    if self.current_indent_level > 0 {
                        self.current_indent_level -= 1;
                    }
                }
                _ => {}
            }
        }

        current
    }

    /// Return the previous token (the last one that was consumed)
    pub fn previous_token(&self) -> Token {
        self.last_token
            .clone()
            .expect("No previous token available")
    }

    /// Check if the current token is an identifier
    pub fn check_identifier(&self) -> bool {
        matches!(
            self.current.as_ref().map(|t| &t.token_type),
            Some(TokenType::Identifier(_))
        )
    }

    /// Print the stack trace of errors
    pub fn print_errors(&self) {
        for (i, error) in self.errors.iter().enumerate() {
            println!("Error {}: {:?}", i + 1, error);
        }
    }

    /// Synchronize the parser state after an error
    ///
    /// This method skips tokens until it finds a synchronization point,
    /// which is typically the start of a new statement or the end of a block.
    fn synchronize(&mut self) {
        if let Some(token) = &self.current {
            if matches!(token.token_type, TokenType::EOF | TokenType::Newline) {
                return;
            }
        } else {
            return;
        }

        while let Some(token) = &self.current {
            if matches!(token.token_type, TokenType::EOF) {
                break;
            }

            if matches!(token.token_type, TokenType::Newline) {
                break;
            }

            self.advance();
        }
    }
}

// Re-export parse function for easier use
pub fn parse(tokens: Vec<Token>) -> Result<Module, Vec<ParseError>> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
