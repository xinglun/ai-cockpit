# Release and upgrade acceptance

## Applicability

Load this guide only for an explicitly authorized release or upgrade
acceptance Contract. A release name, tag, or historical release mentioned in
background text does not select this guide.

## Authoritative inputs

Use the Runtime status and action explanation, the immutable published tag and
downloaded artifact identity, manifests, checksums, SBOM, acceptance receipt,
and the Contract's declared target and verification commands. A source
checkout, moving branch, or workspace binary is not a published artifact
substitute.

## Operations

1. Confirm the exact release Contract, immutable artifact, target, and Runtime
   identity before acceptance.
2. Run only the declared isolated acceptance checks. Bind the source,
   repository identity, root manifests, metadata, digests, and cleaned run
   root in the receipt.
3. Preserve failed downloads, replay output, manifests, and cleanup evidence.
4. Re-query the Runtime at each acceptance boundary, then deliver an Outcome
   that separates implementation, release evidence, projection, and host
   visibility.

## Success conditions

The immutable artifact and target match the Contract, the acceptance receipt
is current and identity-bound, the temporary run root is clean, and the
Runtime admits the next operation. The Outcome states what was accepted and
what remains outside the acceptance boundary.

## Failure evidence

Keep the exact tag, asset digest, target, manifest/checksum/SBOM inputs,
isolation root, cleanup result, command output, and Runtime rejection. A
moving branch or local binary must remain explicitly non-accepted evidence.

## Continue or stop

Continue only when the Runtime projection admits acceptance and all immutable
inputs are present. Stop for an unpublished or mutable artifact, missing
receipt, stale evidence, a failed cleanup, a required human decision, or any
request outside the release Contract. Return to the ordinary guide only when
the Contract no longer concerns release acceptance and Runtime facts confirm
that route.

## Reference

See [installed lifecycle](../../docs/reference/installed-lifecycle.md), [CI
release evidence](../../docs/reference/ci-release-evidence.md), and the
[command reference](../../docs/reference/commands.md).
