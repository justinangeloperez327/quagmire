//! Spanned syntax structures. A parsed module is not yet semantically validated.
use crate::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub names: Vec<Name>,
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub exported: bool,
    pub kind: DeclarationKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationKind {
    Component(Component),
    Function(Function),
    Constant(Binding),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: Name,
    pub parameters: Vec<Parameter>,
    pub members: Vec<Member>,
    pub view: View,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    State(Binding),
    Computed(Binding),
    Constant(Binding),
    Function(Function),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub name: Name,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: Name,
    pub default: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Name,
    pub parameters: Vec<Parameter>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Constant(Binding),
    Let(Binding),
    Assign {
        name: Name,
        value: Expression,
    },
    Call(Expression),
    Return(Option<Expression>),
    If(CodeIf),
    For {
        name: Name,
        iterable: Expression,
        body: Block,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeIf {
    pub branches: Vec<(Expression, Block)>,
    pub otherwise: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
    pub(crate) depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Negate,
    Positive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Or,
    And,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Identifier(Name),
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Group(Box<Expression>),
    Array(Vec<Expression>),
    Object(Vec<Property>),
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Member {
        object: Box<Expression>,
        property: Name,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub key: Name,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub root: Element,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: Name,
    pub attributes: Vec<Attribute>,
    pub children: Vec<ViewNode>,
    pub self_closing: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: Name,
    pub value: AttributeValue,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    Boolean,
    String(String),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewBlock {
    pub nodes: Vec<ViewNode>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewNode {
    pub kind: ViewNodeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewNodeKind {
    Element(Element),
    Text(String),
    Expression(Expression),
    If {
        branches: Vec<(Expression, ViewBlock)>,
        otherwise: Option<ViewBlock>,
    },
    For {
        name: Name,
        iterable: Expression,
        root: Element,
    },
}
