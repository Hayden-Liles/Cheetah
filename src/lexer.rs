use std::fmt;
use std::str::FromStr;
use std::collections::HashSet;

/// Represents the different types of tokens in the Cheetah language
/// 
/// The Cheetah language has Python-like syntax with some custom extensions.
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
    With,
    Assert,
    Async,
    Await,
    Try,
    Except,
    Finally,
    Raise,
    Lambda,
    Global,
    Nonlocal,
    Yield,
    Del,
    Is,
    
    // Identifiers and literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BytesLiteral(Vec<u8>),
    RawString(String),
    FString(String),
    BinaryLiteral(i64),
    OctalLiteral(i64),
    HexLiteral(i64),
    
    // Operators
    Plus,         // +
    Minus,        // -
    Multiply,     // *
    Divide,       // /
    FloorDivide,  // //
    Modulo,       // %
    Power,        // **
    MatrixMul,    // @
    
    Assign,       // =
    PlusAssign,   // +=
    MinusAssign,  // -=
    MulAssign,    // *=
    DivAssign,    // /=
    ModAssign,    // %=
    PowAssign,    // **=
    MatrixMulAssign, // @=
    FloorDivAssign,  // //=
    BitwiseAndAssign, // &=
    BitwiseOrAssign,  // |=
    BitwiseXorAssign, // ^=
    ShiftLeftAssign,  // <<=
    ShiftRightAssign, // >>=
    
    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=
    
    BitwiseAnd,   // &
    BitwiseOr,    // |
    BitwiseXor,   // ^
    BitwiseNot,   // ~
    ShiftLeft,    // <<
    ShiftRight,   // >>
    
    Walrus,       // :=
    Ellipsis,     // ...
    
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
    Arrow,        // ->
    At,           // @ (for decorators)
    
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
#[derive(Debug, Clone)]
pub struct LexerConfig {
    pub tab_width: usize,
    pub enforce_indent_consistency: bool,
    pub standard_indent_size: usize,
    pub allow_trailing_semicolon: bool,
    pub allow_tabs_in_indentation: bool,
    pub strict_line_joining: bool,
}

impl Default for LexerConfig {
    fn default() -> Self {
        LexerConfig {
            tab_width: 4,
            enforce_indent_consistency: true,
            standard_indent_size: 4,
            allow_trailing_semicolon: true,
            allow_tabs_in_indentation: false,
            strict_line_joining: true,
        }
    }
}

/// Error type for lexer errors
#[derive(Debug, Clone)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    pub suggestion: Option<String>,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {}, Column {}: {}", self.line, self.column, self.message)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " - Suggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

