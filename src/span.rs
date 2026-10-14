use std::fmt;
use std::ops::Range;

/// Stable index within one SourceMap. IDs and spans must stay with that map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(pub(crate) usize);

/// A validated, half-open UTF-8 byte range belonging to a source file.
/// Construct through SourceMap::span to validate bounds and character boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub(crate) source: SourceId,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    pub fn source(self) -> SourceId {
        self.source
    }

    pub fn range(self) -> Range<usize> {
        self.start..self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanError {
    UnknownSource,
    InvalidRange,
    NotCharBoundary,
}

impl fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownSource => "source ID is not present in this source map",
            Self::InvalidRange => "span range is reversed or outside the source",
            Self::NotCharBoundary => "span splits a UTF-8 character",
        })
    }
}

impl std::error::Error for SpanError {}
