use crate::interpreter_error::InterpreterError;
use crate::keywords::get_keyword;
use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;
use crate::token_type::TokenType as TT;
use crate::tokenizer::Tokenizer;

pub struct Scanner<'a> {
    iter: Tokenizer<'a>,
    line: usize,
}

pub struct ScanResult<'a> {
    pub tokens: Vec<Token<'a>>,
    pub errors: Vec<InterpreterError>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            iter: Tokenizer::new(source),
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> ScanResult<'a> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        while let Some(character) = self.iter.first() {
            if let Some(res) = self.scan_token(character) {
                match res {
                    Ok(token) => tokens.push(token),
                    Err(err) => errors.push(err),
                }
            }
        }
        tokens.push(Token {
            token_type: TT::EOF,
            lexeme: "",
            literal: None,
            line: self.line,
        });
        ScanResult { tokens, errors }
    }

    fn scan_token(&mut self, character: char) -> Option<Result<Token<'a>, InterpreterError>> {
        match character {
            '(' => Some(Ok(self.build_sized_token(TT::LEFT_PAREN, 1))),
            ')' => Some(Ok(self.build_sized_token(TT::RIGHT_PAREN, 1))),
            '{' => Some(Ok(self.build_sized_token(TT::LEFT_BRACE, 1))),
            '}' => Some(Ok(self.build_sized_token(TT::RIGHT_BRACE, 1))),
            ',' => Some(Ok(self.build_sized_token(TT::COMMA, 1))),
            '.' => Some(Ok(self.build_sized_token(TT::DOT, 1))),
            '-' => Some(Ok(self.build_sized_token(TT::MINUS, 1))),
            '+' => Some(Ok(self.build_sized_token(TT::PLUS, 1))),
            ';' => Some(Ok(self.build_sized_token(TT::SEMICOLON, 1))),
            '*' => Some(Ok(self.build_sized_token(TT::STAR, 1))),

            '!' => Some(Ok(self.build_compound('=', TT::BANG_EQUAL, TT::BANG))),
            '=' => Some(Ok(self.build_compound('=', TT::EQUAL_EQUAL, TT::EQUAL))),
            '<' => Some(Ok(self.build_compound('=', TT::LESS_EQUAL, TT::LESS))),
            '>' => Some(Ok(self.build_compound('=', TT::GREATER_EQUAL, TT::GREATER))),

            '/' => {
                if self.iter.second() == Some('/') {
                    let _ = self.iter.consume_till('\n');
                    // Doesn't need MultiLineTokenInfo because it's not multi line yet
                    None
                } else {
                    Some(Ok(self.build_sized_token(TT::SLASH, 1)))
                }
            }
            ' ' | '\r' | '\t' => None,
            '\n' => {
                self.line += 1;
                None
            }
            '"' => Some(self.build_string()),
            _ => {
                if character.is_ascii_digit() {
                    Some(self.build_number())
                } else if character.is_ascii_alphabetic() || character == '_' {
                    Some(Ok(self.build_identifier()))
                } else {
                    let _ = self.iter.consume_chars(1);
                    Some(Err(InterpreterError {
                        line: self.line,
                        message: "Unexpected character",
                        error_location: None, // TODO fix this, should use value
                    }))
                }
            }
        }
    }

    fn build_sized_token(&mut self, token_type: TokenType, size: usize) -> Token<'a> {
        let lexeme = self.iter.consume_chars(size);
        self.build_token(token_type, lexeme, None)
    }

    fn build_token(
        &self,
        token_type: TokenType,
        lexeme: &'a str,
        literal: Option<LiteralValue<'a>>,
    ) -> Token<'a> {
        Token {
            token_type,
            lexeme,
            literal,
            line: self.line,
        }
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
            self.build_sized_token(one_char_token, 1)
        }
    }

    fn build_string(&mut self) -> Result<Token<'a>, InterpreterError> {
        let (lex_result, lines) = self.iter.consume_string();
        let lexeme = lex_result.ok_or(InterpreterError {
            line: self.line,
            message: "Unterminated string",
            error_location: None, // TODO fix this, should use value
        })?;

        let value = LiteralValue::String(&lexeme[1..lexeme.len() - 1]);
        let token = Ok(self.build_token(TT::STRING, lexeme, Some(value)));
        self.line += lines;
        token
    }

    fn build_number(&mut self) -> Result<Token<'a>, InterpreterError> {
        let lexeme = self.iter.consume_number();
        let value = LiteralValue::Number(
            lexeme
                .parse::<f64>()
                .expect("Tokenizer guarantees valid float syntax"),
        );
        Ok(self.build_token(TT::NUMBER, lexeme, Some(value)))
    }

    fn build_identifier(&mut self) -> Token<'a> {
        let lexeme = self.iter.consume_identifier();
        let token_type = get_keyword(lexeme).unwrap_or(TT::IDENTIFIER);
        self.build_token(token_type, lexeme, None)
    }
}

impl<'a> Tokenizer<'a> {
    fn consume_string(&mut self) -> (Option<&'a str>, usize) {
        let mut lines = 0;
        let mut total_bytes = 0;
        let mut result = self.peek_while(|c| !['\n', '"'].contains(&c));

        while result.character == Some('\n') {
            lines += 1;
            total_bytes += result.bytes + '\n'.len_utf8();
            result = self.peek_while_from(|c| !['\n', '"'].contains(&c), total_bytes);
        }

        total_bytes += result.bytes;
        let output = result.character.map(|_| self.consume(total_bytes));
        (output, lines)
    }

    fn consume_number(&mut self) -> &'a str {
        let integer_part = self.peek_while(|c| c.is_ascii_digit());
        let mut width = integer_part.bytes;

        if integer_part.character == Some('.') {
            let after_dot = self.peek_while_from(|c| c.is_ascii_digit(), width + '.'.len_utf8());
            // Don't consume the '.' there are numbers after it
            if after_dot.bytes > 0 {
                width += '.'.len_utf8() + after_dot.bytes;
            }
        }

        self.consume(width)
    }

    fn consume_identifier(&mut self) -> &'a str {
        self.consume_while(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}
