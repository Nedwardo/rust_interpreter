use crate::interpreter_error::InterpreterError;
use crate::keywords::get_keyword;
use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;
use crate::token_type::TokenType::{
    BANG, BANG_EQUAL, COMMA, DOT, EOF, EQUAL, EQUAL_EQUAL, GREATER, GREATER_EQUAL, IDENTIFIER,
    LEFT_BRACE, LEFT_PAREN, LESS, LESS_EQUAL, MINUS, NUMBER, PLUS, RIGHT_BRACE, RIGHT_PAREN,
    SEMICOLON, SLASH, STAR, STRING,
};

pub struct Scanner<'scanner_lt> {
    source: &'scanner_lt str,
    current_lex_start: usize,
    current: usize,
    line: usize,
}

pub fn build_scanner<'scanner_lt, 'source_lt>(source: &'scanner_lt str) -> Scanner<'scanner_lt>
where
    'source_lt: 'scanner_lt,
{
    Scanner {
        source,
        current_lex_start: 0,
        current: 0,
        line: 0,
    }
}

impl<'source_lt> Scanner<'source_lt> {
    pub fn scan_tokens<'scanner_lt>(&'scanner_lt mut self) -> Vec<Token<'source_lt>> {
        let mut tokens = Vec::new();
        while let Some(character) = self.next() {
            let result = self.scan_token(character);
            if let Ok(Some(token)) = result {
                tokens.push(token);
            }
        }
        tokens.push(Token {
            token_type: EOF,
            lexeme: "",
            literal: None,
            line: self.line,
        });
        tokens
    }

    fn scan_token<'scanner_lt>(
        &'scanner_lt mut self,
        character: char,
    ) -> Result<Option<Token<'source_lt>>, InterpreterError> {
        self.current_lex_start = self.current - 1;
        match character {
            '(' => Ok(Some(self.build_token(LEFT_PAREN))),
            ')' => Ok(Some(self.build_token(RIGHT_PAREN))),
            '{' => Ok(Some(self.build_token(LEFT_BRACE))),
            '}' => Ok(Some(self.build_token(RIGHT_BRACE))),
            ',' => Ok(Some(self.build_token(COMMA))),
            '.' => Ok(Some(self.build_token(DOT))),
            '-' => Ok(Some(self.build_token(MINUS))),
            '+' => Ok(Some(self.build_token(PLUS))),
            ';' => Ok(Some(self.build_token(SEMICOLON))),
            '*' => Ok(Some(self.build_token(STAR))),
            '!' if self.match_next('=') => Ok(Some(self.build_token(BANG_EQUAL))),
            '!' => Ok(Some(self.build_token(BANG))),
            '=' if self.match_next('=') => Ok(Some(self.build_token(EQUAL_EQUAL))),
            '=' => Ok(Some(self.build_token(EQUAL))),
            '<' if self.match_next('=') => Ok(Some(self.build_token(LESS_EQUAL))),
            '<' => Ok(Some(self.build_token(LESS))),
            '>' if self.match_next('=') => Ok(Some(self.build_token(GREATER_EQUAL))),
            '>' => Ok(Some(self.build_token(GREATER))),
            '/' => {
                if self.match_next('/') {
                    self.iter_till('\n');
                    Ok(None)
                } else {
                    Ok(Some(self.build_token(SLASH)))
                }
            }
            ' ' | '\r' | '\t' => Ok(None),
            '\n' => {
                self.line += 1;
                Ok(None)
            }
            '"' => {
                self.scan_string()?;
                let lexeme = self.get_current_lexeme();
                let value = LiteralValue::String(lexeme);
                Ok(Some(self.build_literal_token(STRING, lexeme, value)))
            }
            _ => {
                if character.is_ascii_digit() {
                    self.scan_number();
                    let lexeme = self.get_current_lexeme();
                    let value = LiteralValue::Number(lexeme.parse::<f64>().unwrap());
                    Ok(Some(self.build_literal_token(NUMBER, lexeme, value)))
                } else if character.is_ascii_alphabetic() || character == '_' {
                    self.scan_identifier();
                    let lexeme = self.get_current_lexeme();
                    let token_type = get_keyword(lexeme).unwrap_or(IDENTIFIER);
                    let value = LiteralValue::String(lexeme);
                    Ok(Some(self.build_literal_token(
                        token_type,
                        self.get_current_lexeme(),
                        value,
                    )))
                } else {
                    Err(InterpreterError {
                        line: self.line,
                        message: "Unexpected character",
                        error_location: None,
                    })
                }
            }
        }
    }

    fn build_token<'scanner_lt>(&'scanner_lt self, token_type: TokenType) -> Token<'source_lt> {
        let token: Token = Token {
            token_type,
            lexeme: self.get_current_lexeme(),
            literal: None,
            line: self.line,
        };
        token
    }

    fn build_literal_token<'scanner_lt>(
        &'scanner_lt self,
        token_type: TokenType,
        lexeme: &'source_lt str,
        literal: LiteralValue<'source_lt>,
    ) -> Token<'source_lt> {
        Token {
            token_type,
            lexeme,
            literal: Some(literal),
            line: self.line,
        }
    }
    fn get_current_lexeme<'scanner_lt>(&'scanner_lt self) -> &'source_lt str {
        &self.source[self.current_lex_start..self.current]
    }

    fn next(&mut self) -> Option<char> {
        self.current += 1;
        self.source.chars().nth(self.current)
    }
    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.current + 1)
    }
    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current + 2)
    }

    fn match_next(&mut self, expected: char) -> bool {
        if let Some(actual) = self.peek()
            && actual == expected
        {
            self.current += 1;
            true
        } else {
            false
        }
    }

    fn iter_till(&mut self, expected: char) {
        while let Some(next_char) = self.peek() {
            if next_char == expected {
                break;
            }
            self.next();
        }
    }

    fn scan_string(&mut self) -> Result<(), InterpreterError> {
        self.iter_till('"');
        match self.next() {
            Some(_) => Ok(()),
            None => Err(InterpreterError {
                line: self.line,
                message: ("Unterminated string"),
                error_location: None,
            }),
        }
    }
    fn scan_number(&mut self) {
        while let Some(next_char) = self.peek()
            && next_char.is_ascii_digit()
        {
            self.next();
        }
        if self.peek() == Some('.')
            && let Some(next_digit) = self.peek_next()
            && next_digit.is_ascii_digit()
        {
            self.next();
            while let Some(next_char) = self.peek()
                && next_char.is_ascii_digit()
            {
                self.next();
            }
        }
    }
    fn scan_identifier(&mut self) {
        while let Some(next_char) = self.peek()
            && next_char.is_ascii_alphanumeric()
        {
            self.next();
        }
    }
}
