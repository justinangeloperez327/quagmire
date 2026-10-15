//! Grammar parser using explicit lexer modes. It returns no partial AST on error.
use crate::ast::*;
use crate::lexer::{Keyword as K, LexMode as M, Symbol as S, TokenKind as T};
use crate::{Diagnostic, Lexer, SourceId, SourceMap, Span, SpanError, Token};

const MAX_DEPTH: usize = 128;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    sources: &'a SourceMap,
    source: SourceId,
    last_end: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    pub fn new(sources: &'a SourceMap, source: SourceId) -> Result<Self, SpanError> {
        Ok(Self {
            lexer: Lexer::new(sources, source)?,
            sources,
            source,
            last_end: 0,
            depth: 0,
        })
    }

    /// Parse one complete source module, preserving lexer diagnostics unchanged.
    /// Name resolution, purity, and runtime value checks are separate stages.
    pub fn parse(mut self) -> Result<Module, Diagnostic> {
        let mut imports = Vec::new();
        while self.peek(M::Code)?.kind == T::Keyword(K::Import) {
            imports.push(self.import()?);
        }
        let mut declarations = Vec::new();
        while self.peek(M::Code)?.kind != T::Eof {
            let start = self.peek(M::Code)?.span.range().start;
            let exported = self.eat_keyword(M::Code, K::Export)?;
            let token = self.peek(M::Code)?;
            let kind = match token.kind {
                T::Keyword(K::Component) => DeclarationKind::Component(self.component()?),
                T::Keyword(K::Function) => DeclarationKind::Function(self.function()?),
                T::Keyword(K::Const) => DeclarationKind::Constant(self.binding(K::Const)?),
                _ => return Err(self.error(
                    &token,
                    "QG2001",
                    "expected a component, function, or const declaration; imports must come first",
                )),
            };
            declarations.push(Declaration {
                exported,
                kind,
                span: self.span(start),
            });
        }
        self.take(M::Code)?;
        Ok(Module {
            imports,
            declarations,
            span: self.span(0),
        })
    }

    fn span(&self, start: usize) -> Span {
        self.sources
            .span(self.source, start..self.last_end)
            .expect("parser spans join validated token boundaries")
    }

    fn peek(&self, mode: M) -> Result<Token, Diagnostic> {
        self.lexer.peek(mode)
    }

    fn take(&mut self, mode: M) -> Result<Token, Diagnostic> {
        let token = self.lexer.next_token(mode)?;
        self.last_end = token.span.range().end;
        Ok(token)
    }

    fn error(&self, token: &Token, code: &str, message: &str) -> Diagnostic {
        Diagnostic::error(code, message, token.span)
    }

    fn nested<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        if self.depth >= MAX_DEPTH {
            let offset = self.lexer.offset();
            return Err(Diagnostic::error(
                "QG2004",
                "maximum parser nesting depth (128) exceeded",
                self.sources
                    .span(self.source, offset..offset)
                    .expect("valid cursor"),
            ));
        }
        self.depth += 1;
        let result = parse(self);
        self.depth -= 1;
        result
    }

    fn eat_keyword(&mut self, mode: M, keyword: K) -> Result<bool, Diagnostic> {
        if self.peek(mode)?.kind == T::Keyword(keyword) {
            self.take(mode)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn keyword(&mut self, mode: M, keyword: K) -> Result<Token, Diagnostic> {
        let token = self.take(mode)?;
        if token.kind == T::Keyword(keyword) {
            Ok(token)
        } else {
            Err(self.error(
                &token,
                "QG2001",
                &format!("expected '{}'", format!("{keyword:?}").to_ascii_lowercase()),
            ))
        }
    }

    fn eat_symbol(&mut self, mode: M, symbol: S) -> Result<bool, Diagnostic> {
        if self.peek(mode)?.kind == T::Symbol(symbol) {
            self.take(mode)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn symbol(&mut self, mode: M, symbol: S) -> Result<Token, Diagnostic> {
        let token = self.take(mode)?;
        if token.kind == T::Symbol(symbol) {
            Ok(token)
        } else {
            let spelling = match symbol {
                S::LeftParen => "(",
                S::RightParen => ")",
                S::LeftBrace => "{",
                S::RightBrace => "}",
                S::LeftBracket => "[",
                S::RightBracket => "]",
                S::Semicolon => ";",
                S::Colon => ":",
                S::Assign => "=",
                S::Comma => ",",
                S::Dot => ".",
                _ => "symbol",
            };
            Err(self.error(&token, "QG2001", &format!("expected '{spelling}'")))
        }
    }

    fn name(&mut self, mode: M) -> Result<Name, Diagnostic> {
        let token = self.take(mode)?;
        match token.kind {
            T::Identifier(text) if mode == M::Code => Ok(Name {
                text,
                span: token.span,
            }),
            T::Name(text) if mode == M::Tag => Ok(Name {
                text,
                span: token.span,
            }),
            _ => Err(self.error(&token, "QG2001", "expected a name")),
        }
    }

    fn import(&mut self) -> Result<Import, Diagnostic> {
        let start = self.keyword(M::Code, K::Import)?.span.range().start;
        self.symbol(M::Code, S::LeftBrace)?;
        let mut names = vec![self.name(M::Code)?];
        while self.eat_symbol(M::Code, S::Comma)? {
            if self.peek(M::Code)?.kind == T::Symbol(S::RightBrace) {
                break;
            }
            names.push(self.name(M::Code)?);
        }
        self.symbol(M::Code, S::RightBrace)?;
        self.keyword(M::Code, K::From)?;
        let token = self.take(M::Code)?;
        let T::String(path) = token.kind else {
            return Err(self.error(&token, "QG2001", "expected a quoted import path"));
        };
        self.symbol(M::Code, S::Semicolon)?;
        Ok(Import {
            names,
            path,
            span: self.span(start),
        })
    }

    fn parameters(&mut self) -> Result<Vec<Parameter>, Diagnostic> {
        self.symbol(M::Code, S::LeftParen)?;
        let mut parameters = Vec::new();
        let mut optional = false;
        while self.peek(M::Code)?.kind != T::Symbol(S::RightParen) {
            let name = self.name(M::Code)?;
            let start = name.span.range().start;
            let default = if self.eat_symbol(M::Code, S::Assign)? {
                optional = true;
                Some(self.expr(1)?)
            } else {
                if optional {
                    return Err(Diagnostic::error(
                        "QG2002",
                        "required parameters must precede parameters with defaults",
                        name.span,
                    ));
                }
                None
            };
            parameters.push(Parameter {
                name,
                default,
                span: self.span(start),
            });
            if !self.eat_symbol(M::Code, S::Comma)? {
                break;
            }
        }
        self.symbol(M::Code, S::RightParen)?;
        Ok(parameters)
    }

    fn component(&mut self) -> Result<Component, Diagnostic> {
        let start = self.keyword(M::Code, K::Component)?.span.range().start;
        let name = self.name(M::Code)?;
        if !name.text.starts_with(|c: char| c.is_ascii_uppercase()) {
            return Err(Diagnostic::error(
                "QG2002",
                "component names must start with an uppercase letter",
                name.span,
            ));
        }
        let parameters = self.parameters()?;
        self.symbol(M::Code, S::LeftBrace)?;
        let mut members = Vec::new();
        loop {
            let token = self.peek(M::Code)?;
            let member = match token.kind {
                T::Keyword(K::State) => Member::State(self.binding(K::State)?),
                T::Keyword(K::Computed) => Member::Computed(self.binding(K::Computed)?),
                T::Keyword(K::Const) => Member::Constant(self.binding(K::Const)?),
                T::Keyword(K::Function) => Member::Function(self.function()?),
                T::Keyword(K::View) => break,
                _ => {
                    return Err(self.error(
                        &token,
                        "QG2002",
                        "expected a component member or final view block",
                    ));
                }
            };
            members.push(member);
        }
        let view_start = self.keyword(M::Code, K::View)?.span.range().start;
        self.symbol(M::Code, S::LeftBrace)?;
        self.skip_view_space()?;
        let root = self.element()?;
        self.skip_view_space()?;
        self.symbol(M::Text, S::RightBrace)?;
        let view = View {
            root,
            span: self.span(view_start),
        };
        self.symbol(M::Code, S::RightBrace)?;
        Ok(Component {
            name,
            parameters,
            members,
            view,
            span: self.span(start),
        })
    }

    fn binding(&mut self, keyword: K) -> Result<Binding, Diagnostic> {
        let start = self.keyword(M::Code, keyword)?.span.range().start;
        let name = self.name(M::Code)?;
        self.symbol(M::Code, S::Assign)?;
        let value = self.expr(1)?;
        self.symbol(M::Code, S::Semicolon)?;
        Ok(Binding {
            name,
            value,
            span: self.span(start),
        })
    }

    fn function(&mut self) -> Result<Function, Diagnostic> {
        let start = self.keyword(M::Code, K::Function)?.span.range().start;
        let name = self.name(M::Code)?;
        let parameters = self.parameters()?;
        let body = self.block()?;
        Ok(Function {
            name,
            parameters,
            body,
            span: self.span(start),
        })
    }

    fn block(&mut self) -> Result<Block, Diagnostic> {
        self.nested(Self::block_inner)
    }

    fn block_inner(&mut self) -> Result<Block, Diagnostic> {
        let start = self.symbol(M::Code, S::LeftBrace)?.span.range().start;
        let mut statements = Vec::new();
        while self.peek(M::Code)?.kind != T::Symbol(S::RightBrace) {
            statements.push(self.statement()?);
        }
        self.symbol(M::Code, S::RightBrace)?;
        Ok(Block {
            statements,
            span: self.span(start),
        })
    }

    fn condition(&mut self) -> Result<Expression, Diagnostic> {
        self.symbol(M::Code, S::LeftParen)?;
        let condition = self.expr(1)?;
        self.symbol(M::Code, S::RightParen)?;
        Ok(condition)
    }

    fn loop_header(&mut self) -> Result<(Name, Expression), Diagnostic> {
        self.symbol(M::Code, S::LeftParen)?;
        let name = self.name(M::Code)?;
        self.keyword(M::Code, K::In)?;
        let iterable = self.expr(1)?;
        self.symbol(M::Code, S::RightParen)?;
        Ok((name, iterable))
    }

    fn statement(&mut self) -> Result<Statement, Diagnostic> {
        let token = self.peek(M::Code)?;
        let start = token.span.range().start;
        let kind = match token.kind {
            T::Keyword(K::Const) => StatementKind::Constant(self.binding(K::Const)?),
            T::Keyword(K::Let) => StatementKind::Let(self.binding(K::Let)?),
            T::Keyword(K::Return) => {
                self.take(M::Code)?;
                let value = if self.peek(M::Code)?.kind == T::Symbol(S::Semicolon) {
                    None
                } else {
                    Some(self.expr(1)?)
                };
                self.symbol(M::Code, S::Semicolon)?;
                StatementKind::Return(value)
            }
            T::Keyword(K::If) => {
                let mut branches = Vec::new();
                let mut otherwise = None;
                loop {
                    self.keyword(M::Code, K::If)?;
                    let condition = self.condition()?;
                    let body = self.block()?;
                    branches.push((condition, body));
                    if !self.eat_keyword(M::Code, K::Else)? {
                        break;
                    }
                    if self.peek(M::Code)?.kind != T::Keyword(K::If) {
                        otherwise = Some(self.block()?);
                        break;
                    }
                }
                StatementKind::If(CodeIf {
                    branches,
                    otherwise,
                })
            }
            T::Keyword(K::For) => {
                self.take(M::Code)?;
                let (name, iterable) = self.loop_header()?;
                let body = self.block()?;
                StatementKind::For {
                    name,
                    iterable,
                    body,
                }
            }
            _ => {
                let expression = self.expr(1)?;
                if self.eat_symbol(M::Code, S::Assign)? {
                    let ExpressionKind::Identifier(name) = expression.kind else {
                        return Err(Diagnostic::error(
                            "QG2002",
                            "assignment target must be a simple identifier",
                            expression.span,
                        ));
                    };
                    let value = self.expr(1)?;
                    self.symbol(M::Code, S::Semicolon)?;
                    StatementKind::Assign { name, value }
                } else {
                    let mut inner = &expression;
                    while let ExpressionKind::Group(value) = &inner.kind {
                        inner = value;
                    }
                    if !matches!(inner.kind, ExpressionKind::Call { .. }) {
                        return Err(Diagnostic::error(
                            "QG2002",
                            "only assignments and calls may be expression statements",
                            expression.span,
                        ));
                    }
                    self.symbol(M::Code, S::Semicolon)?;
                    StatementKind::Call(expression)
                }
            }
        };
        Ok(Statement {
            kind,
            span: self.span(start),
        })
    }

    fn expr(&mut self, minimum: u8) -> Result<Expression, Diagnostic> {
        self.nested(|parser| parser.expr_inner(minimum))
    }

    fn build_expr(&self, start: usize, kind: ExpressionKind) -> Result<Expression, Diagnostic> {
        let child_depth = match &kind {
            ExpressionKind::Group(value) | ExpressionKind::Unary { operand: value, .. } => {
                value.depth
            }
            ExpressionKind::Array(values) => values.iter().map(|v| v.depth).max().unwrap_or(0),
            ExpressionKind::Object(values) => {
                values.iter().map(|v| v.value.depth).max().unwrap_or(0)
            }
            ExpressionKind::Binary { left, right, .. } => left.depth.max(right.depth),
            ExpressionKind::Call { callee, arguments } => callee
                .depth
                .max(arguments.iter().map(|v| v.depth).max().unwrap_or(0)),
            ExpressionKind::Member { object, .. } => object.depth,
            ExpressionKind::Index { object, index } => object.depth.max(index.depth),
            _ => 0,
        };
        let span = self.span(start);
        if child_depth >= MAX_DEPTH {
            return Err(Diagnostic::error(
                "QG2004",
                "maximum expression tree depth (128) exceeded",
                span,
            ));
        }
        Ok(Expression {
            kind,
            span,
            depth: child_depth + 1,
        })
    }

    fn expr_inner(&mut self, minimum: u8) -> Result<Expression, Diagnostic> {
        let token = self.take(M::Code)?;
        let start = token.span.range().start;
        let kind = match token.kind {
            T::Identifier(text) => ExpressionKind::Identifier(Name {
                text,
                span: token.span,
            }),
            T::Number(value) => ExpressionKind::Number(value),
            T::String(value) => ExpressionKind::String(value),
            T::Keyword(K::True) => ExpressionKind::Boolean(true),
            T::Keyword(K::False) => ExpressionKind::Boolean(false),
            T::Keyword(K::Null) => ExpressionKind::Null,
            T::Symbol(S::Not | S::Minus | S::Plus) => {
                let operator = match token.kind {
                    T::Symbol(S::Not) => UnaryOperator::Not,
                    T::Symbol(S::Minus) => UnaryOperator::Negate,
                    _ => UnaryOperator::Positive,
                };
                ExpressionKind::Unary {
                    operator,
                    operand: Box::new(self.expr(7)?),
                }
            }
            T::Symbol(S::LeftParen) => {
                let value = self.expr(1)?;
                self.symbol(M::Code, S::RightParen)?;
                ExpressionKind::Group(Box::new(value))
            }
            T::Symbol(S::LeftBracket) => ExpressionKind::Array(self.arguments(S::RightBracket)?),
            T::Symbol(S::LeftBrace) => {
                let mut properties = Vec::new();
                while self.peek(M::Code)?.kind != T::Symbol(S::RightBrace) {
                    let key_token = self.take(M::Code)?;
                    let key = match key_token.kind {
                        T::Identifier(text) | T::String(text) => Name {
                            text,
                            span: key_token.span,
                        },
                        _ => {
                            return Err(self.error(
                                &key_token,
                                "QG2001",
                                "expected an object key followed by ':'",
                            ));
                        }
                    };
                    let property_start = key.span.range().start;
                    self.symbol(M::Code, S::Colon)?;
                    let value = self.expr(1)?;
                    properties.push(Property {
                        key,
                        value,
                        span: self.span(property_start),
                    });
                    if !self.eat_symbol(M::Code, S::Comma)? {
                        break;
                    }
                }
                self.symbol(M::Code, S::RightBrace)?;
                ExpressionKind::Object(properties)
            }
            _ => return Err(self.error(&token, "QG2001", "expected an expression")),
        };
        let mut left = self.build_expr(start, kind)?;
        loop {
            let token = self.peek(M::Code)?;
            let kind = match token.kind {
                T::Symbol(S::Dot) => {
                    self.take(M::Code)?;
                    let property = self.name(M::Code)?;
                    ExpressionKind::Member {
                        object: Box::new(left),
                        property,
                    }
                }
                T::Symbol(S::LeftBracket) => {
                    self.take(M::Code)?;
                    let index = self.expr(1)?;
                    self.symbol(M::Code, S::RightBracket)?;
                    ExpressionKind::Index {
                        object: Box::new(left),
                        index: Box::new(index),
                    }
                }
                T::Symbol(S::LeftParen) => {
                    self.take(M::Code)?;
                    let arguments = self.arguments(S::RightParen)?;
                    ExpressionKind::Call {
                        callee: Box::new(left),
                        arguments,
                    }
                }
                _ => break,
            };
            left = self.build_expr(start, kind)?;
        }
        loop {
            let token = self.peek(M::Code)?;
            let Some((operator, precedence)) = binary(&token.kind) else {
                break;
            };
            if precedence < minimum {
                break;
            }
            if precedence == 4
                && matches!(&left.kind, ExpressionKind::Binary { operator, .. } if comparison(*operator))
            {
                return Err(self.error(
                    &token,
                    "QG2002",
                    "comparison chaining is invalid; combine comparisons with '&&'",
                ));
            }
            self.take(M::Code)?;
            let right = self.expr(precedence + 1)?;
            left = self.build_expr(
                start,
                ExpressionKind::Binary {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            )?;
        }
        Ok(left)
    }

    fn arguments(&mut self, end: S) -> Result<Vec<Expression>, Diagnostic> {
        let mut arguments = Vec::new();
        while self.peek(M::Code)?.kind != T::Symbol(end) {
            arguments.push(self.expr(1)?);
            if !self.eat_symbol(M::Code, S::Comma)? {
                break;
            }
        }
        self.symbol(M::Code, end)?;
        Ok(arguments)
    }

    fn skip_view_space(&mut self) -> Result<(), Diagnostic> {
        while matches!(self.peek(M::Text)?.kind, T::Text(ref text) if text.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')))
        {
            self.take(M::Text)?;
        }
        Ok(())
    }

    fn element(&mut self) -> Result<Element, Diagnostic> {
        self.nested(Self::element_inner)
    }

    fn element_inner(&mut self) -> Result<Element, Diagnostic> {
        let opening = self.take(M::Text)?;
        let start = opening.span.range().start;
        if opening.kind != T::OpenTag {
            return Err(self.error(&opening, "QG2003", "expected an element beginning with '<'"));
        }
        let name = self.name(M::Tag)?;
        let component = name.text.starts_with(|c: char| c.is_ascii_uppercase());
        let valid_name = if component {
            name.text
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        } else {
            name.text.starts_with(|c: char| c.is_ascii_lowercase())
                && name
                    .text
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        };
        if !valid_name {
            return Err(Diagnostic::error(
                "QG2003",
                "invalid element name",
                name.span,
            ));
        }
        let mut attributes: Vec<Attribute> = Vec::new();
        while !matches!(self.peek(M::Tag)?.kind, T::TagEnd | T::SelfCloseTagEnd) {
            let attribute_name = self.name(M::Tag)?;
            if attributes
                .iter()
                .any(|attribute| attribute.name.text == attribute_name.text)
            {
                return Err(Diagnostic::error(
                    "QG2003",
                    "duplicate attribute",
                    attribute_name.span,
                ));
            }
            let attribute_start = attribute_name.span.range().start;
            let value = if self.eat_symbol(M::Tag, S::Assign)? {
                let token = self.take(M::Tag)?;
                match token.kind {
                    T::String(value) => AttributeValue::String(value),
                    T::Symbol(S::LeftBrace) => {
                        let value = self.expr(1)?;
                        self.symbol(M::Code, S::RightBrace)?;
                        AttributeValue::Expression(value)
                    }
                    _ => {
                        return Err(self.error(
                            &token,
                            "QG2003",
                            "expected a quoted attribute value or '{expression}'",
                        ));
                    }
                }
            } else {
                AttributeValue::Boolean
            };
            attributes.push(Attribute {
                name: attribute_name,
                value,
                span: self.span(attribute_start),
            });
        }
        let ending = self.take(M::Tag)?;
        let self_closing = ending.kind == T::SelfCloseTagEnd;
        let mut children = Vec::new();
        if !self_closing {
            if component
                || matches!(
                    name.text.as_str(),
                    "area"
                        | "base"
                        | "br"
                        | "col"
                        | "embed"
                        | "hr"
                        | "img"
                        | "input"
                        | "link"
                        | "meta"
                        | "param"
                        | "source"
                        | "track"
                        | "wbr"
                )
            {
                return Err(Diagnostic::error(
                    "QG2003",
                    "component and void elements must use '/>'",
                    name.span,
                ));
            }
            while self.peek(M::Text)?.kind != T::CloseTag {
                children.push(self.view_node()?);
            }
            self.take(M::Text)?;
            let closing = self.name(M::Tag)?;
            if closing.text != name.text {
                return Err(Diagnostic::error(
                    "QG2003",
                    format!("expected closing tag '</{}>'", name.text),
                    closing.span,
                ));
            }
            let end = self.take(M::Tag)?;
            if end.kind != T::TagEnd {
                return Err(self.error(&end, "QG2003", "expected '>' after closing tag"));
            }
        }
        Ok(Element {
            name,
            attributes,
            children,
            self_closing,
            span: self.span(start),
        })
    }

    fn view_block(&mut self) -> Result<ViewBlock, Diagnostic> {
        self.nested(|parser| {
            let start = parser.symbol(M::Code, S::LeftBrace)?.span.range().start;
            let mut nodes = Vec::new();
            while parser.peek(M::Text)?.kind != T::Symbol(S::RightBrace) {
                nodes.push(parser.view_node()?);
            }
            parser.take(M::Text)?;
            Ok(ViewBlock {
                nodes,
                span: parser.span(start),
            })
        })
    }

    fn has_view_else(&self) -> Result<bool, Diagnostic> {
        let mut lexer = self.lexer.clone();
        loop {
            let token = lexer.next_token(M::Text)?;
            match token.kind {
                T::Text(text) if text.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) => {}
                T::Keyword(K::Else) => return Ok(true),
                _ => return Ok(false),
            }
        }
    }

    fn view_node(&mut self) -> Result<ViewNode, Diagnostic> {
        self.nested(Self::view_node_inner)
    }

    fn view_node_inner(&mut self) -> Result<ViewNode, Diagnostic> {
        let token = self.peek(M::Text)?;
        let start = token.span.range().start;
        let kind = match token.kind {
            T::OpenTag => ViewNodeKind::Element(self.element()?),
            T::Text(_) => {
                let T::Text(value) = self.take(M::Text)?.kind else {
                    unreachable!()
                };
                ViewNodeKind::Text(value)
            }
            T::Symbol(S::LeftBrace) => {
                self.take(M::Text)?;
                let value = self.expr(1)?;
                self.symbol(M::Code, S::RightBrace)?;
                ViewNodeKind::Expression(value)
            }
            T::Keyword(K::If) => {
                self.take(M::Text)?;
                let mut branches = Vec::new();
                let mut otherwise = None;
                loop {
                    let condition = self.condition()?;
                    let body = self.view_block()?;
                    branches.push((condition, body));
                    if !self.has_view_else()? {
                        break;
                    }
                    self.skip_view_space()?;
                    self.keyword(M::Text, K::Else)?;
                    if !self.eat_keyword(M::Code, K::If)? {
                        otherwise = Some(self.view_block()?);
                        break;
                    }
                }
                ViewNodeKind::If {
                    branches,
                    otherwise,
                }
            }
            T::Keyword(K::For) => {
                self.take(M::Text)?;
                let (name, iterable) = self.loop_header()?;
                self.symbol(M::Code, S::LeftBrace)?;
                self.skip_view_space()?;
                let root = self.element()?;
                self.skip_view_space()?;
                self.symbol(M::Text, S::RightBrace)?;
                if !root.attributes.iter().any(|attribute| {
                    attribute.name.text == "key"
                        && matches!(attribute.value, AttributeValue::Expression(_))
                }) {
                    return Err(Diagnostic::error(
                        "QG2003",
                        "view loop root requires 'key={expression}'",
                        root.span,
                    ));
                }
                ViewNodeKind::For {
                    name,
                    iterable,
                    root,
                }
            }
            _ => {
                return Err(self.error(
                    &token,
                    "QG2003",
                    "expected view text, an element, interpolation, or a control directive",
                ));
            }
        };
        Ok(ViewNode {
            kind,
            span: self.span(start),
        })
    }
}

fn binary(kind: &T) -> Option<(BinaryOperator, u8)> {
    Some(match kind {
        T::Symbol(S::Or) => (BinaryOperator::Or, 1),
        T::Symbol(S::And) => (BinaryOperator::And, 2),
        T::Symbol(S::Equal) => (BinaryOperator::Equal, 3),
        T::Symbol(S::NotEqual) => (BinaryOperator::NotEqual, 3),
        T::Symbol(S::Less) => (BinaryOperator::Less, 4),
        T::Symbol(S::LessEqual) => (BinaryOperator::LessEqual, 4),
        T::Symbol(S::Greater) => (BinaryOperator::Greater, 4),
        T::Symbol(S::GreaterEqual) => (BinaryOperator::GreaterEqual, 4),
        T::Symbol(S::Plus) => (BinaryOperator::Add, 5),
        T::Symbol(S::Minus) => (BinaryOperator::Subtract, 5),
        T::Symbol(S::Star) => (BinaryOperator::Multiply, 6),
        T::Symbol(S::Slash) => (BinaryOperator::Divide, 6),
        T::Symbol(S::Percent) => (BinaryOperator::Remainder, 6),
        _ => return None,
    })
}

fn comparison(operator: BinaryOperator) -> bool {
    matches!(
        operator,
        BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
    )
}
