use crate::evaluator::environment::Scope;
use crate::token_type::{OperatorSubset, operator_subset};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum Statement<'a> {
    Declaration {
        name: &'a str,
        expression: Option<Expr<'a>>,
    },
    FunctionDeclaration(Function<'a>),
    Expression(Expr<'a>),
    Print(Expr<'a>),
    Group(Vec<Self>),
    If {
        condition: Expr<'a>,
        true_branch: Box<Self>,
        false_branch: Option<Box<Self>>,
    },
    While {
        condition: Expr<'a>,
        body: Box<Self>,
    },
    Return {
        line: usize,
        value: Option<Expr<'a>>,
    },
    Break,
}

impl<'a> Statement<'a> {
    pub fn r#for(
        initialiser: Option<Self>,
        condition: Option<Expr<'a>>,
        increment: Option<Expr<'a>>,
        body: Box<Self>,
        line: usize,
    ) -> Self {
        let body = match increment {
            Some(inc) => {
                Box::new(Self::Group(vec![*body, Self::Expression(inc)]))
            }
            None => body,
        };
        let flattened_condition = condition
            .unwrap_or_else(|| Expr::literal(Value::Boolean(true), line));

        let while_loop = Self::While {
            condition: flattened_condition,
            body,
        };

        match initialiser {
            None => while_loop,
            Some(init) => Self::Group(vec![init, while_loop]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Expr<'a> {
    pub line: usize,
    pub kind: ExprKind<'a>,
}

#[derive(Clone, Debug)]
pub enum ExprKind<'a> {
    Literal(Value<'a>),
    Identifier(&'a str),
    Unary(Unary<'a>),
    Grouping(Box<Expr<'a>>),
    Binary(Binary<'a>),
    Assignment(Assignment<'a>),
    Logical(Logical<'a>),
    Call(Call<'a>),
    Lambda(Function<'a>),
}

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Function {
        declaration: Function<'a>,
        closure: Scope<'a>,
    },
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

#[derive(Clone, Debug)]
pub struct Assignment<'a> {
    pub name: &'a str,
    pub expr: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct Unary<'a> {
    pub operator: UnaryOperator,
    pub expr: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct Binary<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: BinaryOperator,
    pub right: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct Logical<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: LogicalOperator,
    pub right: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct Call<'a> {
    pub callee: Box<Expr<'a>>,
    pub arguments: Vec<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct Function<'a> {
    pub name: &'a str,
    pub body: FunctionKind<'a>,
    pub params: Vec<&'a str>,
}

#[derive(Clone, Debug)]
pub enum FunctionKind<'a> {
    Lox(Box<Statement<'a>>),
    Rust(fn(Vec<Value<'a>>) -> Value<'a>),
}

operator_subset!(UnaryOperator, {MINUS, BANG});
operator_subset!(BinaryOperator, {
    MINUS,
    PLUS,
    GREATER,
    GREATER_EQUAL,
    BANG_EQUAL,
    EQUAL_EQUAL,
    SLASH,
    STAR,
    COMMA,
    QUESTION_MARK,
    COLON,
    LESS,
    LESS_EQUAL,
});
operator_subset!(LogicalOperator, {OR, AND});

impl<'a> Expr<'a> {
    pub const fn logical(
        left: Box<Self>,
        operator: LogicalOperator,
        right: Box<Self>,
        line: usize,
    ) -> Self {
        Expr {
            line,
            kind: ExprKind::Logical(Logical {
                left,
                operator,
                right,
            }),
        }
    }

    pub const fn binary(
        left: Box<Self>,
        operator: BinaryOperator,
        right: Box<Self>,
        line: usize,
    ) -> Self {
        Expr {
            line,
            kind: ExprKind::Binary(Binary {
                left,
                operator,
                right,
            }),
        }
    }

    pub const fn unary(
        operator: UnaryOperator,
        expr: Box<Self>,
        line: usize,
    ) -> Self {
        Expr {
            line,
            kind: ExprKind::Unary(Unary { operator, expr }),
        }
    }

    pub const fn literal(value: Value<'a>, line: usize) -> Self {
        Expr {
            line,
            kind: ExprKind::Literal(value),
        }
    }

    pub const fn identifier(identifier: &'a str, line: usize) -> Self {
        Expr {
            line,
            kind: ExprKind::Identifier(identifier),
        }
    }

    pub const fn grouping(grouping: Box<Self>, line: usize) -> Self {
        Expr {
            line,
            kind: ExprKind::Grouping(grouping),
        }
    }

    pub const fn assignment(
        name: &'a str,
        expr: Box<Self>,
        line: usize,
    ) -> Self {
        Expr {
            line,
            kind: ExprKind::Assignment(Assignment { name, expr }),
        }
    }

    pub const fn call(callee: Box<Self>, arguments: Vec<Self>) -> Self {
        Expr {
            line: callee.line,
            kind: ExprKind::Call(Call { callee, arguments }),
        }
    }

    pub const fn lambda(function: Function<'a>, line: usize) -> Self {
        Expr {
            line,
            kind: ExprKind::Lambda(function),
        }
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::String(..) => write!(f, "\"{}\"", self.cast_to_string()),
            Self::Function {
                declaration: Function { name, .. },
                ..
            } => write!(f, "<fn {name}>"),
            _ => write!(f, "{}", self.cast_to_string()),
        }
    }
}

impl Value<'_> {
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::String(..) => "String",
            Self::Number(..) => "Number",
            Self::Boolean(..) => "Boolean",
            Self::Nil => "nil",
            Self::Function { .. } => "Function",
        }
    }

    pub fn cast_to_string(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Number(value) => format!("{value}"),
            Self::Boolean(value) => format!("{value}"),
            Self::Nil => "nil".to_owned(),
            Self::Function { .. } => "Function".to_owned(),
        }
    }
}
