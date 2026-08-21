# AI Cockpit repository usage

This repository uses one externally installed `ai-cockpit` Runtime. The binary
is shared; this `.ai/` directory is private to this repository. Never infer a
current repository or Work Item from process state, the working directory, or
Agent prose.

## Agent route

1. Read-only start: `ai-cockpit inspect --repo <repository>` and
   `ai-cockpit status --repo <repository>`.
2. Confirm readiness: `ai-cockpit doctor --repo <repository>` and
   `ai-cockpit agent doctor --repo <repository> --json`.
3. For a new repository, use `ai-cockpit attach --repo <repository>`; this
   creates only AI Cockpit-owned protocol state.
4. Agent discovery is explicit: use `ai-cockpit agent list/install/repair/detach
   --repo <repository> --provider <provider>`. Do not edit global Agent or MCP
   configuration and do not treat a managed prompt as governance authority.
5. Create a skeleton with `ai-cockpit work-item new --repo <repository> --id
   <id> --mode code`. Human-owned intent, scope, acceptance, and authority must
   remain empty or unknown until a person supplies them.
6. For an authorized Work Item, use `start → preflight → checkpoint → verify →
   finish → archive → close`. Every command carries `--repo`.

The Runtime has no global active Work Item, current repository, or project
profile. Repository Protocol, Contract, evidence, knowledge, and adapter
ownership records remain isolated under this repository's `.ai/`.

## Evidence discipline

Do not claim `green`, `passed`, `approved`, `verified`, or `completed` from this
file. Query the Runtime and read the current repository evidence. Missing,
stale, contradictory, or unknown evidence requires a rerun, human decision, or
stop condition.
