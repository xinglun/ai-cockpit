# Release and distribution evidence

The release workflow builds the single `ai-cockpit` binary for macOS arm64,
macOS x86_64, Linux x86_64, and Windows x86_64. Each artifact is accompanied by
SHA-256 checksums and Cargo metadata used as SBOM input.

Checksums and metadata are release evidence; the governance core does not
self-attest them. Production signing, key custody, provenance attestations, and
approval of a release environment remain protected human/CI controls. A release
must attach those receipts before the GA gate is considered green.
