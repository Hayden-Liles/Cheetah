pub mod token;
pub mod config;
pub mod error;
pub mod helpers;

use std::collections::HashSet;
use std::str::FromStr;
pub use token::{Token, TokenType};
pub use config::LexerConfig;
pub use error::LexerError;

pub struct Lexer<'a> {
    input: &'a str,
    chars: std::str::Chars<'a>,
    position: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    current_indent: usize,
    config: LexerConfig,
    errors: Vec<LexerError>,
    paren_level: usize,
    bracket_level: usize,
    brace_level: usize,
    lookahead_buffer: Vec<char>,
    keywords: HashSet<&'static str>,
}

impl<'a> Lexer<'a> {

    pub fn new(input: &'a str) -> Self {
        let mut keywords = HashSet::new();
        for kw in &[
            "def", "return", "if", "elif", "else", "while", "for", "in", "break", 
            "continue", "pass", "import", "from", "as", "True", "False", "None", 
            "and", "or", "not", "class", "with", "assert", "async", "await", "try", 
            "except", "finally", "raise", "lambda", "global", "nonlocal", "yield", 
            "del", "is", "match", "case"
        ] {
            keywords.insert(*kw);
        }
        
        Lexer {
            input,
            chars: input.chars(),
            position: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
            current_indent: 0,
            config: LexerConfig::default(),
            errors: Vec::new(),
            paren_level: 0,
            bracket_level: 0,
            brace_level: 0,
            lookahead_buffer: Vec::new(),
            keywords,
        }
    }

    pub fn with_config(input: &'a str, config: LexerConfig) -> Self {
        let mut lexer = Lexer::new(input);
        lexer.config = config;
        lexer
    }

    pub fn get_errors(&self) -> &[LexerError] {
        &self.errors
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        // Pre-allocate a reasonably sized vector to reduce reallocations
        let estimated_token_count = self.input.len() / 5;  // Rough estimate: 1 token per 5 chars
        let mut tokens = Vec::with_capacity(estimated_token_count);
        let mut pending_indentation_change = true;
        
        while let Some(token) = self.next_token() {
            match token.token_type {
                TokenType::EOF => {
                    // Add dedents for any remaining indents
                    while self.indent_stack.len() > 1 {
                        self.indent_stack.pop();
                        tokens.push(Token::new(
                            TokenType::Dedent,
                            self.line,
                            self.column,
                            "".to_string(),
                        ));
                    }
                    
                    tokens.push(token);
                    break;
                },
                _ => {
                    self.update_nesting_level(&token.token_type);
                    
                    let token_type = token.token_type.clone();
                    let token_line = token.line;
                    
                    if pending_indentation_change && 
                       self.paren_level == 0 && 
                       self.bracket_level == 0 && 
                       self.brace_level == 0 {
                        self.handle_indentation_change(&mut tokens, token_line);
                        pending_indentation_change = false;
                    }
                    
                    tokens.push(token);
                    
                    if matches!(token_type, TokenType::Newline) && 
                       self.paren_level == 0 && 
                       self.bracket_level == 0 && 
                       self.brace_level == 0 {
                        pending_indentation_change = true;
                    }
                }
            }
        }
        
        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        
        if self.is_at_end() {
            return Some(Token::new(TokenType::EOF, self.line, self.column, "".to_string()));
        }
        
        let current_char = self.peek_char();
        
        if current_char == '\n' || current_char == '\r' {
            if self.paren_level > 0 || self.bracket_level > 0 || self.brace_level > 0 {
                self.consume_char();
                self.skip_whitespace();
                return self.next_token();
            }
            return self.handle_newline();
        }
        
        if current_char == '\\' && (self.peek_char_n(1) == '\n' || (self.peek_char_n(1) == '\r' && self.peek_char_n(2) == '\n')) {
            self.consume_char();
            if self.peek_char() == '\r' { self.consume_char(); }
            if self.peek_char() == '\n' { self.consume_char(); }
            while !self.is_at_end() && (self.peek_char() == ' ' || self.peek_char() == '\t') {
                self.consume_char();
            }
            if !self.is_at_end() && self.peek_char() == '#' {
                self.consume_while(|c| c != '\n' && c != '\r');
                if !self.is_at_end() && self.peek_char() == '\n' {
                    self.consume_char();
                } else if !self.is_at_end() && self.peek_char() == '\r' {
                    self.consume_char();
                    if !self.is_at_end() && self.peek_char() == '\n' {
                        self.consume_char();
                    }
                }
            }
            return self.next_token();
        }
        
        if (current_char == 'r' || current_char == 'R' || 
            current_char == 'f' || current_char == 'F' || 
            current_char == 'b' || current_char == 'B') && 
            ((self.peek_char_n(1) == '"' && self.peek_char_n(2) == '"' && self.peek_char_n(3) == '"') ||
            (self.peek_char_n(1) == '\'' && self.peek_char_n(2) == '\'' && self.peek_char_n(3) == '\'')) {
            let prefix = current_char;
            self.consume_char();
            match prefix {
                'r' | 'R' => return Some(self.handle_raw_triple_quoted_string()),
                'f' | 'F' => return Some(self.handle_formatted_triple_quoted_string()),
                'b' | 'B' => return Some(self.handle_bytes_triple_quoted_string()),
                _ => unreachable!()
            }
        }
        
        if (current_char == '"' && self.peek_char_n(1) == '"' && self.peek_char_n(2) == '"') ||
            (current_char == '\'' && self.peek_char_n(1) == '\'' && self.peek_char_n(2) == '\'') {
            return Some(self.handle_triple_quoted_string());
        }
        
        if (current_char == 'r' || current_char == 'R' || 
            current_char == 'f' || current_char == 'F' || 
            current_char == 'b' || current_char == 'B') && 
            (self.peek_char_n(1) == '"' || self.peek_char_n(1) == '\'') {
            let prefix = current_char;
            self.consume_char();
            match prefix {
                'r' | 'R' => return Some(self.handle_raw_string()),
                'f' | 'F' => return Some(self.handle_formatted_string()),
                'b' | 'B' => return Some(self.handle_bytes_string()),
                _ => unreachable!()
            }
        }
        
        if current_char == '"' || current_char == '\'' {
            return Some(self.handle_string());
        }
        
        if current_char.is_alphabetic() || current_char == '_' {
            return Some(self.handle_identifier());
        }
        
        if current_char.is_digit(10) || (current_char == '.' && self.peek_char_n(1).is_digit(10)) {
            return Some(self.handle_number());
        }
        
        if current_char == '#' {
            self.consume_while(|c| c != '\n' && c != '\r');
            if !self.is_at_end() && (self.peek_char() == '\n' || self.peek_char() == '\r') {
                return self.handle_newline();
            }
            return self.next_token();
        }

        if current_char == '.' && self.peek_char_n(1) == '.' && self.peek_char_n(2) == '.' {
            return Some(self.handle_ellipsis());
        }
        
        Some(self.handle_operator_or_delimiter())
    }

