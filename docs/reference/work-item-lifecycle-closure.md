---
author: AI Cockpit maintainers
title: Work Item lifecycle closure
description: Safely close a reviewed Work Item after archive, merge, and exact cleanup.
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-512-reference-docs-batch-33
---

# Work Item lifecycle closure

[简体中文](work-item-lifecycle-closure.zh-CN.md) · [日本語](work-item-lifecycle-closure.ja.md)

Closure is the final handoff after `start → preflight → checkpoint → verify →
finish → archive`. It is not a branch-deletion shortcut. The reviewed PR, exact
Work Item head, archived Contract/Summary/evidence, synchronized base, clean
worktrees, and remote branch absence must all be proven by the Runtime.

## Normal route

```text
verify → finish/archive → push → reviewed PR and hosted checks → merge
→ finalize → finalize-verify → close → synchronize and clean
```

Run the repository-bound close command from the Work Item checkout or the
explicitly registered recovery checkout. It verifies PR state, branch and head
identity, base fast-forward synchronization, archive/decision receipts, clean
worktrees, and remote branch absence before deleting the exact local Work Item
branch. A provider must not auto-delete the branch to bypass this proof.

`ready_on_base` means the invoking checkout is clean and on the synchronized
default branch. `closed_but_current_worktree_detached` means closure succeeded
but another verified worktree owns the base; continue from that printed base
worktree instead of treating the detached checkout as ready.

## Recovery and historical boundaries

Any missing, stale, foreign, or contradictory fact fails closed and preserves
the retry identity. A provider anomaly or stacked-PR recovery uses a separate,
explicit receipt; it never rewrites an immutable archive or turns an open PR
into a merged one. Historical source `make` commands and Python orchestration
are not Runtime commands. The Rust route preserves the same intent—review,
archive, exact cleanup—with explicit `--repo` and repository-local evidence.

### Historical archive quarantine

An explicitly authorized `supersede` recovery may close an immutable archived
predecessor when a legacy optional Task Outcome Markdown artifact no longer
matches its archived manifest digest. The recovery receipt must bind the exact
archive-manifest bytes with `predecessorArchiveManifestDigest`. The Runtime
never rewrites the artifact or manifest; it records a `historical_low`
`historicalArchiveIntegrity` marker in the close receipt and projects the
historical item as yellow, not as a current green verification. Required
Contract/Summary/Outcome bytes, identity, events, and all other artifact
integrity checks remain fail-closed. A missing, foreign, malformed, symlinked,
or differently digested manifest cannot use this quarantine path.

## Successor and historical recovery

A blocked predecessor does not become successful because a corrective successor
exists. Its failed evidence remains immutable and is projected as historical.
`work-item recover` accepts only an identity-bound retry, successor, or
supersede receipt that names the predecessor, successor (when applicable),
repository, archived Contract/Summary/Outcome digests, authority, and reason.
Missing, stale, foreign, or unrelated receipts remain fail-closed; they cannot
mask an unrelated Contract or Summary error.

Legacy resource-finalization records have a separate read-only path:

```sh
ai-cockpit work-item finalize-recovery-plan --repo <path> --id <id>
ai-cockpit work-item finalize-recovery --repo <path> --id <id> --input <receipt.json>
```

For a historical direct merge with no provider PR, the plan may include the
real merge commit and parents. The resulting recovery is explicitly
historical/low-assurance and never invents a PR number or rewrites the old
receipt. The same rule covers shared-worktree `retained` history: the actual
resource disposition is recorded, not changed merely to satisfy a newer
Runtime.

Provider-only post-archive or stacked-PR anomalies are not ordinary closure
shortcuts. They require a separate, human-authorized, append-only evidence
boundary supplied by the provider; the Runtime still requires exact Work Item,
repository, branch/head, archive, and clean-base bindings. An open or
unverifiable PR is never treated as merged, and a provider-specific Make/Python
recovery command from the reference project is not a Rust command.

The final state is therefore both a lifecycle fact and a handoff: `ready_on_base`
means the invoking checkout is clean on the synchronized default branch;
`closed_but_current_worktree_detached` means closure succeeded elsewhere and
the printed base worktree must be used for the next Work Item.