/// The Cheetah lexer
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
    /// Creates a new lexer with default configuration
    pub fn new(input: &'a str) -> Self {
        let mut keywords = HashSet::new();
        // Add all Python keywords
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
            indent_stack: vec![0], // Start with 0 indentation
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
    
    /// Creates a new lexer with custom configuration
    pub fn with_config(input: &'a str, config: LexerConfig) -> Self {
        let mut lexer = Lexer::new(input);
        lexer.config = config;
        lexer
    }
    
    /// Returns any errors encountered during lexing
    pub fn get_errors(&self) -> &[LexerError] {
        &self.errors
    }
    
    /// Adds an error message to the error list
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
    
    /// Adds an error message with suggestion to the error list
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
    
    /// Gets a short context snippet for error reporting
    fn get_error_context(&self) -> String {
        // Get the current line of code
        let lines: Vec<&str> = self.input.lines().collect();
        if self.line <= lines.len() {
            lines[self.line - 1].to_string()
        } else {
            String::new()
        }
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
            
            // Update nesting levels for parentheses, brackets, and braces
            self.update_nesting_level(&token.token_type);
            
            // Store token information before we move it
            let token_type = token.token_type.clone();
            let token_line = token.line;
            
            // Push the token to our collection
            tokens.push(token);
            
            // If we just saw a newline and we're not in a nested structure, check indentation
            if matches!(token_type, TokenType::Newline) && 
               self.paren_level == 0 && self.bracket_level == 0 && self.brace_level == 0 {
                pending_indentation_change = true;
                continue;
            }
            
            // After a newline, check if we need to insert indentation tokens before this token
            if pending_indentation_change {
                self.handle_indentation_change(&mut tokens, token_line);
                pending_indentation_change = false;
            }
        }
        
        tokens
    }
    
    /// Updates the nesting level counters for parentheses, brackets, and braces
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
    
    /// Handles indentation changes after a newline
    fn handle_indentation_change(&mut self, tokens: &mut Vec<Token>, token_line: usize) {
        // Check if indentation increased
        if self.current_indent > *self.indent_stack.last().unwrap_or(&0) {
            // Check for consistency if enabled
            if self.config.enforce_indent_consistency && 
                self.current_indent % self.config.standard_indent_size != 0 {
                self.add_error_with_suggestion(
                    &format!(
                        "Inconsistent indentation. Expected multiple of {} spaces but got {}.", 
                        self.config.standard_indent_size, self.current_indent
                    ),
                    &format!("Use {} spaces for indentation", self.config.standard_indent_size)
                );
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
                    self.add_error_with_suggestion(
                        &msg, 
                        "Ensure indentation matches a previous level"
                    );
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
    }
    
    /// Gets the next token from the input
    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        
        if self.is_at_end() {
            return Some(Token::new(
                TokenType::EOF,
                self.line,
                self.column,
                "".to_string(),
            ));
        }
        
        let current_char = self.peek_char();
        
        // Check for newlines and indentation
        if current_char == '\n' {
            // Skip newlines inside parentheses, brackets, and braces
            if self.paren_level > 0 || self.bracket_level > 0 || self.brace_level > 0 {
                self.consume_char(); // Just consume the newline without generating a token
                self.skip_whitespace(); // Skip any following whitespace
                return self.next_token(); // Continue to the next token
            }
            return self.handle_newline();
        }
        
        // Check for line continuation with backslash
        if current_char == '\\' && self.peek_char_n(1) == '\n' {
            self.consume_char(); // Consume backslash
            self.consume_char(); // Consume newline
            return self.next_token(); // Get the next token after continuation
        }
        
        // Check for string prefixes (r, f, b, etc.)
        if (current_char == 'r' || current_char == 'R' || 
            current_char == 'f' || current_char == 'F' ||
            current_char == 'b' || current_char == 'B') && 
            (self.peek_char_n(1) == '"' || self.peek_char_n(1) == '\'') {
            let prefix = current_char;
            self.consume_char(); // Consume the prefix
            
            // Handle the string based on its prefix
            match prefix {
                'r' | 'R' => return Some(self.handle_raw_string()),
                'f' | 'F' => return Some(self.handle_formatted_string()),
                'b' | 'B' => return Some(self.handle_bytes_string()),
                _ => unreachable!()
            }
        }
        
        // Check for triple-quoted string prefixes
        if (current_char == 'r' || current_char == 'f' || current_char == 'b') && 
            ((self.peek_char_n(1) == '"' && self.peek_char_n(2) == '"' && self.peek_char_n(3) == '"') ||
             (self.peek_char_n(1) == '\'' && self.peek_char_n(2) == '\'' && self.peek_char_n(3) == '\'')) {
            let prefix = current_char;
            self.consume_char(); // Consume the prefix
            
            // Handle the triple-quoted string based on its prefix
            match prefix {
                'r' => return Some(self.handle_raw_triple_quoted_string()),
                'f' => return Some(self.handle_formatted_triple_quoted_string()),
                'b' => return Some(self.handle_bytes_triple_quoted_string()),
                _ => unreachable!()
            }
        }
        
        // Check for triple-quoted strings
        if (current_char == '"' && self.peek_char_n(1) == '"' && self.peek_char_n(2) == '"') ||
           (current_char == '\'' && self.peek_char_n(1) == '\'' && self.peek_char_n(2) == '\'') {
            return Some(self.handle_triple_quoted_string());
        }
        
        // Check for identifiers and keywords
        if current_char.is_alphabetic() || current_char == '_' {
            return Some(self.handle_identifier());
        }
        
        // Check for numeric literals
        if current_char.is_digit(10) || 
           (current_char == '.' && self.peek_char_n(1).is_digit(10)) {
            return Some(self.handle_number());
        }
        
        // Check for regular string literals
        if current_char == '"' || current_char == '\'' {
            return Some(self.handle_string());
        }
        
        // Check for comments
        if current_char == '#' {
            // Skip the comment
            while !self.is_at_end() && self.peek_char() != '\n' {
                self.consume_char();
            }
            
            // If we're at the end of the file after a comment, return EOF
            if self.is_at_end() {
                return Some(Token::new(
                    TokenType::EOF,
                    self.line,
                    self.column,
                    "".to_string(),
                ));
            }
            
            // Otherwise if we reached a newline, handle it
            if !self.is_at_end() && self.peek_char() == '\n' {
                return self.handle_newline();
            }
        }
        
        // Check for ellipsis
        if current_char == '.' && self.peek_char_n(1) == '.' && self.peek_char_n(2) == '.' {
            return Some(self.handle_ellipsis());
        }
        
        // Handle operators and delimiters
        Some(self.handle_operator_or_delimiter())
    }
    
    /// Handles the ellipsis operator (...)
    fn handle_ellipsis(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        
        // Consume the three dots
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
    
    /// Handles newlines and indentation
    fn handle_newline(&mut self) -> Option<Token> {
        let start_col = self.column;
        
        self.consume_char(); // Consume the newline
        
        // Skip empty lines and just count them for line number tracking
        while !self.is_at_end() && self.peek_char() == '\n' {
            self.consume_char();
        }
        
        // Count indentation on the new line
        let indent_size = self.count_indentation();
        
        // Create newline token
        let newline_token = Token::new(
            TokenType::Newline,
            self.line - 1, // Line where the newline started
            start_col,
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
        
        while !self.is_at_end() {
            let c = self.peek_char();
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
            if !self.config.allow_tabs_in_indentation {
                self.add_error_with_suggestion(
                    "Mixed tabs and spaces in indentation",
                    "Use spaces only for indentation"
                );
            }
        }
        
        count
    }
    
    /// Handles identifiers and keywords
    fn handle_identifier(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        
        // Consume all alphanumeric and underscore characters
        self.consume_while(|c| c.is_alphanumeric() || c == '_');
        
        let text = self.get_slice(start_pos, self.position);
        
        // Check if it's a keyword
        let token_type = match text {
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
        };
        
        Token::new(token_type, self.line, start_col, text.to_string())
    }
    
    /// Handles numeric literals (integers, floats, and various bases)
    fn handle_number(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        
        // Check for different number bases (0b, 0o, 0x)
        if self.peek_char() == '0' && !self.is_at_end_n(1) {
            let next_char = self.peek_char_n(1);
            
            // Only consume the '0' and handle special bases
            if next_char == 'b' || next_char == 'B' {
                self.consume_char(); // Consume the '0'
                return self.handle_binary_literal(start_pos, start_col);
            } else if next_char == 'o' || next_char == 'O' {
                self.consume_char(); // Consume the '0'
                return self.handle_octal_literal(start_pos, start_col);
            } else if next_char == 'x' || next_char == 'X' {
                self.consume_char(); // Consume the '0'
                return self.handle_hex_literal(start_pos, start_col);
            }
            // For regular numbers starting with 0, continue normally
        }
        
        let mut is_float = false;
        
        // Parse the integer part
        self.consume_while(|c| c.is_digit(10) || c == '_');
        
        // Check for decimal point followed by a digit
        if !self.is_at_end() && self.peek_char() == '.' && 
           !self.is_at_end_n(1) && self.peek_char_n(1).is_digit(10) {
            is_float = true;
            self.consume_char(); // Consume the '.'
            
            // Parse the fractional part
            self.consume_while(|c| c.is_digit(10) || c == '_');
        }
        
        // Check for exponent (e or E)
        if !self.is_at_end() && 
           (self.peek_char() == 'e' || self.peek_char() == 'E') {
            is_float = true;
            self.consume_char(); // Consume the 'e' or 'E'
            
            // Optional sign
            if !self.is_at_end() && 
               (self.peek_char() == '+' || self.peek_char() == '-') {
                self.consume_char(); // Consume the sign
            }
            
            // Exponent digits
            let exp_start = self.position;
            self.consume_while(|c| c.is_digit(10) || c == '_');
            
            // Check if we have at least one digit in the exponent
            if self.position == exp_start {
                let text = self.get_slice(start_pos, self.position).to_string();
                return Token::error(
                    "Invalid exponent in float literal",
                    self.line,
                    start_col,
                    &text
                );
            }
        }
        
        // Get the text and remove any underscores (numeric separators)
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");
        
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
    
    /// Handles binary literals (0b...)
    fn handle_binary_literal(&mut self, start_pos: usize, start_col: usize) -> Token {
        self.consume_char(); // Consume the 'b' or 'B'
        
        if self.is_at_end() || !self.peek_char().is_digit(2) {
            let text = self.get_slice(start_pos, self.position).to_string();
            self.add_error("Invalid binary literal: missing binary digits");
            return Token::error(
                "Invalid binary literal",
                self.line,
                start_col,
                &text
            );
        }
        
        // Parse binary digits
        self.consume_while(|c| c == '0' || c == '1' || c == '_');
        
        // Get the text and remove any underscores
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");
        
        // Skip the '0b' prefix for parsing
        let value_text = &text[2..];
        
        match i64::from_str_radix(value_text, 2) {
            Ok(value) => Token::new(
                TokenType::BinaryLiteral(value),
                self.line,
                start_col,
                raw_text
            ),
            Err(_) => {
                let err_msg = format!("Invalid binary literal: {}", text);
                self.add_error(&err_msg);
                Token::error(
                    &err_msg,
                    self.line,
                    start_col,
                    &raw_text
                )
            }
        }
    }
    
    /// Handles octal literals (0o...)
    fn handle_octal_literal(&mut self, start_pos: usize, start_col: usize) -> Token {
        self.consume_char(); // Consume the 'o' or 'O'
        
        if self.is_at_end() || !self.peek_char().is_digit(8) {
            let text = self.get_slice(start_pos, self.position).to_string();
            self.add_error("Invalid octal literal: missing octal digits");
            return Token::error(
                "Invalid octal literal",
                self.line,
                start_col,
                &text
            );
        }
        
        // Parse octal digits
        self.consume_while(|c| c.is_digit(8) || c == '_');
        
        // Get the text and remove any underscores
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");
        
        // Skip the '0o' prefix for parsing
        let value_text = &text[2..];
        
        match i64::from_str_radix(value_text, 8) {
            Ok(value) => Token::new(
                TokenType::OctalLiteral(value),
                self.line,
                start_col,
                raw_text
            ),
            Err(_) => {
                let err_msg = format!("Invalid octal literal: {}", text);
                self.add_error(&err_msg);
                Token::error(
                    &err_msg,
                    self.line,
                    start_col,
                    &raw_text
                )
            }
        }
    }
    
    /// Handles hexadecimal literals (0x...)
    fn handle_hex_literal(&mut self, start_pos: usize, start_col: usize) -> Token {
        self.consume_char(); // Consume the 'x' or 'X'
        
        if self.is_at_end() || !self.peek_char().is_ascii_hexdigit() {
            let text = self.get_slice(start_pos, self.position).to_string();
            self.add_error("Invalid hex literal: missing hex digits");
            return Token::error(
                "Invalid hex literal",
                self.line,
                start_col,
                &text
            );
        }
        
        // Parse hex digits
        self.consume_while(|c| c.is_ascii_hexdigit() || c == '_');
        
        // Get the text and remove any underscores
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");
        
        // Skip the '0x' prefix for parsing
        let value_text = &text[2..];
        
        match i64::from_str_radix(value_text, 16) {
            Ok(value) => Token::new(
                TokenType::HexLiteral(value),
                self.line,
                start_col,
                raw_text
            ),
            Err(_) => {
                let err_msg = format!("Invalid hex literal: {}", text);
                self.add_error(&err_msg);
                Token::error(
                    &err_msg,
                    self.line,
                    start_col,
                    &raw_text
                )
            }
        }
    }
    
    /// Handles regular string literals
    fn handle_string(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let quote_char = self.peek_char();
        
        self.consume_char(); // Consume the opening quote
        
        let mut escaped = false;
        let mut string_content = String::new();
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
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
                    'x' => {
                        self.handle_hex_escape(&mut string_content);
                        '\0' // Already added to string_content
                    },
                    'u' => {
                        self.handle_unicode_escape(&mut string_content);
                        '\0' // Already added to string_content
                    },
                    '\n' => {
                        // Line continuation within a string
                        self.consume_char(); // Consume the newline
                        self.skip_whitespace(); // Skip leading whitespace on next line
                        '\0' // Don't add anything for line continuation
                    },
                    _ => {
                        self.add_error(&format!("Unknown escape sequence: \\{}", current_char));
                        current_char // Use the literal character
                    }
                };
                
                // Only add the character for simple escapes, the complex ones handle adding themselves
                if current_char != 'x' && current_char != 'u' && current_char != '\n' {
                    string_content.push(escaped_char);
                    self.consume_char(); // Consume the escape character
                }
                
                escaped = false;
            } else if current_char == '\\' {
                escaped = true;
                self.consume_char(); // Consume the backslash
            } else if current_char == quote_char {
                // End of string
                self.consume_char(); // Consume the closing quote
                break;
            } else if current_char == '\n' {
                // Unterminated string literal
                let text = self.get_slice(start_pos, self.position).to_string();
                self.add_error_with_suggestion(
                    "Unterminated string literal: newline in string",
                    "Add closing quote or use triple quotes for multi-line strings"
                );
                return Token::error(
                    "Unterminated string literal",
                    self.line,
                    start_col,
                    &text
                );
            } else {
                string_content.push(current_char);
                self.consume_char(); // Consume the character
            }
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if we have a proper closing quote
        if self.position > self.input.len() || 
           (self.position == self.input.len() && !text.ends_with(quote_char)) {
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
    
    /// Handles raw string literals (r"...")
    fn handle_raw_string(&mut self) -> Token {
        let start_pos = self.position - 1; // Include the 'r' prefix
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        self.consume_char(); // Consume the opening quote
        
        let mut string_content = String::new();
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if current_char == quote_char {
                // End of string
                self.consume_char(); // Consume the closing quote
                break;
            } else if current_char == '\n' {
                // Unterminated string literal
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
                self.consume_char(); // Consume the character
            }
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if we have a proper closing quote
        if self.position > self.input.len() || 
           (self.position == self.input.len() && !text.ends_with(quote_char)) {
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
    
    /// Handles f-string literals (f"...")
    fn handle_formatted_string(&mut self) -> Token {
        let start_pos = self.position - 1; // Include the 'f' prefix
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        self.consume_char(); // Consume the opening quote
        
        let mut string_content = String::new();
        let mut in_expression = false;
        let mut brace_depth = 0;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if !in_expression && current_char == '{' && self.peek_char_n(1) != '{' {
                // Start of expression
                in_expression = true;
                brace_depth = 1;
                string_content.push(current_char);
                self.consume_char();
            } else if in_expression && current_char == '{' {
                // Nested brace in expression
                brace_depth += 1;
                string_content.push(current_char);
                self.consume_char();
            } else if in_expression && current_char == '}' {
                // End of expression or nested brace
                brace_depth -= 1;
                string_content.push(current_char);
                self.consume_char();
                
                if brace_depth == 0 {
                    in_expression = false;
                }
            } else if !in_expression && current_char == '\\' {
                // Handle escaped characters in string parts
                self.consume_char(); // Consume backslash
                
                if self.is_at_end() {
                    self.add_error("Incomplete escape sequence in f-string");
                    break;
                }
                
                let escape_char = self.peek_char();
                string_content.push('\\');
                string_content.push(escape_char);
                self.consume_char();
            } else if !in_expression && current_char == quote_char {
                // End of string
                self.consume_char(); // Consume the closing quote
                break;
            } else if current_char == '\n' && !in_expression {
                // Unterminated string literal
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
                self.consume_char(); // Consume the character
            }
        }
        
        if in_expression {
            self.add_error("Unterminated expression in f-string: missing '}'");
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if we have a proper closing quote
        if self.position > self.input.len() || 
           (self.position == self.input.len() && !text.ends_with(quote_char)) {
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
    
    /// Handles bytes string literals (b"...")
    fn handle_bytes_string(&mut self) -> Token {
        let start_pos = self.position - 1; // Include the 'b' prefix
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        self.consume_char(); // Consume the opening quote
        
        let mut bytes = Vec::new();
        let mut escaped = false;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if escaped {
                // Handle escape sequences
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
                        self.consume_char(); // Consume 'x'
                        
                        // Read exactly 2 hex digits
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
                // End of bytes string
                self.consume_char(); // Consume the closing quote
                break;
            } else if current_char == '\n' {
                // Unterminated bytes literal
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
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if we have a proper closing quote
        if self.position > self.input.len() || 
           (self.position == self.input.len() && !text.ends_with(quote_char)) {
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
    
    /// Handles raw triple-quoted strings (r"""...""")
    fn handle_raw_triple_quoted_string(&mut self) -> Token {
        let start_pos = self.position - 1; // Include the 'r' prefix
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        // Consume the three opening quotes
        self.consume_char();
        self.consume_char();
        self.consume_char();
        
        let mut string_content = String::new();
        let mut consecutive_quotes = 0;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if current_char == quote_char {
                consecutive_quotes += 1;
                
                // Check if we've found three consecutive quotes
                if consecutive_quotes == 3 {
                    // We found the closing triple-quote
                    break;
                }
                
                self.consume_char();
            } else {
                // If we had some quotes but not three, add them to the content
                for _ in 0..consecutive_quotes {
                    string_content.push(quote_char);
                }
                consecutive_quotes = 0;
                string_content.push(current_char);
                self.consume_char();
            }
        }
        
        // Consume the closing triple quotes
        for _ in 0..consecutive_quotes {
            if !self.is_at_end() {
                self.consume_char();
            }
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if we found a proper closing triple-quote
        if consecutive_quotes < 3 {
            // Unterminated triple-quoted string
            self.add_error("Unterminated raw triple-quoted string");
            return Token::error(
                "Unterminated raw triple-quoted string",
                self.line,
                start_col,
                &text
            );
        }
        
        Token::new(TokenType::RawString(string_content), self.line, start_col, text)
    }
    
    /// Handles formatted triple-quoted strings (f"""...""")
    fn handle_formatted_triple_quoted_string(&mut self) -> Token {
        let start_pos = self.position - 1; // Include the 'f' prefix
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        // Consume the three opening quotes
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
                
                // Check if we've found three consecutive quotes
                if consecutive_quotes == 3 {
                    // We found the closing triple-quote
                    break;
                }
                
                self.consume_char();
            } else if !in_expression && current_char == '{' && 
                      (!self.is_at_end_n(1) && self.peek_char_n(1) != '{') {
                // Start of expression
                
                // If we had some quotes, add them to the content
                for _ in 0..consecutive_quotes {
                    string_content.push(quote_char);
                }
                consecutive_quotes = 0;
                
                in_expression = true;
                brace_depth = 1;
                string_content.push(current_char);
                self.consume_char();
            } else if in_expression && current_char == '{' {
                // Nested brace in expression
                brace_depth += 1;
                string_content.push(current_char);
                self.consume_char();
            } else if in_expression && current_char == '}' {
                // End of expression or nested brace
                brace_depth -= 1;
                string_content.push(current_char);
                self.consume_char();
                
                if brace_depth == 0 {
                    in_expression = false;
                }
            } else {
                // Regular character
                
                // If we had some quotes, add them to the content
                for _ in 0..consecutive_quotes {
                    string_content.push(quote_char);
                }
                consecutive_quotes = 0;
                
                string_content.push(current_char);
                self.consume_char();
            }
        }
        
        // Consume the closing triple quotes
        for _ in 0..consecutive_quotes {
            if !self.is_at_end() {
                self.consume_char();
            }
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        if in_expression {
            self.add_error("Unterminated expression in f-string: missing '}'");
        }
        
        // Check if we found a proper closing triple-quote
        if consecutive_quotes < 3 {
            // Unterminated triple-quoted string
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
    
    /// Handles bytes triple-quoted strings (b"""...""")
    fn handle_bytes_triple_quoted_string(&mut self) -> Token {
        let start_pos = self.position - 1; // Include the 'b' prefix
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        // Consume the three opening quotes
        self.consume_char();
        self.consume_char();
        self.consume_char();
        
        let mut bytes = Vec::new();
        let mut consecutive_quotes = 0;
        let mut escaped = false;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if escaped {
                // Handle escape sequences
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
                        self.consume_char(); // Consume 'x'
                        
                        // Read exactly 2 hex digits
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
                consecutive_quotes += 1;
                
                // Check if we've found three consecutive quotes
                if consecutive_quotes == 3 {
                    // We found the closing triple-quote
                    break;
                }
                
                self.consume_char();
            } else {
                // Regular character
                
                // If we had some quotes, add them to the content
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
        
        // Consume the closing triple quotes
        for _ in 0..consecutive_quotes {
            if !self.is_at_end() {
                self.consume_char();
            }
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if we found a proper closing triple-quote
        if consecutive_quotes < 3 {
            // Unterminated triple-quoted string
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
    
    /// Handles triple-quoted strings ("""...""")
    fn handle_triple_quoted_string(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let quote_char = self.peek_char();
        
        // Consume the three opening quotes
        self.consume_char();
        self.consume_char();
        self.consume_char();
        
        let mut string_content = String::new();
        let mut consecutive_quotes = 0;
        let mut escaped = false;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if escaped {
                // Handle escape sequences
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
                        // Line continuation within a string
                        self.consume_char(); // Consume the newline
                        self.skip_whitespace(); // Skip leading whitespace on next line
                    },
                    _ => {
                        self.add_error(&format!("Unknown escape sequence: \\{}", current_char));
                        string_content.push(current_char);
                    }
                }
                
                escaped = false;
                self.consume_char();
            } else if current_char == '\\' {
                escaped = true;
                self.consume_char();
            } else if current_char == quote_char {
                consecutive_quotes += 1;
                
                // Check if we've found three consecutive quotes
                if consecutive_quotes == 3 {
                    // Consume the third quote to complete the ending triple-quote
                    self.consume_char();
                    break;
                }
                
                self.consume_char();
            } else {
                // If we had some quotes but not three, add them to the content
                for _ in 0..consecutive_quotes {
                    string_content.push(quote_char);
                }
                consecutive_quotes = 0;
                string_content.push(current_char);
                self.consume_char();
            }
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if we found a proper closing triple-quote
        if consecutive_quotes < 3 {
            // Unterminated triple-quoted string
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
    
    /// Handles \x escape sequences in strings (hex values)
    fn handle_hex_escape(&mut self, string_content: &mut String) -> char {
        self.consume_char(); // Consume the 'x'
        
        let mut hex_value = String::with_capacity(2);
        
        // Read exactly 2 hex digits
        for _ in 0..2 {
            if !self.is_at_end() && 
                self.peek_char().is_ascii_hexdigit() {
                hex_value.push(self.peek_char());
                self.consume_char();
            } else {
                self.add_error("Invalid hex escape sequence: expected 2 hex digits");
                return '?'; // Error placeholder
            }
        }
        
        // Convert hex to char
        if let Ok(byte) = u8::from_str_radix(&hex_value, 16) {
            string_content.push(byte as char);
        } else {
            let err_msg = format!("Invalid hex escape sequence: \\x{}", hex_value);
            self.add_error(&err_msg);
        }
        
        // Return null character since we've already added the hex character to string_content
        '\0'
    }
    
    /// Handles \u escape sequences in strings (unicode values)
    fn handle_unicode_escape(&mut self, string_content: &mut String) -> char {
        self.consume_char(); // Consume the 'u'
        
        // Check for opening brace
        let has_braces = !self.is_at_end() && self.peek_char() == '{';
        if has_braces {
            self.consume_char();
        }
        
        // For non-braced format (e.g., \u00A9), read exactly 4 hex digits
        if !has_braces {
            let mut hex_value = String::with_capacity(4);
            
            // Read exactly 4 hex digits
            for _ in 0..4 {
                if !self.is_at_end() && 
                   self.peek_char().is_ascii_hexdigit() {
                    hex_value.push(self.peek_char());
                    self.consume_char();
                } else {
                    self.add_error("Invalid Unicode escape sequence: expected 4 hex digits");
                    return '?'; // Error placeholder
                }
            }
            
            // Convert to Unicode character
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
        // For braced format (e.g., \u{1F600}), read 1-6 hex digits
        else {
            let mut hex_value = String::new();
            
            // Read 1-6 hex digits for Unicode code point
            while !self.is_at_end() && 
                  self.peek_char().is_ascii_hexdigit() && 
                  hex_value.len() < 6 {
                hex_value.push(self.peek_char());
                self.consume_char();
            }
            
            // Check closing brace
            if !self.is_at_end() && self.peek_char() == '}' {
                self.consume_char();
            } else {
                self.add_error("Unclosed Unicode escape sequence: missing closing brace");
                return '?';
            }
            
            // Convert to Unicode character
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
        
        // Return null character since we've already added the Unicode character to string_content
        '\0'
    }
    
    /// Handles operators and delimiters
    fn handle_operator_or_delimiter(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let current_char = self.peek_char();
        
        self.consume_char();
        
        // Check for multi-character operators
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
                    TokenType::MatrixMul
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
            
            // Delimiters
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
    
    /// Gets the next character without consuming it
    fn peek_char(&self) -> char {
        if !self.lookahead_buffer.is_empty() {
            return self.lookahead_buffer[0];
        }
        
        self.chars.clone().next().unwrap_or('\0')
    }
    
    /// Gets a character n positions ahead without consuming anything
    fn peek_char_n(&self, n: usize) -> char {
        if n < self.lookahead_buffer.len() {
            return self.lookahead_buffer[n];
        }
        
        let mut chars_iter = self.chars.clone();
        for _ in 0..n {
            if chars_iter.next().is_none() {
                return '\0';
            }
        }
        
        chars_iter.next().unwrap_or('\0')
    }
    
    /// Checks if we're at the end of the input
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
    
    /// Checks if we're at the end of the input plus n positions
    fn is_at_end_n(&self, n: usize) -> bool {
        self.position + n >= self.input.len()
    }
    
    /// Consumes the current character and advances the position
    fn consume_char(&mut self) {
        if !self.is_at_end() {
            let current_char = if !self.lookahead_buffer.is_empty() {
                self.lookahead_buffer.remove(0)
            } else {
                self.chars.next().unwrap_or('\0')
            };
            
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
        while !self.is_at_end() && predicate(self.peek_char()) {
            self.consume_char();
        }
    }
    
    /// Gets a slice of the input string
    fn get_slice(&self, start: usize, end: usize) -> &str {
        &self.input[start..end]
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

// Unit tests for the lexer
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
        let mut lexer = Lexer::new("123 3.14 0.5 1e10 1.5e-3 1_000_000 0b101 0o755 0xABCD");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::IntLiteral(123));
        assert_eq!(tokens[1].token_type, TokenType::FloatLiteral(3.14));
        assert_eq!(tokens[2].token_type, TokenType::FloatLiteral(0.5));
        assert_eq!(tokens[3].token_type, TokenType::FloatLiteral(1e10));
        assert_eq!(tokens[4].token_type, TokenType::FloatLiteral(1.5e-3));
        assert_eq!(tokens[5].token_type, TokenType::IntLiteral(1000000));
        assert_eq!(tokens[6].token_type, TokenType::BinaryLiteral(5));
        assert_eq!(tokens[7].token_type, TokenType::OctalLiteral(493));
        assert_eq!(tokens[8].token_type, TokenType::HexLiteral(43981));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("\"hello\" 'world' \"escape\\nsequence\" r\"raw\\string\" f\"format {value}\"");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral("hello".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::StringLiteral("world".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::StringLiteral("escape\nsequence".to_string()));
        
        if let TokenType::RawString(s) = &tokens[3].token_type {
            assert_eq!(s, "raw\\string");
        } else {
            panic!("Expected RawString token");
        }
        
        if let TokenType::FString(s) = &tokens[4].token_type {
            assert_eq!(s, "format {value}");
        } else {
            panic!("Expected FString token");
        }
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
        let mut lexer = Lexer::new("+ - * / // % ** @ = += -= *= /= %= **= @= //= &= |= ^= <<= >>= == != < <= > >= & | ^ ~ << >> -> := ...");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::Plus);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Multiply);
        assert_eq!(tokens[3].token_type, TokenType::Divide);
        assert_eq!(tokens[4].token_type, TokenType::FloorDivide);
        assert_eq!(tokens[5].token_type, TokenType::Modulo);
        assert_eq!(tokens[6].token_type, TokenType::Power);
        assert_eq!(tokens[7].token_type, TokenType::MatrixMul);
        assert_eq!(tokens[8].token_type, TokenType::Assign);
        assert_eq!(tokens[9].token_type, TokenType::PlusAssign);
        assert_eq!(tokens[10].token_type, TokenType::MinusAssign);
        assert_eq!(tokens[11].token_type, TokenType::MulAssign);
        assert_eq!(tokens[12].token_type, TokenType::DivAssign);
        assert_eq!(tokens[13].token_type, TokenType::ModAssign);
        assert_eq!(tokens[14].token_type, TokenType::PowAssign);
        assert_eq!(tokens[15].token_type, TokenType::MatrixMulAssign);
        assert_eq!(tokens[16].token_type, TokenType::FloorDivAssign);
        assert_eq!(tokens[17].token_type, TokenType::BitwiseAndAssign);
        assert_eq!(tokens[18].token_type, TokenType::BitwiseOrAssign);
        assert_eq!(tokens[19].token_type, TokenType::BitwiseXorAssign);
        assert_eq!(tokens[20].token_type, TokenType::ShiftLeftAssign);
        assert_eq!(tokens[21].token_type, TokenType::ShiftRightAssign);
        assert_eq!(tokens[22].token_type, TokenType::Equal);
        assert_eq!(tokens[23].token_type, TokenType::NotEqual);
        assert_eq!(tokens[24].token_type, TokenType::LessThan);
        assert_eq!(tokens[25].token_type, TokenType::LessEqual);
        assert_eq!(tokens[26].token_type, TokenType::GreaterThan);
        assert_eq!(tokens[27].token_type, TokenType::GreaterEqual);
        assert_eq!(tokens[28].token_type, TokenType::BitwiseAnd);
        assert_eq!(tokens[29].token_type, TokenType::BitwiseOr);
        assert_eq!(tokens[30].token_type, TokenType::BitwiseXor);
        assert_eq!(tokens[31].token_type, TokenType::BitwiseNot);
        assert_eq!(tokens[32].token_type, TokenType::ShiftLeft);
        assert_eq!(tokens[33].token_type, TokenType::ShiftRight);
        assert_eq!(tokens[34].token_type, TokenType::Arrow);
        assert_eq!(tokens[35].token_type, TokenType::Walrus);
        assert_eq!(tokens[36].token_type, TokenType::Ellipsis);
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
        
        // Look for specific patterns
        
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
    fn test_hex_escape() {
        let mut lexer = Lexer::new("\"\\x41\\x42\"");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral("AB".to_string()));
    }
    
    #[test]
    fn test_unicode_escape() {
        let mut lexer = Lexer::new("\"\\u{1F600}\" \"\\u00A9\"");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral("😀".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::StringLiteral("©".to_string()));
    }
    
    #[test]
    fn test_f_strings() {
        let mut lexer = Lexer::new("f\"Hello, {name}!\" f\"{a + b}\"");
        let tokens = lexer.tokenize();
        
        if let TokenType::FString(s) = &tokens[0].token_type {
            assert_eq!(s, "Hello, {name}!");
        } else {
            panic!("Expected FString token");
        }
        
        if let TokenType::FString(s) = &tokens[1].token_type {
            assert_eq!(s, "{a + b}");
        } else {
            panic!("Expected FString token");
        }
    }
    
    #[test]
    fn test_bytes_literals() {
        let mut lexer = Lexer::new("b\"hello\" b'\\x00\\x01\\x02'");
        let tokens = lexer.tokenize();
        
        if let TokenType::BytesLiteral(bytes) = &tokens[0].token_type {
            assert_eq!(bytes, b"hello");
        } else {
            panic!("Expected BytesLiteral token");
        }
        
        if let TokenType::BytesLiteral(bytes) = &tokens[1].token_type {
            assert_eq!(bytes, &[0, 1, 2]);
        } else {
            panic!("Expected BytesLiteral token");
        }
    }
    
    #[test]
    fn test_numeric_separators() {
        let mut lexer = Lexer::new("1_000_000 0b1010_1100 0xAB_CD");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::IntLiteral(1000000));
        assert_eq!(tokens[1].token_type, TokenType::BinaryLiteral(0b10101100));
        assert_eq!(tokens[2].token_type, TokenType::HexLiteral(0xABCD));
    }
    
    #[test]
    fn test_walrus_operator() {
        let mut lexer = Lexer::new("if (n := len(items)) > 0:");
        let tokens = lexer.tokenize();
        
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Walrus)));
    }
    
    #[test]
    fn test_ellipsis() {
        let mut lexer = Lexer::new("def func(...):");
        let tokens = lexer.tokenize();
        
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Ellipsis)));
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
print(f"Factorial of 5 is {result}")
"#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Just check that we have a reasonable number of tokens
        assert!(tokens.len() > 20);
        
        // Check a few key tokens
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Def)));
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Identifier(ref s) if s == "factorial")));
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Return)));
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::FString(_))));
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Identifier(ref s) if s == "print")));
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
        assert!(errors.iter().any(|e| e.message.contains("indentation")));
    }
    
    #[test]
    fn test_mixed_tabs_spaces() {
        let config = LexerConfig {
            enforce_indent_consistency: true,
            allow_tabs_in_indentation: false,
            ..Default::default()
        };
        
        let mut lexer = Lexer::with_config("if True:\n\t  mixed_indent", config);
        let tokens = lexer.tokenize();
        
        // Should be an error in the errors list
        let errors = lexer.get_errors();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("tabs and spaces")));
    }
    
    #[test]
    fn test_line_continuation() {
        let mut lexer = Lexer::new("x = 1 + \\\n    2");
        let tokens = lexer.tokenize();
        
        // The tokens should be: Identifier(x), Assign, IntLiteral(1), Plus, IntLiteral(2), EOF
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::Assign);
        assert_eq!(tokens[2].token_type, TokenType::IntLiteral(1));
        assert_eq!(tokens[3].token_type, TokenType::Plus);
        assert_eq!(tokens[4].token_type, TokenType::IntLiteral(2));
        assert_eq!(tokens[5].token_type, TokenType::EOF);
    }
    
    #[test]
    fn test_implicit_line_continuation() {
        let mut lexer = Lexer::new("func(\n    arg1,\n    arg2\n)");
        let tokens = lexer.tokenize();
        
        // There should be no Newline tokens between the opening and closing parentheses
        let mut paren_level = 0;
        let mut contains_newline_in_parens = false;
        
        for token in &tokens {
            match token.token_type {
                TokenType::LeftParen => paren_level += 1,
                TokenType::RightParen => paren_level -= 1,
                TokenType::Newline => {
                    if paren_level > 0 {
                        contains_newline_in_parens = true;
                    }
                },
                _ => {}
            }
        }
        
        assert!(!contains_newline_in_parens, "Newlines should be ignored inside parentheses");
    }
}