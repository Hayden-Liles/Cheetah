use std::fmt;
use std::str::FromStr;

/// Represents the different types of tokens in the Cheetah language
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Keywords
    Def,
    Return,
    If,
    Elif,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Pass,
    Import,
    From,
    As,
    True,
    False,
    None,
    And,
    Or,
    Not,
    Class,
    
    // Identifiers and literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    
    // Operators
    Plus,         // +
    Minus,        // -
    Multiply,     // *
    Divide,       // /
    FloorDivide,  // //
    Modulo,       // %
    Power,        // **
    
    Assign,       // =
    PlusAssign,   // +=
    MinusAssign,  // -=
    MulAssign,    // *=
    DivAssign,    // /=
    ModAssign,    // %=
    
    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=
    
    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Dot,          // .
    Colon,        // :
    SemiColon,    // ;
    
    // Indentation (special in Python-like syntax)
    Indent,
    Dedent,
    Newline,
    
    // End of file
    EOF,
    
    // Invalid token
    Invalid(String),
}

/// Represents a token in the Cheetah language
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
    pub lexeme: String, // The actual text of the token
}

impl Token {
    /// Creates a new token
    pub fn new(token_type: TokenType, line: usize, column: usize, lexeme: String) -> Self {
        Token {
            token_type,
            line,
            column,
            lexeme,
        }
    }
    
    /// Creates a new error token
    pub fn error(message: &str, line: usize, column: usize, lexeme: &str) -> Self {
        Token::new(
            TokenType::Invalid(message.to_string()),
            line,
            column,
            lexeme.to_owned(),
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} '{}' at {}:{}", self.token_type, self.lexeme, self.line, self.column)
    }
}

/// Configuration for the lexer
pub struct LexerConfig {
    pub tab_width: usize,
    pub enforce_indent_consistency: bool,
    pub standard_indent_size: usize,
}

impl Default for LexerConfig {
    fn default() -> Self {
        LexerConfig {
            tab_width: 4,
            enforce_indent_consistency: true,
            standard_indent_size: 4,
        }
    }
}