    fn update_nesting_level(&mut self, token_type: &TokenType) {
        match token_type {
            TokenType::LeftParen => self.paren_level += 1,
            TokenType::RightParen => {
                if self.paren_level > 0 {
                    self.paren_level -= 1;
                }
            },
            TokenType::LeftBracket => self.bracket_level += 1,
            TokenType::RightBracket => {
                if self.bracket_level > 0 {
                    self.bracket_level -= 1;
                }
            },
            TokenType::LeftBrace => self.brace_level += 1,
            TokenType::RightBrace => {
                if self.brace_level > 0 {
                    self.brace_level -= 1;
                }
            },
            _ => {}
        }
    }

    fn handle_indentation_change(&mut self, tokens: &mut Vec<Token>, token_line: usize) {
        let current_indent = self.current_indent;
        let previous_indent = *self.indent_stack.last().unwrap_or(&0);
        
        if current_indent > previous_indent {
            if self.config.enforce_indent_consistency && 
                !self.config.allow_tabs_in_indentation && 
                current_indent % self.config.standard_indent_size != 0 {
                let error_message = format!(
                    "Inconsistent indentation. Expected multiple of {} spaces but got {}.",
                    self.config.standard_indent_size, current_indent
                );
                if !self.has_error_for_line(token_line, &error_message) {
                    self.add_error_with_position(
                        &error_message,
                        &format!("Use {} spaces for indentation", self.config.standard_indent_size),
                        token_line,
                        1
                    );
                }
            }
            let indent_token = Token::new(
                TokenType::Indent,
                token_line,
                1,
                " ".repeat(current_indent),
            );
            self.indent_stack.push(current_indent);
            tokens.push(indent_token);
        } else if current_indent < previous_indent {
            let mut _dedent_count = 0;
            
            let valid_indent_level = self.indent_stack.contains(&current_indent);
            
            if !valid_indent_level {
                let msg = format!(
                    "Inconsistent indentation. Current indent level {} doesn't match any previous level.",
                    current_indent
                );
                if !self.has_error_for_line(token_line, &msg) {
                    self.add_error_with_position(
                        &msg,
                        "Ensure indentation matches a previous level",
                        token_line,
                        1
                    );
                }
            }
            
            while self.indent_stack.len() > 1 && current_indent < *self.indent_stack.last().unwrap() {
                self.indent_stack.pop();
                tokens.push(Token::new(TokenType::Dedent, token_line, 1, "".to_string()));
                _dedent_count += 1;
            }
        }
    }

    fn add_error(&mut self, message: &str) {
        let error = LexerError {
            message: message.to_string(),
            line: self.line,
            column: self.column,
            snippet: self.get_error_context(),
            suggestion: None,
        };
        self.errors.push(error);
    }

    fn add_error_with_suggestion(&mut self, message: &str, suggestion: &str) {
        let error = LexerError {
            message: message.to_string(),
            line: self.line,
            column: self.column,
            snippet: self.get_error_context(),
            suggestion: Some(suggestion.to_string()),
        };
        self.errors.push(error);
    }
    
    fn get_error_context(&self) -> String {
        let lines: Vec<&str> = self.input.lines().collect();
        if self.line <= lines.len() {
            lines[self.line - 1].to_string()
        } else {
            String::new()
        }
    }

    fn has_error_for_line(&self, line: usize, message: &str) -> bool {
        self.errors.iter().any(|e| e.line == line && e.message == message)
    }

    fn add_error_with_position(&mut self, message: &str, suggestion: &str, line: usize, column: usize) {
        let error = LexerError {
            message: message.to_string(),
            line: line,
            column: column,
            snippet: self.get_error_context_for_line(line),
            suggestion: Some(suggestion.to_string()),
        };
        self.errors.push(error);
    }

    fn get_error_context_for_line(&self, line: usize) -> String {
        let lines: Vec<&str> = self.input.lines().collect();
        if line <= lines.len() {
            lines[line - 1].to_string()
        } else {
            String::new()
        }
    }

    fn handle_newline(&mut self) -> Option<Token> {
        let start_col = self.column;
        let start_line = self.line;
        
        self.consume_char();
        
        let mut _is_empty_line = false;
        
        while !self.is_at_end() && self.peek_char() == '\n' {
            _is_empty_line = true;
            self.consume_char();
        }
        
        let indent_size = self.count_indentation();
        
        let newline_token = Token::new(
            TokenType::Newline,
            start_line,
            start_col,
            "\n".to_string(),
        );
        
        // Always update current_indent, even for blank lines
        self.current_indent = indent_size;
        
        Some(newline_token)
    }

