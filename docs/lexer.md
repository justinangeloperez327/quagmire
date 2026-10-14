# Lexer

The Rust lexer converts source into tokens with validated byte spans. It covers
code keywords, literals, operators, comments, tag names, attributes, and view
text. It does not parse declarations, validate element nesting, resolve names,
or emit JavaScript.

## API and lexical modes

```rust
use quagmire_compiler::{Keyword, LexMode, Lexer, SourceMap, TokenKind};

let mut sources = SourceMap::new();
let id = sources.add("counter.qg", "state count = 0;");
let mut lexer = Lexer::new(&sources, id).unwrap();
let first = lexer.peek(LexMode::Code).unwrap();
assert_eq!(first.kind, TokenKind::Keyword(Keyword::State));
assert_eq!(lexer.offset(), 0);
assert_eq!(lexer.next_token(LexMode::Code).unwrap(), first);
assert_eq!(sources.slice(first.span).unwrap(), "state");
```

The future parser chooses a mode for every token. Mode selection is explicit
because source characters have different meanings in different grammar contexts:

| Mode | Behavior |
| --- | --- |
| `Code` | Keywords/identifiers, strings, finite decimal numbers, punctuation and operators; skips whitespace and comments |
| `Tag` | Names, quoted values, `=`, `{`, `>`, and `/>`; skips whitespace but does not treat comments as code trivia |
| `Text` | Raw/decoded child text, `<`, `</`, interpolation/control braces, and view directives at node boundaries |

For `<button title={count < 2}>Count: {count}</button>`, read the opening `<` in
text mode, the name and attributes in tag mode, and each interpolation in code
mode. Resume tag/text mode after the interpolation's closing brace. In code
mode `<` is a comparison; in text mode it opens an element. The parser must
track nesting rather than find the next `}` by string search.

`peek(mode)` does not consume source, even on an error. `next_token(mode)` returns
one token or a lexical diagnostic. Stop parsing on a lexical error; the library
does not silently discard it or certify a partial token stream as a valid program.
EOF uses an empty span at the end of source and is repeatable. No convenience API
claims to tokenize a full mixed-mode component without grammar context.

Token spans preserve original spelling, including string quotes and entity
references. String/text payloads are decoded; number payloads are binary64 values.
Identifiers and tag/attribute names are owned strings. Keywords have enum variants.
Tag mode emits `Name` even for words that are reserved in code, so attributes such
as `for` remain usable. Whether a name is a valid native tag, attribute, or
component prop is a parser/semantic responsibility.

## Literals and text

Strings accept the escapes in the [grammar](language/grammar.md). A high-surrogate
Unicode escape must be followed by a low-surrogate escape; lone surrogates are
rejected. Valid pairs decode to one Unicode scalar. Unknown escapes, unescaped
line endings, and unterminated strings produce diagnostics.

Number scanning rejects leading zeroes, malformed exponents, unsupported numeric
suffixes, and nonfinite values. Unary signs are separate tokens. A dot that does
not begin a fractional part remains a postfix punctuation token. Unsupported
compound operators are rejected rather than silently interpreted as other syntax.

Text mode decodes the specified named and numeric character references directly
into a text payload. An encoded `<` or `{` is never rescanned as markup or code.
Only whitespace-only runs containing a line ending are omitted; mixed text and
inline spaces remain. Quoted attribute values use string rules and do not decode
HTML entities. Comment markers remain literal in text mode.

At a child boundary, `if`/`for` followed by optional code whitespace and `(`
introduce directives. `else` followed by `{` or `if (` also produces a directive
token. Those keywords embedded in an existing text run remain text. Comments
between a view directive word and its opening parenthesis are not recognized as
directive separators; place comments inside the code header instead.

## Lexical diagnostics

| Code | Meaning |
| --- | --- |
| `QG1001` | Unexpected code character or unsupported operator |
| `QG1002` | Unterminated string or unescaped line ending |
| `QG1003` | Invalid string/Unicode escape |
| `QG1004` | Invalid or nonfinite number |
| `QG1005` | Unterminated block comment |
| `QG1006` | Invalid tag-mode character or name namespace |
| `QG1007` | Unknown, malformed, or invalid character reference |

These diagnostics use the [source/diagnostic foundation](compiler-foundation.md)
and can be rendered against the same source map. Unknown source IDs are rejected
when constructing a lexer. Every successful non-EOF token consumes source; all
cursor and span boundaries remain valid UTF-8 boundaries.

## Verification and remaining work

`cargo test --locked` covers each mode, keyword boundaries, longest operators,
literal failures, code versus text comments, entities, view whitespace, nested
expression contexts, lookahead, and EOF. Linux and Windows run the same tests.
The documentation API example is also compiled as a Rust documentation test.

The next stage is the parser and abstract syntax tree. It will drive mode changes,
match delimiters and tags, enforce declarations and root-element rules, and parse
the `.qg` example files. Lexer success alone does not prove grammar or semantics.
