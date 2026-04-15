use std::str::Chars;
pub struct Tokenizer<'a> {
    source: &'a str,
    pub location: Location,
}

#[derive(Clone, Copy)]
pub struct Location {
    pub index: usize,
    pub line: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            location: Location::default(),
        }
    }

    pub fn line(&self) -> usize {
        self.location.line
    }

    pub fn remaining(&self) -> &'a str {
        debug_assert!(self.source.is_char_boundary(self.location.index));
        &self.source[self.location.index..]
    }

    pub fn chars(&self) -> Chars<'a> {
        self.remaining().chars()
    }

    pub fn first(&self) -> Option<char> {
        self.chars().next()
    }

    pub fn second(&self) -> Option<char> {
        self.chars().nth(1)
    }

    pub fn slice_from(&self, location: Location) -> &'a str {
        debug_assert!(self.source.is_char_boundary(location.index));
        &self.source[location.index..self.location.index]
    }

    pub fn consume_chars(&mut self, n: usize) -> &'a str {
        let start = self.location;
        for _ in 0..n {
            let result = self.advance();
            if result.is_none() {
                break;
            }
        }
        self.slice_from(start)
    }

    pub fn consume_to_char(&mut self, stop_char: char) -> (&'a str, bool) {
        let start = self.location;
        let mut char_reached = false;
        while let Some(character) = self.advance() {
            if character == stop_char {
                char_reached = true;
                break;
            }
        }
        (self.slice_from(start), char_reached)
    }

    pub fn advance(&mut self) -> Option<char> {
        let result = self.first();
        if let Some(character) = result {
            self.location.add(character);
        }
        result
    }

    pub fn advance_while(&mut self, predicate: impl Fn(char) -> bool) {
        let peek_iter = self.chars();

        for character in peek_iter {
            if !predicate(character) {
                break;
            }
            self.location.add(character);
        }
    }
}

impl Default for Location {
    fn default() -> Location {
        Location { index: 0, line: 1 }
    }
}
impl Location {
    pub fn add(&mut self, character: char) {
        self.index += character.len_utf8();

        if character == '\n' {
            self.line += 1
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_peek() {
        let tokenizer = Tokenizer::new("test");

        assert_eq!(tokenizer.first(), Some('t'));
        assert_eq!(tokenizer.second(), Some('e'));
    }

    #[test]
    fn test_advance() {
        let mut tokenizer = Tokenizer::new("test");

        assert_eq!(tokenizer.advance(), Some('t'));
        assert_eq!(tokenizer.advance(), Some('e'));
        assert_eq!(tokenizer.advance(), Some('s'));
        assert_eq!(tokenizer.advance(), Some('t'));
        assert_eq!(tokenizer.advance(), None);
    }

    #[test]
    fn test_consume() {
        let mut tokenizer = Tokenizer::new("test");

        assert_eq!(tokenizer.consume_chars(3), "tes");
        assert!(tokenizer.location.index == "tes".chars().map(char::len_utf8).sum());

        assert_eq!(tokenizer.first(), Some('t'));
        assert_eq!(tokenizer.second(), None);

        assert_eq!(tokenizer.consume_chars(5), "t");
    }

    #[test]
    fn test_consume_chars() {
        let mut tokenizer = Tokenizer::new("test");
        assert_eq!(tokenizer.consume_chars(3), "tes");
        assert_eq!(tokenizer.first(), Some('t'));

        tokenizer = Tokenizer::new("testy");
        let _ = tokenizer.consume_chars(2);
        let _ = tokenizer.consume_chars(2);
        assert_eq!(tokenizer.first(), Some('y'));
        assert_eq!(tokenizer.second(), None);
    }
}
