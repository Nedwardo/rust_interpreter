use crate::interpreter_error::InterpreterError;
use crate::keywords::get_keyword;
use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;
use crate::token_type::TokenType as TT;
use crate::tokenizer::Tokenizer;

pub struct Scanner<'a> {
    iter: Tokenizer<'a>,
}

pub struct ScanResult<'a> {
    pub tokens: Vec<Token<'a>>,
    pub errors: Vec<InterpreterError<'a>>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            iter: Tokenizer::new(source),
        }
    }

    pub fn scan_tokens(&mut self) -> ScanResult<'a> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        while let Some(character) = self.iter.first() {
            let result = self.scan_token(character);
            println!(
                "Finsihed scanning, Index is now at {:?}, and line is {:?}, next char is {:?}\n",
                self.iter.location.index,
                self.iter.location.line,
                self.iter.first()
            );
            match result {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => {}
                Err(error) => errors.push(error),
            }
        }
        tokens.push(Token::new(TT::EOF, "", None, self.iter.line()));
        ScanResult { tokens, errors }
    }

    fn scan_token(&mut self, character: char) -> Result<Option<Token<'a>>, InterpreterError<'a>> {
        println!(
            "Scanning starting on token {:?}, with index {:?}, and line {:?}",
            character, self.iter.location.index, self.iter.location.line
        );
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

            '/' if self.iter.second() == Some('/') => {
                self.iter.skip_past_char('\n');
                return Ok(None);
            }
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

    fn build_sized_token(&mut self, token_type: TokenType, size: usize) -> Token<'a> {
        let lexeme = self.iter.consume_chars(size);
        Token::new(token_type, lexeme, None, self.iter.line())
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

        let value = LiteralValue::String(&lexeme[1..lexeme.len() - 1]);
        Ok(Token::new(TT::STRING, lexeme, Some(value), line))
    }

    fn build_number(&mut self) -> Token<'a> {
        let lexeme = self.iter.consume_number();
        let value = LiteralValue::Number(
            lexeme
                .parse::<f64>()
                .expect("Tokenizer guarantees valid float syntax"),
        );
        Token::new(TT::NUMBER, lexeme, Some(value), self.iter.line())
    }

    fn build_identifier(&mut self) -> Token<'a> {
        let lexeme = self.iter.consume_identifier();
        let token_type = get_keyword(lexeme).unwrap_or(TT::IDENTIFIER);
        Token::new(token_type, lexeme, None, self.iter.line())
    }

    fn scan_unexpected(&mut self) -> InterpreterError<'a> {
        let character = self.iter.consume_chars(1);
        InterpreterError {
            line: self.iter.line(),
            message: "Unexpected character",
            error_location: Some(character),
        }
    }
}

impl<'a> Tokenizer<'a> {
    fn consume_string(&mut self) -> (&'a str, bool) {
        let start = self.location;

        let first = self.pop();
        debug_assert_eq!(first, Some('"'));

        self.advance_while(|c| c != '"');
        let last = self.pop();

        if last.is_some() {
            debug_assert_eq!(last, Some('"'));
        } else {
            println!("Unterminated string")
        }

        (self.slice_from(start), last.is_some())
    }

    fn consume_number(&mut self) -> &'a str {
        let start = self.location;

        self.advance_while(|c| c.is_ascii_digit());

