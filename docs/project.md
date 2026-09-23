# Project scope and principles

## Purpose

Quagmire is a compiler-driven frontend language and framework. Its purpose is to
help developers express components, state, derived values, events, and views
without making routine interface work depend on framework mechanics.

The intended developer writes `.qg` files. Rust tooling compiles them into
JavaScript and CSS for the browser. A small browser runtime supports the
operations that cannot be resolved at build time.

## Design principles

1. **Developer intent comes first.** Names should explain the task being
   performed. Do not make developers learn internal scheduling or subscription
   mechanics to display a value or handle a click.
2. **Familiar syntax reduces surprise.** Use established punctuation where it
   communicates well. Readability matters more than reducing character count.
3. **Derived values remain derived.** Retain the term `computed` for read-only
   values derived from state. Dependencies are automatic; normal application
   code should not maintain dependency arrays.
4. **Ordinary reactivity should not require Hooks or effects.** Framework
   internals still need to schedule updates and clean up resources. Those
   responsibilities should not become mandatory application boilerplate.
5. **Complexity must have an owner.** The compiler handles work that can be
   resolved at build time. The runtime handles dynamic browser behavior.
   Diagnostics explain errors in terms of the original source.
6. **Performance requires evidence.** Measure compiler time, output size,
   startup, updates, and cleanup. Rust alone does not make the generated browser
   application faster.

## Accepted source conventions

| Convention | Decision |
| --- | --- |
| Source extension | `.qg`, using UTF-8 text |
| Parameters, conditions, calls | Parentheses: `()` |
| Blocks and object literals | Braces: `{}` |
| Arrays and indexing | Brackets: `[]` |
| View elements | HTML-like angle brackets: `<>` |
| Derived state terminology | `computed`, read-only with automatic dependencies |

These conventions are constraints for Group 2, not a complete grammar. The exact
declarations, operator rules, interpolation, event syntax, imports, and error
recovery must be specified there before they are implemented. In particular,
sharing `{}` between objects and blocks requires an explicit grammar rule.

## Initial product scope

The first executable milestone should support a small interactive component
from source to browser: local state, a computed value, an event, rendered content,
and styling. This is a future acceptance target, not a working example in Group 1.

Initial architecture decisions are Rust compiler/tooling, JavaScript module
output, CSS output, and a small runtime that updates the DOM directly. The
initial rendering design does not use a Virtual DOM. Quagmire is independent of
a particular backend framework.

Routing, server rendering, hydration, data fetching conventions, developer tools,
and ecosystem adapters need later designs. WebAssembly is an optional future
integration for CPU-intensive work; ordinary components must not require it.
Quagmire does not aim to replace a backend or require application authors to
write Rust for routine frontend work.

## Group 1 boundary

This group delivers the project policies, architecture, and repository checks.
It does not ship language parsing, a compiler, a runtime, a development server,
or package distribution. See the [roadmap](roadmap.md) for the completion
checklist and the next group.
