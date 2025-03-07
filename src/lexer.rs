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
    #[allow(dead_code)]
    MatrixMul,    // @
    BackSlash,    // \ (for line continuations and other uses)
    
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
    #[allow(dead_code)]
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
        let mut pending_indentation_change = true; // Start with true to handle beginning indentation
        
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
            
            // If pending indentation changes, handle them before adding this token
            if pending_indentation_change && 
               self.paren_level == 0 && self.bracket_level == 0 && self.brace_level == 0 {
                self.handle_indentation_change(&mut tokens, token_line);
                pending_indentation_change = false;
            }
            
            // Push the token to our collection
            tokens.push(token);
            
            // If we just saw a newline and we're not in a nested structure, check indentation on next token
            if matches!(token_type, TokenType::Newline) && 
               self.paren_level == 0 && self.bracket_level == 0 && self.brace_level == 0 {
                pending_indentation_change = true;
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

    fn has_error_for_line(&self, line: usize, message: &str) -> bool {
        self.errors.iter().any(|e| e.line == line && e.message == message)
    }
    
    /// Handles indentation changes after a newline
    fn handle_indentation_change(&mut self, tokens: &mut Vec<Token>, token_line: usize) {
        if self.current_indent > *self.indent_stack.last().unwrap_or(&0) {
            if self.config.enforce_indent_consistency && 
                !self.config.allow_tabs_in_indentation && 
                self.current_indent % self.config.standard_indent_size != 0 {
                let error_message = format!(
                    "Inconsistent indentation. Expected multiple of {} spaces but got {}.",
                    self.config.standard_indent_size, self.current_indent
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
                " ".repeat(self.current_indent),
            );
            self.indent_stack.push(self.current_indent);
            tokens.push(indent_token);
        } else if self.current_indent < *self.indent_stack.last().unwrap_or(&0) {
            let valid_indent_level = self.indent_stack.iter().any(|&i| i == self.current_indent);
            if !valid_indent_level {
                let msg = format!(
                    "Inconsistent indentation. Current indent level {} doesn't match any previous level.",
                    self.current_indent
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
            while self.indent_stack.len() > 1 && self.current_indent < *self.indent_stack.last().unwrap() {
                self.indent_stack.pop();
                tokens.push(Token::new(TokenType::Dedent, token_line, 1, "".to_string()));
            }
        }
    }       
    
    /// Gets the next token from the input
    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        
        if self.is_at_end() {
            return Some(Token::new(TokenType::EOF, self.line, self.column, "".to_string()));
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
        if current_char == '\\' && 
           (self.peek_char_n(1) == '\n' || 
            (self.peek_char_n(1) == '\r' && self.peek_char_n(2) == '\n')) {
            
            self.consume_char(); // Consume backslash
            
            // Handle Windows CRLF or Unix LF line endings
            if self.peek_char() == '\r' {
                self.consume_char(); // Consume \r
            }
            if self.peek_char() == '\n' {
                self.consume_char(); // Consume \n
            }
            
            // Skip whitespace at the start of the next line without
            // checking for indentation consistency
            while !self.is_at_end() && 
                  (self.peek_char() == ' ' || self.peek_char() == '\t') {
                self.consume_char();
            }
            
            // Continue to the next token
            return self.next_token();
        }
        
        // Check for prefixed triple-quoted strings (e.g., r'''...''', f"""...""")
        if (current_char == 'r' || current_char == 'R' || 
            current_char == 'f' || current_char == 'F' || 
            current_char == 'b' || current_char == 'B') && 
            ((self.peek_char_n(1) == '"' && self.peek_char_n(2) == '"' && self.peek_char_n(3) == '"') ||
             (self.peek_char_n(1) == '\'' && self.peek_char_n(2) == '\'' && self.peek_char_n(3) == '\'')) {
            let prefix = current_char;
            self.consume_char(); // Consume the prefix
            match prefix {
                'r' | 'R' => return Some(self.handle_raw_triple_quoted_string()),
                'f' | 'F' => return Some(self.handle_formatted_triple_quoted_string()),
                'b' | 'B' => return Some(self.handle_bytes_triple_quoted_string()),
                _ => unreachable!()
            }
        }
        
        // Check for regular triple-quoted strings (e.g., '''...''', """...""")
        if (current_char == '"' && self.peek_char_n(1) == '"' && self.peek_char_n(2) == '"') ||
           (current_char == '\'' && self.peek_char_n(1) == '\'' && self.peek_char_n(2) == '\'') {
            return Some(self.handle_triple_quoted_string());
        }
        
        // Check for prefixed single-quoted strings (e.g., r"...", f'...')
        if (current_char == 'r' || current_char == 'R' || 
            current_char == 'f' || current_char == 'F' || 
            current_char == 'b' || current_char == 'B') && 
            (self.peek_char_n(1) == '"' || self.peek_char_n(1) == '\'') {
            let prefix = current_char;
            self.consume_char(); // Consume the prefix
            match prefix {
                'r' | 'R' => return Some(self.handle_raw_string()),
                'f' | 'F' => return Some(self.handle_formatted_string()),
                'b' | 'B' => return Some(self.handle_bytes_string()),
                _ => unreachable!()
            }
        }
        
        // Check for regular single-quoted strings (e.g., "...", '...')
        if current_char == '"' || current_char == '\'' {
            return Some(self.handle_string());
        }
        
        // Check for identifiers and keywords
        if current_char.is_alphabetic() || current_char == '_' {
            return Some(self.handle_identifier());
        }
        
        // Check for numeric literals
        if current_char.is_digit(10) || (current_char == '.' && self.peek_char_n(1).is_digit(10)) {
            return Some(self.handle_number());
        }
        
        // Check for comments
        if current_char == '#' {
            // Skip the comment
            while !self.is_at_end() && self.peek_char() != '\n' {
                self.consume_char();
            }
            
            // If we're at the end of the file after a comment, return EOF
            if self.is_at_end() {
                return Some(Token::new(TokenType::EOF, self.line, self.column, "".to_string()));
            }
            
            // Otherwise, if we reached a newline, handle it
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
        let _start_pos = self.position;
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
        let start_line = self.line;
        
        self.consume_char(); // Consume the newline
        
        // Track if we're processing an empty line
        let mut is_empty_line = false;
        
        // Skip empty lines and just count them for line number tracking
        while !self.is_at_end() && self.peek_char() == '\n' {
            is_empty_line = true;
            self.consume_char();
        }
        
        // Count indentation on the new line
        let indent_size = self.count_indentation();
        
        // Create newline token
        let newline_token = Token::new(
            TokenType::Newline,
            start_line, // Line where the newline started
            start_col,
            "\n".to_string(),
        );
        
        // Update the current indentation level, but preserve existing indentation for empty lines
        // This ensures empty lines don't mess up indentation tracking
        if !is_empty_line {
            self.current_indent = indent_size;
        }
        
        // Return the newline token, indentation tokens will be handled separately
        Some(newline_token)
    }
    
    /// Counts the indentation level at the current position
    fn count_indentation(&mut self) -> usize {
        let mut count = 0;
        let mut has_tabs = false;
        let mut has_spaces = false;
        
        // Store the current line for error reporting
        let indentation_line = self.line;
        
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
        
        // Only report tab/space mixing if both are present AND tabs are not allowed
        if has_tabs && has_spaces && !self.config.allow_tabs_in_indentation {
            let msg = "Mixed tabs and spaces in indentation";
            if !self.has_error_for_line(indentation_line, msg) {
                self.add_error_with_position(
                    msg,
                    "Use spaces only for indentation",
                    indentation_line,
                    1
                );
            }
        }
        
        // Only report inconsistent indentation if:
        // 1. Config requires consistency
        // 2. AND the indentation isn't a multiple of standard_indent_size
        // 3. AND EITHER:
        //    a. We're not using tabs at all, OR
        //    b. We're using tabs but they're not allowed (i.e., allow_tabs_in_indentation is false)
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
    
    /// Handles identifiers and keywords
    fn handle_identifier(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        
        // Consume all alphanumeric and underscore characters
        self.consume_while(|c| c.is_alphanumeric() || c == '_');
        
        let text = self.get_slice(start_pos, self.position);
        
        // Check if it's a keyword using the keywords HashSet
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
        let mut decimal_count = 0;
        
        // Parse the integer part
        self.consume_while(|c| c.is_digit(10) || c == '_');
        
        // Check for decimal point followed by a digit
        if !self.is_at_end() && self.peek_char() == '.' {
            decimal_count += 1;
            is_float = true;
            self.consume_char(); // Consume the '.'
            
            // Parse the fractional part
            self.consume_while(|c| c.is_digit(10) || c == '_');
            
            // Check for another decimal point (like in 1.2.3)
            if !self.is_at_end() && self.peek_char() == '.' {
                // This is a multiple decimal point error
                // Consume all remaining characters that could be part of the number
                self.consume_char(); // Consume the second '.'
                self.consume_while(|c| c.is_digit(10) || c == '_' || c == '.');
                
                let raw_text = self.get_slice(start_pos, self.position).to_string();
                self.add_error("Invalid number format: multiple decimal points");
                return Token::error(
                    "Invalid number format: multiple decimal points",
                    self.line,
                    start_col,
                    &raw_text
                );
            }
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
                self.add_error("Invalid exponent in float literal");
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
        
        // Multiple decimal points would be caught here
        if decimal_count > 1 || text.matches('.').count() > 1 {
            self.add_error("Invalid number format: multiple decimal points");
            return Token::error(
                "Invalid number format: multiple decimal points",
                self.line,
                start_col,
                &raw_text
            );
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
    
    /// Handles binary literals (0b...)
    fn handle_binary_literal(&mut self, start_pos: usize, start_col: usize) -> Token {
        self.consume_char(); // Consume 'b' or 'B'
        self.consume_while(|c| c.is_digit(10) || c == '_'); // Consume all digits and '_'
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");
        let value_text = &text[2..]; // Skip "0b"
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
    
    /// Handles octal literals (0o...)
    fn handle_octal_literal(&mut self, start_pos: usize, start_col: usize) -> Token {
        self.consume_char(); // Consume 'o' or 'O'
        self.consume_while(|c| c.is_digit(10) || c == '_'); // Consume all digits and '_'
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");
        let value_text = &text[2..]; // Skip "0o"
        if value_text.is_empty() || value_text.chars().any(|c| !('0'..'8').contains(&c)) {
            let err_msg = format!("Invalid octal literal: {}", text);
            self.add_error(&err_msg);
            return Token::error(&err_msg, self.line, start_col, &raw_text);
        }
        match i64::from_str_radix(value_text, 8) {
            Ok(value) => Token::new(TokenType::OctalLiteral(value), self.line, start_col, raw_text),
            Err(_) => {
                let err_msg = format!("Invalid octal literal: {}", text);
                self.add_error(&err_msg);
                Token::error(&err_msg, self.line, start_col, &raw_text)
            }
        }
    }
    
    /// Handles hexadecimal literals (0x...)
    fn handle_hex_literal(&mut self, start_pos: usize, start_col: usize) -> Token {
        self.consume_char(); // Consume 'x' or 'X'
        self.consume_while(|c| c.is_alphanumeric() || c == '_'); // Consume all alphanumerics and '_'
        let raw_text = self.get_slice(start_pos, self.position).to_string();
        let text = raw_text.replace("_", "");
        let value_text = &text[2..]; // Skip "0x"
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
                    'U' => {
                        self.handle_extended_unicode_escape(&mut string_content);
                        '\0' // Already added to string_content
                    },
                    '\n' => {
                        // Line continuation within a string
                        self.consume_char(); // Consume the newline
                        self.skip_whitespace(); // Skip leading whitespace on next line
                        '\0' // Don't add anything for line continuation
                    },
                    '\r' => {
                        // Handle Windows CRLF line continuation
                        self.consume_char(); // Consume the \r
                        if !self.is_at_end() && self.peek_char() == '\n' {
                            self.consume_char(); // Consume the \n
                        }
                        self.skip_whitespace(); // Skip leading whitespace on next line
                        '\0' // Don't add anything for line continuation
                    },
                    _ => {
                        self.add_error(&format!("Unknown escape sequence: \\{}", current_char));
                        current_char // Use the literal character
                    }
                };
                
                // Only add the character for simple escapes, the complex ones handle adding themselves
                if current_char != 'x' && current_char != 'u' && current_char != 'U' && 
                current_char != '\n' && current_char != '\r' {
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
            } else if current_char == '\n' || current_char == '\r' {
                // Unterminated string literal
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
                self.consume_char(); // Consume the character
            }
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if we have a proper closing quote
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

    /// Handles \U escape sequences in strings (extended unicode values)
    fn handle_extended_unicode_escape(&mut self, string_content: &mut String) -> char {
        self.consume_char(); // Consume the 'U'
        
        let mut hex_value = String::with_capacity(8);
        
        // Read exactly 8 hex digits
        for _ in 0..8 {
            if !self.is_at_end() && self.peek_char().is_ascii_hexdigit() {
                hex_value.push(self.peek_char());
                self.consume_char();
            } else {
                self.add_error("Invalid extended Unicode escape sequence: expected 8 hex digits");
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
            let err_msg = format!("Invalid Unicode escape sequence: \\U{}", hex_value);
            self.add_error(&err_msg);
        }
        
        // Return null character since we've already added the Unicode character to string_content
        '\0'
    }
    
    /// Handles raw string literals (r"...")
    fn handle_raw_string(&mut self) -> Token {
        let start_pos = self.position - 1; // Include the 'r' prefix
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        
        self.consume_char(); // Consume the opening quote
        
        let mut string_content = String::new();
        let mut is_escaped = false;
        
        while !self.is_at_end() {
            let current_char = self.peek_char();
            
            if is_escaped {
                // In raw strings, a backslash followed by any character is not an escape sequence
                // Both characters are added literally
                string_content.push('\\');
                string_content.push(current_char);
                self.consume_char(); // Consume the character after the backslash
                is_escaped = false;
            } else if current_char == '\\' {
                // Mark that we've seen a backslash, but don't add it to string_content yet
                // We'll add it in the next iteration
                is_escaped = true;
                self.consume_char(); // Consume the backslash
            } else if current_char == quote_char {
                // End of string - found the closing quote
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
                // Add the character to the string content
                string_content.push(current_char);
                self.consume_char(); // Consume the character
            }
        }
        
        // If we have an unprocessed backslash at the end, add it to the string
        if is_escaped {
            string_content.push('\\');
        }
        
        // Get the text and immediately clone it to avoid borrow issues
        let text = self.get_slice(start_pos, self.position).to_string();
        
        // Check if we have a proper closing quote
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
    
    /// Handles raw triple-quoted strings (r"""...""")
    fn handle_raw_triple_quoted_string(&mut self) -> Token {
        let start_pos = self.position - 1; // Include the 'r' prefix
        let start_col = self.column - 1;
        let quote_char = self.peek_char();
        println!("[DEBUG] Starting raw triple-quoted string with quote_char: '{}'", quote_char);
        self.consume_char(); // Consume first quote
        self.consume_char(); // Consume second quote
        self.consume_char(); // Consume third quote
        let mut string_content = String::new();
        let mut consecutive_quotes = 0;
        while !self.is_at_end() {
            let current_char = self.peek_char();
            println!("[DEBUG] Current char: '{}', consecutive_quotes: {}", current_char, consecutive_quotes);
            if current_char == quote_char {
                consecutive_quotes += 1;
                self.consume_char(); // Consume the quote
                if consecutive_quotes == 3 {
                    println!("[DEBUG] Found closing triple quotes");
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
        println!("[DEBUG] Raw string lexeme: '{}', content: '{}'", text, string_content);
        if consecutive_quotes < 3 {
            self.add_error("Unterminated raw triple-quoted string");
            return Token::error("Unterminated raw triple-quoted string", self.line, start_col, &text);
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
                self.consume_char(); // Consume the quote
                
                // Check if we've found three consecutive quotes
                if consecutive_quotes == 3 {
                    break; // We've already consumed all three closing quotes
                }
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
                // For any existing consecutive quotes, add them as bytes
                for _ in 0..consecutive_quotes {
                    bytes.push(quote_char as u8);
                }
                consecutive_quotes = 0;
                
                escaped = true;
                self.consume_char();
            } else if current_char == quote_char {
                consecutive_quotes += 1;
                self.consume_char(); // Consume the quote
                
                // Check if we've found three consecutive quotes
                if consecutive_quotes == 3 {
                    break; // We've already consumed all three closing quotes
                }
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
                // If we had some quotes but not three, add them to the content
                for _ in 0..consecutive_quotes {
                    string_content.push(quote_char);
                }
                consecutive_quotes = 0;
                
                escaped = true;
                self.consume_char();
            } else if current_char == quote_char {
                consecutive_quotes += 1;
                self.consume_char();
                
                // Check if we've found three consecutive quotes
                if consecutive_quotes == 3 {
                    // We've already consumed all three quotes
                    break;
                }
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
                    TokenType::At // For decorators
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
            
            // Handle backslash as a legitimate token 
            '\\' => {
                // Line continuation will be handled in next_token
                // Here we're just returning the token
                TokenType::BackSlash
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
            
            // Handle Windows CRLF as a single newline
            if current_char == '\r' && !self.is_at_end_n(1) && self.peek_char_n(1) == '\n' {
                // Consume the \n part of CRLF
                if !self.lookahead_buffer.is_empty() {
                    self.lookahead_buffer.remove(0);
                } else {
                    self.chars.next();
                }
                self.position += 1;
                
                // Handle as a single newline
                self.line += 1;
                self.column = 1;
            } else if current_char == '\n' {
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
    #[allow(dead_code)]
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
    
    // Helper function to simplify token comparison
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
    
    // Test empty input
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
    fn test_complex_indentation() {
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
        let input = "x = 5  # This is a comment\n# This is another comment\ny = 10";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract the token types for easier comparison
        let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
        
        // Expected sequence of token types (comments are skipped)
        let expected = vec![
            TokenType::Identifier("x".to_string()),
            TokenType::Assign,
            TokenType::IntLiteral(5),
            TokenType::Newline,
            TokenType::Identifier("y".to_string()),
            TokenType::Assign,
            TokenType::IntLiteral(10),
            TokenType::EOF,
        ];
        
        assert_eq!(token_types, expected, "Comment handling is incorrect");
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
        let tokens = lexer.tokenize();
        
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
        let tokens = lexer.tokenize();
        
        // We should still get tokens, but there should be errors about mixed indentation
        assert!(lexer.get_errors().len() > 0, "Should report mixed indentation errors");
        
        // Find error about mixed tabs and spaces
        let has_mixed_error = lexer.get_errors().iter().any(|e| 
            e.message.contains("Mixed tabs and spaces"));
        assert!(has_mixed_error, "Should report mixed tabs and spaces error");
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
        let tokens = lexer.tokenize();
        
        // We should still get a string token, but there should be errors
        assert!(lexer.get_errors().len() > 0, "Should report escape sequence errors");
        
        let has_escape_error = lexer.get_errors().iter().any(|e| 
            e.message.contains("escape sequence"));
        assert!(has_escape_error, "Should report an escape sequence error");
    }
    
    // Test for newline handling
    #[test]
    fn test_newline_styles() {
        // Test Unix style (LF)
        let input_lf = "x = 1\ny = 2";
        let mut lexer_lf = Lexer::new(input_lf);
        let tokens_lf = lexer_lf.tokenize();
        
        // Test Windows style (CRLF)
        let input_crlf = "x = 1\r\ny = 2";
        let mut lexer_crlf = Lexer::new(input_crlf);
        let tokens_crlf = lexer_crlf.tokenize();
        
        // Both should produce the same tokens
        assert_eq!(tokens_lf.len(), tokens_crlf.len(), "Different newline styles should produce same token count");
        
        for i in 0..tokens_lf.len() {
            assert_eq!(tokens_lf[i].token_type, tokens_crlf[i].token_type, 
                      "Different newline styles should produce same tokens");
        }
    }
    
    // Test line and column numbers
    #[test]
    fn test_position_tracking() {
        let input = "x = 1\ny = 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Check positions
        assert_eq!(tokens[0].line, 1, "First token should be on line 1"); // x
        assert_eq!(tokens[0].column, 1, "First token should be at column 1");
        
        assert_eq!(tokens[3].line, 2, "Token after newline should be on line 2"); // y
        assert_eq!(tokens[3].column, 1, "First token on new line should be at column 1");
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
        let tokens1 = lexer1.tokenize();
        assert!(lexer1.get_errors().len() > 0, "Default config should report tab errors");
        
        // Custom config allows tabs
        let mut lexer2 = Lexer::with_config(input, LexerConfig {
            allow_tabs_in_indentation: true,
            tab_width: 4,
            ..Default::default()
        });
        let tokens2 = lexer2.tokenize();
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
}