        if self.first() == Some('.') && self.second().is_some_and(|c| c.is_ascii_digit()) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_only_eof() {
        let result = Scanner::new("").scan_tokens();
        let tokens = result.tokens;
        let errors = result.errors;

        assert!(errors.is_empty());
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TT::EOF);
    }

    #[test]
    fn single_char_tokens() {
        let tokens = Scanner::new("(){},.-+;*").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_type).collect();

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
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_type).collect();

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

        assert_eq!(types, expected_types)
    }

    #[test]
    fn slash_is_division_when_not_doubled() {
        let tokens = Scanner::new("a / b").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_type).collect();

        let expected_types = vec![TT::IDENTIFIER, TT::SLASH, TT::IDENTIFIER, TT::EOF];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn line_comment_consumes_to_newline() {
        let tokens = Scanner::new("// this is ignored\n+").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_type).collect();

        let expected_types = vec![TT::PLUS, TT::EOF];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn whitespace_is_skipped_but_tracks_lines() {
        let tokens = Scanner::new("  \t\r\n+\n\n-").scan_tokens().tokens;
        let types: Vec<_> = tokens.iter().map(|t| t.token_type).collect();
        let lines: Vec<_> = tokens.iter().map(|t| t.line).collect();

        let expected_types = vec![TT::PLUS, TT::MINUS, TT::EOF];
        let expected_lines = vec![2, 4, 4];
        assert_eq!(types, expected_types);
        assert_eq!(lines, expected_lines)
    }

    #[test]
    fn string_literal_strips_quotes_in_value() {
        let tokens = Scanner::new(r#""hello""#).scan_tokens().tokens;
        let token = &tokens[0];

        let expected_token_type = TT::STRING;
        let expected_lexeme = r#""hello""#;
        let expected_literal = LiteralValue::String("hello");

        assert_eq!(tokens.len(), 2);
        assert_eq!(token.token_type, expected_token_type);
        assert_eq!(token.lexeme, expected_lexeme);
        assert_eq!(token.literal, Some(expected_literal));
        assert_eq!(tokens[1].token_type, TT::EOF);
    }

    #[test]
    fn empty_string_literal() {
        let tokens = Scanner::new(r#""""#).scan_tokens().tokens;
        let token = &tokens[0];

        let expected_token_type = TT::STRING;
        let expected_literal = LiteralValue::String("");

        assert_eq!(tokens.len(), 2);
        assert_eq!(token.token_type, expected_token_type);
        assert_eq!(token.literal, Some(expected_literal));
        assert_eq!(tokens[1].token_type, TT::EOF);
    }

    #[test]
    fn multiline_string_tracks_lines() {
        let tokens = Scanner::new("\"line1\nline2\"\n+").scan_tokens().tokens;

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token_type, TT::STRING);
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[1].token_type, TT::PLUS);
        assert_eq!(tokens[1].line, 3);
        assert_eq!(tokens[2].token_type, TT::EOF);
    }

    #[test]
    fn unterminated_string_is_error() {
        let result = Scanner::new(r#""no end"#).scan_tokens();
        let error = &result.errors[0];
        let token = &result.tokens[0];

        let expected_error_message = "Unterminated string";

        assert_eq!(result.tokens.len(), 1);
        assert_eq!(token.token_type, TT::EOF);
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
        assert_eq!(token.token_type, TT::EOF);

        assert_eq!(result.errors.len(), 1);
        assert_eq!(error.message, expected_error_message);
    }

    #[test]
    fn test_number() {
        let tokens = Scanner::new("123").scan_tokens().tokens;
        let token = tokens[0];

        let expected_token_type = TT::NUMBER;
        let expected_token_literal = LiteralValue::Number(123.0);

        assert_eq!(tokens.len(), 2);
        assert_eq!(token.token_type, expected_token_type);
        assert_eq!(token.literal, Some(expected_token_literal));
        assert_eq!(tokens[1].token_type, TT::EOF);

        let tokens = Scanner::new("3.15").scan_tokens().tokens;
        let token = tokens[0];

        let expected_token_literal = LiteralValue::Number(3.15);

        assert_eq!(tokens.len(), 2);
        assert_eq!(token.token_type, expected_token_type);
        assert_eq!(token.literal, Some(expected_token_literal));
        assert_eq!(tokens[1].token_type, TT::EOF);
    }

    #[test]
    fn trailing_dot_is_separate_token() {
        let tokens = Scanner::new("123.").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_type).collect();

        let expected_types = vec![TT::NUMBER, TT::DOT, TT::EOF];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn leading_dot_is_separate_token() {
        let tokens = Scanner::new(".123").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_type).collect();

        let expected_types = vec![TT::DOT, TT::NUMBER, TT::EOF];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn identifier_vs_keyword() {
        let tokens = Scanner::new("var foo if").scan_tokens().tokens;
        let types: Vec<_> = tokens.into_iter().map(|t| t.token_type).collect();

        let expected_types = vec![TT::VAR, TT::IDENTIFIER, TT::IF, TT::EOF];

        assert_eq!(types, expected_types);
    }

    #[test]
    fn identifier_with_underscore_and_digits() {
        let tokens = Scanner::new("_foo bar123 _").scan_tokens().tokens;

        let expected_identifier = TT::IDENTIFIER;

        assert_eq!(tokens.len(), 4);

        assert!(
            tokens
                .iter()
                .take(3)
                .all(|t| t.token_type == expected_identifier)
        );
        assert_eq!(tokens[0].lexeme, "_foo");
        assert_eq!(tokens[1].lexeme, "bar123");
        assert_eq!(tokens[2].lexeme, "_");
        assert_eq!(tokens[3].token_type, TT::EOF);
    }

    #[test]
    fn identifier_cannot_start_with_digit() {
        let result = Scanner::new("123abc").scan_tokens();
        let types: Vec<_> = result.tokens.into_iter().map(|t| t.token_type).collect();

        let expected_types = vec![TT::NUMBER, TT::IDENTIFIER, TT::EOF];
        assert_eq!(types, expected_types);
    }

    #[test]
    fn unexpected_char_produces_error_and_continues() {
        let result = Scanner::new("@+@").scan_tokens();
        let types: Vec<_> = result.tokens.into_iter().map(|t| t.token_type).collect();
        let errors = &result.errors;

        let expected_types = vec![TT::PLUS, TT::EOF];
        assert_eq!(types, expected_types);

        let expected_error_message = "Unexpected character";
        assert_eq!(result.errors.len(), 2);
        assert!(errors.iter().all(|e| e.message == expected_error_message))
    }

    #[test]
    fn always_terminates_with_eof() {
        for src in ["", " ", "@", "//x", "\"unterminated"] {
            let tokens = Scanner::new(src).scan_tokens().tokens;

            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].token_type, TT::EOF);
        }
    }
}
