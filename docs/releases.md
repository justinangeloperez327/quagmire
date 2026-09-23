# Version and release policy

## Current status

The repository is unreleased foundation work. Group numbers track implementation
scope; they are not release versions. Completing Group 1 does not create a tag,
a package, or a runnable framework release.

The first executable preview will target `0.1.0-alpha.1` once its documented
features actually work. Further previews use explicit prerelease identifiers.
There is no calendar commitment or release artifact for that version yet.

## Compatibility rules

Use [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html). While the
project is below `1.0.0`, its public interfaces are experimental. Quagmire adopts
the additional convention that breaking changes increment the minor version;
patch releases preserve documented behavior. During an alpha or beta sequence,
prerelease increments may include documented breaking changes.

At `1.0.0`, publish the stable public contract. Afterward, incompatible changes
require a major version, compatible additions use a minor version, and compatible
fixes use a patch version. Prefer deprecation and migration guidance when practical.

The public contract will include documented source syntax and semantics,
configuration, command-line behavior, exported runtime APIs, and supported
environment requirements. Internal compiler structures and generated helper
names are not public APIs. Compiler/runtime compatibility still requires
integration tests and coordinated releases.

Document a minimum supported Rust version when introducing Rust packages. Raising
that minimum, dropping a supported browser generation, or raising a documented
tooling requirement counts as a compatibility change under this policy. Publish
the exact supported environment matrix for each release so a rolling policy does
not silently change an already released version's guarantees.

## Release procedure

1. Confirm the intended scope is implemented and its documentation distinguishes
   supported features from proposals.
2. Run the relevant repository, compiler, runtime, and browser checks that exist
   for that scope. Resolve failures or narrow the release scope transparently.
3. Record changes, migration steps, known limitations, and tested environments in
   the changelog and release notes.
4. Update package versions and lockfiles when packages exist. Keep compiler and
   runtime versions coordinated until a separate compatibility policy is justified.
5. Tag the reviewed commit as `vX.Y.Z`, or `vX.Y.Z-alpha.N` for a preview. Build
   artifacts from that exact commit and record their checksums.
6. Publish only after an explicit release decision. Never replace the contents of
   an existing version; publish a new version for corrections.

Branch validation does not publish packages, create tags, or deploy applications.
Registry names and publishing credentials will be established when distribution
is implemented. A repository name does not establish package-name availability.

## Automation today

The [continuous integration workflow](../.github/workflows/ci.yml) checks the
foundation on branch pushes and pull requests. It has read-only repository
permissions and no release credentials. Rust and browser checks will be added
alongside the corresponding source, rather than reported as passing before any
such implementation exists.
