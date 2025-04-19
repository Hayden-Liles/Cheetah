use super::Lexer;

impl<'a> Lexer<'a> {
    pub fn peek_char(&self) -> char {
        if !self.lookahead_buffer.is_empty() {
            self.lookahead_buffer[0]
        } else {
            self.chars.clone().next().unwrap_or('\0')
        }
    }

    pub fn peek_char_n(&mut self, n: usize) -> char {
        self.ensure_lookahead_buffer(n + 1);

        if n < self.lookahead_buffer.len() {
            self.lookahead_buffer[n]
        } else {
            '\0'
        }
    }

    fn ensure_lookahead_buffer(&mut self, n: usize) {
        while self.lookahead_buffer.len() < n {
            if let Some(c) = self.chars.next() {
                self.lookahead_buffer.push(c);
            } else {
                break;
            }
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
                self.chars.next().expect("Character expected")
            };

            self.position += current_char.len_utf8();

            if current_char == '\r' {
                if let Some(next_char) = self.chars.clone().next() {
                    if next_char == '\n' {
                        self.position += 1;
                        self.chars.next();
                    }
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

    pub fn consume_while<F>(&mut self, predicate: F)
    where
        F: Fn(char) -> bool,
    {
        while !self.is_at_end() && predicate(self.peek_char()) {
            self.consume_char();
        }
    }

    pub fn consume_whitespace(&mut self) {
        self.consume_while(|c| c == ' ' || c == '\t');
    }

    pub fn get_slice(&self, start: usize, end: usize) -> &str {
        debug_assert!(
            self.input.is_char_boundary(start),
            "start must be at a character boundary"
        );
        debug_assert!(
            self.input.is_char_boundary(end),
            "end must be at a character boundary"
        );
        &self.input[start..end]
    }

    pub fn skip_whitespace(&mut self) {
        loop {
            self.consume_whitespace();

            if !self.is_at_end() && self.peek_char() == '#' {
                self.skip_comment();
                continue;
            }

            break;
        }
    }

    pub fn skip_comment(&mut self) {
        if self.peek_char() == '#' {
            let remaining = &self.input[self.position..];
            if let Some(comment_end) = remaining.find(|c| c == '\n' || c == '\r') {
                let old_position = self.position;
                self.position += comment_end;

                let skipped_text = &self.input[old_position..self.position];
                self.column += skipped_text.chars().count();

                self.lookahead_buffer.clear();
                self.chars = self.input[self.position..].chars();
            } else {
                self.consume_while(|c| c != '\n' && c != '\r');
            }
        }
    }
}
