# Security policy

## Project status

Quagmire is in foundation development. There are no published releases or
supported production versions. Security feedback on the repository, tooling, and
design is welcome. Supported release lines will be recorded here when releases
exist; no response-time or patch-time guarantee is offered at this stage.

## Reporting a vulnerability

Use GitHub's **Report a vulnerability** option on the repository's
[Security page](https://github.com/justinangeloperez327/quagmire/security) if that
option is available. Private vulnerability reporting must be enabled by the
repository owner; this policy does not imply that it has been enabled.

If the option is unavailable, open an issue requesting a private reporting
channel, without including the vulnerability details. Wait for the maintainer to
provide a private route before sharing a reproduction or exploit.

In the private report, include the affected commit or version, impact, a minimal
reproduction, and any proposed mitigation. Exclude credentials and private user
data. Avoid publishing exploit details before coordinated disclosure.

## Requirements for future implementation

These are design requirements, not claims about an existing runtime:

- Treat interpolated values as text by default. Any future raw HTML escape hatch
  must be explicit and document the trust boundary.
- Handle attributes, URLs, styles, and event bindings according to their context;
  text escaping alone is not a universal injection defense.
- Generate static JavaScript modules. Normal rendering must not require `eval`,
  `new Function`, or runtime parsing of `.qg` files.
- Do not execute application source merely to parse or diagnose it. Build plugins
  that execute code require an explicit, documented trust boundary.
- Keep client bundles free of server credentials and other secrets. Frontend
  code cannot enforce server authorization.
- Release listeners and subscriptions when their owner is removed. Validate
  malformed input and put limits on compiler resource use where needed.
- Keep continuous integration permissions minimal, pin external actions, and
  review dependency updates. Publishing is separate from branch validation.

See the [architecture](docs/architecture.md) for the planned system boundaries.
