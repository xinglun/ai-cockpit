# Documentation Map

The English documents are canonical for machine-facing terminology. Chinese and
Japanese documents are maintained as semantic equivalents, not summaries.

## Start here

- [Product boundary](architecture/product-boundary.md)
- [Runtime topology](architecture/runtime-topology.md)
- [Versioning](architecture/versioning.md)
- [Bootstrap work-item rules](work-items/README.md)
- [Repository Protocol v1](protocol/v1/specification.md)
- [Performance acceptance](../tests/performance/README.md)
- [Release and distribution](release/distribution.md)
- [Adversarial validation](security/adversarial-validation.md)
- [Work-item roadmap](work-items/WI-03.md)

## Development order

1. Freeze semantics and the protocol.
2. Build the pure governance core.
3. Observe a repository once and reuse the immutable snapshot.
4. Add verification, lifecycle writes, knowledge, attach, and MCP.
5. Prove conformance, performance, adversarial behavior, and thin-repository use.

The current implementation status is recorded in WI-03 through WI-24. Partial
items remain explicitly non-GA until their evidence gates are complete.

Until the Rust runtime can govern itself, development uses the Markdown bootstrap
rules in `docs/work-items`. V1 is never installed into this repository.
