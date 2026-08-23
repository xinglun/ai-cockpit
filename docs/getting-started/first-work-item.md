---
author: AI Cockpit maintainers
title: "First Work Item"
description: "A complete Runtime-native walkthrough from authorized Contract to reviewed close."
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - cli_lifecycle
---

# First Work Item

Use one Work Item, one dedicated branch/worktree, and one pull request. Start
from the latest commit on the repository's discovered remote default branch and
keep every repository-bound command explicit.

```bash
repo=/path/to/repository
id=WI-001-example-change
ai-cockpit start --repo "$repo" --id "$id" --intent "Make the bounded example change." --goal "Deliver reviewed evidence for the example." --scope 'docs/**' --out-of-scope 'src/**' --risk normal --authority authorized --acceptance "The documented example and registered checks pass." --required-evidence verification
```

Review the generated human-owned Contract. It must name the actual source,
scope, out-of-scope boundary, acceptance, verification, authority, remote,
default branch, and base revision. Never edit generated Summary, evidence,
Outcome, archive, or decision receipts by hand.

## Bind the real review resources before implementation

Commit the initial governance bytes, push the dedicated branch, and open a
draft pull request without merging it. Read the actual provider and Git facts;
do not invent a PR URL, branch, worktree, remote, or base branch. Put only those
facts in a temporary `ResourceFinalizationContext` file:

```json
{
  "branch": "feature/example-change",
  "worktree": "/absolute/path/to/worktree",
  "baseBranch": "main",
  "baseRemote": "origin",
  "provider": "github",
  "pullRequest": "https://github.com/owner/repository/pull/123"
}
```

Bind the reviewed context before preflight:

```bash
ai-cockpit work-item finalize-plan --repo "$repo" --id "$id" --input /tmp/WI-001.finalize-context.json
ai-cockpit preflight --repo "$repo" --contract .ai/work-items/active/WI-001-example-change.contract.json
```

If Preflight returns `not_ready` or `needs_human_confirmation`, stop and show
the review to the person. `verification_pending` may advance only to collect the
declared evidence. Record the single serial checkpoint, then implement only the
Contract scope:

```bash
ai-cockpit checkpoint --repo "$repo" --id "$id"
ai-cockpit verify --repo "$repo" --work-item "$id" --command cargo --args test,--workspace --workers 1
ai-cockpit finish --repo "$repo" --id "$id"
```

Use the Contract's project command; the Cargo command is only an example. After
the final edit, verification must be fresh for the same Work Item and snapshot.

## Deliver the visible Outcome, then archive

Render the person-facing handoff as a separate visible message:

```bash
ai-cockpit work-item outcome --repo "$repo" --id "$id"
```

The delivered text begins with `Outcome: 🟢`, `Outcome: 🟡`, or `Outcome: 🔴`
and includes status, unknowns, evidence, human decision, and next action. Only a
current green Outcome can proceed. A JSON lookup or folded tool result is not
the handoff.

```bash
ai-cockpit archive --repo "$repo" --id "$id"
```

## Finalize through merge and cleanup

Commit and push the archive bundle first. Re-read the provider PR after that
push and require a clean worktree. Then obtain a provider-derived receipt bound to the repository ID,
Work Item, Runtime version/digest, archived Contract digest, exact PR, branch,
worktree, and resource context. Before merge, the strict receipt is blocked with
reason `awaiting_merge_close`, an unmerged PR, present branch, clean worktree,
and `failureCodes: ["unmerged_pull_request"]`; it is not a retained success.

```bash
ai-cockpit work-item finalize --repo "$repo" --id "$id" --input /tmp/WI-001.premerge-finalize-receipt.json
```

The Runtime writes the canonical finalization receipt. Commit and push only that
receipt in the next governance commit; do not mix source, documentation, archive,
or other governance changes into this head advance. Require hosted checks, and
let the reviewed pull request merge. Do not merge into local `main` as a shortcut
and do not let the provider delete the branch before cleanup evidence exists.
After merge, point `--repo` at a surviving, fast-forward-synchronized checkout
of the default branch; the removed feature worktree cannot be the command root.
Append provider-derived merge-observation and exact cleanup receipts with
additional `work-item finalize` calls; receipts form an immutable linear chain:

```bash
repo=/path/to/synchronized-default-branch-worktree
ai-cockpit work-item finalize --repo "$repo" --id "$id" --input /tmp/WI-001.merge-observation-receipt.json
ai-cockpit work-item finalize --repo "$repo" --id "$id" --input /tmp/WI-001.cleanup-receipt.json
```

Then verify its unique terminal head:

```bash
ai-cockpit work-item finalize-verify --repo "$repo" --id "$id"
```

Only after the default branch is synchronized, the merged head is bound, the
worktree is clean, and the exact owned local/remote branch is deleted may the
authorized person record the structured close decision:

```bash
decision_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ai-cockpit close --repo "$repo" --id "$id" --human-decision approved --actor human:repository-owner --authority-source repository-review-policy --reason "Reviewed evidence and exact cleanup are complete." --evidence-ref ".ai/evidence/WI-001-example-change.verification.json" --policy-ref "repository-review-policy" --decided-at "$decision_time" --resume-condition none
```

Every failed or unknown transition stays open with its evidence and recovery
condition. Never delete or rewrite records to make the lifecycle look green.
The [Agent workflow reference](../reference/agent-workflow.md) defines the
provider/resource evidence boundary used by the receipt files above.

[Standard adoption guide](standard-adoption-guide.md) | [中文](first-work-item.zh-CN.md) | [日本語](first-work-item.ja.md)
