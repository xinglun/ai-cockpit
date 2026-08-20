# Protocol Compatibility Rules

The compatibility algorithm is intentionally small:

1. Parse the repository protocol version without executing repository material.
2. Reject malformed or unsupported major versions as Red.
3. Accept a supported major version only when required artifact fields validate.
4. Report optional capability gaps as Yellow with an explicit safe action.
5. Never rewrite historical artifacts during compatibility inspection.

The runtime advertises a supported protocol range. The repository declares one
protocol major. Runtime minor and patch releases are not migrations. A major
protocol migration creates a new Work Item, preserves old evidence, and records
the source and target protocol versions.

