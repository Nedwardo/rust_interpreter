pub struct Tokenizer<'a> {
    source: &'a str,
    index: usize,
}

pub struct PeekResult {
    pub character: Option<char>,
    pub bytes: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, index: 0 }
    }

    pub fn remaining(&self) -> &'a str {
        &self.source[self.index..]
    }

    pub fn first(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    pub fn second(&self) -> Option<char> {
        let mut iter = self.remaining().chars();
        iter.next();
        iter.next()
    }

    pub fn consume(&mut self, byte_offset: usize) -> &'a str {
        let start_index = self.index;
        self.index += byte_offset;

        debug_assert!(self.source.is_char_boundary(self.index));
        &self.source[start_index..self.index]
    }

    pub fn consume_chars(&mut self, n: usize) -> &'a str {
        let start_index = self.index;
        let mut chars = self.remaining().chars();
        for _ in 0..n {
            let c = chars.next().expect("consume_chars past end");
            self.index += c.len_utf8();
        }
        &self.source[start_index..self.index]
    }

    pub fn consume_while(&mut self, predicate: impl FnMut(char) -> bool) -> &'a str {
        let result = self.peek_while(predicate);
        self.consume(result.bytes)
    }

    pub fn consume_till(&mut self, stop_char: char) -> &'a str {
        self.consume_while(|c| c != stop_char)
    }

    pub fn peek_while(&self, mut predicate: impl FnMut(char) -> bool) -> PeekResult {
        self.peek_while_from(predicate, 0)
    }

    pub fn peek_while_from(
        &self,
        mut predicate: impl FnMut(char) -> bool,
        byte_offset: usize,
    ) -> PeekResult {
        let iter = self.remaining()[byte_offset..].chars();
        let mut bytes = 0;

        for character in iter {
            if !predicate(character) {
                return PeekResult {
                    character: Some(character),
                    bytes,
                };
            }
            bytes += character.len_utf8();
        }

        PeekResult {
            character: None,
            bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_peek() {
        let input_string = "test";
        let tokenizer = Tokenizer::new(input_string);

        assert_eq!(tokenizer.first(), Some('t'));
        assert_eq!(tokenizer.second(), Some('e'));
    }

    #[test]
    fn test_consume() {
        let input_string = "test";
        let mut tokenizer = Tokenizer::new(input_string);

        assert_eq!(tokenizer.consume(3), "tes");
    }
}
