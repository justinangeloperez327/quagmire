# Contributing to Quagmire

Start with the [project scope](docs/project.md), [architecture](docs/architecture.md),
and [current roadmap](docs/roadmap.md). Group 1 establishes the foundation. Language
design is the next group; executable compiler and runtime work follows it.

## Working locally

For the current foundation, install Git and Python 3.10 or newer. No package
installation, Rust toolchain, or Node.js installation is required for these
repository checks.

```sh
git clone https://github.com/justinangeloperez327/quagmire.git
cd quagmire
git switch -c your-change
python3 scripts/check_repository.py
git diff --check
```

The foundation is initially delivered on `group-1-project-foundation`. Until that
branch is merged into `main`, check it out before creating a dependent branch:

```sh
git switch group-1-project-foundation
git switch -c your-dependent-change
```

The checker examines tracked and non-ignored untracked text files, so it also
works before staging a new document. It checks required files, UTF-8 text, line
endings, final newlines, trailing whitespace, and inline Markdown links to local
files. It does not check external URLs, heading fragments, reference-style links,
or general Markdown grammar. Use inline links for repository file references.

## Making a change

1. Inspect the relevant files and open work before assuming a feature is missing.
2. Create a focused branch from the appropriate base. Keep implementation within
   the agreed group or issue.
3. Explain a public syntax or architecture proposal with examples, tradeoffs, and
   alternatives. Keep proposed syntax labeled as proposed until it is accepted.
4. Implement the change, update its documentation, and run the relevant checks.
5. Use a descriptive commit message and summarize behavior and verification when
   submitting a pull request.

Prefer names that describe developer intent. Fewer characters are not a reason
to introduce surprising syntax. Keep dependencies purposeful and explain their
benefit. Do not claim that generated code, features, or performance exist before
they have been implemented and verified.

Use UTF-8, LF line endings, a final newline, and no trailing whitespace. Follow
[EditorConfig](.editorconfig); Python uses four-space indentation. Do not commit
credentials, build output, installed dependencies, or local environment files.
Commit dependency lockfiles once package manifests are introduced.

## Checks as implementation grows

The [current workflow](.github/workflows/ci.yml) runs the foundation checker. Add
Rust formatting, Clippy, and tests with the first Rust sources, along with a
documented minimum supported Rust version and a pinned development toolchain.
Introduce browser tests when there is executable browser behavior to test.

Compiler tests should cover valid programs, invalid programs, useful diagnostics,
and generated output. Runtime tests should check observable DOM behavior,
updates, and cleanup. Add regression tests for meaningful failure modes; avoid
tests that only repeat implementation details.

## Reporting a problem

For ordinary bugs, include the commit or version, expected and actual behavior,
and a minimal reproduction. For language proposals, include the task a developer
is trying to express and the proposed source example. Report sensitive security
issues using the [security policy](SECURITY.md).
