use crate::{SourceMap, Span, SpanError};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Error => "error",
            Self::Warning => "warning",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            span,
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Render plain text at the start of the span, underlining its first line.
    /// For multiline spans, report that the selection continues on another line.
    pub fn render(&self, sources: &SourceMap) -> Result<String, SpanError> {
        sources.slice(self.span)?;
        let file = sources
            .get(self.span.source())
            .ok_or(SpanError::UnknownSource)?;
        let range = self.span.range();
        let position = file.position(range.start).ok_or(SpanError::InvalidRange)?;
        let line = file.line(position.line).ok_or(SpanError::InvalidRange)?;
        let start = file
            .line_start(position.line)
            .ok_or(SpanError::InvalidRange)?;
        // A span beginning within a CRLF ending is valid, but its marker is
        // displayed at the end of the visible line, not beyond its snippet.
        let local_start = (range.start - start).min(line.len());
        let local_end = range.end.saturating_sub(start).min(line.len());
        let indent = visible(&line[..local_start]).chars().count();
        let width = visible(&line[local_start..local_end])
            .chars()
            .count()
            .max(1);
        let number = position.line.to_string();
        let padding = " ".repeat(number.len());
        let mut output = format!(
            "{}[{}]: {}\n --> {}:{}:{}\n{} |\n{} | {}\n{} | {}{}\n",
            self.severity,
            visible(&self.code),
            visible(&self.message),
            visible(&file.path().to_string_lossy()),
            position.line,
            position.column,
            padding,
            number,
            visible(line),
            padding,
            " ".repeat(indent),
            "^".repeat(width),
        );
        if range.end > start + line.len() {
            output.push_str(" = note: selection includes a line ending or later lines\n");
        }
        if let Some(help) = &self.help {
            output.push_str(&format!(" = help: {}\n", visible(help)));
        }
        Ok(output)
    }
}

// Preserve Unicode text, expand tabs, and escape terminal control characters.
// Marker widths count Unicode scalars, not terminal cells or grapheme clusters.
fn visible(text: &str) -> String {
    let mut result = String::new();
    for character in text.chars() {
        if character == '\t' {
            result.push_str("    ");
        } else if character.is_control() {
            result.extend(character.escape_default());
        } else {
            result.push(character);
        }
    }
    result
}
