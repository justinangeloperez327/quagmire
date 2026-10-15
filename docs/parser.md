# Parser and syntax tree

The Rust parser reads one complete `.qg` module into a spanned abstract syntax
tree. It drives the lexer's code, tag, and text modes and implements the
[first-subset grammar](language/grammar.md). A successful parse establishes syntax
and structural validity. Name resolution, purity, types, module loading, runtime
behavior, and JavaScript generation are separate stages.

## Parse a source

```rust
use quagmire_compiler::{Parser, SourceMap};
use quagmire_compiler::ast::DeclarationKind;

let mut sources = SourceMap::new();
let id = sources.add("app.qg",
    "export component App() { view { <p>Hello</p> } }");
let module = Parser::new(&sources, id).unwrap().parse().unwrap();
assert_eq!(module.declarations.len(), 1);
let DeclarationKind::Component(component) = &module.declarations[0].kind else {
    panic!("expected a component");
};
assert_eq!(component.name.text, "App");
assert_eq!(component.view.root.name.text, "p");
```

`Parser::new` rejects source IDs absent from the supplied map. `parse` consumes
its parser and returns either a complete `Module` or one `Diagnostic`. It stops
at the first error and never returns a partially valid module as a successful
result. Automatic multi-error recovery is not implemented. Lexer diagnostics
retain their codes and source spans; parser diagnostics can be rendered against
the same source map with `Diagnostic::render`.

## Supported syntax

- Named imports followed by exported or local components, functions, and constants.
- Component props with defaults; state, computed, constant, and function members;
  exactly one final view with one root element.
- Functions, local constants/variables, identifier assignments, call statements,
  returns, conditional statements, and array loop syntax.
- Primitive, array, and object expressions; grouping, unary/binary operations,
  named property access, indexing, and calls with trailing commas.
- Native and component elements, attributes, text, interpolation, conditional
  branches, and view loops with one keyed root element.

The parser uses precedence climbing for expressions and left-associative binary
operators. Parenthesized expressions remain explicit `Group` nodes. Ordering
comparisons cannot be chained without grouping; `a < b < c` is rejected, while
`(a < b) < c` parses and still needs type checking. Assignment remains a statement.

The parser also checks uppercase component declaration names, required/default
parameter ordering, simple assignment targets, call-only expression statements,
matching closing tags, exact duplicate attribute names, void/component
self-closing syntax, and expression-valued keys on loop roots. Case-normalized
HTML attribute validation and value/type checks remain semantic work.

## Syntax tree

The `ast` module exposes data structures for modules, imports, declarations,
components, parameters, bindings, functions, blocks, statements, expressions,
object properties, elements, attributes, and view nodes. Meaningful syntax nodes
carry source spans; names keep both their text and original token span. String
and text values are decoded, while spans retain access to original spelling.

Conditionals store ordered branches and an optional final alternative. Code
loops and view loops are distinct node kinds. Attributes distinguish bare boolean,
quoted string, and expression values. Declaration and member variants retain
whether a binding was `const`, `let`, `state`, or `computed`.

The module span covers the whole source, including leading/trailing trivia.
Declaration spans include an `export` modifier when present; the contained
component/function span begins at its own declaration keyword. Expression spans
cover their complete syntax, including grouping or postfix operations.

## Diagnostics and resource limits

| Code | Meaning |
| --- | --- |
| `QG2001` | Expected declaration, token, name, or expression |
| `QG2002` | Invalid declaration/statement shape or comparison chain |
| `QG2003` | Invalid element, view structure, attribute, or loop key syntax |
| `QG2004` | Parser nesting or expression-tree depth limit exceeded |

Recursive parser nesting and resulting expression trees are limited to 128
levels. This also bounds left-associated operator/property chains, which do not
necessarily create deep parser recursion. Limits produce diagnostics; they are
not permission to emit partial output. Total source-size/time limits and more
advanced error recovery remain later work.

## Verification and boundaries

`cargo test --locked` parses both checked-in `.qg` examples and covers precedence,
postfix expressions, declarations, code/view control flow, entity decoding,
comments in text, nested object interpolation, malformed input, lexical failures,
and depth limits. This guide's API example is a compiled documentation test.
Linux and Windows run the same compiler tests in continuous integration.

A parsed import is not a resolved import. A parsed call is not proof that its
callee exists or is pure. Likewise, a syntactically keyed list has not had its
keys evaluated. Semantic analysis, reactive dependency analysis, and code
emission are the next implementation stages; there is no runnable framework yet.
