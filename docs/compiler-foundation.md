# Compiler foundation

Quagmire's first Rust implementation provides source loading, validated spans,
positions, and plain-text diagnostics. It does not parse `.qg` or emit JavaScript. The [lexer](lexer.md) now adds
context-driven tokenization on this foundation.

The package is one library, `quagmire-compiler`, with no third-party dependencies.
It uses Rust 1.88 or newer, edition 2024. Development and continuous integration
are pinned to Rust 1.88.0 with formatting and Clippy components. The manifest's
`0.1.0-alpha.1` is an unreleased development version; package publishing is disabled.

## Source and diagnostics

```rust
use quagmire_compiler::{Diagnostic, SourceMap};

let mut sources = SourceMap::new();
let id = sources.add("counter.qg", "state count = 0");
let span = sources.span(id, 15..15)?;
let diagnostic = Diagnostic::error("QG1002", "expected ';'", span)
    .with_help("terminate the state declaration with a semicolon");
println!("{}", diagnostic.render(&sources)?);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`SourceMap::load(path)` reads a UTF-8 `.qg` file and propagates I/O errors.
It preserves all bytes, including original line endings, and does not execute
the source. Failed loads leave the source map unchanged. `add` accepts in-memory
source with a display path that need not exist.

Source IDs are stable indices within their owning map. Keep IDs, spans, and
diagnostics with that map; they are not global identities and must not be moved
between separate maps. Repeated loading currently adds a new source. Module
resolution, source-root enforcement, and path deduplication are later work.

`span(id, start..end)` validates a half-open byte range, including empty spans.
It rejects reversed/out-of-bounds ranges and boundaries inside a UTF-8 character.
Private span fields prevent unchecked construction. `slice` revalidates before
reading. Empty spans at EOF are valid for missing-token diagnostics.

Positions retain the byte offset and use one-based lines and columns. Columns
count Unicode scalar values, not UTF-16 code units, grapheme clusters, or terminal
cells. LF, CRLF, and standalone CR are recognized as line endings without changing
offsets. A trailing line ending creates an empty final line. Empty input has one
empty line and a valid EOF position.

Diagnostics carry severity, code, message, a primary span, and optional help.
The renderer prints the path and start position, a source line, and a marker.
Multiline spans underline their first line and report continuation. Tabs expand
to four spaces; terminal controls are escaped in paths, messages, and snippets.
Wide characters and combining sequences may not align with terminal display
cells. Full terminal-width handling, multiple labels, and color are later work.

The diagnostic codes in tests/examples demonstrate formatting; parser error-code
assignments will be established with the parser implementation.

## Module boundaries

| Module | Responsibility | Current implementation |
| --- | --- | --- |
| `source` | File loading and positions | Implemented |
| `span` | Source IDs and validated ranges | Implemented |
| `diagnostic` | Diagnostic data and text rendering | Implemented |
| `lexer` | Code, tag, and view text tokenization | Implemented |
| `parser` | Grammar parsing and recovery | Reserved |
| `ast` | Declaration, expression, and view structures | Reserved |
| `semantic` | Names, assignments, and purity | Reserved |
| `reactive` | Reactive dependency analysis | Reserved |
| `view` | View lowering and validation | Reserved |
| `optimize` | Behavior-preserving transformations | Reserved |
| `codegen` | JavaScript, CSS, and source maps | Reserved |

Reserved modules contain responsibility documentation only. They have no stub
success results or pretend compilation APIs. Split crates only when an actual
integration or ownership requirement justifies it.

## Verify locally

Install Rust using rustup; the checked-in toolchain file selects the required
version and components. From the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python3 scripts/check_repository.py
```

Tests exercise Unicode offsets, line endings, EOF, span failures, stable source
IDs, actual source file loading, invalid UTF-8, and diagnostic formatting.
Continuous integration runs the Rust checks on Linux and Windows. These checks
validate compiler foundations, not browser behavior or the language examples'
syntax. Parser and abstract syntax tree implementation is the next step.
