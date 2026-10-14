# Language specification examples

These `.qg` files illustrate the [Group 2 grammar](../../docs/language/grammar.md).
They are not runnable applications: Quagmire has no parser, compiler, or runtime
yet. No package installation or build command is available.

- [Counter](counter.qg) demonstrates props, local state, a computed value, and
  an event handler.
- [App](app.qg) imports the counter and renders a keyed list and conditional view.

The future compiler should accept these files, preserve independent state for
counter instances, recompute the doubled value after a click, and maintain list
identity by key. Until then, repository checks validate text and document links,
not `.qg` syntax or browser behavior.
