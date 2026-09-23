# Architecture

**Status: accepted foundation direction; compiler and runtime not implemented.**

## Responsibilities and boundaries

| Layer | Responsibility | Planned output |
| --- | --- | --- |
| `.qg` source | Express application intent with the agreed language | Components and related source modules |
| Rust compiler | Read source, parse it, analyze it, and generate browser code | JavaScript modules, CSS, source maps, diagnostics |
| Rust tooling | Coordinate compilation and, later, development workflows | Build artifacts and developer feedback |
| Browser runtime | Own dynamic state, update scheduling, DOM updates, and cleanup | Observable interface behavior |
| Browser CSS engine | Apply styling, transitions, and suitable animations | Visual presentation and motion |
| Optional WebAssembly module | Perform an explicitly selected CPU-intensive task | Results passed through a documented boundary |

Rust runs in the development/build toolchain. Normal browser applications receive
JavaScript and CSS; they do not need the compiler or a WebAssembly runtime for
Quagmire itself. A future JavaScript ecosystem adapter may coordinate the Rust
compiler without changing these responsibilities.

## Compiler pipeline

The intended compilation stages are:

1. Read UTF-8 `.qg` files and retain source locations.
2. Tokenize and parse against the grammar established in Group 2.
3. Analyze names, expressions, component structure, and reactive dependencies.
4. Lower the program into an internal representation suitable for code generation.
5. Emit JavaScript modules, CSS, and source maps with deterministic output.

Errors should identify the original file and location, explain what is wrong,
and suggest an actionable correction when possible. Invalid input must not
silently produce runnable output. Optimizations must preserve behavior and source
mapping; performance claims need measurements.

The boundaries above do not mandate one crate per stage. Introduce modules and
crates as implementation needs become clear. The minimum Rust version, toolchain,
dependencies, command-line interface, and module resolution rules will be chosen
with the first relevant implementation, rather than inferred from another project.

## Browser execution and reactivity

The initial renderer generates direct DOM creation and update operations. It
does not build a general Virtual DOM tree. A small runtime coordinates behavior
that remains dynamic; it must not parse `.qg` in the browser.

The intended model is writable state and read-only `computed` values with
automatic dependencies. Computing a value should not itself mutate state or
perform external side effects. The compiler should identify dependencies where
possible; any dynamic dependency tracking belongs inside the framework. The
dependency algorithm, update ordering, batching, cycle detection, and cleanup
rules require explicit design and behavior tests in later groups.

Normal component authors should not supply Hooks, dependency arrays, or an
`effect` call just to keep the screen synchronized. Event handling, network work,
subscriptions, and resource disposal still need clear semantics. Naming those
application-facing facilities is outside this foundation group.

Compiler and runtime must share a documented contract for generated calls and
ownership. Initially, release them together and test them together. Keep the
internal contract separate from the public language and runtime API.

## CSS and animation

Generate standard CSS for styling, transitions, and keyframe animations where
those mechanisms meet the requested behavior. Do not add a per-frame JavaScript
loop to an animation that CSS can express adequately.

JavaScript can still change state or classes to start motion, coordinate events,
or support a future animation feature that requires it. The decision to use CSS
does not promise that all interaction is JavaScript-free. Styling boundaries,
class naming, and any style scoping syntax require later specifications.

Motion features must respect the browser's
[`prefers-reduced-motion`](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion)
setting. A reduced-motion mode must preserve essential information and controls.
Prefer progressive enhancement for newer CSS capabilities.

## WebAssembly

WebAssembly is optional and outside the first browser milestone. Consider it for
work such as image processing or a substantial numerical calculation only after
measurement justifies it. Account for module download, startup, data transfer,
and integration cost. It is not the default target for DOM rendering and must
not be required for a basic component.

## Quality gates

When the relevant implementations exist, verify:

- Diagnostics and emitted output against valid and invalid source fixtures.
- Real DOM behavior, computed updates, event handling, and teardown.
- Browser compatibility according to the [support policy](browser-support.md).
- Text and attribute handling according to the [security requirements](../SECURITY.md).
- Build time, output size, startup, and updates using reproducible benchmarks.

Group 1's checker validates repository files only. These compiler and browser
gates become mandatory alongside the features they exercise.
