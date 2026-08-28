---
author: AI Cockpit maintainers
title: "WI-360 — Release adopter close cleanup"
workItemId: WI-360-release-adopter-close-cleanup
description: "修复 staged/N-1 adopter acceptance 的资源收尾与临时运行目录清理。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-360-release-adopter-close-cleanup
authority: canonical
---

# WI-360: Release adopter close cleanup

## Purpose

Make the staged and N-1 release adopter acceptance harnesses complete the
Runtime lifecycle without leaving a feature branch or worktree retained at
`close`.

## Scope

- `tests/release/adopter_acceptance.sh`
- `tests/release/adopter_upgrade_acceptance.sh`
- their static regression wrappers
- the three-language release distribution documentation

The harness remains post-release acceptance logic. It does not relax Runtime
resource-finalization rules and does not alter the immutable `v0.2.36` failed
release truth.

## Design

Each fixture uses a surviving control checkout and a dedicated lifecycle
checkout. After archive, the harness commits the generated archive records,
fast-forwards the control checkout, removes the exact lifecycle checkout and
branch, and records a `disposition: deleted` finalization receipt. `finalize`,
`finalize-verify`, and `close` then run from the surviving control checkout.

The EXIT trap still writes the acceptance receipt and checksums before removing
the validated temporary `run_root`. Failure and interruption paths preserve the
receipt, record cleanup state, and return non-zero if cleanup fails.

## Acceptance

- staged adopter lifecycle reaches `close` with `disposition: deleted`;
- both old and new N-1 lifecycle paths do the same;
- no lifecycle receipt claims an unperformed retained resource state;
- static tests reject retained close receipts and require branch/worktree
  deletion;
- tri-language documentation describes the control-worktree transition and
  the immutable failed `v0.2.36` staged history;
- source checkout and forbidden HOME/XDG roots remain unchanged;
- the exact temporary run root is removed on success and failure.

## Verification evidence

The release harness is validated independently with its static wrappers and
the published-artifact staged/N-1 acceptance jobs. The receipt records runtime
identity, repository identity, lifecycle outputs, isolation manifests, cleanup
state, and checksums. A failed post-release acceptance never rewrites Release
publication truth.
