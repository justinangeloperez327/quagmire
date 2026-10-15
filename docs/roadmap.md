# Implementation roadmap

This document records the compiler implementation deliverables.
Completion here describes repository work, not published or production-ready
framework capabilities.

## Group 1 — Project Foundation

- [x] Define Quagmire's purpose and scope: [project](project.md).
- [x] Establish developer intent over framework mechanics: [principles](project.md).
- [x] Choose Rust for compiler and tooling: [architecture](architecture.md).
- [x] Choose JavaScript as the ordinary browser target: [architecture](architecture.md).
- [x] Use CSS for suitable animation and motion: [architecture](architecture.md).
- [x] Keep WebAssembly optional for CPU-intensive work: [architecture](architecture.md).
- [x] Choose `.qg` as the source extension: [source conventions](project.md).
- [x] Choose and add the [MIT license](../LICENSE).
- [x] Add the [README](../README.md), [contribution guide](../CONTRIBUTING.md),
  [security policy](../SECURITY.md), and [changelog](../CHANGELOG.md).
- [x] Document the compiler/runtime boundaries: [architecture](architecture.md).
- [x] Define planned [browser targets](browser-support.md).
- [x] Define [versioning and release strategy](releases.md).
- [x] Add working [continuous integration](../.github/workflows/ci.yml) and
  [local foundation checks](../scripts/check_repository.py).

## Group 2 — Language Design and Grammar

**Specification complete; syntax parsing implemented; runtime not implemented.** The language
contract follows the accepted punctuation and `computed` terminology from the
[project decisions](project.md). Start with the [grammar](language/grammar.md).

- [x] Define `component`, `state`, `computed`, `function`, and `view` declarations.
- [x] Define `if`, `else`, `for`, `in`, and `return` behavior and syntax.
- [x] Define imports, exports, parameters, variables, and literals.
- [x] Define expressions, operators, precedence, and associativity.
- [x] Define HTML-like elements, attributes, events, interpolation, and nesting.
- [x] Specify escaping and the separation between source syntax and emitted text.
- [x] Add `docs/language/grammar.md`, `docs/language/components.md`,
  `docs/language/expressions.md`, and `docs/language/views.md` with valid and invalid
  examples, ambiguity decisions, and the first supported subset.
- [x] Add illustrative [source examples](../examples/language/README.md) and
  require the language documents in repository checks.

Lexer and parser implementation are recorded below. Semantic analysis, code
emission, runtime behavior, and browser tooling remain later work.

## Group 3 — Rust Compiler Foundation

- [x] Establish one Rust library and stage module boundaries for `lexer`, `parser`,
  `ast`, `semantic`, `reactive`, `view`, `optimize`, and `codegen`.
- [x] Implement UTF-8 source loading and stable map-local source IDs.
- [x] Implement byte offsets, one-based lines/columns, and validated spans.
- [x] Implement diagnostics with original source locations and optional help.
- [x] Pin Rust 1.88.0, commit the lockfile, and add Linux/Windows Rust checks.
- [x] Document and test the [foundation APIs](compiler-foundation.md).

## Group 4 — Lexer

- [x] Implement code, tag, and text modes with source spans and lookahead.
- [x] Tokenize keywords, identifiers, names, literals, operators, and punctuation.
- [x] Handle comments, string escapes, finite numbers, and view entities.
- [x] Preserve text semantics and expose lexical diagnostics.
- [x] Test nested view/code contexts, malformed input, UTF-8 spans, and EOF.
- [x] Document the [lexer API](lexer.md) and its parser-driven mode contract.

The parser now drives lexer modes and validates complete syntax. The lexer alone
does not enforce grammar or semantics.

## Parser and Abstract Syntax Tree

- [x] Parse full modules, imports, declarations, functions, statements, and expressions.
- [x] Drive lexical modes for elements, attributes, text, interpolation, and controls.
- [x] Preserve spans and declaration intent in typed syntax nodes.
- [x] Enforce structural grammar, element matching, and keyed-list root syntax.
- [x] Report fail-fast diagnostics without returning a successful partial module.
- [x] Bound recursive parsing and expression-tree depth.
- [x] Parse checked-in examples and test invalid input and precedence.

Next: semantic analysis for scopes, names, immutable bindings, parameter/call
rules, purity, and source/module constraints. Parsing alone does not validate
those requirements or produce runnable browser output.
