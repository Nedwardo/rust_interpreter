use crate::interpreter_error::InterpreterError;
use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;
use crate::token_type::TokenType::{
    BANG, BANG_EQUAL, COMMA, DOT, EOF, EQUAL, EQUAL_EQUAL, GREATER, GREATER_EQUAL, LEFT_BRACE,
    LEFT_PAREN, LESS, LESS_EQUAL, MINUS, PLUS, RIGHT_BRACE, RIGHT_PAREN, SEMICOLON, STAR,
};

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    current_lex_start: usize,
    current_lex_size: usize,
    current: usize,
    line: usize,
}

pub fn build_scanner<'a>(source: &'a str) -> Scanner<'a> {
    Scanner {
        source,
        tokens: Vec::new(),
        current_lex_start: 0,
        current_lex_size: 0,
        current: 0,
        line: 0,
    }
}

impl<'a> Scanner<'a> {
    pub fn scan_tokens(&mut self) -> &Vec<Token<'a>> {
        self.next().map(|character| self.scan_token(character));
        self.tokens.push(Token {
            token_type: EOF,
            lexeme: "",
            literal: None,
            line: self.line,
        });
        &self.tokens
    }

    fn scan_token(&mut self, character: char) -> Result<Token<'_>, InterpreterError> {
        let token: TokenType;
        match character {
            '(' => Ok(self.build_token(LEFT_PAREN)),
            ')' => Ok(self.build_token(RIGHT_PAREN)),
            '{' => Ok(self.build_token(LEFT_BRACE)),
            '}' => Ok(self.build_token(RIGHT_BRACE)),
            ',' => Ok(self.build_token(COMMA)),
            '.' => Ok(self.build_token(DOT)),
            '-' => Ok(self.build_token(MINUS)),
            '+' => Ok(self.build_token(PLUS)),
            ';' => Ok(self.build_token(SEMICOLON)),
            '*' => Ok(self.build_token(STAR)),
            '!' => {
                if self.match_next('=') {
                    token = BANG_EQUAL
                } else {
                    token = BANG;
                }
                Ok(self.build_token(token))
            }
            '=' => {
                if self.match_next('=') {
                    token = EQUAL_EQUAL;
                } else {
                    token = EQUAL;
                }
                Ok(self.build_token(token))
            }
            '<' => {
                if self.match_next('=') {
                    token = LESS_EQUAL;
                } else {
                    token = LESS;
                }
                Ok(self.build_token(token))
            }
            '>' => {
                if self.match_next('=') {
                    token = GREATER_EQUAL;
                } else {
                    token = GREATER;
                }
                Ok(self.build_token(token))
            }
            _ => Err(InterpreterError {
                line: self.line,
                message: "Unexpected character",
                error_location: None,
            }),
        }
    }

    fn build_token(&self, token_type: TokenType) -> Token<'_> {
        Token {
            token_type,
            lexeme: self.get_current_lexeme(),
            literal: None,
            line: self.line,
        }
    }

    fn build_literal_token(
        &self,
        token_type: TokenType,
        literal: Option<LiteralValue>,
    ) -> Token<'_> {
        Token {
            token_type,
            lexeme: self.get_current_lexeme(),
            literal,
            line: self.line,
        }
    }
    fn get_current_lexeme(&self) -> &str {
        &self.source[self.current_lex_start..self.current_lex_start + self.current_lex_size]
    }

    fn match_next(&mut self, expected: char) -> bool {
        if let Some(actual) = self.source.chars().nth(self.current + 1)
            && actual == expected
        {
            self.current += 1;
            self.current_lex_size += 1;
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> Option<char> {
        self.current += 1;
        self.source.chars().nth(self.current)
    }
}
