use std::str::Chars;
pub struct Tokenizer<'a> {
    source: &'a str,
    pub(crate) location: Location,
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

    fn remaining(&self) -> &'a str {
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
        for character in self.remaining().chars().take(n) {
            self.location.bump(character)
        }
        self.slice_from(start)
    }

    pub fn skip_past_char(&mut self, stop_char: char) {
        while let Some(character) = self.pop() {
            if character == stop_char {
                break;
            }
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        let result = self.first();
        if let Some(character) = result {
            self.location.bump(character);
        }
        result
    }

    pub fn advance_while(&mut self, predicate: impl Fn(char) -> bool) {
        let peek_iter = self.chars();

        for character in peek_iter {
            if !predicate(character) {
                break;
            }
            self.location.bump(character);
        }
    }
}

impl Default for Location {
    fn default() -> Location {
        Location { index: 0, line: 1 }
    }
}
impl Location {
    pub fn bump(&mut self, character: char) {
        println!("char is {:?}.", character);
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
    fn test_pop() {
        let mut tokenizer = Tokenizer::new("test");

        assert_eq!(tokenizer.pop(), Some('t'));
        assert_eq!(tokenizer.pop(), Some('e'));
        assert_eq!(tokenizer.pop(), Some('s'));
        assert_eq!(tokenizer.pop(), Some('t'));
        assert_eq!(tokenizer.pop(), None);
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

    #[test]
    fn empty_source() {
        let mut tokenizer = Tokenizer::new("");
        assert_eq!(tokenizer.first(), None);
        assert_eq!(tokenizer.second(), None);
        assert_eq!(tokenizer.pop(), None);
        assert_eq!(tokenizer.consume_chars(5), "");
        assert_eq!(tokenizer.line(), 1);
    }

    #[test]
    fn consume_zero_chars_is_noop() {
        let mut tokenizer = Tokenizer::new("abc");
        assert_eq!(tokenizer.consume_chars(0), "");
        assert_eq!(tokenizer.first(), Some('a'));
        assert_eq!(tokenizer.location.index, 0);
    }

    #[test]
    fn line_starts_at_one_and_increments_on_newline() {
        let mut tokenizer = Tokenizer::new("a\nb\n\nc");
        assert_eq!(tokenizer.line(), 1);
        assert_eq!(tokenizer.pop(), Some('a'));
        assert_eq!(tokenizer.line(), 1);
        assert_eq!(tokenizer.pop(), Some('\n'));
        assert_eq!(tokenizer.line(), 2);
        assert_eq!(tokenizer.pop(), Some('b'));
        assert_eq!(tokenizer.pop(), Some('\n'));
        assert_eq!(tokenizer.pop(), Some('\n'));
        assert_eq!(tokenizer.line(), 4);
    }

    #[test]
    fn advance_while_stops_at_predicate() {
        let mut tokenizer = Tokenizer::new("12345abc");
        tokenizer.advance_while(|c| c.is_ascii_digit());
        assert_eq!(tokenizer.first(), Some('a'));
        assert_eq!(tokenizer.location.index, 5);
    }

    #[test]
    fn advance_while_handles_eof() {
        let mut tokenizer = Tokenizer::new("12345");
        tokenizer.advance_while(|c| c.is_ascii_digit());
        assert_eq!(tokenizer.first(), None);
    }

    #[test]
    fn advance_while_empty_match_is_noop() {
        let mut tokenizer = Tokenizer::new("abc");
        tokenizer.advance_while(|c| c.is_ascii_digit());
        assert_eq!(tokenizer.first(), Some('a'));
        assert_eq!(tokenizer.location.index, 0);
    }

    #[test]
    fn consume_to_char_finds_target() {
        let mut tokenizer = Tokenizer::new("hello\nworld");
        tokenizer.skip_past_char('\n');
        assert_eq!(tokenizer.first(), Some('w'));
        assert_eq!(tokenizer.line(), 2); // newline was consumed
    }

    #[test]
    fn consume_to_char_eof_without_match() {
        let mut tokenizer = Tokenizer::new("hello");
        tokenizer.skip_past_char('\n');
        assert_eq!(tokenizer.first(), None);
    }

    #[test]
    fn slice_from_returns_span_between_locations() {
        let mut tokenizer = Tokenizer::new("foobar");
        let start = tokenizer.location;
        tokenizer.consume_chars(3);
        assert_eq!(tokenizer.slice_from(start), "foo");
    }

    #[test]
    fn remaining_reflects_cursor() {
        let mut tokenizer = Tokenizer::new("foobar");
        assert_eq!(tokenizer.remaining(), "foobar");
        tokenizer.consume_chars(3);
        assert_eq!(tokenizer.remaining(), "bar");
        tokenizer.consume_chars(10);
        assert_eq!(tokenizer.remaining(), "");
    }

    #[test]
    fn handles_multibyte_utf8() {
        let mut tokenizer = Tokenizer::new("é🦀z");
        assert_eq!(tokenizer.first(), Some('é'));
        assert_eq!(tokenizer.pop(), Some('é'));
        assert_eq!(tokenizer.location.index, 2);
        assert_eq!(tokenizer.pop(), Some('🦀'));
        assert_eq!(tokenizer.location.index, 6);
        assert_eq!(tokenizer.pop(), Some('z'));
        assert_eq!(tokenizer.pop(), None);
    }

    #[test]
    fn consume_chars_with_multibyte() {
        let mut tokenizer = Tokenizer::new("é🦀z");
        assert_eq!(tokenizer.consume_chars(2), "é🦀");
        assert_eq!(tokenizer.first(), Some('z'));
    }

    #[test]
    fn advance_while_counts_newlines() {
        let mut tokenizer = Tokenizer::new("\n\n\nx");
        tokenizer.advance_while(|c| c == '\n');
        assert_eq!(tokenizer.line(), 4);
        assert_eq!(tokenizer.first(), Some('x'));
    }
}
