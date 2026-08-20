# AI Cockpit

AI Cockpit is a new Rust repository-governance runtime for AI-assisted engineering.
It is a single-binary, CLI-first product with a local MCP adapter and a versioned
Repository Protocol.

This repository is not a V1 upgrade, migration, or Rust port. The V1 template is
used only as a specification source, behavioral oracle, conformance corpus source,
and historical evidence reference. Runtime code, Python modules, Makefile.ai,
installer files, and runtime schemas are not copied into this repository's target
repositories.

Northbound is MCP and CLI. Southbound is the Repository Protocol. The Rust
governance core remains independent from both adapters and from application code.

Read the [documentation map](docs/README.md) before contributing.

