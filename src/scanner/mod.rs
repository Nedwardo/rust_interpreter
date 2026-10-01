mod scanner_error;
use crate::token::Span;
use crate::token::Token;
use crate::token_type::KeywordToken as KT;
use crate::token_type::LiteralToken as LT;
use crate::token_type::TokenType as TT;
use core::str::Chars;
use log::debug;
use scanner_error::ScannerError as Error;

pub fn scan(source: &'_ str) -> Result<Vec<Token>, Vec<Error>> {
    Scanner::new(source).scan_tokens()
}

pub struct Scanner<'a> {
    iter: Cursor<'a>,
}

pub struct Cursor<'a> {
    source: &'a str,
    index: usize,
}

impl<'a> Scanner<'a> {
    const fn new(source: &'a str) -> Self {
        Scanner {
            iter: Cursor::new(source),
        }
    }

    fn scan_tokens(&mut self) -> Result<Vec<Token>, Vec<Error>> {
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

        if errors.is_empty() {
            debug!("TT: {tokens:?}");
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    fn scan_token(&mut self, character: char) -> Result<Option<Token>, Error> {
        let token = match character {
            '(' => self.build_single_char_token(KT::LeftParen),
            ')' => self.build_single_char_token(KT::RightParen),
            '{' => self.build_single_char_token(KT::LeftBrace),
            '}' => self.build_single_char_token(KT::RightBrace),
            ',' => self.build_single_char_token(KT::Comma),
            '.' => self.build_single_char_token(KT::Dot),
            '-' => self.build_single_char_token(KT::Minus),
            '+' => self.build_single_char_token(KT::Plus),
            ';' => self.build_single_char_token(KT::Semicolon),
            '*' => self.build_single_char_token(KT::Star),
            '?' => self.build_single_char_token(KT::QuestionMark),
            ':' => self.build_single_char_token(KT::Colon),

            '!' => self.build_compound('=', KT::BangEqual, KT::Bang),
            '=' => self.build_compound('=', KT::EqualEqual, KT::Equal),
            '<' => self.build_compound('=', KT::LessEqual, KT::Less),
            '>' => self.build_compound('=', KT::GreaterEqual, KT::Greater),

            '/' if self.iter.second() == Some('/') => {
                self.iter.consume_comment();
                return Ok(None);
            }
            '/' => self.build_single_char_token(KT::Slash),
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

    fn build_single_char_token(&mut self, keyword_token: KT) -> Token {
        self.build_sized_token(TT::Keyword(keyword_token), 1)
    }

    fn build_sized_token(&mut self, token_type: TT, size: usize) -> Token {
        let span = self.iter.consume_chars(size);
        Token::new(token_type, span)
    }

    fn build_compound(
        &mut self,
        char_flag: char,
        two_char_token: KT,
        one_char_token: KT,
    ) -> Token {
        if self.iter.second() == Some(char_flag) {
            self.build_sized_token(TT::Keyword(two_char_token), 2)
        } else {
            self.build_single_char_token(one_char_token)
        }
    }

    fn build_string(&mut self) -> Result<Token, Error> {
        match self.iter.consume_string() {
            Ok(span) => Ok(Token::new(TT::Literal(LT::String), span)),
            Err(span) => Err(Error {
                message: "Unterminated string",
                error_location: span,
            }),
        }
    }

    fn build_number(&mut self) -> Token {
        let span = self.iter.consume_number();
        Token::new(TT::Literal(LT::Number), span)
    }

    fn build_identifier(&mut self) -> Token {
        let (span, lexeme) = self.iter.consume_identifier();
        if let Some(keyword) = TT::from_lexeme(lexeme) {
            return Token::new(keyword, span);
        }

        Token::new(TT::Literal(LT::Identifier), span)
    }

    fn scan_unexpected(&mut self) -> Error {
        let character = self.iter.consume_chars(1);
        Error {
            message: "Unexpected character",
            error_location: character,
        }
    }
}

impl<'a> Cursor<'a> {
    const fn new(source: &'a str) -> Self {
        Self { source, index: 0 }
    }

    fn remaining(&self) -> &'a str {
        debug_assert!(self.source.is_char_boundary(self.index));
        &self.source[self.index..]
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

    fn consume_chars(&mut self, n: usize) -> Span {
        let start = self.index;
        for character in self.remaining().chars().take(n) {
            self.index += character.len_utf8();
        }
        (start, self.index)
    }

    fn pop(&mut self) -> Option<char> {
        let result = self.first();
        if let Some(character) = result {
            self.index += character.len_utf8();
        }

        result
    }

    fn advance_while(&mut self, predicate: impl Fn(char) -> bool) {
        let peek_iter = self.chars();

        for character in peek_iter {
            if !predicate(character) {
                break;
            }
            self.index += character.len_utf8();
        }
    }

    fn consume_string(&mut self) -> Result<Span, Span> {
        let start = self.index;

        let first = self.pop();
        debug_assert_eq!(first, Some('"'));

        self.advance_while(|c| c != '"');
        let terminated_by_quote = self.pop().is_some();
        let span = (start, self.index);

        if terminated_by_quote {
            Ok(span)
        } else {
            Err(span)
        }
    }

    fn consume_number(&mut self) -> Span {
        let start = self.index;

        self.advance_while(|c| c.is_ascii_digit());

        if self.first() == Some('.')
            && self.second().is_some_and(|c| c.is_ascii_digit())
        {
            self.pop();
            self.advance_while(|c| c.is_ascii_digit());
        }
        (start, self.index)
    }

    fn consume_identifier(&mut self) -> (Span, &'a str) {
        let start = self.index;
        self.advance_while(|c| c.is_ascii_alphanumeric() || c == '_');
        ((start, self.index), &self.source[start..self.index])
    }

    fn consume_comment(&mut self) -> Span {
        let start = self.index;
        self.advance_while(|c| c != '\n');
        (start, self.index)
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
    use crate::error_utils::HydratedStageError;
    #[test]
    fn empty_input_yields_empty_output() {
        let tokens = Scanner::new("").scan_tokens().unwrap();

        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn single_char_tokens() {
        let tokens = Scanner::new("(){},.-+;*").scan_tokens().unwrap();
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![
            TT::Keyword(KT::LeftParen),
            TT::Keyword(KT::RightParen),
            TT::Keyword(KT::LeftBrace),
            TT::Keyword(KT::RightBrace),
            TT::Keyword(KT::Comma),
            TT::Keyword(KT::Dot),
            TT::Keyword(KT::Minus),
            TT::Keyword(KT::Plus),
            TT::Keyword(KT::Semicolon),
            TT::Keyword(KT::Star),
        ];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn compound_operators_prefer_two_char() {
        let tokens = Scanner::new("!= == <= >= ! = < >").scan_tokens().unwrap();
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![
            TT::Keyword(KT::BangEqual),
            TT::Keyword(KT::EqualEqual),
            TT::Keyword(KT::LessEqual),
            TT::Keyword(KT::GreaterEqual),
            TT::Keyword(KT::Bang),
            TT::Keyword(KT::Equal),
            TT::Keyword(KT::Less),
            TT::Keyword(KT::Greater),
        ];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn slash_is_division_when_not_doubled() {
        let tokens = Scanner::new("a / b").scan_tokens().unwrap();
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![
            TT::Literal(LT::Identifier),
            TT::Keyword(KT::Slash),
            TT::Literal(LT::Identifier),
        ];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn string_literal_strips_quotes_in_value() {
        let tokens = Scanner::new(r#""hello""#).scan_tokens().unwrap();
        let token = &tokens[0];

        let expected_token_type = TT::Literal(LT::String);

        assert_eq!(tokens.len(), 1);
        assert_eq!(token.token_kind, expected_token_type);
    }

    #[test]
    fn empty_string_literal() {
        let tokens = Scanner::new(r#""""#).scan_tokens().unwrap();
        let token = &tokens[0];

        let expected_token_type = TT::Literal(LT::String);

        assert_eq!(tokens.len(), 1);
        assert_eq!(token.token_kind, expected_token_type);
    }

    #[test]
    fn unterminated_string_is_error() {
        let program = "\"no end";
        let error = Scanner::new(program).scan_tokens().unwrap_err();
        let expected_error_message = concat!(
            "Error during scanning: Unterminated string\n",
            r#"   1 | "no end"#,
            "\n",
            r#"     | ^^^^^^^"#,
            "\n"
        );

        assert_eq!(
            HydratedStageError::hydrate_errors(error, program).to_string(),
            expected_error_message
        );
    }

    #[test]
    fn lone_quote_is_unterminated_not_panic() {
        let program = "\"";
        let error = Scanner::new(program).scan_tokens().unwrap_err();
        let expected_error_message = concat!(
            "Error during scanning: Unterminated string\n",
            r#"   1 | ""#,
            "\n",
            r#"     | ^"#,
            "\n"
        );

        assert_eq!(
            HydratedStageError::hydrate_errors(error, program).to_string(),
            expected_error_message
        );
    }

    #[test]
    fn scan_number() {
        let mut tokens = Scanner::new("123").scan_tokens().unwrap();
        let mut token = tokens[0];

        let expected_token_type = TT::Literal(LT::Number);

        assert_eq!(tokens.len(), 1);
        assert_eq!(token.token_kind, expected_token_type);

        tokens = Scanner::new("3.15").scan_tokens().unwrap();
        token = tokens[0];

        assert_eq!(tokens.len(), 1);
        assert_eq!(token.token_kind, expected_token_type);
    }

    #[test]
    fn trailing_dot_is_separate_token() {
        let tokens = Scanner::new("123.").scan_tokens().unwrap();
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types =
            vec![TT::Literal(LT::Number), TT::Keyword(KT::Dot)];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn leading_dot_is_separate_token() {
        let tokens = Scanner::new(".123").scan_tokens().unwrap();
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types =
            vec![TT::Keyword(KT::Dot), TT::Literal(LT::Number)];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn identifier_vs_keyword() {
        let tokens = Scanner::new("var foo if").scan_tokens().unwrap();
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_kind).collect();

        let expected_types = vec![
            TT::Keyword(KT::Var),
            TT::Literal(LT::Identifier),
            TT::Keyword(KT::If),
        ];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn identifier_with_underscore_and_digits() {
        let tokens = Scanner::new("_foo bar123 _").scan_tokens().unwrap();
        let types: Vec<_> = tokens.iter().map(|t| t.token_kind).collect();

        let expected_types = vec![
            TT::Literal(LT::Identifier),
            TT::Literal(LT::Identifier),
            TT::Literal(LT::Identifier),
        ];

        assert_eq!(tokens.len(), 3);

        assert_eq!(types, expected_types);
    }

    #[test]
    fn identifier_cannot_start_with_digit() {
        let result = Scanner::new("123abc").scan_tokens();
        let types: Vec<_> =
            result.unwrap().into_iter().map(|t| t.token_kind).collect();

        let expected_types =
            vec![TT::Literal(LT::Number), TT::Literal(LT::Identifier)];
        assert_eq!(types, expected_types);
    }

    #[test]
    fn comment_skips_until_eol() {
        let result = Scanner::new("123//some words if\n+").scan_tokens();
        let types: Vec<_> =
            result.unwrap().into_iter().map(|t| t.token_kind).collect();

        let expected_types =
            vec![TT::Literal(LT::Number), TT::Keyword(KT::Plus)];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn multiple_errors_are_produced() {
        let program = "@+`";
        let error = Scanner::new(program).scan_tokens().unwrap_err();

        let expected_error_message = concat!(
            "Error during scanning: Unexpected character\n",
            "   1 | @+`\n",
            "     | ^\n",
            "\n",
            "Error during scanning: Unexpected character\n",
            "   1 | @+`\n",
            "     |   ^\n",
        );

        assert_eq!(
            HydratedStageError::hydrate_errors(error, program).to_string(),
            expected_error_message
        );
    }
}

#[allow(
    clippy::indexing_slicing,
    clippy::min_ident_chars,
    clippy::unwrap_used,
    reason = "tests"
)]
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

        assert_eq!(tokenizer.consume_chars(3), (0, 3));

        assert_eq!(tokenizer.first(), Some('t'));
        assert_eq!(tokenizer.second(), None);

        assert_eq!(tokenizer.consume_chars(5), (3, 4));
    }

    #[test]
    fn consume_chars() {
        let mut tokenizer = Cursor::new("test");
        assert_eq!(tokenizer.consume_chars(3), (0, 3));
        assert_eq!(tokenizer.first(), Some('t'));

        tokenizer = Cursor::new("testy");
        let _: Span = tokenizer.consume_chars(2);
        let _: Span = tokenizer.consume_chars(2);
        assert_eq!(tokenizer.first(), Some('y'));
        assert_eq!(tokenizer.second(), None);
    }

    #[test]
    fn empty_source() {
        let mut tokenizer = Cursor::new("");
        assert_eq!(tokenizer.first(), None);
        assert_eq!(tokenizer.second(), None);
        assert_eq!(tokenizer.pop(), None);
        assert_eq!(tokenizer.consume_chars(5), (0, 0));
        assert_eq!(tokenizer.index, 0);
    }

    #[test]
    fn consume_zero_chars_is_noop() {
        let mut tokenizer = Cursor::new("abc");
        assert_eq!(tokenizer.consume_chars(0), (0, 0));
        assert_eq!(tokenizer.first(), Some('a'));
        assert_eq!(tokenizer.index, 0);
    }

    #[test]
    fn advance_while_stops_at_predicate() {
        let mut tokenizer = Cursor::new("12345abc");
        tokenizer.advance_while(|c| c.is_ascii_digit());
        assert_eq!(tokenizer.first(), Some('a'));
        assert_eq!(tokenizer.index, 5);
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
        assert_eq!(tokenizer.index, 0);
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
        assert_eq!(tokenizer.index, 2);
        assert_eq!(tokenizer.pop(), Some('🦀'));
        assert_eq!(tokenizer.index, 6);
        assert_eq!(tokenizer.pop(), Some('z'));
        assert_eq!(tokenizer.pop(), None);
    }

    #[test]
    fn consume_chars_with_multibyte() {
        let mut tokenizer = Cursor::new("é🦀z");
        assert_eq!(tokenizer.consume_chars(2), (0, 6));
        assert_eq!(tokenizer.first(), Some('z'));
    }
}
