# Language specification examples

These `.qg` files illustrate the [Group 2 grammar](../../docs/language/grammar.md).
The Rust parser now accepts both files in automated tests. They are not runnable
applications: semantic analysis, code generation, and a browser runtime are not
implemented. No application build command is available.

- [Counter](counter.qg) demonstrates props, local state, a computed value, and
  an event handler.
- [App](app.qg) imports the counter and renders a keyed list and conditional view.

The future runtime should preserve independent state for counter instances,
recompute the doubled value after a click, and maintain list identity by key.
Rust tests validate parsing; Python repository checks validate text and links.
Neither check demonstrates runtime or browser behavior.
