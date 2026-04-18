use crate::interpreter_error::InterpreterError;
use crate::token::TokenKind::{Comment, Identifier, SelfContained, Value};
use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;
use crate::token_type::TokenType as TT;
use core::str::Chars;

pub struct Scanner<'a> {
    iter: Cursor<'a>,
}

pub struct ScanResult<'a> {
    pub tokens: Vec<Token<'a>>,
    pub errors: Vec<InterpreterError<'a>>,
}

pub struct Cursor<'a> {
    source: &'a str,
    location: Location,
}

#[derive(Clone, Copy)]
pub struct Location {
    pub index: usize,
    pub line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            iter: Cursor::new(source),
        }
    }

    pub fn scan_tokens(&mut self) -> ScanResult<'a> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        while let Some(character) = self.iter.first() {
            let result = self.scan_token(character);
            match result {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => {}
                Err(error) => errors.push(error),
            }
        }
        tokens.push(Token::new(SelfContained(TT::EOF), self.iter.line()));
        ScanResult { tokens, errors }
    }

    fn scan_token(
        &mut self,
        character: char,
    ) -> Result<Option<Token<'a>>, InterpreterError<'a>> {
        let token = match character {
            '(' => self.build_single_char_token(TT::LEFT_PAREN),
            ')' => self.build_single_char_token(TT::RIGHT_PAREN),
            '{' => self.build_single_char_token(TT::LEFT_BRACE),
            '}' => self.build_single_char_token(TT::RIGHT_BRACE),
            ',' => self.build_single_char_token(TT::COMMA),
            '.' => self.build_single_char_token(TT::DOT),
            '-' => self.build_single_char_token(TT::MINUS),
            '+' => self.build_single_char_token(TT::PLUS),
            ';' => self.build_single_char_token(TT::SEMICOLON),
            '*' => self.build_single_char_token(TT::STAR),

            '!' => self.build_compound('=', TT::BANG_EQUAL, TT::BANG),
            '=' => self.build_compound('=', TT::EQUAL_EQUAL, TT::EQUAL),
            '<' => self.build_compound('=', TT::LESS_EQUAL, TT::LESS),
            '>' => self.build_compound('=', TT::GREATER_EQUAL, TT::GREATER),

            '/' if self.iter.second() == Some('/') => self.build_comment(),
            '/' => self.build_single_char_token(TT::SLASH),
            ' ' | '\r' | '\t' | '\n' => {
                self.iter.pop();
                return Ok(None);
            }
            '"' => self.build_string()?,
            c if c.is_ascii_digit() => self.build_number(),
            c if c.is_ascii_alphabetic() || c == '_' => self.build_identifier(),
            _ => return Err(self.scan_unexpected()),
        };
        Ok(Some(token))
    }

    fn build_single_char_token(&mut self, token_type: TokenType) -> Token<'a> {
        self.build_sized_token(token_type, 1)
    }

    fn build_sized_token(
        &mut self,
        token_type: TokenType,
        size: usize,
    ) -> Token<'a> {
        let _ = self.iter.consume_chars(size);
        Token::new(SelfContained(token_type), self.iter.line())
    }

    fn build_compound(
        &mut self,
        char_flag: char,
        two_char_token: TokenType,
        one_char_token: TokenType,
    ) -> Token<'a> {
        if self.iter.second() == Some(char_flag) {
            self.build_sized_token(two_char_token, 2)
        } else {
            self.build_single_char_token(one_char_token)
        }
    }

    fn build_string(&mut self) -> Result<Token<'a>, InterpreterError<'a>> {
        let line = self.iter.line();
        let (lexeme, success) = self.iter.consume_string();
        if !success {
            return Err(InterpreterError {
                line,
                message: "Unterminated string",
                error_location: Some(lexeme),
            });
        }

        debug_assert!(
            lexeme.is_char_boundary(1)
                && lexeme.is_char_boundary(lexeme.len() - 1),
            r#"The first and last chars are '"'"#
        );
        let literal_value = LiteralValue::String(&lexeme[1..lexeme.len() - 1]);
        Ok(Token::new(Value(literal_value), line))
    }

    fn build_number(&mut self) -> Token<'a> {
        let lexeme = self.iter.consume_number();
        let value = LiteralValue::Number(
            lexeme
                .parse::<f64>()
                .expect("Consume number guarantees valid float syntax"),
        );
        Token::new(Value(value), self.iter.line())
    }

    fn build_identifier(&mut self) -> Token<'a> {
        let lexeme = self.iter.consume_identifier();
        let token_type = TT::from_lexeme(lexeme);

        let token_kind = token_type.map_or(Identifier(lexeme), |operator| {
            LiteralValue::from_keyword(operator)
                .map_or(SelfContained(operator), |literal_value| {
                    Value(literal_value)
                })
        });
        Token::new(token_kind, self.iter.line())
    }

    fn scan_unexpected(&mut self) -> InterpreterError<'a> {
        let character = self.iter.consume_chars(1);
        InterpreterError {
            line: self.iter.line(),
            message: "Unexpected character",
            error_location: Some(character),
        }
    }

    fn build_comment(&mut self) -> Token<'a> {
        let lexeme = self.iter.consume_comment();
        Token::new(Comment(lexeme), self.iter.line())
    }
}