    fn count_indentation(&mut self) -> usize {
        let mut count = 0;
        let mut has_tabs = false;
        let mut _has_spaces = false;
        
        let indentation_line = self.line;
        
        while !self.is_at_end() {
            let c = self.peek_char();
            if c == ' ' {
                _has_spaces = true;
                count += 1;
                self.consume_char();
            } else if c == '\t' {
                has_tabs = true;
                count += self.config.tab_width;
                self.consume_char();
            } else {
                break;
            }
        }
        
        if has_tabs && !self.config.allow_tabs_in_indentation {
            let msg = "Tabs are not allowed in indentation";
            if !self.has_error_for_line(indentation_line, msg) {
                self.add_error_with_position(
                    msg,
                    "Use spaces only for indentation",
                    indentation_line,
                    1
                );
            }
        }
        
        if self.config.enforce_indent_consistency && 
            count % self.config.standard_indent_size != 0 && 
            (!has_tabs || !self.config.allow_tabs_in_indentation) {
            
            let msg = format!(
                "Inconsistent indentation. Expected multiple of {} spaces but got {}.", 
                self.config.standard_indent_size, count
            );
            
            if !self.has_error_for_line(indentation_line, &msg) {
                self.add_error_with_position(
                    &msg,
                    &format!("Use {} spaces for indentation", self.config.standard_indent_size),
                    indentation_line,
                    1
                );
            }
        }
        
        count
    }

    fn handle_identifier(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        
        // Fast-path for common identifiers
        self.consume_while(|c| c.is_alphanumeric() || c == '_');
        
        let text = self.get_slice(start_pos, self.position);
        
        // Fast lookup with direct keyword matching
        let token_type = if self.keywords.contains(text) {
            // Use a function-wide static HashMap for faster lookup than match
            match text {
                "def" => TokenType::Def,
                "return" => TokenType::Return,
                "if" => TokenType::If,
                "elif" => TokenType::Elif,
                "else" => TokenType::Else,
                "while" => TokenType::While,
                "for" => TokenType::For,
                "in" => TokenType::In,
                "break" => TokenType::Break,
                "continue" => TokenType::Continue,
                "pass" => TokenType::Pass,
                "True" => TokenType::True,
                "False" => TokenType::False,
                "None" => TokenType::None,
                // Common operators
                "and" => TokenType::And,
                "or" => TokenType::Or,
                "not" => TokenType::Not,
                "is" => TokenType::Is,
                // Other keywords with direct mapping
                "import" => TokenType::Import,
                "from" => TokenType::From,
                "as" => TokenType::As,
                "class" => TokenType::Class,
                "with" => TokenType::With,
                "assert" => TokenType::Assert,
                "async" => TokenType::Async,
                "await" => TokenType::Await,
                "try" => TokenType::Try,
                "except" => TokenType::Except, 
                "finally" => TokenType::Finally,
                "raise" => TokenType::Raise,
                "lambda" => TokenType::Lambda,
                "global" => TokenType::Global,
                "nonlocal" => TokenType::Nonlocal,
                "yield" => TokenType::Yield,
                "del" => TokenType::Del,
                "match" => TokenType::Match,
                "case" => TokenType::Case,
                _ => TokenType::Identifier(text.to_string()),
            }
        } else {
            TokenType::Identifier(text.to_string())
        };
        
        Token::new(token_type, self.line, start_col, text.to_string())
    }

    fn handle_number(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
    
        // Handle special prefixes (0b, 0o, 0x)
        if self.peek_char() == '0' && !self.is_at_end_n(1) {
            let next_char = self.peek_char_n(1);
            if next_char == 'b' || next_char == 'B' {
                self.consume_char();
                self.consume_char();
                return self.handle_binary_literal(start_pos, start_col);
            } else if next_char == 'o' || next_char == 'O' {
                self.consume_char();
                self.consume_char();
                return self.handle_octal_literal(start_pos, start_col);
            } else if next_char == 'x' || next_char == 'X' {
                self.consume_char();
                self.consume_char();
                return self.handle_hex_literal(start_pos, start_col);
            }
        }
    
        let mut is_float = false;
        let mut _decimal_count = 0;
    
        // Handle the case where number starts with a dot
        if self.peek_char() == '.' {
            _decimal_count += 1;
            is_float = true;
            self.consume_char();
            
            if self.is_at_end() || !self.peek_char().is_digit(10) {
                let text = self.get_slice(start_pos, self.position).to_string();
                self.add_error("Invalid float literal: must have at least one digit after decimal point");
                return Token::error("Invalid float literal", self.line, start_col, &text);
            }
            
            // Consume digits after decimal point
            self.consume_while(|c| c.is_digit(10) || c == '_');
        } else {
            // Consume the integer part
            self.consume_while(|c| c.is_digit(10) || c == '_');
            
            // Handle decimal point
            if !self.is_at_end() && self.peek_char() == '.' {
                _decimal_count += 1;
                is_float = true;
                self.consume_char();
                
                // Consume digits after decimal point if any
                self.consume_while(|c| c.is_digit(10) || c == '_');
            }
        }
    
        // Handle exponent part (e.g., 1e10, 1.5e-5)
        if !self.is_at_end() && (self.peek_char() == 'e' || self.peek_char() == 'E') {
            is_float = true;
            self.consume_char();
    
            // Handle optional sign
            if !self.is_at_end() && (self.peek_char() == '+' || self.peek_char() == '-') {
                self.consume_char();
            }
    
            // Exponent must have at least one digit
            if self.is_at_end() || !self.peek_char().is_digit(10) {
                let text = self.get_slice(start_pos, self.position).to_string();
                self.add_error("Invalid exponent: must start with a digit");
                return Token::error("Invalid exponent", self.line, start_col, &text);
            }
    
            // Consume exponent digits
            self.consume_while(|c| c.is_digit(10) || c == '_');
        }
    
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");  // Remove underscores for parsing
    
        // Check for multiple decimal points (which would be an error)
        if !self.is_at_end() && self.peek_char() == '.' && 
           (!self.is_at_end_n(1) && (self.peek_char_n(1).is_digit(10) || self.peek_char_n(1) == '.')) {
            self.add_error("Invalid number format: multiple decimal points");
            self.consume_char();  // Consume the second decimal point
            self.consume_while(|c| c.is_digit(10) || c == '_' || c == '.');  // Continue consuming to recover
            let full_text = self.get_slice(start_pos, self.position).to_string();
            return Token::error("Invalid number format: multiple decimal points", self.line, start_col, &full_text);
        }
    
        // Parse the token based on whether it's a float or integer
        let token_type = if is_float {
            match f64::from_str(&text) {
                Ok(value) => TokenType::FloatLiteral(value),
                Err(_) => {
                    let err_msg = format!("Invalid float literal: {}", text);
                    self.add_error(&err_msg);
                    TokenType::Invalid(err_msg)
                }
            }
        } else {
            match i64::from_str(&text) {
                Ok(value) => TokenType::IntLiteral(value),
                Err(_) => {
                    let err_msg = format!("Invalid integer literal: {}", text);
                    self.add_error(&err_msg);
                    TokenType::Invalid(err_msg)
                }
            }
        };
    
        Token::new(token_type, self.line, start_col, raw_text)
    }

