use crate::token_type::TokenType;
use crate::token_type::TokenType::{
    AND, CLASS, ELSE, FALSE, FOR, FUN, IF, NIL, OR, PRINT, RETURN, SUPER, THIS,
    TRUE, VAR, WHILE,
};

pub fn get_keyword(keyword: &str) -> Option<TokenType> {
    match keyword {
        "and" => Some(AND),
        "class" => Some(CLASS),
        "else" => Some(ELSE),
        "false" => Some(FALSE),
        "for" => Some(FOR),
        "fun" => Some(FUN),
        "if" => Some(IF),
        "nil" => Some(NIL),
        "or" => Some(OR),
        "print" => Some(PRINT),
        "return" => Some(RETURN),
        "super" => Some(SUPER),
        "this" => Some(THIS),
        "true" => Some(TRUE),
        "var" => Some(VAR),
        "while" => Some(WHILE),
        _ => None,
    }
}
