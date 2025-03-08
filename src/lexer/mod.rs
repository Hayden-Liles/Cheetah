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
            "del", "is"
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
        let mut tokens = Vec::new();
        let mut pending_indentation_change = true;
        
        while let Some(token) = self.next_token() {
            if token.token_type == TokenType::EOF {
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
            }
            
            self.update_nesting_level(&token.token_type);
            
            let token_type = token.token_type.clone();
            let token_line = token.line;
            
            if pending_indentation_change && 
                self.paren_level == 0 && self.bracket_level == 0 && self.brace_level == 0 {
                self.handle_indentation_change(&mut tokens, token_line);
                pending_indentation_change = false;
            }
            
            tokens.push(token);
            
            if matches!(token_type, TokenType::Newline) && 
                self.paren_level == 0 && self.bracket_level == 0 && self.brace_level == 0 {
                pending_indentation_change = true;
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
        
        let mut is_empty_line = false;
        
        while !self.is_at_end() && self.peek_char() == '\n' {
            is_empty_line = true;
            self.consume_char();
        }
        
        let indent_size = self.count_indentation();
        
        let newline_token = Token::new(
            TokenType::Newline,
            start_line,
            start_col,
            "\n".to_string(),
        );
        
        if !is_empty_line {
            self.current_indent = indent_size;
        }
        
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
        
        self.consume_while(|c| c.is_alphanumeric() || c == '_');
        
        let text = self.get_slice(start_pos, self.position);
        
        let token_type = if self.keywords.contains(text) {
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
                "import" => TokenType::Import,
                "from" => TokenType::From,
                "as" => TokenType::As,
                "True" => TokenType::True,
                "False" => TokenType::False,
                "None" => TokenType::None,
                "and" => TokenType::And,
                "or" => TokenType::Or,
                "not" => TokenType::Not,
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
                "is" => TokenType::Is,
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
    
        if self.peek_char() == '0' && !self.is_at_end_n(1) {
            let next_char = self.peek_char_n(1);
            if next_char == 'b' || next_char == 'B' {
                self.consume_char();
                return self.handle_binary_literal(start_pos, start_col);
            } else if next_char == 'o' || next_char == 'O' {
                self.consume_char();
                return self.handle_octal_literal(start_pos, start_col);
            } else if next_char == 'x' || next_char == 'X' {
                self.consume_char();
                return self.handle_hex_literal(start_pos, start_col);
            }
        }
    
        let mut is_float = false;
        let mut decimal_count = 0;
    
        self.consume_while(|c| c.is_digit(10) || c == '_');
    
        if !self.is_at_end() && self.peek_char() == '.' && self.peek_char_n(1).is_digit(10) {
            decimal_count += 1;
            is_float = true;
            self.consume_char();
            self.consume_while(|c| c.is_digit(10) || c == '_');
        }
    
        if !self.is_at_end() && (self.peek_char() == 'e' || self.peek_char() == 'E') {
            is_float = true;
            self.consume_char();
    
            if !self.is_at_end() && (self.peek_char() == '+' || self.peek_char() == '-') {
                self.consume_char();
            }
    
            if self.is_at_end() || !self.peek_char().is_digit(10) {
                let text = self.get_slice(start_pos, self.position).to_string();
                self.add_error("Invalid exponent: must start with a digit");
                return Token::error("Invalid exponent", self.line, start_col, &text);
            }
    
            self.consume_char();
            while !self.is_at_end() {
                let c = self.peek_char();
                if c.is_digit(10) {
                    self.consume_char();
                } else if c == '_' {
                    self.consume_char();
                    if self.is_at_end() || !self.peek_char().is_digit(10) {
                        let text = self.get_slice(start_pos, self.position).to_string();
                        self.add_error("Invalid underscore in exponent");
                        return Token::error("Invalid underscore in exponent", self.line, start_col, &text);
                    }
                } else {
                    break;
                }
            }
        }
    
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");
    
        if decimal_count > 1 || text.matches('.').count() > 1 {
            self.add_error("Invalid number format: multiple decimal points");
            return Token::error("Invalid number format: multiple decimal points", self.line, start_col, &raw_text);
        }
    
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
        self.consume_char();
        
        let mut digit_str = String::new();
        let mut has_digit = false;
        
        while !self.is_at_end() {
            let c = self.peek_char();
            if c.is_digit(8) {
                digit_str.push(c);
                has_digit = true;
                self.consume_char();
            } else if c == '_' {
                self.consume_char();
            } else {
                break;
            }
        }

        let raw_text = self.get_slice(start_pos, self.position).to_string();
        
        if !has_digit {
            let err_msg = "Invalid octal literal: no digits after '0o'";
            self.add_error(err_msg);
            return Token::error(err_msg, self.line, start_col, &raw_text);
        }

        match i64::from_str_radix(&digit_str, 8) {
            Ok(value) => Token::new(TokenType::OctalLiteral(value), self.line, start_col, raw_text),
            Err(_) => {
                let err_msg = format!("Invalid octal literal: 0o{}", digit_str);
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

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to simplify token comparison
    #[allow(dead_code)]
    fn assert_tokens(input: &str, expected_tokens: Vec<TokenType>) {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), expected_tokens.len() + 1, "Token count mismatch"); // +1 for EOF
        
        for (i, expected_type) in expected_tokens.iter().enumerate() {
            assert_eq!(&tokens[i].token_type, expected_type, 
                        "Token type mismatch at position {}. Expected {:?}, got {:?}", 
                        i, expected_type, tokens[i].token_type);
        }
        
        // Check that the last token is EOF
        assert_eq!(tokens.last().unwrap().token_type, TokenType::EOF);
    }

    // Test for newline handling
    #[test]
    fn test_newline_styles() {
        let input_lf = "x = 1\ny = 2";
        let mut lexer_lf = Lexer::new(input_lf);
        let tokens_lf = lexer_lf.tokenize();
        
        let input_crlf = "x = 1\r\ny = 2";
        let mut lexer_crlf = Lexer::new(input_crlf);
        let tokens_crlf = lexer_crlf.tokenize();
        
        if tokens_lf.len() != tokens_crlf.len() {
            println!("LF tokens ({}): {:?}", tokens_lf.len(), tokens_lf);
            println!("CRLF tokens ({}): {:?}", tokens_crlf.len(), tokens_crlf);
        }
        
        assert_eq!(tokens_lf.len(), tokens_crlf.len(), "Different newline styles should produce same token count");
        for i in 0..tokens_lf.len() {
            assert_eq!(tokens_lf[i].token_type, tokens_crlf[i].token_type, 
                    "Different newline styles should produce same tokens");
        }
    }

    // Test for unicode support in strings and identifiers
    #[test]
    fn test_unicode_support() {
        // Testing indentation handling with a helper function
        fn assert_tokens_ignore_indentation(input: &str, expected_token_types: Vec<TokenType>) {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize();
            
            // Filter out Indent and Dedent tokens
            let filtered_tokens: Vec<_> = tokens
                .into_iter()
                .filter(|token| !matches!(token.token_type, TokenType::Indent | TokenType::Dedent))
                .collect();
            
            assert_eq!(filtered_tokens.len(), expected_token_types.len() + 1, 
                    "Token count mismatch (ignoring indentation tokens)"); // +1 for EOF
            
            for (i, expected_type) in expected_token_types.iter().enumerate() {
                assert_eq!(&filtered_tokens[i].token_type, expected_type, 
                        "Token type mismatch at position {}. Expected {:?}, got {:?}", 
                        i, expected_type, filtered_tokens[i].token_type);
            }
            
            // Check that the last token is EOF
            assert_eq!(filtered_tokens.last().unwrap().token_type, TokenType::EOF);
        }

        // Unicode in identifiers
        assert_tokens_ignore_indentation(
            "π = 3.14159\nñame = \"José\"\n你好 = \"Hello\"",
            vec![
                TokenType::Identifier("π".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(3.14159),
                TokenType::Newline,
                TokenType::Identifier("ñame".to_string()),
                TokenType::Assign,
                TokenType::StringLiteral("José".to_string()),
                TokenType::Newline,
                TokenType::Identifier("你好".to_string()),
                TokenType::Assign,
                TokenType::StringLiteral("Hello".to_string()),
            ]
        );
        
        // Unicode in string literals
        assert_tokens_ignore_indentation(
            "message = \"Hello, 世界!\"",
            vec![
                TokenType::Identifier("message".to_string()),
                TokenType::Assign,
                TokenType::StringLiteral("Hello, 世界!".to_string()),
            ]
        );
        
        // Unicode escape sequences
        assert_tokens_ignore_indentation(
            r#"emoji = "\u{1F600}""#, // 😀 emoji
            vec![
                TokenType::Identifier("emoji".to_string()),
                TokenType::Assign,
                TokenType::StringLiteral("😀".to_string()),
            ]
        );
    }
    
    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::EOF);
    }
    
    // Test keywords
    #[test]
    fn test_keywords() {
        assert_tokens(
            "def if elif else while for in break continue pass return",
            vec![
                TokenType::Def,
                TokenType::If,
                TokenType::Elif,
                TokenType::Else,
                TokenType::While,
                TokenType::For,
                TokenType::In,
                TokenType::Break,
                TokenType::Continue,
                TokenType::Pass,
                TokenType::Return,
            ]
        );
        
        assert_tokens(
            "import from as True False None and or not",
            vec![
                TokenType::Import,
                TokenType::From,
                TokenType::As,
                TokenType::True,
                TokenType::False,
                TokenType::None,
                TokenType::And,
                TokenType::Or,
                TokenType::Not,
            ]
        );
        
        assert_tokens(
            "class with assert async await try except finally raise",
            vec![
                TokenType::Class,
                TokenType::With,
                TokenType::Assert,
                TokenType::Async,
                TokenType::Await,
                TokenType::Try,
                TokenType::Except,
                TokenType::Finally,
                TokenType::Raise,
            ]
        );
        
        assert_tokens(
            "lambda global nonlocal yield del is",
            vec![
                TokenType::Lambda,
                TokenType::Global,
                TokenType::Nonlocal,
                TokenType::Yield,
                TokenType::Del,
                TokenType::Is,
            ]
        );
    }
    
    // Test identifiers
    #[test]
    fn test_identifiers() {
        assert_tokens(
            "variable _private name123 camelCase snake_case",
            vec![
                TokenType::Identifier("variable".to_string()),
                TokenType::Identifier("_private".to_string()),
                TokenType::Identifier("name123".to_string()),
                TokenType::Identifier("camelCase".to_string()),
                TokenType::Identifier("snake_case".to_string()),
            ]
        );
        
        // Test identifier that looks like keyword but isn't
        assert_tokens(
            "defining ifdef",
            vec![
                TokenType::Identifier("defining".to_string()),
                TokenType::Identifier("ifdef".to_string()),
            ]
        );
    }
    
    // Test integer literals
    #[test]
    fn test_integer_literals() {
        assert_tokens(
            "123 0 -42 1_000_000",
            vec![
                TokenType::IntLiteral(123),
                TokenType::IntLiteral(0),
                TokenType::Minus,
                TokenType::IntLiteral(42),
                TokenType::IntLiteral(1000000),
            ]
        );
    }
    
    // Test different numeric bases
    #[test]
    fn test_different_bases() {
        assert_tokens(
            "0b1010 0B1100 0o777 0O123 0xABC 0Xdef",
            vec![
                TokenType::BinaryLiteral(10),
                TokenType::BinaryLiteral(12),
                TokenType::OctalLiteral(511), // 777 octal = 511 decimal
                TokenType::OctalLiteral(83),  // 123 octal = 83 decimal
                TokenType::HexLiteral(2748),  // ABC hex = 2748 decimal
                TokenType::HexLiteral(3567),  // def hex = 3567 decimal
            ]
        );
    }
    
    // Test float literals
    #[test]
    fn test_float_literals() {
        assert_tokens(
            "3.14 .5 2. 1e10 1.5e-5 1_000.5 1e+10",
            vec![
                TokenType::FloatLiteral(3.14),
                TokenType::FloatLiteral(0.5),
                TokenType::FloatLiteral(2.0),
                TokenType::FloatLiteral(1e10),
                TokenType::FloatLiteral(1.5e-5),
                TokenType::FloatLiteral(1000.5),
                TokenType::FloatLiteral(1e10),
            ]
        );
    }
    
    // Test string literals
    #[test]
    fn test_string_literals() {
        assert_tokens(
            r#""hello" 'world'"#,
            vec![
                TokenType::StringLiteral("hello".to_string()),
                TokenType::StringLiteral("world".to_string()),
            ]
        );
        
        // Test strings with escape sequences
        assert_tokens(
            r#""hello\nworld" 'escaped\'quote' "tab\tchar" 'bell\a'"#,
            vec![
                TokenType::StringLiteral("hello\nworld".to_string()),
                TokenType::StringLiteral("escaped'quote".to_string()),
                TokenType::StringLiteral("tab\tchar".to_string()),
                TokenType::StringLiteral("bell\u{0007}".to_string()),
            ]
        );
        
        // Test hex and Unicode escapes
        assert_tokens(
            r#""\x41\x42C" "\u00A9 copyright""#,
            vec![
                TokenType::StringLiteral("ABC".to_string()),
                TokenType::StringLiteral("© copyright".to_string()),
            ]
        );
    }
    
    // Test raw strings
    #[test]
    fn test_raw_strings() {
        assert_tokens(
            r#"r"raw\nstring" R'another\tone'"#,
            vec![
                TokenType::RawString("raw\\nstring".to_string()),
                TokenType::RawString("another\\tone".to_string()),
            ]
        );
    }
    
    // Test formatted strings (f-strings)
    #[test]
    fn test_formatted_strings() {
        assert_tokens(
            r#"f"Hello, {name}!" F'Value: {2 + 2}'"#,
            vec![
                TokenType::FString("Hello, {name}!".to_string()),
                TokenType::FString("Value: {2 + 2}".to_string()),
            ]
        );
        
        // Test nested expressions
        assert_tokens(
            r#"f"Nested: {value if condition else {inner}}""#,
            vec![
                TokenType::FString("Nested: {value if condition else {inner}}".to_string()),
            ]
        );
    }
    
    // Test bytes literals
    #[test]
    fn test_bytes_literals() {
        assert_tokens(
            r#"b"bytes" B'\x00\xff'"#,
            vec![
                TokenType::BytesLiteral(b"bytes".to_vec()),
                TokenType::BytesLiteral(vec![0, 255]),
            ]
        );
    }
    
    // Test triple-quoted strings
    #[test]
    fn test_triple_quoted_strings() {
        assert_tokens(
            r#""""Triple quoted string"""'''Another triple quoted'''"#,
            vec![
                TokenType::StringLiteral("Triple quoted string".to_string()),
                TokenType::StringLiteral("Another triple quoted".to_string()),
            ]
        );
        
        // Test with newlines inside
        assert_tokens(
            "\"\"\"Multi\nline\nstring\"\"\"",
            vec![
                TokenType::StringLiteral("Multi\nline\nstring".to_string()),
            ]
        );
    }
    
    // Test prefixed triple-quoted strings
    #[test]
    fn test_prefixed_triple_quoted_strings() {
        assert_tokens(
            r#"r"""Raw\nTriple"""f'''Format {x}'''"#,
            vec![
                TokenType::RawString("Raw\\nTriple".to_string()),
                TokenType::FString("Format {x}".to_string()),
            ]
        );
        
        assert_tokens(
            "b\"\"\"Bytes\nWith\nNewlines\"\"\"",
            vec![
                TokenType::BytesLiteral(b"Bytes\nWith\nNewlines".to_vec()),
            ]
        );
    }
    
    // Test operators
    #[test]
    fn test_basic_operators() {
        assert_tokens(
            "+ - * / % ** // @ & | ^ ~ << >>",
            vec![
                TokenType::Plus,
                TokenType::Minus,
                TokenType::Multiply,
                TokenType::Divide,
                TokenType::Modulo,
                TokenType::Power,
                TokenType::FloorDivide,
                TokenType::At,
                TokenType::BitwiseAnd,
                TokenType::BitwiseOr,
                TokenType::BitwiseXor,
                TokenType::BitwiseNot,
                TokenType::ShiftLeft,
                TokenType::ShiftRight,
            ]
        );
    }
    
    // Test comparison operators
    #[test]
    fn test_comparison_operators() {
        assert_tokens(
            "== != < <= > >=",
            vec![
                TokenType::Equal,
                TokenType::NotEqual,
                TokenType::LessThan,
                TokenType::LessEqual,
                TokenType::GreaterThan,
                TokenType::GreaterEqual,
            ]
        );
    }
    
    // Test assignment operators
    #[test]
    fn test_assignment_operators() {
        assert_tokens(
            "= += -= *= /= %= **= //= &= |= ^= <<= >>=",
            vec![
                TokenType::Assign,
                TokenType::PlusAssign,
                TokenType::MinusAssign,
                TokenType::MulAssign,
                TokenType::DivAssign,
                TokenType::ModAssign,
                TokenType::PowAssign,
                TokenType::FloorDivAssign,
                TokenType::BitwiseAndAssign,
                TokenType::BitwiseOrAssign,
                TokenType::BitwiseXorAssign,
                TokenType::ShiftLeftAssign,
                TokenType::ShiftRightAssign,
            ]
        );
    }
    
    // Test special operators
    #[test]
    fn test_special_operators() {
        assert_tokens(
            ":= ...",
            vec![
                TokenType::Walrus,
                TokenType::Ellipsis,
            ]
        );
    }
    
    // Test delimiters
    #[test]
    fn test_delimiters() {
        assert_tokens(
            "( ) [ ] { } , . : ; -> \\",
            vec![
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBracket,
                TokenType::RightBracket,
                TokenType::LeftBrace,
                TokenType::RightBrace,
                TokenType::Comma,
                TokenType::Dot,
                TokenType::Colon,
                TokenType::SemiColon,
                TokenType::Arrow,
                TokenType::BackSlash,
            ]
        );
    }
    
    // Test indentation
    #[test]
    fn test_indentation() {
        let input = "def test():\n    print('indented')\n    if True:\n        print('nested')\n";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract the token types for easier comparison
        let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
        
        // Expected sequence of token types
        let expected = vec![
            TokenType::Def,
            TokenType::Identifier("test".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::Colon,
            TokenType::Newline,
            TokenType::Indent,
            TokenType::Identifier("print".to_string()),
            TokenType::LeftParen,
            TokenType::StringLiteral("indented".to_string()),
            TokenType::RightParen,
            TokenType::Newline,
            TokenType::If,
            TokenType::True,
            TokenType::Colon,
            TokenType::Newline,
            TokenType::Indent,
            TokenType::Identifier("print".to_string()),
            TokenType::LeftParen,
            TokenType::StringLiteral("nested".to_string()),
            TokenType::RightParen,
            TokenType::Newline,
            TokenType::Dedent,
            TokenType::Dedent,
            TokenType::EOF,
        ];
        
        assert_eq!(token_types, expected, "Indentation tokens don't match expected");
    }
    
    // Test nested indentation
    #[test]
    fn test_complex_indentation2() {
        let input = "if x:\n    if y:\n        print('nested')\n    print('outer')\nprint('no indent')";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Count indents and dedents
        let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
        let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
        
        assert_eq!(indent_count, 2, "Should have 2 indents");
        assert_eq!(dedent_count, 2, "Should have 2 dedents");
    }
    
    // Test comments
    #[test]
    fn test_comments() {
        // Test inline comment
        let input = "x = 5 # comment\ny = 10";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let expected = vec![
            Token::new(TokenType::Identifier("x".to_string()), 1, 1, "x".to_string()),
            Token::new(TokenType::Assign, 1, 3, "=".to_string()),
            Token::new(TokenType::IntLiteral(5), 1, 5, "5".to_string()),
            Token::new(TokenType::Newline, 1, 16, "\n".to_string()), // Corrected to column 16
            Token::new(TokenType::Identifier("y".to_string()), 2, 1, "y".to_string()),
            Token::new(TokenType::Assign, 2, 3, "=".to_string()),
            Token::new(TokenType::IntLiteral(10), 2, 5, "10".to_string()),
            Token::new(TokenType::EOF, 2, 7, "".to_string()),
        ];
        assert_eq!(tokens, expected, "Inline comment not handled correctly");

        // Test standalone comment
        let input = "x = 5\n# comment\ny = 10";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let expected = vec![
            Token::new(TokenType::Identifier("x".to_string()), 1, 1, "x".to_string()),
            Token::new(TokenType::Assign, 1, 3, "=".to_string()),
            Token::new(TokenType::IntLiteral(5), 1, 5, "5".to_string()),
            Token::new(TokenType::Newline, 1, 6, "\n".to_string()),
            Token::new(TokenType::Newline, 2, 10, "\n".to_string()), // Corrected column
            Token::new(TokenType::Identifier("y".to_string()), 3, 1, "y".to_string()),
            Token::new(TokenType::Assign, 3, 3, "=".to_string()),
            Token::new(TokenType::IntLiteral(10), 3, 5, "10".to_string()),
            Token::new(TokenType::EOF, 3, 7, "".to_string()),
        ];
        assert_eq!(tokens, expected, "Standalone comment line not handled correctly");
    }
    
    // Test for line continuation
    #[test]
    fn test_line_continuation() {
        let input = "x = 1 + \\\n    2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract the token types for easier comparison
        let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
        
        // Expected sequence of token types
        let expected = vec![
            TokenType::Identifier("x".to_string()),
            TokenType::Assign,
            TokenType::IntLiteral(1),
            TokenType::Plus,
            TokenType::IntLiteral(2),
            TokenType::EOF,
        ];
        
        assert_eq!(token_types, expected, "Line continuation not handled correctly");
    }
    
    // Test for nested expressions
    #[test]
    fn test_nested_expressions() {
        let input = "result = (a + b) * (c - d) / (e ** f)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract the token types for easier comparison
        let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
        
        // Expected sequence of token types
        let expected = vec![
            TokenType::Identifier("result".to_string()),
            TokenType::Assign,
            TokenType::LeftParen,
            TokenType::Identifier("a".to_string()),
            TokenType::Plus,
            TokenType::Identifier("b".to_string()),
            TokenType::RightParen,
            TokenType::Multiply,
            TokenType::LeftParen,
            TokenType::Identifier("c".to_string()),
            TokenType::Minus,
            TokenType::Identifier("d".to_string()),
            TokenType::RightParen,
            TokenType::Divide,
            TokenType::LeftParen,
            TokenType::Identifier("e".to_string()),
            TokenType::Power,
            TokenType::Identifier("f".to_string()),
            TokenType::RightParen,
            TokenType::EOF,
        ];
        
        assert_eq!(token_types, expected, "Nested expressions not parsed correctly");
    }
    
    // Test for error handling
    #[test]
    fn test_unterminated_string() {
        let input = "\"unterminated";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Check if we have an error token
        assert!(matches!(tokens[0].token_type, TokenType::Invalid(_)), 
                "Unterminated string should produce an Invalid token");
        assert_eq!(lexer.get_errors().len(), 1, "Should report exactly one error");
    }
    
    #[test]
    fn test_invalid_indentation() {
        let input = "def test():\n  print('indented')\n    print('invalid indent')";
        let mut lexer = Lexer::new(input);
        let _tokens = lexer.tokenize();
        
        // We should still get tokens, but there should be errors
        assert!(lexer.get_errors().len() > 0, "Should report indentation errors");
        
        // Find error about inconsistent indentation
        let has_indent_error = lexer.get_errors().iter().any(|e| 
            e.message.contains("indentation") || e.message.contains("indent"));
        assert!(has_indent_error, "Should report an indentation-related error");
    }
    
    #[test]
    fn test_mixed_tabs_spaces() {
        let input = "def test():\n\t  print('mixed tabs and spaces')";
        let mut lexer = Lexer::with_config(input, LexerConfig {
            allow_tabs_in_indentation: false,
            ..Default::default()
        });
        let _tokens = lexer.tokenize();
        
        // We should still get tokens, but there should be errors about mixed indentation
        assert!(lexer.get_errors().len() > 0, "Should report mixed indentation errors");
        
        // Find error about mixed tabs and spaces
        let has_mixed_error = lexer.get_errors().iter().any(|e| 
            e.message.contains("Tabs are not allowed"));
        assert!(has_mixed_error, "Should report tabs in indentation error");        
    }
    
    #[test]
    fn test_invalid_number_format() {
        let input = "123.456.789";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Check if we have an error token
        assert!(matches!(tokens[0].token_type, TokenType::Invalid(_)),
                "Invalid number format should produce an Invalid token");
        assert_eq!(lexer.get_errors().len(), 1, "Should report exactly one error");
    }
    
    // Test invalid escape sequences
    #[test]
    fn test_invalid_escape_sequences() {
        let input = r#""Invalid escape: \z""#;
        let mut lexer = Lexer::new(input);
        let _tokens = lexer.tokenize();
        
        // We should still get a string token, but there should be errors
        assert!(lexer.get_errors().len() > 0, "Should report escape sequence errors");
        
        let has_escape_error = lexer.get_errors().iter().any(|e| 
            e.message.contains("Unknown escape sequence"));
        assert!(has_escape_error, "Should report an escape sequence error");        
    }
    
    
    
    // Test line and column numbers
    #[test]
    fn test_position_tracking() {
        let input = "x = 1\ny = 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Find the specific tokens we want to check
        let x_token = tokens.iter()
            .find(|t| matches!(&t.token_type, TokenType::Identifier(id) if id == "x"))
            .unwrap();
        let y_token = tokens.iter()
            .find(|t| matches!(&t.token_type, TokenType::Identifier(id) if id == "y"))
            .unwrap();
        
        // Check positions
        assert_eq!(x_token.line, 1, "x token should be on line 1");
        assert_eq!(x_token.column, 1, "x token should be at column 1");
        
        assert_eq!(y_token.line, 2, "y token should be on line 2");
        assert_eq!(y_token.column, 1, "y token should be at column 1");
    }
    
    // Test ignoring newlines inside parentheses, brackets, and braces
    #[test]
    fn test_newlines_in_groupings() {
        let input = "func(\n    arg1,\n    arg2\n)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract token types
        let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
        
        // No newline tokens should appear between parentheses
        let expected = vec![
            TokenType::Identifier("func".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("arg1".to_string()),
            TokenType::Comma,
            TokenType::Identifier("arg2".to_string()),
            TokenType::RightParen,
            TokenType::EOF,
        ];
        
        assert_eq!(token_types, expected, "Newlines in groupings not handled correctly");
    }
    
    // Test indentation with empty lines
    #[test]
    fn test_empty_lines() {
        let input = "def test():\n    print('line 1')\n\n    print('line 2')";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Empty lines shouldn't affect indentation
        let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
        let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
        
        assert_eq!(indent_count, 1, "Should have 1 indent");
        assert_eq!(dedent_count, 1, "Should have 1 dedent");
    }
    
    // Test for custom lexer config
    #[test]
    fn test_custom_lexer_config() {
        let input = "def test():\n\tprint('using tabs')";
        
        // Default config doesn't allow tabs
        let mut lexer1 = Lexer::new(input);
        let _tokens1 = lexer1.tokenize();
        assert!(lexer1.get_errors().len() > 0, "Default config should report tab errors");
        
        // Custom config allows tabs
        let mut lexer2 = Lexer::with_config(input, LexerConfig {
            allow_tabs_in_indentation: true,
            tab_width: 4,
            ..Default::default()
        });
        let _tokens2 = lexer2.tokenize();
        assert_eq!(lexer2.get_errors().len(), 0, "Custom config should allow tabs");
    }
    
    // Test for a comprehensive real-world code example
    #[test]
    fn test_comprehensive_code() {
        let input = r#"
def factorial(n):
    """
    Calculate the factorial of a number.
    
    Args:
        n: A positive integer
        
    Returns:
        The factorial of n
    """
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)

class Calculator:
    def __init__(self, value=0):
        self.value = value
    
    def add(self, x):
        self.value += x
        return self
    
    def multiply(self, x):
        self.value *= x
        return self

# Test the calculator
calc = Calculator(5)
result = calc.add(3).multiply(2).value
print(f"Result: {result}")  # Should be 16

# Binary, octal, and hex examples
binary = 0b1010  # 10
octal = 0o777   # 511
hexa = 0xABC    # 2748

# Raw string and bytes
raw_data = r"C:\Users\path\to\file"
bytes_data = b"\x00\x01\x02"
"#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // We should have a lot of tokens and no errors
        assert!(tokens.len() > 50, "Comprehensive example should produce many tokens");
        assert_eq!(lexer.get_errors().len(), 0, "Comprehensive example should not have errors");
        
        // Check a few key tokens to ensure it parsed correctly
        let has_def = tokens.iter().any(|t| t.token_type == TokenType::Def);
        let has_class = tokens.iter().any(|t| t.token_type == TokenType::Class);
        let has_docstring = tokens.iter().any(|t| 
            matches!(&t.token_type, TokenType::StringLiteral(s) if s.contains("Calculate the factorial")));
        
        assert!(has_def, "Should have 'def' tokens");
        assert!(has_class, "Should have 'class' tokens");
        assert!(has_docstring, "Should have docstring token");
    }

    // Add these test functions to your lexer.rs file's tests module

    #[test]
    fn test_invalid_identifiers() {
        let input = "123abc = 5";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // The lexer should tokenize this as IntLiteral(123) followed by Identifier("abc"), 
        // not as an Invalid token
        assert_eq!(tokens[0].token_type, TokenType::IntLiteral(123), "Should recognize 123 as an integer");
        assert_eq!(tokens[1].token_type, TokenType::Identifier("abc".to_string()), "Should recognize abc as an identifier");
    }

    // Test edge cases for indentation with empty lines and comments
    #[test]
    fn test_indentation_edge_cases() {
        // Test empty lines within indented blocks
        let input = "def test():\n    line1()\n\n    # Comment\n\n    line2()";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Count indents and dedents
        let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
        let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
        
        assert_eq!(indent_count, 1, "Should have 1 indent");
        assert_eq!(dedent_count, 1, "Should have 1 dedent");
        
        // Find the line2 token
        let line2 = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "line2")
        ).unwrap();
        
        // It should have proper indentation (same as line1)
        let line1 = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "line1")
        ).unwrap();
        
        assert_eq!(line2.column, line1.column, "line2 should have same indentation as line1");
    }

    #[test]
    fn test_walrus_operator() {
        assert_tokens(
            "if (n := len(items)) > 0: print(n)",
            vec![
                TokenType::If,
                TokenType::LeftParen,
                TokenType::Identifier("n".to_string()),
                TokenType::Walrus,
                TokenType::Identifier("len".to_string()),
                TokenType::LeftParen,
                TokenType::Identifier("items".to_string()),
                TokenType::RightParen,
                TokenType::RightParen,
                TokenType::GreaterThan,
                TokenType::IntLiteral(0),
                TokenType::Colon,
                TokenType::Identifier("print".to_string()),
                TokenType::LeftParen,
                TokenType::Identifier("n".to_string()),
                TokenType::RightParen,
            ]
        );
    }

    // Test for handling mixed line endings
    #[test]
    fn test_mixed_line_endings() {
        let input = "x = 1\ny = 2\r\nz = 3\n";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // We should have 3 lines with proper line numbers
        let x_token = tokens.iter().find(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "x")).unwrap();
        let y_token = tokens.iter().find(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "y")).unwrap();
        let z_token = tokens.iter().find(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "z")).unwrap();
        
        assert_eq!(x_token.line, 1, "x should be on line 1");
        assert_eq!(y_token.line, 2, "y should be on line 2");
        assert_eq!(z_token.line, 3, "z should be on line 3");
    }

    // Test for handling line continuation in different contexts
    #[test]
    fn test_line_continuation_contexts() {
        // Line continuation in lists
        assert_tokens(
            "items = [\n    1,\n    2,\n    3\n]",
            vec![
                TokenType::Identifier("items".to_string()),
                TokenType::Assign,
                TokenType::LeftBracket,
                TokenType::IntLiteral(1),
                TokenType::Comma,
                TokenType::IntLiteral(2),
                TokenType::Comma,
                TokenType::IntLiteral(3),
                TokenType::RightBracket,
            ]
        );
        
        // Line continuation in function calls
        assert_tokens(
            "result = func(\n    arg1,\n    arg2\n)",
            vec![
                TokenType::Identifier("result".to_string()),
                TokenType::Assign,
                TokenType::Identifier("func".to_string()),
                TokenType::LeftParen,
                TokenType::Identifier("arg1".to_string()),
                TokenType::Comma,
                TokenType::Identifier("arg2".to_string()),
                TokenType::RightParen,
            ]
        );
        
        // Explicit line continuation with backslash
        assert_tokens(
            "result = 1 + \\\n    2 + \\\n    3",
            vec![
                TokenType::Identifier("result".to_string()),
                TokenType::Assign,
                TokenType::IntLiteral(1),
                TokenType::Plus,
                TokenType::IntLiteral(2),
                TokenType::Plus,
                TokenType::IntLiteral(3),
            ]
        );
    }

    // Test for position tracking with nested structures
    #[test]
    fn test_position_tracking_nested() {
        let input = "nested = [(1, 2), (3, 4)]";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Find the specific nested tokens
        let left_bracket = tokens.iter().find(|t| t.token_type == TokenType::LeftBracket).unwrap();
        let first_paren = tokens.iter().find(|t| t.token_type == TokenType::LeftParen).unwrap();
        let second_paren = tokens.iter().filter(|t| t.token_type == TokenType::LeftParen).nth(1).unwrap();
        
        // Check relative positions
        assert!(left_bracket.column < first_paren.column, "Left bracket should be before first parenthesis");
        assert!(first_paren.column < second_paren.column, "First parenthesis should be before second parenthesis");
    }


    // Test edge cases for number formats
    #[test]
    fn test_number_format_edge_cases() {
        // Test scientific notation edge cases
        assert_tokens(
            "a = 1e10\nb = 1.5e+20\nc = 1.5e-10\nd = .5e3",
            vec![
                TokenType::Identifier("a".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(1e10),
                TokenType::Newline,
                TokenType::Identifier("b".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(1.5e20),
                TokenType::Newline,
                TokenType::Identifier("c".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(1.5e-10),
                TokenType::Newline,
                TokenType::Identifier("d".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(0.5e3),
            ]
        );
        
        // Test number format errors
        let input = "good = 123\nbad = 123.456.789\nrecovered = 42";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // We should have an error for the invalid number
        assert!(!lexer.get_errors().is_empty(), "Should detect invalid number format");
        
        // But we should still tokenize valid content after the error
        let recovered = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "recovered")
        );
        assert!(recovered.is_some(), "Lexer should recover and find tokens after the error");
    }

    // Test for recovery from errors
    #[test]
    fn test_error_recovery() {
        // Test recovery from unterminated string
        let input = r#"good = "valid"
    bad = "unterminated
    recovered = 42"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // We should have at least one error
        assert!(!lexer.get_errors().is_empty(), "Should detect unterminated string error");
        
        // But we should also have valid tokens after the error
        let recovered = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "recovered")
        );
        assert!(recovered.is_some(), "Lexer should recover and find tokens after the error");
        
        // Test recovery from invalid indentation
        let input = "def test():\n   print('3 spaces')\n      print('6 spaces')\nprint('valid')";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // We should have at least one error
        assert!(!lexer.get_errors().is_empty(), "Should detect indentation error");
        
        // But we should also have valid tokens after the error
        let recovered = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "print") && 
            t.lexeme == "print" && 
            t.line > 3
        );
        assert!(recovered.is_some(), "Lexer should recover and find tokens after the indentation error");
    }

        // Test for complex line continuation
    #[test]
    fn test_complex_line_continuation() {
        assert_tokens(
            "long_string = \"This is a very \\\n    long string that \\\n    spans multiple lines\"",
            vec![
                TokenType::Identifier("long_string".to_string()),
                TokenType::Assign,
                TokenType::StringLiteral("This is a very long string that spans multiple lines".to_string()),
            ]
        );
        
        // Test line continuation inside expressions
        assert_tokens(
            "result = (1 + \\\n          2) * \\\n         3",
            vec![
                TokenType::Identifier("result".to_string()),
                TokenType::Assign,
                TokenType::LeftParen,
                TokenType::IntLiteral(1),
                TokenType::Plus,
                TokenType::IntLiteral(2),
                TokenType::RightParen,
                TokenType::Multiply,
                TokenType::IntLiteral(3),
            ]
        );
    }

    // Test for complex nested structures
    #[test]
    fn test_complex_nesting() {
        assert_tokens(
            "x = [1, (2, 3), {'a': 4, 'b': [5, 6]}]",
            vec![
                TokenType::Identifier("x".to_string()),
                TokenType::Assign,
                TokenType::LeftBracket,
                TokenType::IntLiteral(1),
                TokenType::Comma,
                TokenType::LeftParen,
                TokenType::IntLiteral(2),
                TokenType::Comma,
                TokenType::IntLiteral(3),
                TokenType::RightParen,
                TokenType::Comma,
                TokenType::LeftBrace,
                TokenType::StringLiteral("a".to_string()),
                TokenType::Colon,
                TokenType::IntLiteral(4),
                TokenType::Comma,
                TokenType::StringLiteral("b".to_string()),
                TokenType::Colon,
                TokenType::LeftBracket,
                TokenType::IntLiteral(5),
                TokenType::Comma,
                TokenType::IntLiteral(6),
                TokenType::RightBracket,
                TokenType::RightBrace,
                TokenType::RightBracket,
            ]
        );
    }

    // Test for string escape edge cases
    #[test]
    fn test_string_escape_edge_cases() {
        // Test octal escapes
        assert_tokens(
            r#""\1\22\377""#,
            vec![
                TokenType::StringLiteral("\u{0001}\u{0012}\u{00FF}".to_string()),
            ]
        );
        
        // Test Unicode escapes
        assert_tokens(
            r#""\u00A9\u2764\u{1F600}""#, // copyright, heart, smile emoji
            vec![
                TokenType::StringLiteral("©❤😀".to_string()),
            ]
        );
        
        // Test raw strings with backslashes and quotes
        assert_tokens(
            r#"r"C:\path\to\file" r'\'quoted\''"#,
            vec![
                TokenType::RawString(r"C:\path\to\file".to_string()),
                TokenType::RawString(r"\'quoted\'".to_string()),
            ]
        );
    }

        // Test for complex indentation patterns
    #[test]
    fn test_complex_indentation() {
        let input = "def outer():\n    if condition:\n        nested()\n    else:\n        if another:\n            deep_nested()\n        result = 42\n    return result";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Count indents and dedents
        let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
        let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
        
        assert_eq!(indent_count, 4, "Should have 4 indents");
        assert_eq!(dedent_count, 4, "Should have 4 dedents");
    }

    // Test for numeric separators
    #[test]
    fn test_numeric_separators() {
        assert_tokens(
            "a = 1_000_000\nb = 0b1010_1010\nc = 0o777_333\nd = 0xFF_FF_FF\ne = 3.14_15_92",
            vec![
                TokenType::Identifier("a".to_string()),
                TokenType::Assign,
                TokenType::IntLiteral(1000000),
                TokenType::Newline,
                TokenType::Identifier("b".to_string()),
                TokenType::Assign,
                TokenType::BinaryLiteral(170), // 0b10101010
                TokenType::Newline,
                TokenType::Identifier("c".to_string()),
                TokenType::Assign,
                TokenType::OctalLiteral(261851), // 0o777333
                TokenType::Newline,
                TokenType::Identifier("d".to_string()),
                TokenType::Assign,
                TokenType::HexLiteral(16777215), // 0xFFFFFF
                TokenType::Newline,
                TokenType::Identifier("e".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(3.141592),
            ]
        );
    }

        // Test for complex comments and docstrings
    #[test]
    fn test_comments_and_docstrings() {
        // Test inline comments
        let input = "x = 5 # This is a comment\ny = 10 # Another comment";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), 8, "Should have 7 tokens plus EOF"); // x = 5 \n y = 10 EOF
        
        // Test docstrings (triple-quoted strings)
        let input = "def func():\n    \"\"\"This is a docstring.\n    Multi-line.\n    \"\"\"\n    pass";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract just the docstring
        let docstring = tokens.iter().find_map(|t| {
            if let TokenType::StringLiteral(s) = &t.token_type {
                Some(s.as_str())
            } else {
                None
            }
        });
        
        assert_eq!(docstring, Some("This is a docstring.\n    Multi-line.\n    "), "Docstring not parsed correctly");
    }

        // Test for f-string variants and edge cases
    #[test]
    fn test_fstring_variants() {
        // Test basic f-string
        assert_tokens(
            r#"f"Hello, {name}!""#,
            vec![
                TokenType::FString("Hello, {name}!".to_string()),
            ]
        );
        
        // Test nested expressions in f-strings
        assert_tokens(
            r#"f"Value: {2 + 3 * {4 + 5}}""#,
            vec![
                TokenType::FString("Value: {2 + 3 * {4 + 5}}".to_string()),
            ]
        );
        
        // Test f-string with dictionary unpacking
        assert_tokens(
            r#"f"Items: {', '.join(f'{k}={v}' for k, v in items.items())}""#,
            vec![
                TokenType::FString("Items: {', '.join(f'{k}={v}' for k, v in items.items())}".to_string()),
            ]
        );
        
        // Test triple-quoted f-strings
        let input = "f\"\"\"Name: {name}\nAge: {age}\n\"\"\"";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert!(matches!(tokens[0].token_type, TokenType::FString(_)), 
                "Triple-quoted f-string should be recognized as an FString token");
    }

    #[test]
    fn test_recovery_after_deep_indentation_error() {
        let input = "def outer():\n    if x:\n        nested()\n   bad_indent()\n    recovered()";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert!(lexer.get_errors().len() > 0, "Should report indentation error");
        let recovered = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "recovered")
        );
        assert!(recovered.is_some(), "Should recover after indentation error");
    }

    #[test]
    fn test_large_string_literal() {
        let large_string = "a".repeat(10_000);
        let input = format!("\"{}\"", large_string);
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), 2, "Should have StringLiteral and EOF");
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral(large_string), 
                    "Should handle large string correctly");
        assert_eq!(lexer.get_errors().len(), 0, "Should process large string without errors");
    }

    #[test]
    fn test_deep_nesting() {
        let input = "(".repeat(1000) + &")".repeat(1000);
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), 2001, "Should have 2000 tokens plus EOF");
        assert_eq!(lexer.get_errors().len(), 0, "Should handle deep nesting without errors");
    }

        #[test]
    fn test_leading_zeros_in_decimal() {
        let input = "x = 0123";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[2].token_type, TokenType::IntLiteral(123), 
                    "Should parse 0123 as 123, treating leading zero as insignificant");
        // Note: If your lexer should reject leading zeros, replace with Invalid token check
    }

    #[test]
    fn test_standalone_backslash() {
        let input = "x = 1 \\ y = 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        let backslash_idx = tokens.iter().position(|t| t.token_type == TokenType::BackSlash).unwrap();
        assert_eq!(tokens[backslash_idx + 1].token_type, TokenType::Identifier("y".to_string()), 
                    "Should tokenize content after standalone backslash");
    }

    #[test]
    fn test_ellipsis_vs_dots() {
        assert_tokens(
            "x = ... y = .. z = . . .",
            vec![
                TokenType::Identifier("x".to_string()),
                TokenType::Assign,
                TokenType::Ellipsis,
                TokenType::Identifier("y".to_string()),
                TokenType::Assign,
                TokenType::Dot,
                TokenType::Dot,
                TokenType::Identifier("z".to_string()),
                TokenType::Assign,
                TokenType::Dot,
                TokenType::Dot,
                TokenType::Dot,
            ]
        );
    }

    #[test]
    fn test_surrogate_pairs() {
        assert_tokens(
            r#""\U0001F600""#, // 😀 emoji (requires surrogate pair in UTF-16)
            vec![
                TokenType::StringLiteral("😀".to_string()),
            ]
        );
    }

    #[test]
    fn test_invalid_unicode_escape() {
        let input = r#""\u12""#; // Incomplete Unicode escape
        let mut lexer = Lexer::new(input);
        let _tokens = lexer.tokenize();
        
        assert_eq!(lexer.get_errors().len(), 1, "Should report an error for invalid Unicode escape");
    }

        #[test]
    fn test_mixed_tabs_and_spaces_with_recovery() {
        let input = "def test():\n    print('ok')\n\t  print('mixed')\n    print('recovered')";
        let mut lexer = Lexer::with_config(input, LexerConfig {
            allow_tabs_in_indentation: false,
            ..Default::default()
        });
        let tokens = lexer.tokenize();
        
        assert!(lexer.get_errors().len() > 0, "Should report mixed indentation error");
        let recovered = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "print") && 
            t.lexeme == "print" && 
            t.line == 4
        );
        assert!(recovered.is_some(), "Should recover and tokenize after mixed indentation");
    }

    #[test]
    fn test_indentation_with_comments_and_empty_lines() {
        let input = "def func():\n    x = 1\n\n    # Comment\n\n    y = 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
        let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
        assert_eq!(indent_count, 1, "Should have 1 indent");
        assert_eq!(dedent_count, 1, "Should have 1 dedent");
    }

    #[test]
    fn test_unterminated_triple_quoted_string() {
        let input = r#"x = """incomplete docstring"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert!(matches!(tokens[2].token_type, TokenType::Invalid(_)), 
                "Unterminated triple-quoted string should produce an Invalid token");
        assert_eq!(lexer.get_errors().len(), 1, "Should report one error for unterminated string");
    }

    #[test]
    fn test_escaped_quotes_in_single_quoted_string() {
        assert_tokens(
            r#"'He said \"Hello\"'"#,
            vec![
                TokenType::StringLiteral("He said \"Hello\"".to_string()),
            ]
        );
    }

    #[test]
    fn test_string_with_line_continuation() {
        let input = "\"Line split \\\n    here\"";
        assert_tokens(input, vec![TokenType::StringLiteral("Line split here".to_string())]);
    }

    #[test]
    fn test_cr_only_newlines() {
        let input = "x = 1\ry = 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let x = tokens.iter().find(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "x")).unwrap();
        let y = tokens.iter().find(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "y")).unwrap();
        assert_eq!(x.line, 1, "x should be on line 1");
        assert_eq!(y.line, 2, "y should be on line 2: {:?}", tokens);
    }

    #[test]
    fn test_multiline_comments() {
        let input = "# Line 1\n# Line 2\nx = 1";
        assert_tokens(input, vec![
            TokenType::Newline,
            TokenType::Newline,
            TokenType::Identifier("x".to_string()),
            TokenType::Assign,
            TokenType::IntLiteral(1),
        ]);
    }

    #[test]
    fn test_float_with_underscore_in_exponent() {
        let input = "x = 1.5e_10";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[2].token_type, TokenType::Invalid(_)), "Expected Invalid: {:?}", tokens[2]);
        assert_eq!(lexer.get_errors().len(), 1, "Errors: {:?}", lexer.get_errors());
    }

    #[test]
    fn test_mixed_newline_styles() {
        let input = "a = 1\nb = 2\r\nc = 3\rd = 4";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let lines: Vec<_> = tokens.iter()
            .filter(|t| matches!(&t.token_type, TokenType::Identifier(_)))
            .map(|t| t.line)
            .collect();
        assert_eq!(lines, vec![1, 2, 3, 4], "Line numbers: {:?}", lines);
    }

    #[test]
    fn test_comment_after_line_continuation() {
        let input = "x = 1 + \\\n# Comment\n    2";
        assert_tokens(input, vec![
            TokenType::Identifier("x".to_string()),
            TokenType::Assign,
            TokenType::IntLiteral(1),
            TokenType::Plus,
            TokenType::IntLiteral(2),
        ]);
    }

    #[test]
    fn test_multiple_errors_one_line() {
        let input = "x = \"unterminated\\z 123.456.789";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(lexer.get_errors().len() >= 2, "Expected 2+ errors, got: {:?}", lexer.get_errors());
        assert!(tokens.iter().any(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "x")));
    }

}