    fn handle_binary_literal(&mut self, start_pos: usize, start_col: usize) -> Token {
        self.consume_char();
        self.consume_while(|c| c.is_digit(10) || c == '_');
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");
        let value_text = &text[2..];
        if value_text.is_empty() || value_text.chars().any(|c| c != '0' && c != '1') {
            let err_msg = format!("Invalid binary literal: {}", text);
            self.add_error(&err_msg);
            return Token::error(&err_msg, self.line, start_col, &raw_text);
        }
        match i64::from_str_radix(value_text, 2) {
            Ok(value) => Token::new(TokenType::BinaryLiteral(value), self.line, start_col, raw_text),
            Err(_) => {
                let err_msg = format!("Invalid binary literal: {}", text);
                self.add_error(&err_msg);
                Token::error(&err_msg, self.line, start_col, &raw_text)
            }
        }
    }        
    
    fn handle_octal_literal(&mut self, start_pos: usize, start_col: usize) -> Token {
        self.consume_char(); // Skip the 'o' or 'O'
        
        // Directly consume all octal digits (0-7) and underscores
        let mut seen_digit = false;
        
        // Consume all digits (and underscores) that are part of the octal literal
        while !self.is_at_end() {
            let c = self.peek_char();
            if c >= '0' && c <= '7' {
                seen_digit = true;
                self.consume_char();
            } else if c == '_' {
                self.consume_char();
            } else {
                break;
            }
        }
        
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        
        if !seen_digit {
            let err_msg = "Invalid octal literal: no digits after '0o'";
            self.add_error(err_msg);
            return Token::error(err_msg, self.line, start_col, &raw_text);
        }
        
        // Extract just the digits (remove the "0o" prefix and any underscores)
        let digit_text = raw_text[2..].replace("_", "");
        
        match i64::from_str_radix(&digit_text, 8) {
            Ok(value) => Token::new(TokenType::OctalLiteral(value), self.line, start_col, raw_text),
            Err(_) => {
                let err_msg = format!("Invalid octal literal: {}", raw_text);
                self.add_error(&err_msg);
                Token::error(&err_msg, self.line, start_col, &raw_text)
            }
        }
    }
    
    fn handle_hex_literal(&mut self, start_pos: usize, start_col: usize) -> Token {
        self.consume_char();
        self.consume_while(|c| c.is_alphanumeric() || c == '_');
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");
        let value_text = &text[2..];
        if value_text.is_empty() || value_text.chars().any(|c| !c.is_ascii_hexdigit()) {
            let err_msg = format!("Invalid hex literal: {}", text);
            self.add_error(&err_msg);
            return Token::error(&err_msg, self.line, start_col, &raw_text);
        }
        match i64::from_str_radix(value_text, 16) {
            Ok(value) => Token::new(TokenType::HexLiteral(value), self.line, start_col, raw_text),
            Err(_) => {
                let err_msg = format!("Invalid hex literal: {}", text);
                self.add_error(&err_msg);
                Token::error(&err_msg, self.line, start_col, &raw_text)
            }
        }
    }

    fn handle_string(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let quote_char = self.peek_char();
        
        self.consume_char();
        
        let mut escaped = false;
        let mut string_content = String::new();
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if escaped {
                let escaped_char = match current_char {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    'b' => '\u{0008}',
                    'f' => '\u{000C}',
                    'a' => '\u{0007}',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    '0'..='7' => {
                        self.handle_octal_escape(&mut string_content);
                        '\0'
                    },
                    'x' => {
                        self.handle_hex_escape(&mut string_content);
                        '\0'
                    },
                    'u' => {
                        self.handle_unicode_escape(&mut string_content);
                        '\0'
                    },
                    'U' => {
                        self.handle_extended_unicode_escape(&mut string_content);
                        '\0'
                    },
                    '\n' => {
                        self.consume_char();
                        self.skip_whitespace();
                        '\0'
                    },
                    '\r' => {
                        self.consume_char();
                        if !self.is_at_end() && self.peek_char() == '\n' {
                            self.consume_char();
                        }
                        self.skip_whitespace();
                        '\0'
                    },
                    _ => {
                        self.add_error(&format!("Unknown escape sequence: \\{}", current_char));
                        current_char
                    }
                };
                
                if !matches!(current_char, '0'..='7' | 'x' | 'u' | 'U' | '\n' | '\r') {
                    string_content.push(escaped_char);
                    self.consume_char();
                }
                
                escaped = false;
            } else if current_char == '\\' {
                escaped = true;
                self.consume_char();
            } else if current_char == quote_char {
                self.consume_char();
                break;
            } else if current_char == '\n' || current_char == '\r' {
                let text = self.get_slice(start_pos, self.position).to_string();
                self.add_error_with_suggestion(
                    "Unterminated string literal: newline in string",
                    "Add closing quote or use triple quotes for multi-line strings"
                );
                return Token::error(
                    "Unterminated string literal: newline in string",
                    self.line,
                    start_col,
                    &text
                );        
            } else {
                string_content.push(current_char);
                self.consume_char();
            }
        }
        
