use crate::interpreter_error::InterpreterError;
use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;
use crate::token_type::TokenType::{
    BANG, BANG_EQUAL, COMMA, DOT, EOF, EQUAL, EQUAL_EQUAL, GREATER, GREATER_EQUAL, LEFT_BRACE,
    LEFT_PAREN, LESS, LESS_EQUAL, MINUS, PLUS, RIGHT_BRACE, RIGHT_PAREN, SEMICOLON, SLASH, STAR,
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
        let token: TokenType;
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
            '!' => {
                if self.match_next('=') {
                    token = BANG_EQUAL
                } else {
                    token = BANG;
                }
                Ok(Some(self.build_token(token)))
            }
            '=' => {
                if self.match_next('=') {
                    token = EQUAL_EQUAL;
                } else {
                    token = EQUAL;
                }
                Ok(Some(self.build_token(token)))
            }
            '<' => {
                if self.match_next('=') {
                    token = LESS_EQUAL;
                } else {
                    token = LESS;
                }
                Ok(Some(self.build_token(token)))
            }
            '>' => {
                if self.match_next('=') {
                    token = GREATER_EQUAL;
                } else {
                    token = GREATER;
                }
                Ok(Some(self.build_token(token)))
            }
            '/' => {
                if self.match_next('/') {
                    self.iter_till('\n');
                    Ok(None)
                } else {
                    Ok(Some(self.build_token(SLASH)))
                }
            }
            _ => Err(InterpreterError {
                line: self.line,
                message: "Unexpected character",
                error_location: None,
            }),
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
        literal: Option<LiteralValue>,
    ) -> Token<'source_lt> {
        Token {
            token_type,
            lexeme: self.get_current_lexeme(),
            literal,
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
}
