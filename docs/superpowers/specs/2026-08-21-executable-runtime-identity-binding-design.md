# Executable Runtime Identity Binding Design

## Context

AI Cockpit records `runtimeVersion` and `runtimeDigest` in inspection and
verification evidence. The current CLI and MCP adapters derive the digest from
the literal bytes `ai-cockpit-0.1.0`. That proves only a version label, not the
executable that produced the evidence. A locally built candidate demonstrates
the contradiction: its file SHA-256 differs from the published runtime digest.

This defect blocks WI-30 self-governance cutover because repository evidence
cannot be bound to the exact runtime that generated it.

## Decision

The CLI adapter will construct one immutable `RuntimeContext` per invocation by
resolving and streaming the current executable through SHA-256. The version is
`env!("CARGO_PKG_VERSION")` and the protocol is
`cockpit_protocol::PROTOCOL_VERSION`; neither is inferred from repository
state. Failure to resolve, open, read, or validate the executable identity is a
fatal adapter error.

The same context is passed through inspect, doctor, CLI verification evidence,
and MCP initialization/verification. MCP APIs accept the context explicitly;
they do not compute a second identity or retain a version-string fallback.

## Component Boundaries

### CLI runtime identity adapter

`crates/cockpit-cli/src/runtime_identity.rs` owns executable discovery and
streaming SHA-256 computation. It returns the existing protocol
`RuntimeContext` type and performs no repository access.

### CLI consumers

`crates/cockpit-cli/src/main.rs` constructs the context once after argument
parsing and reuses it for every output or evidence path that claims runtime
identity. `--version` remains Clap-owned and reports the package version.

### MCP adapter

`crates/cockpit-mcp/src/lib.rs` receives `&RuntimeContext` at its public request
and stdio serving boundaries. MCP `serverInfo` exposes the same version and
digest, and verification evidence receives the same context fields.

## Rejected Approaches

- Hashing a version string is the current defect and does not bind executable
  bytes.
- Embedding a build-time self-digest is circular and becomes stale if signing
  or packaging changes the executable.
- Recomputing identity independently in CLI and MCP risks disagreement and
  violates the one-context-per-invocation boundary.

## Error Handling

Runtime identity discovery is fail closed. No command that enters the normal
adapter dispatch may emit a placeholder digest after identity failure. MCP
tests inject an explicit test `RuntimeContext`; production MCP receives the
context computed by the CLI process.

## Verification

- RED/GREEN integration test compares `inspect.runtimeDigest` with an
  independently computed SHA-256 of `CARGO_BIN_EXE_ai-cockpit`.
- Doctor and MCP initialize expose the same exact identity.
- CLI and MCP verification evidence persist the supplied exact digest.
- A missing executable path is rejected by the runtime identity adapter.
- Mutation checks restoring any literal version-string digest must fail.
- Focused tests, two workspace runs, Clippy, rustfmt, diff, workflow YAML,
  multilingual counterparts, and the locked V1 Oracle remain required.

## Scope Boundary

This Work Item does not commit, push, attach this repository, confirm a project
profile, publish a release, or implement evidence reuse. WI-32 will address the
separate fail-closed evidence-reuse planner after this identity boundary is
accepted.
