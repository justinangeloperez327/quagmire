# Browser support policy

**These are planned targets, not a claim of tested support.** Group 1 has no
browser runtime or generated application. No browser compatibility has been
certified yet.

## Initial target matrix

For each future release, record exact tested versions in its release notes. The
planned rolling policy is:

| Browser | Target at release time |
| --- | --- |
| Google Chrome | Latest two stable major versions |
| Microsoft Edge | Latest two stable major versions |
| Mozilla Firefox | Latest two stable major versions and the current Extended Support Release |
| Apple Safari on macOS | Latest two stable major versions |
| Apple Safari on iOS and iPadOS | Latest two stable major versions |

Internet Explorer, legacy Edge, preview browser channels, and embedded WebViews
are outside the initial supported matrix. Embedded applications can work, but
need their own compatibility evidence before support is promised.

## Output baseline

- Emit JavaScript using ECMAScript 2020 syntax or earlier and native ECMAScript
  modules as the initial code-generation ceiling. This is a compiler target,
  not a blanket guarantee for every browser or built-in API.
- Validate each required DOM and JavaScript API against the target matrix.
  Syntax compatibility does not establish API compatibility.
- Use standard CSS. Newer visual features need a working fallback or progressive
  enhancement; core controls must remain usable without them.
- Keep WebAssembly optional. Basic interface behavior cannot depend on it.
- Do not silently ship broad polyfill bundles. Any required compatibility layer
  must have a documented purpose and measured cost.

The initial emission ceiling can be revised through a documented compatibility
decision. A bundler or ecosystem adapter must not silently widen the support
claim beyond the tested output.

## Verification before claiming support

Once there is executable output, automate tests in Chromium, Firefox, and WebKit
engines. Engine automation is useful coverage, but does not by itself verify
every released browser in the matrix. Add checks in the named shipping browsers,
including real Safari and iOS Safari environments, before claiming full support.

Tests should cover mounting and removal, events, state and computed updates,
conditional/list rendering as implemented, keyboard use, and reduced motion.
Keep failures reproducible and record coverage gaps in release notes. If the
target matrix cannot be validated for a release, narrow the published support
claim explicitly rather than labeling untested versions as supported.

Apply the [release policy](releases.md) to changes in supported browsers.