impl<'a> Cursor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            location: Location::default(),
        }
    }

    const fn line(&self) -> usize {
        self.location.line
    }

    fn remaining(&self) -> &'a str {
        debug_assert!(self.source.is_char_boundary(self.location.index));
        &self.source[self.location.index..]
    }

    fn chars(&self) -> Chars<'a> {
        self.remaining().chars()
    }

    fn first(&self) -> Option<char> {
        self.chars().next()
    }

    fn second(&self) -> Option<char> {
        self.chars().nth(1)
    }

    fn slice_from(&self, location: Location) -> &'a str {
        debug_assert!(self.source.is_char_boundary(location.index));
        &self.source[location.index..self.location.index]
    }

    fn consume_chars(&mut self, n: usize) -> &'a str {
        let start = self.location;
        for character in self.remaining().chars().take(n) {
            self.location.bump(character);
        }
        self.slice_from(start)
    }

    fn pop(&mut self) -> Option<char> {
        let result = self.first();
        if let Some(character) = result {
            self.location.bump(character);
        }
        result
    }

    fn advance_while(&mut self, predicate: impl Fn(char) -> bool) {
        let peek_iter = self.chars();

        for character in peek_iter {
            if !predicate(character) {
                break;
            }
            self.location.bump(character);
        }
    }

    fn consume_string(&mut self) -> (&'a str, bool) {
        let start = self.location;

        let first = self.pop();
        debug_assert_eq!(first, Some('"'));

        self.advance_while(|c| c != '"');
        let terminated_by_quote = self.pop().is_some();

        (self.slice_from(start), terminated_by_quote)
    }

    fn consume_number(&mut self) -> &'a str {
        let start = self.location;

        self.advance_while(|c| c.is_ascii_digit());

        if self.first() == Some('.')
            && self.second().is_some_and(|c| c.is_ascii_digit())
        {
            self.pop();
            self.advance_while(|c| c.is_ascii_digit());
        }
        self.slice_from(start)
    }

    fn consume_identifier(&mut self) -> &'a str {
        let start = self.location;
        self.advance_while(|c| c.is_ascii_alphanumeric() || c == '_');
        self.slice_from(start)
    }

    fn consume_comment(&mut self) -> &'a str {
        let start = self.location;
        self.advance_while(|c| c != '\n');
        self.slice_from(start)
    }
}

impl Default for Location {
    fn default() -> Self {
        Self { index: 0, line: 1 }
    }
}

impl Location {
    pub const fn bump(&mut self, character: char) {
        self.index += character.len_utf8();

        if character == '\n' {
            self.line += 1;
        }
    }
}

#[cfg(test)]
mod cursor_tests {
    use super::*;
    #[test]
    fn peek() {
        let tokenizer = Cursor::new("test");

        assert_eq!(tokenizer.first(), Some('t'));
        assert_eq!(tokenizer.second(), Some('e'));
    }

