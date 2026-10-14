//! Foundations for the Quagmire compiler. Parsing and code generation are not
//! implemented yet; unimplemented stage modules reserve their responsibilities.

#![doc = include_str!("../docs/compiler-foundation.md")]
#![doc = include_str!("../docs/lexer.md")]

pub mod ast;
pub mod codegen;
pub mod diagnostic;
pub mod lexer;
pub mod optimize;
pub mod parser;
pub mod reactive;
pub mod semantic;
pub mod source;
pub mod span;
pub mod view;

pub use diagnostic::{Diagnostic, Severity};
pub use lexer::{Keyword, LexMode, Lexer, Symbol, Token, TokenKind};
pub use source::{Position, SourceFile, SourceMap};
pub use span::{SourceId, Span, SpanError};
