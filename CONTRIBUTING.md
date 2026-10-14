# Contributing to Quagmire

Start with the [project scope](docs/project.md), [architecture](docs/architecture.md),
and [current roadmap](docs/roadmap.md). Group 1 establishes the foundation and
Group 2 specifies the [language](docs/language/grammar.md). Executable compiler
and runtime work follows those contracts. The [compiler foundation](docs/compiler-foundation.md)
provides source loading and diagnostics.

## Working locally

For text and link checks, install Git and Python 3.10 or newer; these checks
need no third-party Python packages. For Rust development, also install rustup
and use the pinned toolchain described in the compiler foundation guide.

```sh
git clone https://github.com/justinangeloperez327/quagmire.git
cd quagmire
git switch main
git pull --ff-only
python3 scripts/check_repository.py
git diff --check
```

The checker examines tracked and non-ignored untracked text files, so it also
works before staging a new document. It checks required files, UTF-8 text, line
endings, final newlines, trailing whitespace, and inline Markdown links to local
files. It does not check external URLs, heading fragments, reference-style links,
or general Markdown grammar. Use inline links for repository file references.

## Making a change

1. Inspect the relevant files and open work before assuming a feature is missing.
2. Work directly on `main` for this project and keep implementation within the
   agreed scope. Do not create a branch or pull request unless requested.
3. Explain a public syntax or architecture proposal with examples, tradeoffs, and
   alternatives. Keep proposed syntax labeled as proposed until it is accepted.
4. Implement the change, update its documentation, and run the relevant checks.
5. Use a descriptive commit message, commit and push directly to `main`, and
   summarize behavior and verification.

Prefer names that describe developer intent. Fewer characters are not a reason
to introduce surprising syntax. Keep dependencies purposeful and explain their
benefit. Do not claim that generated code, features, or performance exist before
they have been implemented and verified.

Use UTF-8, LF line endings, a final newline, and no trailing whitespace. Follow
[EditorConfig](.editorconfig); Python uses four-space indentation. Do not commit
credentials, build output, installed dependencies, or local environment files.
Commit dependency lockfiles once package manifests are introduced.

## Checks as implementation grows

The [current workflow](.github/workflows/ci.yml) runs the foundation checker plus
Rust formatting, Clippy, and tests on Linux and Windows. Rust 1.88 is the minimum;
`rust-toolchain.toml` pins development to 1.88.0. Run the commands from the
[compiler foundation guide](docs/compiler-foundation.md) before publishing Rust
changes. Introduce browser tests when there is executable browser behavior.

Compiler tests should cover valid programs, invalid programs, useful diagnostics,
and generated output. Runtime tests should check observable DOM behavior,
updates, and cleanup. Add regression tests for meaningful failure modes; avoid
tests that only repeat implementation details.

## Reporting a problem

For ordinary bugs, include the commit or version, expected and actual behavior,
and a minimal reproduction. For language proposals, include the task a developer
is trying to express and the proposed source example. Report sensitive security
issues using the [security policy](SECURITY.md).
