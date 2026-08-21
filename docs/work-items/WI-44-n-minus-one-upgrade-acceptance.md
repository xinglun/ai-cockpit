---
author: AI Cockpit maintainers
title: "WI-44 — N-1 adopter upgrade acceptance"
description: "A reproducible post-release proof that an existing adopter can migrate explicitly and continue operating."
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: implementation-acceptance
capabilityClaims:
  - n_minus_one_upgrade
keywords: [work-item, release, upgrade, migration, adopter]
---

# WI-44 — N-1 adopter upgrade acceptance

## Intent and boundary

This Work Item establishes a post-release acceptance harness using only two
immutable public Release archives: the previous Runtime and the new Runtime.
It does not build source, run a workspace binary, mutate Release truth, or test
a second technology stack.

Runtime-only upgrades are expected to leave an attached repository unchanged.
When the Repository Protocol schema changes, the new Runtime must instead
report `MIGRATION_REQUIRED` and wait for an explicit, approved migration.

## Acceptance flow

`tests/release/adopter_upgrade_acceptance.sh` downloads and verifies both
archives, creates an isolated Cargo adopter, attaches it with the old Runtime,
records a real Work Item and evidence, then checks:

1. old schema state is detected by the new Runtime;
2. migration plan is read-only and unapproved apply fails closed;
3. approved migration writes a Runtime-bound receipt without changing old
   evidence bytes;
4. the new Runtime reaches `COMPATIBLE`, verifies Agent discovery, closes the
   old Work Item, and executes a new verification;
5. `acceptance.json`, runtime identities, isolation evidence, history digests,
   and `SHA256SUMS` are produced.

The script records `releasePublished: true` even when post-release acceptance
fails. A failed acceptance can never turn an already published Release into an
unpublished result or authorize reuse of its failed receipt.

## Reproduction

Run only after the new public Release exists:

```bash
tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --from-tag v0.1.1 \
  --to-tag v0.2.0 \
  --target aarch64-apple-darwin \
  --output ./release-adopter-upgrade-acceptance
```

The static test is runnable before publication:

```bash
bash tests/release/adopter_upgrade_acceptance_test.sh
```

The output is an acceptance artifact, not a pre-release gate. It must be
attached to the release evidence for the exact public tag and archive digest.
