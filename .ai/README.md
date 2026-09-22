# AI Cockpit repository usage

Use the shared, externally installed `ai-cockpit` Runtime with an explicit
`--repo <repository>`; this `.ai/` directory is private to the repository.

## Read-only entry route

Read [`AGENTS.md`](../AGENTS.md); consult [`glossary.md`](glossary.md) only for
unclear terms or protocol semantics. Runtime queries use the explicit path:

```text
ai-cockpit inspect --repo <repository>
ai-cockpit status --repo <repository>
ai-cockpit doctor --repo <repository>
ai-cockpit agent doctor --repo <repository> --json
```

For ordinary work, load [`ordinary-work-item`](../agents/skills/ordinary-work-item.md);
load recovery, Provider, or release guidance only when Runtime facts and the
Contract match. Runtime `safeActions`, blockers, evidence freshness, and
action explanation are authoritative; guides never grant an action.

## Repository-owned records

`.ai/agent-interface.json` is the adapter interface; `.ai/project/` holds
read-only repository declarations; `.ai/work-items/` holds lifecycle records;
`.ai/evidence/` holds identity-bound evidence; and `.ai/decisions/` holds
Runtime-generated decisions. Do not hand-edit these records or global Agent
and MCP configuration; preserve history and query-reported blockers.

## Runtime surface discovery

Use `ai-cockpit --help`, group help, and `ai-cockpit capability show --repo <repository>`
before guessing arguments. CLI, schema, and capability metadata
are mechanical fact sources; human rationale is in `docs/reference/`. See the
[task guide index](../agents/skills/README.md) for task entry points.

For a resource-bound Work Item, the public boundary order is: archive → synchronize default branch → perform exact provider cleanup under the accepted plan → record finalize receipt → finalize-verify → close.