    #[test]
    fn pop() {
        let mut tokenizer = Cursor::new("test");

        assert_eq!(tokenizer.pop(), Some('t'));
        assert_eq!(tokenizer.pop(), Some('e'));
        assert_eq!(tokenizer.pop(), Some('s'));
        assert_eq!(tokenizer.pop(), Some('t'));
        assert_eq!(tokenizer.pop(), None);
    }

    #[test]
    fn consume() {
        let mut tokenizer = Cursor::new("test");

        assert_eq!(tokenizer.consume_chars(3), "tes");
        assert!(
            tokenizer.location.index == "tes".chars().map(char::len_utf8).sum()
        );

        assert_eq!(tokenizer.first(), Some('t'));
        assert_eq!(tokenizer.second(), None);

        assert_eq!(tokenizer.consume_chars(5), "t");
    }

    #[test]
    fn consume_chars() {
        let mut tokenizer = Cursor::new("test");
        assert_eq!(tokenizer.consume_chars(3), "tes");
        assert_eq!(tokenizer.first(), Some('t'));

        tokenizer = Cursor::new("testy");
        let _: &str = tokenizer.consume_chars(2);
        let _: &str = tokenizer.consume_chars(2);
        assert_eq!(tokenizer.first(), Some('y'));
        assert_eq!(tokenizer.second(), None);
    }

    #[test]
    fn empty_source() {
        let mut tokenizer = Cursor::new("");
        assert_eq!(tokenizer.first(), None);
        assert_eq!(tokenizer.second(), None);
        assert_eq!(tokenizer.pop(), None);
        assert_eq!(tokenizer.consume_chars(5), "");
        assert_eq!(tokenizer.line(), 1);
    }

    #[test]
    fn consume_zero_chars_is_noop() {
        let mut tokenizer = Cursor::new("abc");
        assert_eq!(tokenizer.consume_chars(0), "");
        assert_eq!(tokenizer.first(), Some('a'));
        assert_eq!(tokenizer.location.index, 0);
    }

    #[test]
    fn line_starts_at_one_and_increments_on_newline() {
        let mut tokenizer = Cursor::new("a\nb\n\nc");
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
        let mut tokenizer = Cursor::new("12345abc");
        tokenizer.advance_while(|c| c.is_ascii_digit());
        assert_eq!(tokenizer.first(), Some('a'));
        assert_eq!(tokenizer.location.index, 5);
    }

    #[test]
    fn advance_while_handles_eof() {
        let mut tokenizer = Cursor::new("12345");
        tokenizer.advance_while(|c| c.is_ascii_digit());
        assert_eq!(tokenizer.first(), None);
    }

    #[test]
    fn advance_while_empty_match_is_noop() {
        let mut tokenizer = Cursor::new("abc");
        tokenizer.advance_while(|c| c.is_ascii_digit());
        assert_eq!(tokenizer.first(), Some('a'));
        assert_eq!(tokenizer.location.index, 0);
    }

    #[test]
    fn slice_from_returns_span_between_locations() {
        let mut tokenizer = Cursor::new("foobar");
        let start = tokenizer.location;
        tokenizer.consume_chars(3);
        assert_eq!(tokenizer.slice_from(start), "foo");
    }

    #[test]
    fn remaining_reflects_cursor() {
        let mut tokenizer = Cursor::new("foobar");
        assert_eq!(tokenizer.remaining(), "foobar");
        tokenizer.consume_chars(3);
        assert_eq!(tokenizer.remaining(), "bar");
        tokenizer.consume_chars(10);
        assert_eq!(tokenizer.remaining(), "");
    }

    #[test]
    fn handles_multibyte_utf8() {
        let mut tokenizer = Cursor::new("é🦀z");
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
        let mut tokenizer = Cursor::new("é🦀z");
        assert_eq!(tokenizer.consume_chars(2), "é🦀");
        assert_eq!(tokenizer.first(), Some('z'));
    }

    #[test]
    fn advance_while_counts_newlines() {
        let mut tokenizer = Cursor::new("\n\n\nx");
        tokenizer.advance_while(|c| c == '\n');
        assert_eq!(tokenizer.line(), 4);
        assert_eq!(tokenizer.first(), Some('x'));
    }
}

