use crate::span::{SourceId, Span, SpanError};
use std::ffi::OsStr;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// Byte offset plus one-based line and Unicode scalar column.
/// Columns are not grapheme, UTF-16, or terminal display-cell indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub struct SourceFile {
    path: PathBuf,
    text: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    fn new(path: PathBuf, text: String) -> Self {
        let mut line_starts = vec![0];
        let bytes = text.as_bytes();
        let mut offset = 0;
        while offset < bytes.len() {
            match bytes[offset] {
                b'\r' => {
                    offset += 1;
                    if bytes.get(offset) == Some(&b'\n') {
                        offset += 1;
                    }
                    line_starts.push(offset);
                }
                b'\n' => {
                    offset += 1;
                    line_starts.push(offset);
                }
                _ => offset += 1,
            }
        }
        Self {
            path,
            text,
            line_starts,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Return a one-based line without its line ending. A trailing newline
    /// creates an empty final line so EOF has a well-defined position.
    pub fn line(&self, number: usize) -> Option<&str> {
        let index = number.checked_sub(1)?;
        let start = *self.line_starts.get(index)?;
        let end = self
            .line_starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.text.len());
        Some(self.text[start..end].trim_end_matches(['\r', '\n']))
    }

    pub fn position(&self, offset: usize) -> Option<Position> {
        if !self.text.is_char_boundary(offset) {
            return None;
        }
        let index = self.line_starts.partition_point(|&start| start <= offset) - 1;
        let column = self.text[self.line_starts[index]..offset].chars().count() + 1;
        Some(Position {
            offset,
            line: index + 1,
            column,
        })
    }

    pub(crate) fn line_start(&self, number: usize) -> Option<usize> {
        self.line_starts.get(number.checked_sub(1)?).copied()
    }
}

/// Owns immutable UTF-8 sources. IDs are stable because sources are never removed.
/// Each load is a new source; path deduplication/module resolution comes later.
#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an in-memory source. Its path is a display label and need not exist.
    pub fn add(&mut self, path: impl Into<PathBuf>, text: impl Into<String>) -> SourceId {
        let id = SourceId(self.files.len());
        self.files.push(SourceFile::new(path.into(), text.into()));
        id
    }

    /// Read a .qg file without newline normalization or executing source.
    /// I/O and UTF-8 failures leave this map unchanged.
    pub fn load(&mut self, path: impl AsRef<Path>) -> io::Result<SourceId> {
        let path = path.as_ref();
        if path.extension() != Some(OsStr::new("qg")) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Quagmire source files must use the .qg extension",
            ));
        }
        let text = std::fs::read_to_string(path)?;
        Ok(self.add(path, text))
    }

    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        self.files.get(id.0)
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn span(&self, source: SourceId, range: Range<usize>) -> Result<Span, SpanError> {
        let file = self.get(source).ok_or(SpanError::UnknownSource)?;
        if range.start > range.end || range.end > file.text.len() {
            return Err(SpanError::InvalidRange);
        }
        if !file.text.is_char_boundary(range.start) || !file.text.is_char_boundary(range.end) {
            return Err(SpanError::NotCharBoundary);
        }
        Ok(Span {
            source,
            start: range.start,
            end: range.end,
        })
    }

    pub fn slice(&self, span: Span) -> Result<&str, SpanError> {
        self.span(span.source, span.range())?;
        Ok(&self.get(span.source).ok_or(SpanError::UnknownSource)?.text[span.range()])
    }
}
