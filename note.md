Lexer should return TokenKind<'a>(???? Beter type name(this is just stolen)) (Has data)
TokenType should not store any data and just be a list of symbols
TokenKind<'a> should be 1 to 1 with TokenType, and compariable (PartialEq?????)
Expr's should store a Union of TokenKind (How to do this????)
Can I lose the boxes??????
