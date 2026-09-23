# Implementation roadmap

This document records the Group 1 deliverables and the immediate Group 2 handoff.
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

**Next; not implemented in Group 1.** Specify the language before writing the
parser. Accepted punctuation and `computed` terminology carry forward from the
[project decisions](project.md).

- [ ] Define `component`, `state`, `computed`, `function`, and `view` declarations.
- [ ] Define `if`, `else`, `for`, `in`, and `return` behavior and syntax.
- [ ] Define imports, exports, parameters, variables, and literals.
- [ ] Define expressions, operators, precedence, and associativity.
- [ ] Define HTML-like elements, attributes, events, interpolation, and nesting.
- [ ] Specify escaping and the separation between source syntax and emitted text.
- [ ] Add `docs/language/grammar.md`, `docs/language/components.md`,
  `docs/language/expressions.md`, and `docs/language/views.md` with valid and invalid
  examples, ambiguity decisions, and the first supported subset.

Lexer/parser implementation, compilation, runtime behavior, and browser tooling
remain later work. Do not mark them complete based on design documents alone.
