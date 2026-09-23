# Quagmire

A frontend language and framework designed around what developers want their
interfaces to do.

Quagmire will compile `.qg` source files with Rust tooling into JavaScript modules
and CSS. The browser will run the generated JavaScript with a small runtime that
updates the Document Object Model (DOM) directly.

**Status: Group 1 — Project Foundation.** This repository currently contains the
project decisions, documentation, and working repository checks. The grammar,
compiler, browser runtime, and command-line tools are not implemented yet. There
is no installable Quagmire release.

## Direction

- Express interface intent with familiar names and syntax.
- Keep state and read-only `computed` values understandable; track dependencies
  automatically.
- Make ordinary application development possible without Hooks, dependency
  arrays, or an `effect` API.
- Use Rust for the compiler and tooling, JavaScript for browser interaction, and
  CSS for suitable transitions and animations.
- Generate direct DOM updates with no Virtual DOM in the initial architecture.
- Reserve WebAssembly for optional, measured CPU-intensive work.

Rust is an implementation choice, not a browser performance guarantee. Output
size, update behavior, build time, and developer experience must be measured as
the implementation grows.

## Project guide

| Document | Purpose |
| --- | --- |
| [Project scope and principles](docs/project.md) | Goals, boundaries, and accepted source conventions |
| [Architecture](docs/architecture.md) | Compiler, output, runtime, and animation responsibilities |
| [Browser support](docs/browser-support.md) | Planned browser targets and compatibility gates |
| [Release policy](docs/releases.md) | Versioning, compatibility, and release requirements |
| [Roadmap](docs/roadmap.md) | Group 1 checklist and the Group 2 handoff |
| [Contributing](CONTRIBUTING.md) | Development workflow and local checks |
| [Security](SECURITY.md) | Reporting issues and implementation requirements |
| [Changelog](CHANGELOG.md) | Recorded changes; no release has been published |

## Run the foundation checks

From a Git checkout, use Python 3.10 or newer:

```sh
python3 scripts/check_repository.py
```

These checks validate required foundation files, text formatting, and local file
links in Markdown. They need no third-party Python packages. The same command
runs in [continuous integration](.github/workflows/ci.yml) on branch pushes and
pull requests. It does not compile Quagmire or test browsers.

Group 2 will define the language grammar before compiler implementation starts.

## License

[MIT](LICENSE) — Copyright (c) 2026 Justin Angelo Perez.
