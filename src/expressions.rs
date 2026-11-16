use crate::evaluator::environment::{Frame, VariableBinding};
use crate::operator_subset;
use crate::token::{Span, Token};
use crate::token_type::KeywordToken as KT;
use crate::token_type::LiteralToken as LT;
use crate::token_type::TokenType as TT;
use crate::token_type::TokenTypeSubset;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Statement<'a> {
    Declaration {
        binding: VariableBinding<'a>,
        expression: Option<Expr<'a>>,
    },
    FunctionDeclaration(FunctionType<'a>),
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
        span: Span,
        value: Option<Expr<'a>>,
    },
    ClassDeclaration(Class<'a>),
    Break,
}

impl<'a> Statement<'a> {
    #[must_use]
    pub fn r#for(
        initialiser: Option<Self>,
        condition: Option<Expr<'a>>,
        increment: Option<Expr<'a>>,
        body: Box<Self>,
        span: Span,
    ) -> Self {
        let body = match increment {
            Some(inc) => {
                Box::new(Self::Group(vec![*body, Self::Expression(inc)]))
            }
            None => body,
        };
        let flattened_condition = condition
            .unwrap_or_else(|| Expr::literal(Value::Boolean(true), span));

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
    pub span: Span,
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
    Get(Get<'a>),
    Set(Set<'a>),
    Lambda(FunctionType<'a>),
    This(VariableBinding<'a>),
}

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Function(FunctionValue<'a>),
    Class(Class<'a>),
    Instance(Rc<RefCell<Instance<'a>>>),
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

#[derive(Clone, Debug)]
pub enum ValueError {
    NotANumber(Span),
    NotAValue(Span),
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
pub struct Get<'a> {
    pub object: Box<Expr<'a>>,
    pub target: VariableBinding<'a>,
}

#[derive(Clone, Debug)]
pub struct Set<'a> {
    pub object: Box<Expr<'a>>,
    pub target: VariableBinding<'a>,
    pub value: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct FunctionType<'a> {
    pub binding: VariableBinding<'a>,
    pub body: FunctionKind<'a>,
    pub params: Vec<VariableBinding<'a>>,
}

#[derive(Clone, Debug)]
pub enum FunctionKind<'a> {
    Lox(Box<Statement<'a>>),
    Rust(fn(Vec<Value<'a>>) -> Value<'a>),
    Constructor(Class<'a>),
    CustomConstructor(Box<Statement<'a>>),
}

#[derive(Clone)]
pub struct FunctionValue<'a> {
    pub function: FunctionType<'a>,
    pub closure: Frame<'a>,
}

impl<'a> FunctionValue<'a> {
    #[must_use]
    pub fn bind(&self, instance: Rc<RefCell<Instance<'a>>>) -> Self {
        let closure = self.closure.clone();
        closure.insert("this", Some(Value::Instance(instance)));
        Self {
            function: self.function.clone(),
            closure,
        }
    }
}

impl fmt::Debug for FunctionValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionValue")
            .field("function", &self.function)
            .field("closure", &"{closure}")
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct Class<'a> {
    pub superclass: Option<VariableBinding<'a>>,
    pub binding: VariableBinding<'a>,
    pub methods: Vec<FunctionType<'a>>,
}

impl<'a> Class<'a> {
    #[must_use]
    pub fn constructor(&self) -> FunctionValue<'a> {
        FunctionValue {
            function: FunctionType {
                binding: VariableBinding {
                    span: (0, 0),
                    name: self.binding.name,
                },
                body: FunctionKind::Constructor(self.clone()),
                params: self
                    .fields()
                    .get("init")
                    .map_or_else(Vec::new, |init| init.params.clone()),
            },
            closure: Frame::new(),
        }
    }

    #[must_use]
    pub fn fields(&self) -> HashMap<&'a str, &FunctionType<'a>> {
        self.methods
            .iter()
            .map(|f| (f.binding.name, f))
            .collect::<HashMap<_, _>>()
    }

    #[must_use]
    pub const fn this_binding(&self) -> VariableBinding<'a> {
        VariableBinding {
            span: self.binding.span,
            name: "this",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Instance<'a> {
    pub class: Class<'a>,
    pub fields: HashMap<&'a str, Value<'a>>,
}

impl<'a> Instance<'a> {
    #[must_use]
    pub fn new(class: Class<'a>, superclass: &Option<Class<'a>>) -> Self {
        let fields = superclass
            .iter()
            .flat_map(|superclass| superclass.fields())
            .chain(class.fields())
            .map(|(name, field)| {
                if name == "init"
                    && let FunctionKind::Lox(body) = &field.body
                {
                    (
                        name,
                        Value::from(FunctionType {
                            binding: field.binding.clone(),
                            body: FunctionKind::CustomConstructor(body.clone()),
                            params: field.params.clone(),
                        }),
                    )
                } else {
                    (name, Value::from(field.clone()))
                }
            })
            .collect::<HashMap<_, _>>();

        Self { class, fields }
    }

    #[must_use]
    pub fn get(&self, target: &'a str) -> Option<&Value<'a>> {
        self.fields.get(target)
    }

    pub fn set(&mut self, target: &'a str, value: Value<'a>) {
        self.fields.insert(target, value);
    }
}

operator_subset!(UnaryOperator, {Minus, Bang});
operator_subset!(BinaryOperator, {
    Minus,
    Plus,
    Greater,
    GreaterEqual,
    BangEqual,
    EqualEqual,
    Slash,
    Star,
    Comma,
    QuestionMark,
    Colon,
    Less,
    LessEqual,
});
operator_subset!(LogicalOperator, {Or, And});

impl<'a> Expr<'a> {
    #[must_use]
    pub const fn logical(
        left: Box<Self>,
        operator: LogicalOperator,
        right: Box<Self>,
        span: Span,
    ) -> Self {
        Expr {
            span,
            kind: ExprKind::Logical(Logical {
                left,
                operator,
                right,
            }),
        }
    }

    #[must_use]
    pub const fn binary(
        left: Box<Self>,
        operator: BinaryOperator,
        right: Box<Self>,
        span: Span,
    ) -> Self {
        Expr {
            span,
            kind: ExprKind::Binary(Binary {
                left,
                operator,
                right,
            }),
        }
    }

    #[must_use]
    pub const fn unary(
        operator: UnaryOperator,
        expr: Box<Self>,
        span: Span,
    ) -> Self {
        Expr {
            span,
            kind: ExprKind::Unary(Unary { operator, expr }),
        }
    }

    #[must_use]
    pub const fn literal(value: Value<'a>, span: Span) -> Self {
        Expr {
            span,
            kind: ExprKind::Literal(value),
        }
    }

    #[must_use]
    pub const fn identifier(identifier: &'a str, span: Span) -> Self {
        Expr {
            span,
            kind: ExprKind::Identifier(identifier),
        }
    }

    #[must_use]
    pub const fn grouping(grouping: Box<Self>, span: Span) -> Self {
        Expr {
            span,
            kind: ExprKind::Grouping(grouping),
        }
    }

    #[must_use]
    pub const fn assignment(
        name: &'a str,
        expr: Box<Self>,
        span: Span,
    ) -> Self {
        Expr {
            span,
            kind: ExprKind::Assignment(Assignment { name, expr }),
        }
    }

    #[must_use]
    pub const fn call(callee: Box<Self>, arguments: Vec<Self>) -> Self {
        Expr {
            span: callee.span,
            kind: ExprKind::Call(Call { callee, arguments }),
        }
    }

    #[must_use]
    pub const fn lambda(function: FunctionType<'a>) -> Self {
        Expr {
            span: function.binding.span,
            kind: ExprKind::Lambda(function),
        }
    }

    #[must_use]
    pub const fn get(object: Box<Self>, binding: VariableBinding<'a>) -> Self {
        Expr {
            span: object.span,
            kind: ExprKind::Get(Get {
                object,
                target: binding,
            }),
        }
    }

    #[must_use]
    pub const fn set(
        object: Box<Self>,
        target: VariableBinding<'a>,
        value: Box<Self>,
        span: Span,
    ) -> Self {
        Expr {
            span,
            kind: ExprKind::Set(Set {
                object,
                target,
                value,
            }),
        }
    }

    #[must_use]
    pub const fn this(span: Span) -> Self {
        Expr {
            span,
            kind: ExprKind::This(VariableBinding { span, name: "this" }),
        }
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::String(value) => {
                write!(f, "{value}")
            }
            Self::Number(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Nil => write!(f, "nil"),
            Self::Function(FunctionValue {
                function: FunctionType { binding, .. },
                ..
            }) => write!(f, "Function<{}>", binding.name),
            Self::Class(Class { binding, .. }) => {
                write!(f, "Class<{}>", binding.name)
            }
            Self::Instance(instance) => {
                write!(
                    f,
                    "Instance<of {}>",
                    instance.borrow().class.binding.name
                )
            }
        }
    }
}

impl<'a> Value<'a> {
    #[must_use]
    pub const fn token_types() -> &'static [TT] {
        &[
            TT::Literal(LT::String),
            TT::Keyword(KT::True),
            TT::Keyword(KT::False),
            TT::Literal(LT::Number),
            TT::Keyword(KT::Nil),
        ]
    }

    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::String(..) => "String",
            Self::Number(..) => "Number",
            Self::Boolean(..) => "Boolean",
            Self::Nil => "nil",
            Self::Function { .. } => "Function",
            Self::Class { .. } => "Class",
            Self::Instance(..) => "Instance",
        }
    }

    #[must_use]
    pub fn cast_to_string(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Number(value) => format!("{value}"),
            Self::Boolean(value) => format!("{value}"),
            Self::Nil => "nil".to_owned(),
            Self::Function(..) => "Function".to_owned(),
            Self::Class(Class { binding, .. }) => binding.name.to_owned(),
            Self::Instance(instance) => {
                format!("{} instance", instance.borrow().class.binding.name)
            }
        }
    }

    /// # Errors
    ///
    /// Will error if the number is invalid
    pub fn try_from(
        token: &Token,
        source: &'a str,
    ) -> Result<Self, ValueError> {
        let sub_string = &source[token.span.0..token.span.1];
        match token.token_kind {
            TT::Literal(LT::String) => {
                Ok(Self::String(sub_string[1..sub_string.len() - 1].to_owned()))
            }
            TT::Keyword(KT::True) => Ok(Self::Boolean(true)),
            TT::Keyword(KT::False) => Ok(Self::Boolean(false)),
            TT::Literal(LT::Number) => sub_string
                .parse::<f64>()
                .map_or(Err(ValueError::NotANumber(token.span)), |v| {
                    Ok(Self::Number(v))
                }),
            TT::Keyword(KT::Nil) => Ok(Self::Nil),
            _ => Err(ValueError::NotAValue(token.span)),
        }
    }
}

impl<'a> From<FunctionType<'a>> for Value<'a> {
    fn from(value: FunctionType<'a>) -> Self {
        Value::Function(FunctionValue {
            function: value,
            closure: Frame::new(),
        })
    }
}
