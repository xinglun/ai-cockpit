# Release and distribution evidence

The release workflow builds the single `ai-cockpit` binary for macOS arm64,
macOS x86_64, Linux arm64, Linux x86_64, and Windows x86_64. Each artifact is accompanied by
SHA-256 checksums, Cargo metadata, an SPDX SBOM, and GitHub build-provenance
attestations. Unix targets are packaged as `.tar.gz`; Windows is packaged as
`.zip`.

Checksums and metadata are release evidence; the governance core does not
self-attest them. The workflow signs checksums when the protected `COSIGN_*`
secrets are configured. Production key custody and approval of a release
environment remain protected human/CI controls. A release must attach those
receipts before the GA gate is considered green.
