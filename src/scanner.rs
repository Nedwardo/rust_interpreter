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
        Ok(Some(match character {
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
                self.iter.consume_to_char('\n');
                return Ok(None);
            }
            '/' => self.build_single_char_token(TT::SLASH),
            ' ' | '\r' | '\t' | '\n' => {
                self.iter.advance();
                return Ok(None);
            }
            '"' => self.build_string()?,
            c if c.is_ascii_digit() => self.build_number(),
            c if c.is_ascii_alphabetic() || c == '_' => self.build_identifier(),
            _ => return Err(self.scan_unexpected()),
        }))
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
        debug_assert_eq!(self.first(), Some('"'));

        self.advance();
        self.advance_while(|c| c != '"');
        let quote_terminated = self.advance().is_some();

        (self.slice_from(start), quote_terminated)
    }

    fn consume_number(&mut self) -> &'a str {
        let start = self.location;

        self.advance_while(|c| c.is_ascii_digit());

        if self.first() == Some('.') && self.second().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
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
