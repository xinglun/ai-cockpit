<!-- AI_COCKPIT_ADAPTER_BEGIN provider=codex adapterVersion=1 repositoryId=sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b -->

This repository is attached to AI Cockpit.

Canonical interface: `.ai/agent-interface.json`.

Use the repository-bound AI Cockpit Runtime as the governance interface. Do
not infer current state from this file, process state, the working directory,
or Agent prose. Query the Runtime with an explicit repository path.

<!-- AI_COCKPIT_ADAPTER_END -->

## Public boundary

Read `.ai/README.md`, `.ai/glossary.md`, and the current machine-readable
governance records before editing. The Contract is the human-owned source for
intent, scope, acceptance, authority, and changes to those decisions. The
Runtime is authoritative for current state, evidence validity, action
admission, blockers, and next-action explanations. Runtime output is authoritative for action admission; a guide recommendation is not permission.

Keep one active Work Item, branch, worktree, and repository context for a
change. Work only inside the active Contract. Preserve existing tests,
evidence, history, and generated records; do not hand-edit Runtime-generated
Contract, Summary, receipt, archive, decision, or status files. Do not edit
global Agent or MCP configuration, publish, push, merge, mutate tags, or
change provider resources unless the current Contract and the human request
explicitly authorize that work.

The installed Runtime is shared. Use its current help and capability surface
before guessing arguments, and pass `--repo <repository>` to every
repository-bound command. The normal read-only discovery is:

```text
ai-cockpit inspect --repo <repository>
ai-cockpit status --repo <repository>
ai-cockpit doctor --repo <repository>
ai-cockpit agent doctor --repo <repository> --json
```

If discovery or preflight reports missing authority, unknowns, contradictory
evidence, or a required human decision, stop and show that reason. Re-query
before an action after the Contract or repository snapshot changes. Query
paths are read-only and do not start verification or repair processes.

## Task guide routing

Load only the guide selected by the current Runtime facts and task:

- Ordinary implementation, verification, archive, and local cleanup: [ordinary-work-item](agents/skills/ordinary-work-item.md). This is the default route.
- An actual failed, timed-out, stale, or invalid verification result: [verification-failure-recovery](agents/skills/verification-failure-recovery.md).
- A Work Item with an explicitly bound external Provider resource: [provider-resource-finalization](agents/skills/provider-resource-finalization.md) only.
- An explicitly authorized release or upgrade acceptance: [release-upgrade-acceptance](agents/skills/release-upgrade-acceptance.md) only.

Load `provider-resource-finalization` only for a bound Provider resource and
load `release-upgrade-acceptance` only for an explicitly authorized release or
upgrade acceptance.

The default route does not require loading the Provider or release guides, and
the guides do not maintain a second action allow-list or state machine. Read
the [guide index](agents/skills/README.md) for the routing inputs and deeper
References. Runtime `safeActions`, blockers, evidence freshness, and action
explanation determine what may happen now.

## Outcome delivery

When work reaches a handoff boundary, deliver a separate visible human
Outcome beginning with `Outcome: 🟢`, `Outcome: 🟡`, or `Outcome: 🔴`. Use the
repository-bound `work-item outcome` or MCP Outcome handoff rather than a
machine record lookup alone. Include current status, issue count, blockers or
stopping reason, completed work, evidence, unknowns or risks, resolved issues,
the human decision required or recorded, verification, impact (mark an
unproven benefit as an inference), and next action.

Missing, folded-only, stale, yellow, red, contradictory, or malformed Outcome
evidence fails closed; direct human-visible delivery is required for a green
terminal.

Report implementation, external-resource cleanup, documentation projection,
and host delivery separately. Distinguish generated, returned, host-accepted,
and host-displayed Outcome states; when the host provides no display
confirmation, say that visibility is unknown. A log line, file link, or “done”
alone is not a complete Outcome.

## Stop and preservation rules

Never claim green, passed, verified, completed, released, or user-visible
from this file. Use current Runtime evidence. Preserve failed, stale,
unsupported, malformed, unknown, or contradictory evidence and keep the
blocking Outcome visible. If an in-scope defect is found, amend and revalidate
the current Contract before expanding work. Repair an in-scope defect in the
current Work Item before opening another Work Item or Issue when its scope,
authority, and base permit. Create a successor only for a genuinely different
scope, authority, or base, an unsafe repair, immutable failed delivery, or
explicit human direction.
