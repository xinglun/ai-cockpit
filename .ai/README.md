# AI Cockpit repository usage

This repository uses one externally installed `ai-cockpit` Runtime. The
binary is shared; this `.ai/` directory is private to the repository. Never
infer a current repository or Work Item from process state, the working
directory, or Agent prose.

## Read-only entry route

Read [`AGENTS.md`](../AGENTS.md), then consult [`glossary.md`](glossary.md) only
when a term or protocol meaning is unclear. Use the explicit repository path
for Runtime queries:

```text
ai-cockpit inspect --repo <repository>
ai-cockpit status --repo <repository>
ai-cockpit doctor --repo <repository>
ai-cockpit agent doctor --repo <repository> --json
```

For ordinary work, load [`ordinary-work-item`](../agents/skills/ordinary-work-item.md).
Load the recovery, Provider, or release guide only when the Runtime facts and
Contract match that guide's applicability. Runtime `safeActions`, blockers,
evidence freshness, and action explanation are authoritative; guides explain
how to perform an admitted action and never grant one.

## Repository-owned records

- `.ai/agent-interface.json` is the canonical adapter interface.
- `.ai/project/` contains optional repository declarations consumed as
  read-only Runtime inputs.
- `.ai/work-items/` contains Contract, Summary, lifecycle, and archive
  records. Runtime commands generate these records.
- `.ai/evidence/` contains identity-bound verification and delegated evidence.
- `.ai/decisions/` contains Runtime-generated observation and decision records.

Do not hand-edit generated Contract, Summary, receipt, archive, decision, or
status records; do not edit global Agent or MCP configuration. Keep historical
records unchanged. If a query reports missing authority, unknowns,
contradiction, stale evidence, or a required human decision, preserve the
reason and stop until the Contract or human decision resolves it.

## Runtime surface discovery

Use `ai-cockpit --help`, the relevant group help, and
`ai-cockpit capability show --repo <repository>` before guessing arguments.
The current CLI, protocol schema, and capability metadata are the sources for
mechanical command facts. Human guidance and design rationale live in
`docs/reference/`; see the [task guide index](../agents/skills/README.md) for
the small task-specific entry points.

## Outcome boundary

Use the repository-bound `work-item outcome` command or the MCP Outcome
handoff for the visible human result. The Outcome must preserve current
status, evidence, unknowns, decision, and next action; a machine lookup,
log, or file link is not a substitute. Distinguish generated, returned,
host-accepted, and host-displayed states when display confirmation is absent.
