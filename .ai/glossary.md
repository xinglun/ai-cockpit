# AI Cockpit glossary

- **Runtime** — the externally installed shared `ai-cockpit` executable. It
  observes repository state and evaluates evidence; it does not invent human
  authority.
- **Repository Context** — one request-scoped repository binding resolved from
  an explicit `--repo` path and repository-owned Protocol state.
- **Work Item** — one bounded change with a human-owned Contract, evidence,
  decision, and lifecycle records.
- **Contract** — the Work Item declaration of intent, scope, out-of-scope
  boundaries, authority, acceptance, sources, verification, and required
  evidence.
- **Evidence** — an identity- and freshness-bound fact that may be accepted,
  invalidated, or require a human decision.
- **Preflight Review** — the Runtime review of the Contract and current
  snapshot before an action. A warning or successful command is not itself
  authorization.
- **Human Decision** — an auditable decision supplied by a person or an
  explicitly authorized authority; a recommendation is not a decision.
- **Outcome** — the visible human handoff containing status, evidence,
  unknowns, decision, and next action with a 🟢/🟡/🔴 marker.
- **Green** — sufficient current evidence for the admitted step.
- **Yellow** — investigate, preserve evidence, or obtain confirmation.
- **Red** — a required control failed; stop and keep the failure visible.

For protocol semantics, operation details, and compatibility notes, consult
the relevant [Reference](../docs/reference/README.md) page or the [task guide
index](../agents/skills/README.md). This glossary is not a second state
machine or command inventory.
