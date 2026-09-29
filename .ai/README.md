# AI Cockpit repository usage

Use the shared, externally installed `ai-cockpit` Runtime with an explicit
`--repo <repository>`; this `.ai/` directory is private to the repository.

## Read-only entry route

Read [`AGENTS.md`](../AGENTS.md); consult [`glossary.md`](glossary.md) when a
term or protocol is unclear. Pass the explicit repository path to Runtime:

```text
ai-cockpit inspect --repo <repository>
ai-cockpit status --repo <repository>
ai-cockpit doctor --repo <repository>
ai-cockpit agent doctor --repo <repository> --json
```

Load [`ordinary-work-item`](../agents/skills/ordinary-work-item.md) for ordinary
work; load recovery, Provider, or release guides only when Runtime facts and
Contract match. Runtime admission, blockers, and evidence freshness govern;
guides never grant actions.

## Repository-owned records

`.ai/agent-interface.json` declares the adapter; `.ai/project/` holds
repository declarations, `.ai/work-items/` lifecycle records, `.ai/evidence/`
bound evidence, and `.ai/decisions/` Runtime-generated decisions. Preserve
history; never hand-edit generated data or global Agent/MCP configuration.

## Runtime surface discovery

Use `ai-cockpit --help`, group help, and `capability show --repo <repository>`
before guessing. CLI/schema/capability metadata define the interface; see
[`docs/reference`](../docs/reference/README.md) for rationale and the
[task guide index](../agents/skills/README.md) for routing.

Use `work-item amend --request` for reasoned plan changes and separate
read-only check/explicit record actions for environment drift; only record
writes a shared event. See [commands](../docs/reference/commands.md),
[Contract fields](../docs/reference/contract-fields.md), and
[agent workflow](../docs/reference/agent-workflow.md).

For a resource-bound Work Item, the public boundary order is: archive → synchronize default branch → perform exact provider cleanup under the accepted plan → record finalize receipt → finalize-verify → close.
