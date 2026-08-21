# AI Cockpit glossary

- **Runtime** — the externally installed shared `ai-cockpit` executable.
- **Repository Context** — one request-scoped repository binding resolved from
  the explicit `--repo` path and repository-owned Protocol state.
- **Repository Protocol** — repository-local storage and identity contract under
  `.ai/`; it is not a copy of the Rust Runtime.
- **Agent Discovery / Adapter** — an explicit, owned, reversible provider
  integration that helps an Agent find the repository interface. It is not
  governance policy and does not prove compliance.
- **Work Item** — one bounded change with Contract, evidence, human decision,
  and lifecycle records.
- **Contract** — declared intent, goal, scope, authority, acceptance, and
  required evidence for a Work Item.
- **Snapshot** — an observed repository state used to bind decisions and
  evidence.
- **Receipt** — content-bound verification evidence that may be reused only
  when all authorized identity bindings still match.
- **Green / Yellow / Red** — proceed with sufficient evidence / investigate or
  obtain confirmation / stop because a required control failed.
- **Runtime-only upgrade** — a compatible executable change that does not
  modify repository-owned `.ai/` state.
- **Repository migration** — an explicit, reviewed, versioned change to
  repository Protocol or configuration state; it never rewrites historical
  Work Items or evidence.
