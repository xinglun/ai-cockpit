---
author: AI Cockpit maintainers
title: "WI-262 Release version-consistency cleanup"
workItemId: WI-262-release-version-consistency-cleanup
description: "Make post-release version consistency cleanup deterministic and fail closed."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_cleanup
  - release_truth_preservation
  - isolated_release_regression
---

# WI-262: Release version-consistency cleanup

## Goal

The post-release `tests/release/version_consistency.sh` check downloads
`release-manifest.json` into an isolated directory. The previous EXIT trap
removed only the metadata file and attempted `rmdir`, so a successful check
left the downloaded manifest behind while silently ignoring cleanup failure.

This Work Item makes cleanup an explicit postcondition. Both successful and
manifest-validation-failure paths remove the isolated download directory. A
cleanup failure is reported as a fail-closed result with
`release truth unchanged`; the script never rewrites or unpublishes a public
Release.

## Scope

- `tests/release/version_consistency.sh`
- `tests/release/version_consistency_test.sh`
- the three language versions of this Work Item document

The regression uses an isolated temporary root, a fake `gh` provider, and an
injected cleanup failure. It proves that the success and manifest-failure
paths leave no temporary files, while the injected failure is visible and
does not alter Release truth.

## Verification

```text
bash -n tests/release/version_consistency.sh
bash tests/release/version_consistency_test.sh
cargo test --locked --workspace
```

The test wrapper does not build a source fallback or contact GitHub. It binds
the fake provider to the workspace version and asserts the cleanup result.

## Acceptance boundary

Cleanup is operational hygiene, not publication authority. A cleanup failure
must remain visible in the command result and evidence, but it must not turn a
published Release into an unpublished one or modify any Release metadata.
