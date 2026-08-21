# Governance Scaffolding Without Governance Decisions

## Goal

Introduce repository and Work Item scaffolding without allowing the Runtime,
the Agent, or a discovery rule to invent intent, authority, acceptance, or a
positive governance state.

## Architecture

The installed Rust Core remains one shared, stateless Runtime. Every command
receives an explicit `--repo` and creates a request-scoped Repository Context;
there is no process-level current repository, active Work Item, or global
profile. `attach` owns only the minimum repository protocol scaffold and a
stable repository identity. It does not install provider instructions, mutate
`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursor/**`, or global MCP settings.

The Repository Context owns `.ai/` state. `attach --repo <path>` creates the
minimum directories and protocol files, including `agent-interface.json` as a
discovery manifest. The manifest is a repository-local fact: it identifies the
repository and available Runtime capabilities, but it is not an Agent prompt,
provider configuration, or authorization grant.

`work-item new --repo <path> --id <id> --mode <mode>` and the transitional flat
`start` command call one shared contract-scaffold API. The new command writes a
validator-readable active Contract whose deterministic fields come from the
current Repository Snapshot and attached profile. Human fields remain empty or
`unknown`; the state is `not_ready`. The scaffold never writes `passed`,
`approved`, `verified`, or `completed`.

`profile propose --repo <path>` derives a candidate amendment from the current
snapshot and observed profile. It is read-only by default and labels its output
`candidate`/`proposed`. An explicit future apply flow is the only operation that
may change the formal profile baseline and its digest.

## Deterministic versus human-owned fields

The scaffold may fill `repositoryId`, `baseRevision`,
`projectProfileDigest`, and `repositorySnapshotDigest`. It must not fill
`intent`, `scope`, `acceptanceCriteria`, or `authority`; those values are
`""`, `[]`, or `"unknown"` until a person supplies them. Any validator result
with missing human inputs is `not_ready`/`unknown`, never green.

## Idempotence and isolation

Repeated attach on one repository preserves all identity and manifest bytes and
does not create meaningless changes. Two repositories may be attached and
scaffolded concurrently; their repository IDs, snapshots, profiles, Contracts,
and evidence paths must remain independent. The Core must not use an ambient
working directory or a global mutable project binding to select state.

## Acceptance

1. Attach is idempotent and creates only the minimum Runtime scaffold.
2. A/B parallel attach and Work Item creation produce isolated identities and
   files.
3. New Work Item skeletons contain all four deterministic facts and no invented
   human decisions or positive completion state.
4. Existing Contract validation can read a skeleton and reports `not_ready` or
   `unknown` while human fields are missing.
5. Profile proposals cannot alter formal baseline bytes/digest and are marked
   candidate/proposed.
6. No provider files or global MCP configuration are changed.
7. English, Chinese, and Japanese user documentation describe the same
   scaffold/decision boundary and next-step output.
