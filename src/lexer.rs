//! Context-driven tokenization for Quagmire code, tag attributes, and view text.
//! The parser selects the mode; this lexer does not guess grammar structure.

use crate::{Diagnostic, SourceId, SourceMap, Span, SpanError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexMode {
    Code,
    Tag,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Component,
    State,
    Computed,
    Function,
    View,
    If,
    Else,
    For,
    In,
    Return,
    Import,
    Export,
    From,
    Let,
    Const,
    True,
    False,
    Null,
}

impl Keyword {
    fn from_word(word: &str) -> Option<Self> {
        Some(match word {
            "component" => Self::Component,
            "state" => Self::State,
            "computed" => Self::Computed,
            "function" => Self::Function,
            "view" => Self::View,
            "if" => Self::If,
            "else" => Self::Else,
            "for" => Self::For,
            "in" => Self::In,
            "return" => Self::Return,
            "import" => Self::Import,
            "export" => Self::Export,
            "from" => Self::From,
            "let" => Self::Let,
            "const" => Self::Const,
            "true" => Self::True,
            "false" => Self::False,
            "null" => Self::Null,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
    Dot,
    Colon,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Not,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(String),
    /// A tag or attribute name, including hyphens and one optional namespace.
    Name(String),
    Number(f64),
    String(String),
    /// Decoded view text. Entity decoding never creates markup tokens.
    Text(String),
    Symbol(Symbol),
    OpenTag,
    CloseTag,
    TagEnd,
    SelfCloseTagEnd,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    sources: &'a SourceMap,
    source: SourceId,
    text: &'a str,
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(sources: &'a SourceMap, source: SourceId) -> Result<Self, SpanError> {
        let text = sources.get(source).ok_or(SpanError::UnknownSource)?.text();
        Ok(Self {
            sources,
            source,
            text,
            offset: 0,
        })
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Look ahead in a chosen mode without consuming source or changing state.
    pub fn peek(&self, mode: LexMode) -> Result<Token, Diagnostic> {
        self.clone().next_token(mode)
    }

    /// Consume one token. Code skips comments/whitespace; tag mode skips
    /// whitespace only. Text mode preserves inline spaces and literal comments.
    /// On a lexical error, stop parsing this input; no executable output is valid.
    pub fn next_token(&mut self, mode: LexMode) -> Result<Token, Diagnostic> {
        match mode {
            LexMode::Code => self.code(),
            LexMode::Tag => self.tag(),
            LexMode::Text => self.view_text(),
        }
    }

    fn current(&self) -> Option<char> {
        self.text[self.offset..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let character = self.current()?;
        self.offset += character.len_utf8();
        Some(character)
    }

    fn starts(&self, value: &str) -> bool {
        self.text[self.offset..].starts_with(value)
    }

    fn token(&self, start: usize, kind: TokenKind) -> Token {
        Token {
            kind,
            // Cursor movements consume whole UTF-8 characters or ASCII tokens.
            span: self
                .sources
                .span(self.source, start..self.offset)
                .expect("lexer cursor must remain at valid source boundaries"),
        }
    }

    fn error(&self, start: usize, code: &str, message: &str) -> Diagnostic {
        Diagnostic::error(code, message, self.token(start, TokenKind::Eof).span)
    }

    fn whitespace(&mut self) {
        while self.current().is_some_and(code_space) {
            self.advance();
        }
    }

    fn trivia(&mut self) -> Result<(), Diagnostic> {
        loop {
            self.whitespace();
            if self.starts("//") {
                while self.current().is_some_and(|c| c != '\n' && c != '\r') {
                    self.advance();
                }
            } else if self.starts("/*") {
                let start = self.offset;
                self.offset += 2;
                while self.current().is_some() && !self.starts("*/") {
                    self.advance();
                }
                if !self.starts("*/") {
                    return Err(self.error(start, "QG1005", "unterminated block comment"));
                }
                self.offset += 2;
            } else {
                return Ok(());
            }
        }
    }

    fn code(&mut self) -> Result<Token, Diagnostic> {
        self.trivia()?;
        let start = self.offset;
        let Some(character) = self.current() else {
            return Ok(self.token(start, TokenKind::Eof));
        };
        if ident_start(character) {
            self.advance();
            while self.current().is_some_and(ident_continue) {
                self.advance();
            }
            let word = &self.text[start..self.offset];
            let kind = Keyword::from_word(word)
                .map(TokenKind::Keyword)
                .unwrap_or_else(|| TokenKind::Identifier(word.to_owned()));
            return Ok(self.token(start, kind));
        }
        if character.is_ascii_digit() {
            return self.number();
        }
        if character == '"' || character == '\'' {
            return self.string();
        }
        for (operator, symbol) in [
            ("===", Symbol::Equal),
            ("!==", Symbol::NotEqual),
            ("<=", Symbol::LessEqual),
            (">=", Symbol::GreaterEqual),
            ("&&", Symbol::And),
            ("||", Symbol::Or),
        ] {
            if self.starts(operator) {
                self.offset += operator.len();
                return Ok(self.token(start, TokenKind::Symbol(symbol)));
            }
        }
        for operator in [
            "==", "!=", "++", "--", "+=", "-=", "*=", "/=", "%=", "=>", "**", "??", "?.", "...",
            "<<", ">>",
        ] {
            if self.starts(operator) {
                self.offset += operator.len();
                return Err(self.error(
                    start,
                    "QG1001",
                    "operator is outside the first language subset",
                ));
            }
        }
        self.advance();
        let symbol = match character {
            '(' => Symbol::LeftParen,
            ')' => Symbol::RightParen,
            '{' => Symbol::LeftBrace,
            '}' => Symbol::RightBrace,
            '[' => Symbol::LeftBracket,
            ']' => Symbol::RightBracket,
            ';' => Symbol::Semicolon,
            ',' => Symbol::Comma,
            '.' => Symbol::Dot,
            ':' => Symbol::Colon,
            '=' => Symbol::Assign,
            '+' => Symbol::Plus,
            '-' => Symbol::Minus,
            '*' => Symbol::Star,
            '/' => Symbol::Slash,
            '%' => Symbol::Percent,
            '!' => Symbol::Not,
            '<' => Symbol::Less,
            '>' => Symbol::Greater,
            _ => return Err(self.error(start, "QG1001", "unexpected character in code")),
        };
        Ok(self.token(start, TokenKind::Symbol(symbol)))
    }

    fn number(&mut self) -> Result<Token, Diagnostic> {
        let start = self.offset;
        while self.current().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }
        let integer_end = self.offset;
        if self.starts(".")
            && self.text[self.offset + 1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
        {
            self.advance();
            while self.current().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        let mut bad_exponent = false;
        if self.current().is_some_and(|c| c == 'e' || c == 'E') {
            self.advance();
            if self.current().is_some_and(|c| c == '+' || c == '-') {
                self.advance();
            }
            let exponent_start = self.offset;
            while self.current().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
            bad_exponent = exponent_start == self.offset;
        }
        let suffix = self.current().is_some_and(ident_start);
        if suffix {
            while self.current().is_some_and(ident_continue) {
                self.advance();
            }
        }
        let raw = &self.text[start..self.offset];
        let value = raw.parse::<f64>().ok();
        let leading_zero = integer_end - start > 1 && raw.starts_with('0');
        if leading_zero || bad_exponent || suffix || !value.is_some_and(f64::is_finite) {
            return Err(self.error(start, "QG1004", "invalid or nonfinite decimal number"));
        }
        Ok(self.token(start, TokenKind::Number(value.expect("validated number"))))
    }

    fn string(&mut self) -> Result<Token, Diagnostic> {
        let start = self.offset;
        let quote = self.advance().expect("string starts with a quote");
        let mut value = String::new();
        while let Some(character) = self.advance() {
            if character == quote {
                return Ok(self.token(start, TokenKind::String(value)));
            }
            if character == '\n' || character == '\r' {
                return Err(self.error(start, "QG1002", "unescaped line ending in string"));
            }
            if character != '\\' {
                value.push(character);
                continue;
            }
            match self.advance() {
                Some('\\') => value.push('\\'),
                Some('"') => value.push('"'),
                Some('\'') => value.push('\''),
                Some('n') => value.push('\n'),
                Some('r') => value.push('\r'),
                Some('t') => value.push('\t'),
                Some('u') => value.push(self.unicode_escape(start)?),
                Some(_) => return Err(self.error(start, "QG1003", "unsupported string escape")),
                None => return Err(self.error(start, "QG1002", "unterminated string")),
            }
        }
        Err(self.error(start, "QG1002", "unterminated string"))
    }

    fn hex_quad(&mut self, start: usize) -> Result<u32, Diagnostic> {
        let mut value = 0;
        for _ in 0..4 {
            match self.advance().and_then(|c| c.to_digit(16)) {
                Some(digit) => value = value * 16 + digit,
                None => {
                    return Err(self.error(
                        start,
                        "QG1003",
                        "Unicode escape requires four hexadecimal digits",
                    ));
                }
            }
        }
        Ok(value)
    }

    fn unicode_escape(&mut self, start: usize) -> Result<char, Diagnostic> {
        let mut value = self.hex_quad(start)?;
        if (0xd800..=0xdbff).contains(&value) {
            if !self.starts("\\u") {
                return Err(self.error(
                    start,
                    "QG1003",
                    "high surrogate requires a following low surrogate escape",
                ));
            }
            self.offset += 2;
            let low = self.hex_quad(start)?;
            if !(0xdc00..=0xdfff).contains(&low) {
                return Err(self.error(start, "QG1003", "invalid Unicode surrogate pair"));
            }
            value = 0x10000 + ((value - 0xd800) << 10) + (low - 0xdc00);
        }
        char::from_u32(value)
            .ok_or_else(|| self.error(start, "QG1003", "unpaired Unicode surrogate"))
    }

    fn tag(&mut self) -> Result<Token, Diagnostic> {
        self.whitespace();
        let start = self.offset;
        let Some(character) = self.current() else {
            return Ok(self.token(start, TokenKind::Eof));
        };
        if self.starts("/>") {
            self.offset += 2;
            return Ok(self.token(start, TokenKind::SelfCloseTagEnd));
        }
        if character == '"' || character == '\'' {
            return self.string();
        }
        if ident_start(character) {
            self.advance();
            while self.current().is_some_and(name_continue) {
                self.advance();
            }
            if self.starts(":") {
                self.advance();
                if !self.current().is_some_and(ident_start) {
                    return Err(self.error(start, "QG1006", "namespace requires a name after ':'"));
                }
                self.advance();
                while self.current().is_some_and(name_continue) {
                    self.advance();
                }
            }
            return Ok(self.token(
                start,
                TokenKind::Name(self.text[start..self.offset].to_owned()),
            ));
        }
        self.advance();
        let kind = match character {
            '>' => TokenKind::TagEnd,
            '=' => TokenKind::Symbol(Symbol::Assign),
            '{' => TokenKind::Symbol(Symbol::LeftBrace),
            _ => return Err(self.error(start, "QG1006", "unexpected character in tag")),
        };
        Ok(self.token(start, kind))
    }

    fn directive(&self, offset: usize) -> Option<(Keyword, usize)> {
        let rest = &self.text[offset..];
        for (word, keyword) in [
            ("if", Keyword::If),
            ("for", Keyword::For),
            ("else", Keyword::Else),
        ] {
            let Some(after) = rest.strip_prefix(word) else {
                continue;
            };
            if after.chars().next().is_some_and(ident_continue) {
                continue;
            }
            let after = after.trim_start_matches(code_space);
            let follows = if keyword == Keyword::Else {
                after.starts_with('{')
                    || after.strip_prefix("if").is_some_and(|tail| {
                        !tail.chars().next().is_some_and(ident_continue)
                            && tail.trim_start_matches(code_space).starts_with('(')
                    })
            } else {
                after.starts_with('(')
            };
            if follows {
                return Some((keyword, offset + word.len()));
            }
        }
        None
    }

    fn view_text(&mut self) -> Result<Token, Diagnostic> {
        let mut start = self.offset;
        // Newline-only indentation is insignificant at a child boundary.
        // Preserve inline spaces when there is no newline.
        let whitespace_end = self.text[start..]
            .char_indices()
            .find(|(_, c)| !code_space(*c))
            .map(|(i, _)| start + i)
            .unwrap_or(self.text.len());
        let prefix = &self.text[start..whitespace_end];
        let next_character = self.text[whitespace_end..].chars().next();
        let ends_text_run = next_character.is_none()
            || next_character.is_some_and(|c| matches!(c, '<' | '{' | '}'))
            || self.directive(whitespace_end).is_some();
        if prefix.contains(['\n', '\r']) && ends_text_run {
            self.offset = whitespace_end;
            start = self.offset;
        }
        let Some(character) = self.current() else {
            return Ok(self.token(start, TokenKind::Eof));
        };
        if let Some((keyword, end)) = self.directive(self.offset) {
            self.offset = end;
            return Ok(self.token(start, TokenKind::Keyword(keyword)));
        }
        // A directive preceded only by inline spaces still starts a node.
        if let Some((_, _)) = self.directive(whitespace_end) {
            if whitespace_end > self.offset {
                self.offset = whitespace_end;
                return Ok(self.token(start, TokenKind::Text(prefix.to_owned())));
            }
        }
        if self.starts("</") {
            self.offset += 2;
            return Ok(self.token(start, TokenKind::CloseTag));
        }
        if character == '<' {
            self.advance();
            return Ok(self.token(start, TokenKind::OpenTag));
        }
        if character == '{' || character == '}' {
            self.advance();
            let symbol = if character == '{' {
                Symbol::LeftBrace
            } else {
                Symbol::RightBrace
            };
            return Ok(self.token(start, TokenKind::Symbol(symbol)));
        }
        let mut value = String::new();
        while let Some(character) = self.current() {
            if matches!(character, '<' | '{' | '}') {
                break;
            }
            if character == '&' {
                value.push(self.entity()?);
            } else {
                self.advance();
                value.push(character);
            }
        }
        Ok(self.token(start, TokenKind::Text(value)))
    }

    fn entity(&mut self) -> Result<char, Diagnostic> {
        let start = self.offset;
        self.advance();
        while self
            .current()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '#')
        {
            self.advance();
        }
        if !self.starts(";") {
            return Err(self.error(
                start,
                "QG1007",
                "character reference requires a terminating ';'",
            ));
        }
        let name = &self.text[start + 1..self.offset];
        self.advance();
        let value = match name {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => {
                let number = if let Some(hex) =
                    name.strip_prefix("#x").or_else(|| name.strip_prefix("#X"))
                {
                    (!hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit()))
                        .then(|| u32::from_str_radix(hex, 16).ok())
                        .flatten()
                } else if let Some(decimal) = name.strip_prefix('#') {
                    (!decimal.is_empty() && decimal.chars().all(|c| c.is_ascii_digit()))
                        .then(|| decimal.parse::<u32>().ok())
                        .flatten()
                } else {
                    None
                };
                number.and_then(char::from_u32)
            }
        };
        value.ok_or_else(|| self.error(start, "QG1007", "unknown or invalid character reference"))
    }
}

fn ident_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn ident_continue(character: char) -> bool {
    ident_start(character) || character.is_ascii_digit()
}

fn name_continue(character: char) -> bool {
    ident_continue(character) || character == '-'
}

fn code_space(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\r' | '\n')
}