#[allow(
    clippy::indexing_slicing,
    clippy::min_ident_chars,
    clippy::unwrap_used,
    reason = "tests"
)]
#[cfg(test)]
mod tokenizer_tests {
    use super::*;
    use crate::token::TokenKind;
    #[test]
    fn empty_input_yields_only_eof() {
        let result = Scanner::new("").scan_tokens();
        let tokens = result.tokens;
        let errors = result.errors;

        assert!(errors.is_empty());
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_kind, TT::EOF);
    }

    #[test]
    fn single_char_tokens() {
        let tokens = Scanner::new("(){},.-+;*").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![
            TT::LEFT_PAREN,
            TT::RIGHT_PAREN,
            TT::LEFT_BRACE,
            TT::RIGHT_BRACE,
            TT::COMMA,
            TT::DOT,
            TT::MINUS,
            TT::PLUS,
            TT::SEMICOLON,
            TT::STAR,
            TT::EOF,
        ];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn compound_operators_prefer_two_char() {
        let tokens = Scanner::new("!= == <= >= ! = < >").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![
            TT::BANG_EQUAL,
            TT::EQUAL_EQUAL,
            TT::LESS_EQUAL,
            TT::GREATER_EQUAL,
            TT::BANG,
            TT::EQUAL,
            TT::LESS,
            TT::GREATER,
            TT::EOF,
        ];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn slash_is_division_when_not_doubled() {
        let tokens = Scanner::new("a / b").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types =
            vec![TT::IDENTIFIER, TT::SLASH, TT::IDENTIFIER, TT::EOF];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn comment_consumes_to_newline() {
        let tokens = Scanner::new("// this is ignored\n+").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![TT::COMMENT, TT::PLUS, TT::EOF];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn whitespace_is_skipped_but_tracks_lines() {
        let tokens = Scanner::new("  \t\r\n+\n\n-").scan_tokens().tokens;
        let types: Vec<_> = tokens.iter().map(|t| t.token_kind).collect();
        let lines: Vec<_> = tokens.iter().map(|t| t.line).collect();

        let expected_types = vec![TT::PLUS, TT::MINUS, TT::EOF];
        let expected_lines = vec![2, 4, 4];
        assert_eq!(types, expected_types);
        assert_eq!(lines, expected_lines);
    }

    #[test]
    fn string_literal_strips_quotes_in_value() {
        let tokens = Scanner::new(r#""hello""#).scan_tokens().tokens;
        let token = &tokens[0];

        let expected_token_type = TT::STRING;
        let expected_literal = TokenKind::Value(LiteralValue::String("hello"));

        assert_eq!(tokens.len(), 2);
        assert_eq!(token.token_kind, expected_token_type);
        assert_eq!(token.token_kind, expected_literal);
        assert_eq!(tokens[1].token_kind, TT::EOF);
    }

    #[test]
    fn empty_string_literal() {
        let tokens = Scanner::new(r#""""#).scan_tokens().tokens;
        let token = &tokens[0];

        let expected_token_type = TT::STRING;
        let expected_literal = TokenKind::Value(LiteralValue::String(""));

        assert_eq!(tokens.len(), 2);
        assert_eq!(token.token_kind, expected_token_type);
        assert_eq!(token.token_kind, expected_literal);
        assert_eq!(tokens[1].token_kind, TT::EOF);
    }

    #[test]
    fn multiline_string_tracks_lines() {
        let tokens = Scanner::new("\"line1\nline2\"\n+").scan_tokens().tokens;

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token_kind, TT::STRING);
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[1].token_kind, TT::PLUS);
        assert_eq!(tokens[1].line, 3);
        assert_eq!(tokens[2].token_kind, TT::EOF);
    }

    #[test]
    fn unterminated_string_is_error() {
        let result = Scanner::new(r#""no end"#).scan_tokens();
        let error = &result.errors[0];
        let token = &result.tokens[0];

        let expected_error_message = "Unterminated string";

        assert_eq!(result.tokens.len(), 1);
        assert_eq!(token.token_kind, TT::EOF);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(error.message, expected_error_message);
    }

    #[test]
    fn lone_quote_is_unterminated_not_panic() {
        let result = Scanner::new("\"").scan_tokens();
        let error = &result.errors[0];
        let token = result.tokens[0];

        let expected_error_message = "Unterminated string";

        assert_eq!(result.tokens.len(), 1);
        assert_eq!(token.token_kind, TT::EOF);

        assert_eq!(result.errors.len(), 1);
        assert_eq!(error.message, expected_error_message);
    }

    #[test]
    fn scan_number() {
        let mut tokens = Scanner::new("123").scan_tokens().tokens;
        let mut token = tokens[0];

        let expected_token_type = TT::NUMBER;
        let mut expected_token_literal = 123.0;

        assert_eq!(tokens.len(), 2);
        assert_eq!(token.token_kind, expected_token_type);
        assert_eq!(token.token_kind, expected_token_literal);
        assert_eq!(tokens[1].token_kind, TT::EOF);

        tokens = Scanner::new("3.15").scan_tokens().tokens;
        token = tokens[0];

        expected_token_literal = 3.15;

        assert_eq!(tokens.len(), 2);
        assert_eq!(token.token_kind, expected_token_type);
        assert_eq!(token.token_kind, expected_token_literal);
        assert_eq!(tokens[1].token_kind, TT::EOF);
    }

    #[test]
    fn trailing_dot_is_separate_token() {
        let tokens = Scanner::new("123.").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![TT::NUMBER, TT::DOT, TT::EOF];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn leading_dot_is_separate_token() {
        let tokens = Scanner::new(".123").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![TT::DOT, TT::NUMBER, TT::EOF];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn identifier_vs_keyword() {
        let tokens = Scanner::new("var foo if").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![TT::VAR, TT::IDENTIFIER, TT::IF, TT::EOF];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn identifier_with_underscore_and_digits() {
        let tokens = Scanner::new("_foo bar123 _").scan_tokens().tokens;
        let types: Vec<_> = tokens.iter().map(|t| t.token_kind).collect();

        let expected_types =
            vec![TT::IDENTIFIER, TT::IDENTIFIER, TT::IDENTIFIER, TT::EOF];

        assert_eq!(tokens.len(), 4);

        assert_eq!(types, expected_types);
        assert_eq!(tokens[0].token_kind, Identifier("_foo"));
        assert_eq!(tokens[1].token_kind, Identifier("bar123"));
        assert_eq!(tokens[2].token_kind, Identifier("_"));
        assert_eq!(tokens[3].token_kind, TT::EOF);
    }

    #[test]
    fn identifier_cannot_start_with_digit() {
        let result = Scanner::new("123abc").scan_tokens();
        let types: Vec<_> =
            result.tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![TT::NUMBER, TT::IDENTIFIER, TT::EOF];
        assert_eq!(types, expected_types);
    }

    #[test]
    fn comment_skips_until_eol() {
        let result = Scanner::new("123//some words if\n+").scan_tokens();
        let types: Vec<_> =
            result.tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![TT::NUMBER, TT::COMMENT, TT::PLUS, TT::EOF];
        assert_eq!(types, expected_types);
    }

    #[test]
    fn unexpected_char_produces_error_and_continues() {
        let result = Scanner::new("@+@").scan_tokens();
        let types: Vec<_> =
            result.tokens.into_iter().map(|t| t.token_kind).collect();
        let errors = &result.errors;

        let expected_types = vec![TT::PLUS, TT::EOF];
        assert_eq!(types, expected_types);

        let expected_error_message = "Unexpected character";
        assert_eq!(result.errors.len(), 2);
        assert!(errors.iter().all(|e| e.message == expected_error_message));
    }

    #[test]
    fn always_terminates_with_eof() {
        for src in ["", " ", "@", "//x", "\"unterminated"] {
            let tokens = Scanner::new(src).scan_tokens().tokens;

            assert_eq!(tokens.iter().last().unwrap().token_kind, TT::EOF);
        }
    }
}
