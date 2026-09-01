---
author: AI Cockpit maintainers
title: Repository workflow
description: The repository-scoped Work Item, review, archive, and cleanup workflow.
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - repository_workflow
---

# Repository workflow

[English](repository-workflow.md) · [简体中文](repository-workflow.zh-CN.md) · [日本語](repository-workflow.ja.md)

AI Cockpit uses one Work Item, one dedicated branch/worktree, and one reviewed
Pull Request for a bounded change. The installed Runtime is shared by
machines, but Contract, evidence, and repository identity are request-scoped.

## Start to review

1. Fetch the latest remote default branch and record its remote, branch, and
   revision in the Contract.
2. Create a dedicated linked worktree and branch from that revision.
3. Run `ai-cockpit start --repo <worktree> --id <id> --intent <text> --goal <text>`
   with explicit scope, out-of-scope boundaries, authority, acceptance, and
   required evidence.
4. Run preflight and checkpoint. A yellow or red result is a visible review
   condition; it is not permission to edit or finish.
5. Change only the declared scope. Record verification with explicit argv and
   the same `--repo`, then run `finish` and `archive`.
6. Push the exact branch, open one reviewed PR, and wait for its required
   hosted checks. Do not merge into local `main` as a substitute for review.

### Finalization context is explicit

The `resourceContext` written by `start` is provisional until an explicit
`work-item finalize-plan` binds the reviewed PR, provider, base, branch, and
worktree. Both `pending` and `pending:<stable-reference>` are provisional
sentinels; neither can authorize `finish` or `archive`. Run
`finalize-plan` with the real reviewed resource before those terminal steps.

## Repository-wide serial boundary

Before writing a new Contract, the Runtime checks every linked worktree for an
active Contract/Summary pair. An active item in another non-detached worktree,
or an incomplete pair, blocks a new item. A replacement does not silently
terminate its predecessor; use an explicit recovery/supersede decision and
preserve the predecessor bytes.

## Merge, close, and cleanup

The delivery order is:

```text
latest remote default base → dedicated branch/worktree → implement
→ verify/finish/archive → reviewed PR → merge → finalize-verify → close
→ synchronize default branch → remove exact branch/worktree
```

Do not delete a branch before its PR is merged, and do not let provider-side
auto-delete bypass finalization. New Work Items require a structured human
decision, archived evidence, the merged PR identity, a deleted finalization
receipt, a fast-forward-synchronized default branch, and clean worktrees. A
verified historical shared-worktree or direct-merge receipt may use the narrow
`retained` exception with `historical_low` assurance and explicit repository-
bound Git facts; it never applies to new Work Items or upgrades historical
evidence. Any failed postcondition remains visible and fail-closed.

Immediately after close, synchronize the documentation projection:

```sh
python3 tests/docs/promote_closed_work_item.py --repo <repository> --check-all
```

If the check reports stale documentation, create a narrow documentation-
promotion Work Item, run the helper, and rerun the check before claiming
`ready_on_base`. The helper updates reader-facing status/parity only; it does
not rewrite Contract, evidence, archive, or decision history.

## Recovery and adoption

Recovery is append-only and identity-bound. A changed snapshot, stale receipt,
or provider conflict must be recorded as a retry, successor, or supersede
decision; old evidence is not edited to make a later state green. Installation,
upgrade, adapter setup, and historical finalization recovery are separate
repository operations and use an immutable public Release where applicable.
`work-item finalize-recovery --repo <path> --id <id> --input <receipt.json>` is
the only compatibility path for an immutable legacy finalization: it binds the
predecessor digest, repository/Work Item/Contract base, current Runtime, actor,
authority, reason, and timestamp without editing the predecessor. No command
selects a process-wide current project, and no provider-global Agent or MCP
configuration is modified.

New Runtime-created successors must carry the exact predecessor Work Item,
Contract digest, recovery path, and repository bindings. For a historical
successor created before those Contract fields existed, Runtime permits a
narrow compatibility path only when the recovery receipt itself binds the
predecessor and successor and the successor has a verified archive, strict
verification evidence, and a confirmed close decision. The resulting
append-only recovery receipt is marked `successorBindingMode:
legacy_terminal_evidence`; missing, foreign, stale, malformed, symlinked, or
incomplete evidence remains `recovery_decision_invalid` and cannot authorize a
transition. This compatibility projection never turns an unfinished successor
green and never rewrites predecessor bytes.

A predecessor has one selected successor lineage. After a valid `successor`
receipt exists, another `successor` decision for a different Work Item is
rejected with the stable `recovery_decision_invalid:competing_successor`
boundary. Continue or explicitly `supersede` the selected lineage instead;
never leave competing successors for a human to resolve from filenames. This
rule makes recovery graphs deterministic and keeps every predecessor's
terminal decision auditable without rewriting its historical bytes.

When an archived predecessor contains an older successor attempt whose target
was never bound, a newer valid `supersede` receipt may resolve that historical
residue. Runtime only treats the older record as historical when the newer
receipt is valid and wins by its recorded decision time; malformed, foreign,
tampered, or newer-invalid records remain fail-closed. No Contract, Summary,
Outcome, Events, Evidence, or recovery receipt bytes are rewritten.

This is a semantic Rust-native workflow. The reference source's `make`
commands, Python modules, and generated history are comparison material, not
commands or Runtime authority in this repository.