/// The Cheetah lexer
pub struct Lexer<'a> {
    input: &'a str,
    chars: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    current_indent: usize,
    config: LexerConfig,
    errors: Vec<String>,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer with default configuration
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            chars: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0], // Start with 0 indentation
            current_indent: 0,
            config: LexerConfig::default(),
            errors: Vec::new(),
        }
    }
    
    /// Creates a new lexer with custom configuration
    pub fn with_config(input: &'a str, config: LexerConfig) -> Self {
        Lexer {
            input,
            chars: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0], // Start with 0 indentation
            current_indent: 0,
            config,
            errors: Vec::new(),
        }
    }
    
    /// Returns any errors encountered during lexing
    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }
    
    /// Adds an error message to the error list
    fn add_error(&mut self, message: &str) {
        let error_message = format!("Line {}, Column {}: {}", self.line, self.column, message);
        self.errors.push(error_message);
    }
    
    /// Tokenizes the input string into a vector of tokens
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut pending_indentation_change = false;
        
        // Process input until EOF
        while let Some(token) = self.next_token() {
            if token.token_type == TokenType::EOF {
                // Handle any remaining dedents at the end of the file
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
            
            // Store token information before we move it
            let token_type = token.token_type.clone();
            let token_line = token.line;
            
            // Push the token to our collection
            tokens.push(token);
            
            // If we just saw a newline, check indentation for the next token
            if matches!(token_type, TokenType::Newline) {
                pending_indentation_change = true;
                continue;
            }
            
            // After a newline, check if we need to insert indentation tokens before this token
            if pending_indentation_change {
                // Check if indentation increased
                if self.current_indent > *self.indent_stack.last().unwrap_or(&0) {
                    // Check for consistency if enabled
                    if self.config.enforce_indent_consistency && 
                       self.current_indent % self.config.standard_indent_size != 0 {
                        self.add_error(&format!(
                            "Inconsistent indentation. Expected multiple of {} spaces but got {}.", 
                            self.config.standard_indent_size, self.current_indent
                        ));
                    }
                    
                    // Insert an indent token BEFORE the current token
                    let indent_token = Token::new(
                        TokenType::Indent,
                        token_line,
                        1, // Indent is always at the start of the line
                        " ".repeat(self.current_indent),
                    );
                    
                    // Update indentation stack
                    self.indent_stack.push(self.current_indent);
                    
                    // Insert the Indent token before the current token
                    tokens.insert(tokens.len() - 1, indent_token);
                } 
                // Check if indentation decreased
                else if self.current_indent < *self.indent_stack.last().unwrap_or(&0) {
                    // Generate dedent tokens as needed
                    let mut dedent_tokens = Vec::new();
                    
                    while self.indent_stack.len() > 1 && self.current_indent < *self.indent_stack.last().unwrap() {
                        let _last_indent = self.indent_stack.pop().unwrap();
                        
                        // Check if we're going back to a valid indentation level
                        if self.indent_stack.last().unwrap() != &self.current_indent && 
                           self.indent_stack.iter().all(|i| i != &self.current_indent) {
                            let msg = format!(
                                "Inconsistent indentation. Current indent level {} doesn't match any previous level.",
                                self.current_indent
                            );
                            self.add_error(&msg);
                        }
                        
                        dedent_tokens.push(Token::new(
                            TokenType::Dedent,
                            token_line,
                            1,
                            "".to_string(),
                        ));
                    }
                    
                    // Insert all dedent tokens before the current token
                    for (i, dedent) in dedent_tokens.into_iter().enumerate() {
                        tokens.insert(tokens.len() - 1 - i, dedent);
                    }
                }
                
                pending_indentation_change = false;
            }
        }
        
        tokens
    }
    
    /// Gets the next token from the input
    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        
        if self.position >= self.chars.len() {
            return Some(Token::new(
                TokenType::EOF,
                self.line,
                self.column,
                "".to_string(),
            ));
        }
        
        let current_char = self.chars[self.position];
        
        // Check for newlines and indentation
        if current_char == '\n' {
            return self.handle_newline();
        }
        
        // Check for identifiers and keywords
        if current_char.is_alphabetic() || current_char == '_' {
            return Some(self.handle_identifier());
        }
        
        // Check for numeric literals
        if current_char.is_digit(10) {
            return Some(self.handle_number());
        }
        
        // Check for triple-quoted strings
        if (current_char == '"' && self.check_next('"') && self.check_next_n('"', 2)) ||
           (current_char == '\'' && self.check_next('\'') && self.check_next_n('\'', 2)) {
            return Some(self.handle_triple_quoted_string());
        }
        
        // Check for regular string literals
        if current_char == '"' || current_char == '\'' {
            return Some(self.handle_string());
        }
        
        // Check for comments
        if current_char == '#' {
            // Skip the comment
            while self.position < self.chars.len() && self.chars[self.position] != '\n' {
                self.consume_char();
            }
            
            // If we're at the end of the file after a comment, return EOF
            if self.position >= self.chars.len() {
                return Some(Token::new(
                    TokenType::EOF,
                    self.line,
                    self.column,
                    "".to_string(),
                ));
            }
            
            // Otherwise if we reached a newline, handle it
            if self.position < self.chars.len() && self.chars[self.position] == '\n' {
                return self.handle_newline();
            }
        }
        
        // Handle operators and delimiters
        Some(self.handle_operator_or_delimiter())
    }
    
    /// Handles newlines and indentation
    fn handle_newline(&mut self) -> Option<Token> {
        self.consume_char(); // Consume the newline
        
        // Skip empty lines and just count them for line number tracking
        while self.position < self.chars.len() && self.chars[self.position] == '\n' {
            self.consume_char();
        }
        
        // Count indentation on the new line
        let indent_size = self.count_indentation();
        
        // Create newline token
        let newline_token = Token::new(
            TokenType::Newline,
            self.line - 1, // Line where the newline started
            self.column - 1,
            "\n".to_string(),
        );
        
        // Update the current indentation level
        self.current_indent = indent_size;
        
        // Return the newline token, indentation tokens will be handled separately
        Some(newline_token)
    }
    
    /// Counts the indentation level at the current position
    fn count_indentation(&mut self) -> usize {
        let mut count = 0;
        let mut has_tabs = false;
        let mut has_spaces = false;
        
        let _start_pos = self.position;
        
        while self.position < self.chars.len() {
            let c = self.chars[self.position];
            if c == ' ' {
                has_spaces = true;
                count += 1;
                self.consume_char();
            } else if c == '\t' {
                has_tabs = true;
                // Convert tab to spaces according to config
                count += self.config.tab_width;
                self.consume_char();
            } else {
                break;
            }
        }
        
        // Check for mixed tabs and spaces
        if has_tabs && has_spaces && self.config.enforce_indent_consistency {
            self.add_error("Mixed tabs and spaces in indentation");
        }
        
        count
    }
    
    /// Handles identifiers and keywords
    fn handle_identifier(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        
        // Consume all alphanumeric and underscore characters
        self.consume_while(|c| c.is_alphanumeric() || c == '_');
        
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if it's a keyword
        let token_type = match text.as_str() {
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
            _ => TokenType::Identifier(text.to_string()),
        };
        
        Token::new(token_type, self.line, start_col, text.to_string())
    }
    
    /// Handles numeric literals (integers, floats)
    fn handle_number(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let mut is_float = false;
        
        // Parse the integer part
        self.consume_while(|c| c.is_digit(10));
        
        // Check for decimal point
        if self.position < self.chars.len() && self.chars[self.position] == '.' {
            is_float = true;
            self.consume_char();
            
            // Parse the fractional part
            self.consume_while(|c| c.is_digit(10));
        }
        
        // Check for exponent (e or E)
        if self.position < self.chars.len() && 
           (self.chars[self.position] == 'e' || self.chars[self.position] == 'E') {
            is_float = true;
            self.consume_char();
            
            // Optional sign
            if self.position < self.chars.len() && 
               (self.chars[self.position] == '+' || self.chars[self.position] == '-') {
                self.consume_char();
            }
            
            // Exponent digits
            let exp_start = self.position;
            self.consume_while(|c| c.is_digit(10));
            
            // Check if we have at least one digit in the exponent
            if self.position == exp_start {
                let text = self.get_slice(start_pos, self.position);
                return Token::error(
                    "Invalid exponent in float literal",
                    self.line,
                    start_col,
                    text
                );
            }
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        let token_type = if is_float {
            match f64::from_str(&text) {
                Ok(value) => TokenType::FloatLiteral(value),
                Err(_) => {
                    self.add_error(&format!("Invalid float literal: {}", text));
                    TokenType::Invalid(format!("Invalid float: {}", text))
                }
            }
        } else {
            match i64::from_str(&text) {
                Ok(value) => TokenType::IntLiteral(value),
                Err(_) => {
                    self.add_error(&format!("Invalid integer literal: {}", text));
                    TokenType::Invalid(format!("Invalid integer: {}", text))
                }
            }
        };
        
        Token::new(token_type, self.line, start_col, text.to_string())
    }
    
    /// Handles string literals
    fn handle_string(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let quote_char = self.chars[self.position];
        
        self.consume_char(); // Consume the opening quote
        
        let mut escaped = false;
        let mut string_content = String::new();
        
        while self.position < self.chars.len() {
            let current_char = self.chars[self.position];
            
            if escaped {
                // Handle escape sequences
                let escaped_char = match current_char {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    'b' => '\u{0008}', // Backspace
                    'f' => '\u{000C}', // Form feed
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    'x' => self.handle_hex_escape(&mut string_content),
                    'u' => self.handle_unicode_escape(&mut string_content),
                    _ => {
                        self.add_error(&format!("Unknown escape sequence: \\{}", current_char));
                        current_char // Use the literal character
                    }
                };
                
                if current_char != 'x' && current_char != 'u' {
                    string_content.push(escaped_char);
                }
                escaped = false;
            } else if current_char == '\\' {
                escaped = true;
            } else if current_char == quote_char {
                // End of string
                self.consume_char(); // Consume the closing quote
                break;
            } else if current_char == '\n' {
                // Unterminated string literal
                self.add_error("Unterminated string literal: newline in string");
                let text = self.get_slice(start_pos, self.position);
                return Token::error(
                    "Unterminated string literal",
                    self.line,
                    start_col,
                    text
                );
            } else {
                string_content.push(current_char);
            }
            
            self.consume_char();
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if we have a proper closing quote
        if self.position > self.chars.len() || 
           (self.position == self.chars.len() && (text.chars().last() != Some(quote_char))) {
            self.add_error("Unterminated string literal");
            return Token::error(
                "Unterminated string literal",
                self.line,
                start_col,
                &text
            );
        }
        
        Token::new(TokenType::StringLiteral(string_content), self.line, start_col, text.to_string())
    }
    
    /// Handles triple-quoted strings (multi-line strings)
    fn handle_triple_quoted_string(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let quote_char = self.chars[self.position];
        
        // Consume the three quotes
        self.consume_char();
        self.consume_char();
        self.consume_char();
        
        let mut string_content = String::new();
        let mut consecutive_quotes = 0;
        
        while self.position < self.chars.len() {
            let current_char = self.chars[self.position];
            
            if current_char == quote_char {
                consecutive_quotes += 1;
                
                // Check if we've reached the end (three consecutive quotes)
                if consecutive_quotes == 3 {
                    break;
                }
            } else {
                // If we had some quotes but not three, add them to the content
                for _ in 0..consecutive_quotes {
                    string_content.push(quote_char);
                }
                consecutive_quotes = 0;
                string_content.push(current_char);
            }
            
            self.consume_char();
        }
        
        // Consume the three closing quotes if we found them
        if consecutive_quotes == 3 {
            self.consume_char();
            self.consume_char();
            self.consume_char();
        } else {
            // Unterminated triple-quoted string
            let text_str = self.get_slice(start_pos, self.position).to_string();
            self.add_error("Unterminated triple-quoted string");
            return Token::error(
                "Unterminated triple-quoted string",
                self.line,
                start_col,
                &text_str
            );
        }
        
        let text = self.get_slice(start_pos, self.position).to_string();
        Token::new(TokenType::StringLiteral(string_content), self.line, start_col, text.to_string())
    }
    
    /// Handles \x escape sequences in strings (hex values)
    fn handle_hex_escape(&mut self, string_content: &mut String) -> char {
        self.consume_char(); // Consume the 'x'
        
        let mut hex_value = String::with_capacity(2);
        let mut escape_count = 0;
        
        // Read exactly 2 hex digits
        for _ in 0..2 {
            if self.position < self.chars.len() && 
               self.chars[self.position].is_ascii_hexdigit() {
                hex_value.push(self.chars[self.position]);
                self.consume_char();
                escape_count += 1;
            } else {
                self.add_error("Invalid hex escape sequence: expected 2 hex digits");
                return '?'; // Error placeholder
            }
        }
        
        // Clone hex_value to avoid borrow issues
        let hex_value_clone = hex_value.clone();
        
        // Convert hex to char
        if let Ok(byte) = u8::from_str_radix(&hex_value, 16) {
            let c = byte as char;
            string_content.push(c);
        } else {
            let err_msg = format!("Invalid hex escape sequence: \\x{}", hex_value_clone);
            self.add_error(&err_msg);
        }
        
        '\0' // Return null char as the actual char is added to string_content
    }
    
    /// Handles \u escape sequences in strings (unicode values)
    fn handle_unicode_escape(&mut self, string_content: &mut String) -> char {
        self.consume_char(); // Consume the 'u'
        
        // Check for opening brace
        let has_braces = self.position < self.chars.len() && self.chars[self.position] == '{';
        if has_braces {
            self.consume_char();
        }
        
        let start_pos = self.position;
        
        // Read 1-6 hex digits for Unicode code point
        self.consume_while(|c| c.is_ascii_hexdigit());
        
        let hex_str = self.get_slice(start_pos, self.position);
        
        // Store the hex string to avoid borrow issues
        let hex_string = hex_str.to_string();
        
        // Check closing brace if needed
        if has_braces {
            if self.position < self.chars.len() && self.chars[self.position] == '}' {
                self.consume_char();
            } else {
                self.add_error("Unclosed Unicode escape sequence: missing closing brace");
                return '?';
            }
        }
        
        // Convert to Unicode character
        if let Ok(code_point) = u32::from_str_radix(&hex_string, 16) {
            match char::from_u32(code_point) {
                Some(c) => {
                    string_content.push(c);
                    '\0' // Return null char as the actual char is added to string_content
                },
                None => {
                    let err_msg = format!("Invalid Unicode code point: U+{:X}", code_point);
                    self.add_error(&err_msg);
                    '?' // Error placeholder
                }
            }
        } else {
            let err_msg = format!("Invalid Unicode escape sequence: \\u{{{}}}", hex_string);
            self.add_error(&err_msg);
            '?' // Error placeholder
        }
    }
    
    /// Handles operators and delimiters
    fn handle_operator_or_delimiter(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let current_char = self.chars[self.position];
        
        self.consume_char();
        
        // Check for two-character operators
        let text_str = self.get_slice(start_pos, self.position).to_string();
        let token_type = match current_char {
            '+' => {
                if self.check_next('=') {
                    self.consume_char();
                    TokenType::PlusAssign
                } else {
                    TokenType::Plus
                }
            },
            '-' => {
                if self.check_next('=') {
                    self.consume_char();
                    TokenType::MinusAssign
                } else {
                    TokenType::Minus
                }
            },
            '*' => {
                if self.check_next('*') {
                    self.consume_char();
                    TokenType::Power
                } else if self.check_next('=') {
                    self.consume_char();
                    TokenType::MulAssign
                } else {
                    TokenType::Multiply
                }
            },
            '/' => {
                if self.check_next('/') {
                    self.consume_char();
                    TokenType::FloorDivide
                } else if self.check_next('=') {
                    self.consume_char();
                    TokenType::DivAssign
                } else {
                    TokenType::Divide
                }
            },
            '%' => {
                if self.check_next('=') {
                    self.consume_char();
                    TokenType::ModAssign
                } else {
                    TokenType::Modulo
                }
            },
            '=' => {
                if self.check_next('=') {
                    self.consume_char();
                    TokenType::Equal
                } else {
                    TokenType::Assign
                }
            },
            '!' => {
                if self.check_next('=') {
                    self.consume_char();
                    TokenType::NotEqual
                } else {
                    self.add_error("Unexpected character: !");
                    TokenType::Invalid("Unexpected character: !".to_string())
                }
            },
            '<' => {
                if self.check_next('=') {
                    self.consume_char();
                    TokenType::LessEqual
                } else {
                    TokenType::LessThan
                }
            },
            '>' => {
                if self.check_next('=') {
                    self.consume_char();
                    TokenType::GreaterEqual
                } else {
                    TokenType::GreaterThan
                }
            },
            
            // Delimiters
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            ':' => TokenType::Colon,
            ';' => TokenType::SemiColon,
            
            // Invalid characters
            _ => {
                let msg = format!("Unexpected character: {}", current_char);
                self.add_error(&msg);
                TokenType::Invalid(msg)
            }
        };
        
        let text = self.get_slice(start_pos, self.position);
        Token::new(token_type, self.line, start_col, text.to_string())
    }
    
    /// Checks if the next character is the expected character
    fn check_next(&self, expected: char) -> bool {
        self.position < self.chars.len() && self.chars[self.position] == expected
    }
    
    /// Checks if the character at position + n is the expected character
    fn check_next_n(&self, expected: char, n: usize) -> bool {
        self.position + n < self.chars.len() && self.chars[self.position + n] == expected
    }
    
    /// Consumes the current character and advances the position
    fn consume_char(&mut self) {
        if self.position < self.chars.len() {
            let current_char = self.chars[self.position];
            
            if current_char == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            
            self.position += 1;
        }
    }
    
    /// Consumes characters while a predicate is true
    fn consume_while<F>(&mut self, predicate: F) 
    where
        F: Fn(char) -> bool
    {
        while self.position < self.chars.len() && predicate(self.chars[self.position]) {
            self.consume_char();
        }
    }
    
    /// Gets a slice of the input string
    fn get_slice(&self, start: usize, end: usize) -> &str {
        let start_byte = self.byte_index(start);
        let end_byte = self.byte_index(end);
        &self.input[start_byte..end_byte]
    }
    
    /// Converts a character index to a byte index
    fn byte_index(&self, char_index: usize) -> usize {
        // Count the bytes up to the character index
        let mut byte_index = 0;
        let input_bytes = self.input.as_bytes();
        let mut char_count = 0;
        
        while char_count < char_index && byte_index < input_bytes.len() {
            // UTF-8 encoding: skip the correct number of bytes
            let width = if (input_bytes[byte_index] & 0x80) == 0 {
                1
            } else if (input_bytes[byte_index] & 0xE0) == 0xC0 {
                2
            } else if (input_bytes[byte_index] & 0xF0) == 0xE0 {
                3
            } else {
                4
            };
            
            byte_index += width;
            char_count += 1;
        }
        
        byte_index
    }
    
    /// Skips whitespace (spaces, tabs) but not newlines
    fn skip_whitespace(&mut self) {
        self.consume_while(|c| c == ' ' || c == '\t' || c == '\r');
    }
    
    /// Prints source line with error location for better error reporting
    pub fn format_error_location(&self, line: usize, column: usize) -> String {
        let mut result = String::new();
        let lines: Vec<&str> = self.input.lines().collect();
        
        if line <= lines.len() {
            let source_line = lines[line - 1];
            result.push_str(&format!("Line {}: {}\n", line, source_line));
            
            // Add caret pointing to the error position
            if column <= source_line.len() + 1 {
                result.push_str(&format!("{}^\n", " ".repeat(column + 6)));
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("if elif else def return while for in break continue");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::If);
        assert_eq!(tokens[1].token_type, TokenType::Elif);
        assert_eq!(tokens[2].token_type, TokenType::Else);
        assert_eq!(tokens[3].token_type, TokenType::Def);
        assert_eq!(tokens[4].token_type, TokenType::Return);
        assert_eq!(tokens[5].token_type, TokenType::While);
        assert_eq!(tokens[6].token_type, TokenType::For);
        assert_eq!(tokens[7].token_type, TokenType::In);
        assert_eq!(tokens[8].token_type, TokenType::Break);
        assert_eq!(tokens[9].token_type, TokenType::Continue);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("x y variable_name _private var123");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::Identifier("y".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Identifier("variable_name".to_string()));
        assert_eq!(tokens[3].token_type, TokenType::Identifier("_private".to_string()));
        assert_eq!(tokens[4].token_type, TokenType::Identifier("var123".to_string()));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("123 3.14 0.5 1e10 1.5e-3");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::IntLiteral(123));
        assert_eq!(tokens[1].token_type, TokenType::FloatLiteral(3.14));
        assert_eq!(tokens[2].token_type, TokenType::FloatLiteral(0.5));
        assert_eq!(tokens[3].token_type, TokenType::FloatLiteral(1e10));
        assert_eq!(tokens[4].token_type, TokenType::FloatLiteral(1.5e-3));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("\"hello\" 'world' \"escape\\nsequence\"");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral("hello".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::StringLiteral("world".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::StringLiteral("escape\nsequence".to_string()));
    }
    
    #[test]
    fn test_triple_quoted_strings() {
        let mut lexer = Lexer::new("\"\"\"This is a\nmulti-line\nstring\"\"\" '''Another\none'''");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral("This is a\nmulti-line\nstring".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::StringLiteral("Another\none".to_string()));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / // % ** = += -= *= /= %= == != < <= > >=");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::Plus);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Multiply);
        assert_eq!(tokens[3].token_type, TokenType::Divide);
        assert_eq!(tokens[4].token_type, TokenType::FloorDivide);
        assert_eq!(tokens[5].token_type, TokenType::Modulo);
        assert_eq!(tokens[6].token_type, TokenType::Power);
        assert_eq!(tokens[7].token_type, TokenType::Assign);
        assert_eq!(tokens[8].token_type, TokenType::PlusAssign);
        assert_eq!(tokens[9].token_type, TokenType::MinusAssign);
        assert_eq!(tokens[10].token_type, TokenType::MulAssign);
        assert_eq!(tokens[11].token_type, TokenType::DivAssign);
        assert_eq!(tokens[12].token_type, TokenType::ModAssign);
        assert_eq!(tokens[13].token_type, TokenType::Equal);
        assert_eq!(tokens[14].token_type, TokenType::NotEqual);
        assert_eq!(tokens[15].token_type, TokenType::LessThan);
        assert_eq!(tokens[16].token_type, TokenType::LessEqual);
        assert_eq!(tokens[17].token_type, TokenType::GreaterThan);
        assert_eq!(tokens[18].token_type, TokenType::GreaterEqual);
    }

    #[test]
    fn test_delimiters() {
        let mut lexer = Lexer::new("( ) [ ] { } , . : ;");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens[1].token_type, TokenType::RightParen);
        assert_eq!(tokens[2].token_type, TokenType::LeftBracket);
        assert_eq!(tokens[3].token_type, TokenType::RightBracket);
        assert_eq!(tokens[4].token_type, TokenType::LeftBrace);
        assert_eq!(tokens[5].token_type, TokenType::RightBrace);
        assert_eq!(tokens[6].token_type, TokenType::Comma);
        assert_eq!(tokens[7].token_type, TokenType::Dot);
        assert_eq!(tokens[8].token_type, TokenType::Colon);
        assert_eq!(tokens[9].token_type, TokenType::SemiColon);
    }

    #[test]
    fn test_indentation() {
        let mut lexer = Lexer::new("if True:\n    print(\"indented\")\n    if False:\n        nested\n    back\noutside");
        let tokens = lexer.tokenize();
        
        // Print the tokens for debugging
        println!("Tokens in indentation test:");
        for (i, token) in tokens.iter().enumerate() {
            println!("{}: {:?} -> {}", i, token.token_type, token.lexeme);
        }
        
        // Now look for specific patterns
        
        // Find if the token sequence includes indentation after "if True:"
        let mut if_true_index = None;
        for i in 0..tokens.len().saturating_sub(3) {
            if let (TokenType::If, TokenType::True, TokenType::Colon, TokenType::Newline) = 
               (&tokens[i].token_type, &tokens[i+1].token_type, &tokens[i+2].token_type, &tokens[i+3].token_type) {
                if_true_index = Some(i);
                break;
            }
        }
        
        // If we found "if True:" followed by newline, check for indent after it
        let if_true_followed_by_indent = if let Some(idx) = if_true_index {
            // The indent should be right after the newline
            idx + 4 < tokens.len() && matches!(tokens[idx+4].token_type, TokenType::Indent)
        } else {
            false
        };
        
        // Find if the token sequence includes indentation after "if False:"
        let mut if_false_index = None;
        for i in 0..tokens.len().saturating_sub(3) {
            if let (TokenType::If, TokenType::False, TokenType::Colon, TokenType::Newline) = 
               (&tokens[i].token_type, &tokens[i+1].token_type, &tokens[i+2].token_type, &tokens[i+3].token_type) {
                if_false_index = Some(i);
                break;
            }
        }
        
        // If we found "if False:" followed by newline, check for indent after it
        let if_false_followed_by_indent = if let Some(idx) = if_false_index {
            // The indent should be right after the newline
            idx + 4 < tokens.len() && matches!(tokens[idx+4].token_type, TokenType::Indent)
        } else {
            false
        };
        
        // Make sure there's at least one Dedent token
        let has_dedent = tokens.iter().any(|t| matches!(t.token_type, TokenType::Dedent));
        
        // Assert the conditions
        assert!(if_true_followed_by_indent, "Missing indentation after 'if True:'");
        assert!(if_false_followed_by_indent, "Missing indentation after 'if False:'");
        assert!(has_dedent, "Missing Dedent token");
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("# This is a comment\nx = 5 # inline comment");
        let tokens = lexer.tokenize();
        
        // Comments should be skipped, but newlines are preserved in Python-like syntax
        assert_eq!(tokens[0].token_type, TokenType::Newline);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Assign);
        assert_eq!(tokens[3].token_type, TokenType::IntLiteral(5));
        assert_eq!(tokens[4].token_type, TokenType::EOF);
    }

    #[test]
    fn test_complete_program() {
        let input = r#"
def factorial(n):
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)

result = factorial(5)
print("Factorial of 5 is", result)
"#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Just check that we have a reasonable number of tokens
        assert!(tokens.len() > 20);
        
        // Check a few key tokens
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Def)));
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Identifier(ref s) if s == "factorial")));
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Return)));
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Identifier(ref s) if s == "print")));
    }
    
    #[test]
    fn test_error_recovery() {
        let mut lexer = Lexer::new("x = @invalid\ny = 10");
        let tokens = lexer.tokenize();
        
        // Check that we get an Invalid token but continue lexing
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Invalid(_))));
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Identifier(ref s) if s == "y")));
        assert!(!lexer.get_errors().is_empty());
    }
    
    #[test]
    fn test_unicode_escape() {
        let mut lexer = Lexer::new(r#""\u{1F600}" "\u00A9""#);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral("😀".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::StringLiteral("©".to_string()));
    }
    
    #[test]
    fn test_hex_escape() {
        let mut lexer = Lexer::new(r#""\x41\x42""#);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral("AB".to_string()));
    }
    
    #[test]
    fn test_inconsistent_indentation() {
        let config = LexerConfig {
            enforce_indent_consistency: true,
            standard_indent_size: 4,
            ..Default::default()
        };
        
        let mut lexer = Lexer::with_config("if True:\n   bad_indent\n    good_indent", config);
        let tokens = lexer.tokenize();
        
        // Should be an error in the errors list
        let errors = lexer.get_errors();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("indentation")));
    }
    
    #[test]
    fn test_mixed_tabs_spaces() {
        let config = LexerConfig {
            enforce_indent_consistency: true,
            ..Default::default()
        };
        
        let mut lexer = Lexer::with_config("if True:\n\t  mixed_indent", config);
        let tokens = lexer.tokenize();
        
        // Should be an error in the errors list
        let errors = lexer.get_errors();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("tabs and spaces")));
    }
}