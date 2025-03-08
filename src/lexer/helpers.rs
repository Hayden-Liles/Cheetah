use super::Lexer;

impl<'a> Lexer<'a> {
    pub fn peek_char(&self) -> char {
        if !self.lookahead_buffer.is_empty() {
            self.lookahead_buffer[0]
        } else {
            self.chars.clone().next().unwrap_or('\0')
        }
    }

    pub fn peek_char_n(&self, n: usize) -> char {
        if n < self.lookahead_buffer.len() {
            self.lookahead_buffer[n]
        } else {
            let mut chars_iter = self.chars.clone();
            for _ in 0..n {
                if chars_iter.next().is_none() { return '\0'; }
            }
            chars_iter.next().unwrap_or('\0')
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    pub fn is_at_end_n(&self, n: usize) -> bool {
        self.position + n >= self.input.len()
    }

    pub fn consume_char(&mut self) {
        if !self.is_at_end() {
            let current_char = if !self.lookahead_buffer.is_empty() {
                self.lookahead_buffer.remove(0)
            } else {
                self.chars.next().unwrap_or('\0')
            };
            self.position += current_char.len_utf8();
            if current_char == '\r' {
                if !self.is_at_end() && self.peek_char() == '\n' {
                    self.position += 1;
                    self.chars.next();
                }
                self.line += 1;
                self.column = 1;
            } else if current_char == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    pub fn consume_while<F>(&mut self, predicate: F) where F: Fn(char) -> bool {
        while !self.is_at_end() && predicate(self.peek_char()) {
            self.consume_char();
        }
    }

    pub fn get_slice(&self, start: usize, end: usize) -> &str {
        let mut valid_start = start;
        let mut valid_end = end;
        while valid_start > 0 && !self.input.is_char_boundary(valid_start) {
            valid_start -= 1;
        }
        while valid_end < self.input.len() && !self.input.is_char_boundary(valid_end) {
            valid_end += 1;
        }
        &self.input[valid_start..valid_end]
    }

    pub fn skip_whitespace(&mut self) {
        self.consume_while(|c| c == ' ' || c == '\t');
        if !self.is_at_end() && self.peek_char() == '#' {
            self.consume_while(|c| c != '\n' && c != '\r');
        }
    }
}