        let text = self.get_slice(start_pos, self.position).to_string();
        
        if self.position >= self.input.len() && !text.ends_with(quote_char) {
            self.add_error_with_suggestion(
                "Unterminated string literal",
                "Add closing quote"
            );
            return Token::error(
                "Unterminated string literal",
                self.line,
                start_col,
                &text
            );
        }
        
        Token::new(TokenType::StringLiteral(string_content), self.line, start_col, text)
    }

    fn handle_raw_string(&mut self) -> Token {
        let start_pos = self.position - 1;
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        self.consume_char();
        
        let mut string_content = String::new();
        let mut is_escaped = false;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if is_escaped {
                string_content.push('\\');
                string_content.push(current_char);
                self.consume_char();
                is_escaped = false;
            } else if current_char == '\\' {
                is_escaped = true;
                self.consume_char();
            } else if current_char == quote_char {
                self.consume_char();
                break;
            } else if current_char == '\n' {
                let text = self.get_slice(start_pos, self.position).to_string();
                self.add_error_with_suggestion(
                    "Unterminated raw string literal: newline in string",
                    "Add closing quote or use triple quotes for multi-line strings"
                );
                return Token::error(
                    "Unterminated raw string literal",
                    self.line,
                    start_col,
                    &text
                );        
            } else {
                string_content.push(current_char);
                self.consume_char();
            }
        }
        
        if is_escaped {
            string_content.push('\\');
        }
        
        let text = self.get_slice(start_pos, self.position).to_string();
        
        if self.position >= self.input.len() && !text.ends_with(quote_char) {
            self.add_error("Unterminated raw string literal");
            return Token::error(
                "Unterminated raw string literal",
                self.line,
                start_col,
                &text
            );
        }
        
        Token::new(TokenType::RawString(string_content), self.line, start_col, text)
    }

    fn handle_formatted_string(&mut self) -> Token {
        let start_pos = self.position - 1;
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        self.consume_char();
        
        let mut string_content = String::new();
        let mut in_expression = false;
        let mut brace_depth = 0;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if !in_expression && current_char == '{' && self.peek_char_n(1) != '{' {
                in_expression = true;
                brace_depth = 1;
                string_content.push(current_char);
                self.consume_char();
            } else if in_expression && current_char == '{' {
                brace_depth += 1;
                string_content.push(current_char);
                self.consume_char();
            } else if in_expression && current_char == '}' {
                brace_depth -= 1;
                string_content.push(current_char);
                self.consume_char();
                
                if brace_depth == 0 {
                    in_expression = false;
                }
            } else if !in_expression && current_char == '\\' {
                self.consume_char();
                
                if self.is_at_end() {
                    self.add_error("Incomplete escape sequence in f-string");
                    break;
                }
                
                let escape_char = self.peek_char();
                string_content.push('\\');
                string_content.push(escape_char);
                self.consume_char();
            } else if !in_expression && current_char == quote_char {
                self.consume_char();
                break;
            } else if current_char == '\n' && !in_expression {
                let text = self.get_slice(start_pos, self.position).to_string();
                self.add_error("Unterminated f-string literal: newline in string");
                return Token::error(
                    "Unterminated f-string literal",
                    self.line,
                    start_col,
                    &text
                );
            } else {
                string_content.push(current_char);
                self.consume_char();
            }
        }
        
        if in_expression {
            self.add_error("Unterminated expression in f-string: missing '}'");
        }
        
        let text = self.get_slice(start_pos, self.position).to_string();
        
        if self.position >= self.input.len() && !text.ends_with(quote_char) {
            self.add_error("Unterminated f-string literal");
            return Token::error(
                "Unterminated f-string literal",
                self.line,
                start_col,
                &text
            );
        }
        
        Token::new(TokenType::FString(string_content), self.line, start_col, text)
    }

    fn handle_bytes_string(&mut self) -> Token {
        let start_pos = self.position - 1;
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        self.consume_char();
        
        let mut bytes = Vec::new();
        let mut escaped = false;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if escaped {
                match current_char {
                    'n' => bytes.push(b'\n'),
                    't' => bytes.push(b'\t'),
                    'r' => bytes.push(b'\r'),
                    '\\' => bytes.push(b'\\'),
                    '\'' => bytes.push(b'\''),
                    '"' => bytes.push(b'"'),
                    'x' => {
                        self.consume_char();
                        
                        let mut hex_value = String::with_capacity(2);
                        for _ in 0..2 {
                            if !self.is_at_end() && self.peek_char().is_ascii_hexdigit() {
                                hex_value.push(self.peek_char());
                                self.consume_char();
                            } else {
                                self.add_error("Invalid hex escape in bytes literal");
                                break;
                            }
                        }
                        
                        if let Ok(byte) = u8::from_str_radix(&hex_value, 16) {
                            bytes.push(byte);
                        }
                        
                        escaped = false;
                        continue;
                    },
                    _ => {
                        self.add_error(&format!("Invalid escape sequence in bytes literal: \\{}", current_char));
                        bytes.push(current_char as u8);
                    }
                }
                
                self.consume_char();
                escaped = false;
            } else if current_char == '\\' {
                escaped = true;
                self.consume_char();
            } else if current_char == quote_char {
                self.consume_char();
                break;
            } else if current_char == '\n' {
                let text = self.get_slice(start_pos, self.position).to_string();
                self.add_error("Unterminated bytes literal: newline in string");
                return Token::error(
                    "Unterminated bytes literal",
                    self.line,
                    start_col,
                    &text
                );
            } else if !current_char.is_ascii() {
                self.add_error("Non-ASCII character in bytes literal");
                self.consume_char();
            } else {
                bytes.push(current_char as u8);
                self.consume_char();
            }
        }
        
        let text = self.get_slice(start_pos, self.position).to_string();
        
        if self.position >= self.input.len() && !text.ends_with(quote_char) {
            self.add_error("Unterminated bytes literal");
            return Token::error(
                "Unterminated bytes literal",
                self.line,
                start_col,
                &text
            );
        }
        
        Token::new(TokenType::BytesLiteral(bytes), self.line, start_col, text)
    }

    fn handle_triple_quoted_string(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let quote_char = self.peek_char();
        
        self.consume_char();
        self.consume_char();
        self.consume_char();
        
        let mut string_content = String::new();
        let mut consecutive_quotes = 0;
        let mut escaped = false;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if escaped {
                match current_char {
                    'n' => string_content.push('\n'),
                    't' => string_content.push('\t'),
                    'r' => string_content.push('\r'),
                    'b' => string_content.push('\u{0008}'), // Backspace
                    'f' => string_content.push('\u{000C}'), // Form feed
                    '\\' => string_content.push('\\'),
                    '\'' => string_content.push('\''),
                    '"' => string_content.push('"'),
                    'x' => {
                        self.handle_hex_escape(&mut string_content);
                        escaped = false;
                        continue;
                    },
                    'u' => {
                        self.handle_unicode_escape(&mut string_content);
                        escaped = false;
                        continue;
                    },
                    '\n' => {
                        self.consume_char();
                        self.skip_whitespace();
                    },
                    _ => {
                        self.add_error(&format!("Unknown escape sequence: \\{}", current_char));
                        string_content.push(current_char);
                    }
                }
                
                escaped = false;
                self.consume_char();
            } else if current_char == '\\' {
                for _ in 0..consecutive_quotes {
                    string_content.push(quote_char);
                }
                consecutive_quotes = 0;
                
                escaped = true;
                self.consume_char();
            } else if current_char == quote_char {
                consecutive_quotes += 1;
                self.consume_char();
                
                if consecutive_quotes == 3 {
                    break;
                }
            } else {
                for _ in 0..consecutive_quotes {
                    string_content.push(quote_char);
                }
                consecutive_quotes = 0;
                
                string_content.push(current_char);
                self.consume_char();
            }
        }
        
        let text = self.get_slice(start_pos, self.position).to_string();
        
        if consecutive_quotes < 3 {
            self.add_error("Unterminated triple-quoted string");
            return Token::error(
                "Unterminated triple-quoted string",
                self.line,
                start_col,
                &text
            );
        }
        
        Token::new(TokenType::StringLiteral(string_content), self.line, start_col, text)
    }

    fn handle_raw_triple_quoted_string(&mut self) -> Token {
        let start_pos = self.position - 1;
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        self.consume_char();
        self.consume_char();
        self.consume_char();
        
        let mut string_content = String::new();
        let mut consecutive_quotes = 0;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if current_char == quote_char {
                consecutive_quotes += 1;
                self.consume_char();
                
                if consecutive_quotes == 3 {
                    break;
                }
            } else {
                for _ in 0..consecutive_quotes {
                    string_content.push(quote_char);
                }
                consecutive_quotes = 0;
                
                string_content.push(current_char);
                self.consume_char();
            }
        }
        
        let text = self.get_slice(start_pos, self.position).to_string();
        
        if consecutive_quotes < 3 {
            self.add_error("Unterminated raw triple-quoted string");
            return Token::error("Unterminated raw triple-quoted string", self.line, start_col, &text);
        }
        
        Token::new(TokenType::RawString(string_content), self.line, start_col, text)
    }

    fn handle_formatted_triple_quoted_string(&mut self) -> Token {
        let start_pos = self.position - 1;
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        self.consume_char();
        self.consume_char();
        self.consume_char();
        
        let mut string_content = String::new();
        let mut consecutive_quotes = 0;
        let mut in_expression = false;
        let mut brace_depth = 0;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if !in_expression && current_char == quote_char {
                consecutive_quotes += 1;
                self.consume_char();
                
                if consecutive_quotes == 3 {
                    break;
                }
            } else if !in_expression && current_char == '{' && 
                      self.peek_char_n(1) != '{' {
                
                for _ in 0..consecutive_quotes {
                    string_content.push(quote_char);
                }
                consecutive_quotes = 0;
                
                in_expression = true;
                brace_depth = 1;
                string_content.push(current_char);
                self.consume_char();
            } else if in_expression && current_char == '{' {
                brace_depth += 1;
                string_content.push(current_char);
                self.consume_char();
            } else if in_expression && current_char == '}' {
                brace_depth -= 1;
                string_content.push(current_char);
                self.consume_char();
                
                if brace_depth == 0 {
                    in_expression = false;
                }
            } else {
                
                if consecutive_quotes > 0 && !in_expression {
                    for _ in 0..consecutive_quotes {
                        string_content.push(quote_char);
                    }
                    consecutive_quotes = 0;
                }
                
                string_content.push(current_char);
                self.consume_char();
            }
        }
        
        let text = self.get_slice(start_pos, self.position).to_string();
        
        if in_expression {
            self.add_error("Unterminated expression in f-string: missing '}'");
        }
        
        if consecutive_quotes < 3 {
            self.add_error("Unterminated formatted triple-quoted string");
            return Token::error(
                "Unterminated formatted triple-quoted string",
                self.line,
                start_col,
                &text
            );
        }
        
        Token::new(TokenType::FString(string_content), self.line, start_col, text)
    }

    fn handle_bytes_triple_quoted_string(&mut self) -> Token {
        let start_pos = self.position - 1;
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        self.consume_char();
        self.consume_char();
        self.consume_char();
        
        let mut bytes = Vec::new();
        let mut consecutive_quotes = 0;
        let mut escaped = false;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if escaped {
                match current_char {
                    'n' => bytes.push(b'\n'),
                    't' => bytes.push(b'\t'),
                    'r' => bytes.push(b'\r'),
                    'b' => bytes.push(b'\x08'), // Backspace
                    'f' => bytes.push(b'\x0C'), // Form feed
                    '\\' => bytes.push(b'\\'),
                    '\'' => bytes.push(b'\''),
                    '"' => bytes.push(b'"'),
                    'x' => {
                        self.consume_char();
                        
                        let mut hex_value = String::with_capacity(2);
                        for _ in 0..2 {
                            if !self.is_at_end() && self.peek_char().is_ascii_hexdigit() {
                                hex_value.push(self.peek_char());
                                self.consume_char();
                            } else {
                                self.add_error("Invalid hex escape in bytes literal");
                                break;
                            }
                        }
                        
                        if let Ok(byte) = u8::from_str_radix(&hex_value, 16) {
                            bytes.push(byte);
                        }
                        
                        escaped = false;
                        continue;
                    },
                    _ => {
                        self.add_error(&format!("Invalid escape sequence in bytes literal: \\{}", current_char));
                        bytes.push(current_char as u8);
                    }
                }
                
                self.consume_char();
                escaped = false;
            } else if current_char == '\\' {
                for _ in 0..consecutive_quotes {
                    bytes.push(quote_char as u8);
                }
                consecutive_quotes = 0;
                
                escaped = true;
                self.consume_char();
            } else if current_char == quote_char {
                consecutive_quotes += 1;
                self.consume_char();
                
                if consecutive_quotes == 3 {
                    break;
                }
            } else {
                
                for _ in 0..consecutive_quotes {
                    bytes.push(quote_char as u8);
                }
                consecutive_quotes = 0;
                
                if !current_char.is_ascii() {
                    self.add_error("Non-ASCII character in bytes literal");
                } else {
                    bytes.push(current_char as u8);
                }
                
                self.consume_char();
            }
        }
        
        let text = self.get_slice(start_pos, self.position).to_string();
        
        if consecutive_quotes < 3 {
            self.add_error("Unterminated bytes triple-quoted string");
            return Token::error(
                "Unterminated bytes triple-quoted string",
                self.line,
                start_col,
                &text
            );
        }
        
        Token::new(TokenType::BytesLiteral(bytes), self.line, start_col, text)
    }

    fn handle_operator_or_delimiter(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let current_char = self.peek_char();
        
        self.consume_char();
        
        let token_type = match current_char {
            '+' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::PlusAssign
                } else {
                    TokenType::Plus
                }
            },
            '-' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::MinusAssign
                } else if !self.is_at_end() && self.peek_char() == '>' {
                    self.consume_char();
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            },
            '*' => {
                if !self.is_at_end() && self.peek_char() == '*' {
                    self.consume_char();
                    if !self.is_at_end() && self.peek_char() == '=' {
                        self.consume_char();
                        TokenType::PowAssign
                    } else {
                        TokenType::Power
                    }
                } else if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::MulAssign
                } else {
                    TokenType::Multiply
                }
            },
            '/' => {
                if !self.is_at_end() && self.peek_char() == '/' {
                    self.consume_char();
                    if !self.is_at_end() && self.peek_char() == '=' {
                        self.consume_char();
                        TokenType::FloorDivAssign
                    } else {
                        TokenType::FloorDivide
                    }
                } else if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::DivAssign
                } else {
                    TokenType::Divide
                }
            },
            '%' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::ModAssign
                } else {
                    TokenType::Modulo
                }
            },
            '@' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::MatrixMulAssign
                } else {
                    TokenType::At
                }
            },
            '&' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::BitwiseAndAssign
                } else {
                    TokenType::BitwiseAnd
                }
            },
            '|' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::BitwiseOrAssign
                } else {
                    TokenType::BitwiseOr
                }
            },
            '^' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::BitwiseXorAssign
                } else {
                    TokenType::BitwiseXor
                }
            },
            '~' => TokenType::BitwiseNot,
            '=' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::Equal
                } else {
                    TokenType::Assign
                }
            },
            '!' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::NotEqual
                } else {
                    self.add_error_with_suggestion(
                        "Unexpected character: !",
                        "Use 'not' instead of ! for boolean negation"
                    );
                    TokenType::Invalid("Unexpected character: !".to_string())
                }
            },
            '<' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::LessEqual
                } else if !self.is_at_end() && self.peek_char() == '<' {
                    self.consume_char();
                    if !self.is_at_end() && self.peek_char() == '=' {
                        self.consume_char();
                        TokenType::ShiftLeftAssign
                    } else {
                        TokenType::ShiftLeft
                    }
                } else {
                    TokenType::LessThan
                }
            },
            '>' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::GreaterEqual
                } else if !self.is_at_end() && self.peek_char() == '>' {
                    self.consume_char();
                    if !self.is_at_end() && self.peek_char() == '=' {
                        self.consume_char();
                        TokenType::ShiftRightAssign
                    } else {
                        TokenType::ShiftRight
                    }
                } else {
                    TokenType::GreaterThan
                }
            },
            ':' => {
                if !self.is_at_end() && self.peek_char() == '=' {
                    self.consume_char();
                    TokenType::Walrus
                } else {
                    TokenType::Colon
                }
            },
            
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            ';' => {
                if !self.config.allow_trailing_semicolon {
                    self.add_error_with_suggestion(
                        "Semicolons are not used in Python-like syntax",
                        "Remove the semicolon"
                    );
                }
                TokenType::SemiColon
            },
            
            '\\' => {
                TokenType::BackSlash
            },
            
            _ => {
                let msg = format!("Unexpected character: {}", current_char);
                self.add_error(&msg);
                TokenType::Invalid(msg)
            }
        };
        
        let text = self.get_slice(start_pos, self.position);
        Token::new(token_type, self.line, start_col, text.to_string())
    }

    fn handle_ellipsis(&mut self) -> Token {
        let _start_pos = self.position;
        let start_col = self.column;
        
        self.consume_char();
        self.consume_char();
        self.consume_char();
        
        Token::new(
            TokenType::Ellipsis,
            self.line,
            start_col,
            "...".to_string()
        )
    }

    fn handle_octal_escape(&mut self, string_content: &mut String) -> char {
        let mut octal_value = String::with_capacity(3);
        let mut digit_count = 0;
        
        octal_value.push(self.peek_char());
        self.consume_char();
        digit_count += 1;
        
        while digit_count < 3 && !self.is_at_end() && self.peek_char().is_digit(8) {
            octal_value.push(self.peek_char());
            self.consume_char();
            digit_count += 1;
        }
        
        if let Ok(byte) = u8::from_str_radix(&octal_value, 8) {
            string_content.push(byte as char);
        } else {
            let err_msg = format!("Invalid octal escape sequence: \\{}", octal_value);
            self.add_error(&err_msg);
        }
        
        '\0'
    }

    fn handle_hex_escape(&mut self, string_content: &mut String) -> char {
        self.consume_char();
        
        let mut hex_value = String::with_capacity(2);
        
        for _ in 0..2 {
            if !self.is_at_end() && 
                self.peek_char().is_ascii_hexdigit() {
                hex_value.push(self.peek_char());
                self.consume_char();
            } else {
                self.add_error("Invalid hex escape sequence: expected 2 hex digits");
                return '?';
            }
        }
        
        if let Ok(byte) = u8::from_str_radix(&hex_value, 16) {
            string_content.push(byte as char);
        } else {
            let err_msg = format!("Invalid hex escape sequence: \\x{}", hex_value);
            self.add_error(&err_msg);
        }
        
        '\0'
    }

    fn handle_unicode_escape(&mut self, string_content: &mut String) -> char {
        self.consume_char();
        
        let has_braces = !self.is_at_end() && self.peek_char() == '{';
        if has_braces {
            self.consume_char();
        }
        
        if !has_braces {
            let mut hex_value = String::with_capacity(4);
            
            for _ in 0..4 {
                if !self.is_at_end() && 
                   self.peek_char().is_ascii_hexdigit() {
                    hex_value.push(self.peek_char());
                    self.consume_char();
                } else {
                    self.add_error("Invalid Unicode escape sequence: expected 4 hex digits");
                    return '?';
                }
            }
            
            if let Ok(code_point) = u32::from_str_radix(&hex_value, 16) {
                if let Some(c) = char::from_u32(code_point) {
                    string_content.push(c);
                } else {
                    let err_msg = format!("Invalid Unicode code point: U+{:X}", code_point);
                    self.add_error(&err_msg);
                }
            } else {
                let err_msg = format!("Invalid Unicode escape sequence: \\u{}", hex_value);
                self.add_error(&err_msg);
            }
        }
        else {
            let mut hex_value = String::new();
            
            while !self.is_at_end() && 
                  self.peek_char().is_ascii_hexdigit() && 
                  hex_value.len() < 6 {
                hex_value.push(self.peek_char());
                self.consume_char();
            }
            
            if !self.is_at_end() && self.peek_char() == '}' {
                self.consume_char();
            } else {
                self.add_error("Unclosed Unicode escape sequence: missing closing brace");
                return '?';
            }
            
            if hex_value.is_empty() {
                self.add_error("Empty Unicode escape sequence: \\u{}");
                return '?';
            }
            
            if let Ok(code_point) = u32::from_str_radix(&hex_value, 16) {
                if let Some(c) = char::from_u32(code_point) {
                    string_content.push(c);
                } else {
                    let err_msg = format!("Invalid Unicode code point: U+{:X}", code_point);
                    self.add_error(&err_msg);
                }
            } else {
                let err_msg = format!("Invalid Unicode escape sequence: \\u{{{}}}", hex_value);
                self.add_error(&err_msg);
            }
        }
        
        '\0'
    }

    fn handle_extended_unicode_escape(&mut self, string_content: &mut String) -> char {
        self.consume_char();
        
        let mut hex_value = String::with_capacity(8);
        
        for _ in 0..8 {
            if !self.is_at_end() && self.peek_char().is_ascii_hexdigit() {
                hex_value.push(self.peek_char());
                self.consume_char();
            } else {
                self.add_error("Invalid extended Unicode escape sequence: expected 8 hex digits");
                return '?';
            }
        }
        
        if let Ok(code_point) = u32::from_str_radix(&hex_value, 16) {
            if let Some(c) = char::from_u32(code_point) {
                string_content.push(c);
            } else {
                let err_msg = format!("Invalid Unicode code point: U+{:X}", code_point);
                self.add_error(&err_msg);
            }
        } else {
            let err_msg = format!("Invalid Unicode escape sequence: \\U{}", hex_value);
            self.add_error(&err_msg);
        }
        
        '\0'
    }
}