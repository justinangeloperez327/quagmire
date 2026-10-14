use quagmire_compiler::{Diagnostic, Position, Severity, SourceMap, SpanError};
use std::fs;

#[test]
fn unicode_offsets_lines_and_eof_are_consistent() {
    let mut sources = SourceMap::new();
    let id = sources.add("unicode.qg", "é🙂\r\nnext\rlast\n");
    let file = sources.get(id).unwrap();
    assert_eq!(file.line_count(), 4);
    assert_eq!(file.line(1), Some("é🙂"));
    assert_eq!(file.line(2), Some("next"));
    assert_eq!(file.line(3), Some("last"));
    assert_eq!(file.line(4), Some(""));
    assert_eq!(file.line(0), None);
    assert_eq!(file.line(5), None);
    assert_eq!(
        file.position(6),
        Some(Position {
            offset: 6,
            line: 1,
            column: 3
        })
    );
    assert_eq!(file.position(8).unwrap().line, 2);
    assert_eq!(file.position(file.text().len()).unwrap().line, 4);
    assert_eq!(file.position(1), None);
    assert_eq!(file.position(file.text().len() + 1), None);
    assert_eq!(
        sources.slice(sources.span(id, 2..6).unwrap()).unwrap(),
        "🙂"
    );
}

#[test]
fn spans_reject_invalid_ranges_and_accept_empty_eof() {
    let mut sources = SourceMap::new();
    let id = sources.add("input.qg", "é");
    let start = 2;
    assert_eq!(sources.span(id, start..0), Err(SpanError::InvalidRange));
    assert_eq!(sources.span(id, 0..3), Err(SpanError::InvalidRange));
    assert_eq!(sources.span(id, 0..1), Err(SpanError::NotCharBoundary));
    assert_eq!(sources.slice(sources.span(id, 2..2).unwrap()).unwrap(), "");
    let missing = SourceMap::new();
    assert_eq!(missing.span(id, 0..0), Err(SpanError::UnknownSource));
}

#[test]
fn source_ids_remain_stable_after_later_additions() {
    let mut sources = SourceMap::new();
    let first = sources.add("first.qg", "first");
    let first_span = sources.span(first, 0..5).unwrap();
    let second = sources.add("second.qg", "second");
    assert_ne!(first, second);
    assert_eq!(sources.len(), 2);
    assert_eq!(sources.slice(first_span).unwrap(), "first");
}

#[test]
fn empty_source_has_a_first_line_and_eof_position() {
    let mut sources = SourceMap::new();
    assert!(sources.is_empty());
    let id = sources.add("empty.qg", "");
    let file = sources.get(id).unwrap();
    assert_eq!(file.line(1), Some(""));
    assert_eq!(file.position(0).unwrap().column, 1);
    let rendered = Diagnostic::error(
        "QG1001",
        "expected a declaration",
        sources.span(id, 0..0).unwrap(),
    )
    .render(&sources)
    .unwrap();
    assert!(rendered.contains("empty.qg:1:1"));
    assert!(rendered.contains("| ^"));
}

#[test]
fn diagnostics_preserve_source_location_and_help() {
    let mut sources = SourceMap::new();
    let id = sources.add("counter.qg", "state count = 0\n");
    let diagnostic = Diagnostic::error("QG1002", "expected ';'", sources.span(id, 15..15).unwrap())
        .with_help("terminate the state declaration with a semicolon");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.render(&sources).unwrap(),
        "error[QG1002]: expected ';'\n --> counter.qg:1:16\n  |\n1 | state count = 0\n  |                ^\n = help: terminate the state declaration with a semicolon\n"
    );
}

#[test]
fn multiline_and_tab_diagnostics_do_not_split_characters() {
    let mut sources = SourceMap::new();
    let id = sources.add("view.qg", "\té🙂\nsecond");
    let diagnostic = Diagnostic::error(
        "QG1003",
        "invalid expression",
        sources.span(id, 1..8).unwrap(),
    );
    let output = diagnostic.render(&sources).unwrap();
    assert!(output.contains("view.qg:1:2"));
    assert!(output.contains("|     ^^"));
    assert!(output.contains("selection includes a line ending or later lines"));
}

#[test]
fn terminal_controls_are_escaped_in_diagnostics() {
    let mut sources = SourceMap::new();
    let id = sources.add("bad\u{1b}.qg", "\u{1b}bad");
    let output = Diagnostic::error("QG1004", "bad\u{1b}", sources.span(id, 0..1).unwrap())
        .render(&sources)
        .unwrap();
    assert!(!output.contains('\u{1b}'));
    assert!(output.contains("\\u{1b}"));
}

#[test]
fn real_examples_load_without_normalizing_or_executing() {
    let mut sources = SourceMap::new();
    for name in ["counter.qg", "app.qg"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/language")
            .join(name);
        let id = sources.load(&path).unwrap();
        assert_eq!(
            sources.get(id).unwrap().text().as_bytes(),
            fs::read(&path).unwrap()
        );
    }
    let count = sources.len();
    assert_eq!(
        sources.load("wrong.txt").unwrap_err().kind(),
        std::io::ErrorKind::InvalidInput
    );
    assert_eq!(
        sources.load("does-not-exist.qg").unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    assert_eq!(sources.len(), count);
}

#[test]
fn invalid_utf8_does_not_add_a_source() {
    let path = std::env::temp_dir().join(format!("quagmire-invalid-{}.qg", std::process::id()));
    fs::write(&path, [0xff, 0xfe]).unwrap();
    let mut sources = SourceMap::new();
    let result = sources.load(&path);
    fs::remove_file(path).unwrap();
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
    assert!(sources.is_empty());
}
