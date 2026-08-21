# Agent Discovery / Adapter Layer Design

**Date:** 2026-08-21  
**Work Item:** WI-39  
**Status:** Design baseline for review

## Goal

Add an explicit, repository-bound Agent Discovery / Adapter layer without
turning Runtime installation into repository attachment, adapter installation
into connection, or connection into governance compliance.

The three layers remain independent:

```text
Layer 1  Runtime installation
         one external ai-cockpit binary per machine

Layer 2  Repository attachment
         ai-cockpit attach --repo <target>

Layer 3  Agent discovery / adapter
         ai-cockpit agent install --repo <target> --provider <provider>
```

The Core remains request-scoped and stateless. Repository identity, protocol
state, adapter ownership, Contracts, evidence, knowledge, and profiles remain
repository-local.

## Non-goals and ownership boundaries

- `attach` does not install or edit Agent-provider files.
- No command writes `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursor/**`,
  `.codex/**`, user-home MCP settings, or other host-global configuration by
  default.
- An adapter is a bootstrap/discovery surface, not a prompt, governance rule,
  policy authority, evidence source, or Work Item state cache.
- Adapter text must say that the Runtime is authoritative and that Agents must
  query the Runtime; it must not encode Red/Yellow/Green or infer current
  state from stale text.
- MCP remains an optional northbound adapter. CLI calls the Core directly;
  CLI does not depend on MCP.
- Public Release, Homebrew/tap changes, and provider-global installation are
  outside WI-39.

## Canonical discovery facts

`.ai/agent-interface.json` remains the canonical repository-local discovery
surface created by `attach`. WI-39 may add explicitly versioned interface and
compatibility fields, but the file must continue to bind:

- the repository ID and manifest-parent root binding;
- the minimum Repository Protocol version;
- available CLI/MCP interfaces and their transport facts;
- capability names, without governance semantics; and
- an adapter compatibility/version marker.

Any schema extension must be strict and version-gated. Unknown fields, a
repository-ID mismatch, a root-binding mismatch, or an unsupported interface
version must fail closed rather than being silently ignored.

Adapters must locate the canonical manifest first, resolve its parent as the
repository root, read the repository ID, and then call the Runtime with that
explicit repository context. Provider adapters must not independently guess a
root from `cwd` or from an arbitrary project file.

## Adapter contract

The repository layer exposes one provider-neutral contract. A concrete
provider implements discovery and produces a plan before any write:

```rust
trait AgentAdapter {
    fn provider(&self) -> AgentProvider;
    fn detect(&self, repo: &RepositoryContext) -> DetectionResult;
    fn plan_install(&self, repo: &RepositoryContext) -> AdapterPlan;
    fn verify(&self, repo: &RepositoryContext) -> AdapterStatus;
    fn plan_repair(&self, repo: &RepositoryContext) -> AdapterPlan;
    fn plan_detach(&self, repo: &RepositoryContext) -> AdapterPlan;
}
```

`AdapterPlan` must identify every target path, operation, current digest,
expected digest, repository ID, and conflict. A plan with an unresolved
conflict is not executable. Installation is explicit and atomic; repeated
installation of the same managed content is byte-stable.

The first provider set is deliberately small and uses shared managed-section
logic wherever the host format permits it: `generic-agents-md`, `codex`,
`claude`, `gemini`, and `cursor`. A provider-specific global configuration is
never inferred from detection and is not written by `auto`.

## Ownership and safe mutation

Each installed adapter has a repository-local ownership record:

```text
.ai/adapters/<provider>.json
```

The strict record contains at least `provider`, `adapterVersion`, `target`,
`mode`, `repositoryId`, and `installedDigest`. Managed text is enclosed by
stable provider/version/repository markers. The record and managed section
must agree before the adapter is considered installed or verified.

Install rules:

1. Detect first and return a plan; never overwrite unrelated content.
2. Refuse duplicate markers, malformed markers, symlink/reparse targets, and
   repository-ID mismatches.
3. Use no-follow, handle-relative/atomic writes where the platform supports
   them, and preserve unrelated user content byte-for-byte.
4. `--provider auto` detects safe candidates and installs only the explicitly
   selected safe plans; it never means “modify every discovered file.”

Detach rules:

- If the current managed-section digest equals `installedDigest`, remove only
  the owned section and ownership record.
- If the section changed, is duplicated, or no longer matches the target,
  refuse automatically with a conflict and require a reviewed recovery path.
- There is no `--force` shortcut in WI-39.

Repair follows the same plan/verify/ownership rules. It never overwrites a
user-modified managed section without an explicit, separately reviewed
recovery workflow.

## State model and exit codes

`agent doctor` derives state from current repository facts; it does not trust a
cached state string:

```text
UNATTACHED
ATTACHED
DISCOVERY_AVAILABLE
ADAPTER_INSTALLED
CONNECTED
VERIFIED
DEGRADED
CONFLICT
```

The state is not required to be linear. For example, CLI can be verified while
MCP is merely available; MCP failure may produce `DEGRADED` rather than block
CLI use. `CONFLICT` always requires human intervention.

The JSON doctor result includes `schemaVersion`, `state`, `repositoryId`,
attachment/manifest status, adapter statuses, interface statuses, `problems`,
and `safeActions`. It never claims `VERIFIED` unless a repository-bound probe
returns the expected repository ID.

Standard process exit codes are:

```text
0  READY / VERIFIED
1  DEGRADED but usable
2  configuration or evidence error
3  human intervention required
4  incompatible protocol
```

## CLI and MCP surface

All commands require explicit repository context:

```text
ai-cockpit agent list   --repo <repo>
ai-cockpit agent install --repo <repo> --provider <provider|auto>
ai-cockpit agent doctor  --repo <repo> [--json]
ai-cockpit agent repair  --repo <repo> [--provider <provider>]
ai-cockpit agent detach  --repo <repo> --provider <provider>
```

The output must distinguish facts, available safe actions, conflicts, and
human decisions. `agent doctor` is the acceptance command for this layer.

MCP configuration export may be added only as an explicit, provider-specific
command. WI-39 does not modify Claude Desktop, Cursor, Codex, or other global
MCP configuration. A repository-local provider configuration is allowed only
when the provider contract explicitly supports it and ownership is recorded.

## Skill boundary

Provider skills/rule packs are optional usability enhancements. They may
explain when to call `status`, `knowledge query`, or `doctor`, but they are not
Runtime, Repository Protocol, Evidence, or Governance Authority. A skill being
installed does not imply discovery, connection, verification, or compliance.
Skill content is not copied into canonical `.ai` facts.

## Verification strategy

The implementation plan must include tests for:

- explicit repository binding and A/B parallel isolation;
- manifest resolution, repository-ID binding, strict unknown-field handling,
  and unsupported-version failure;
- detection-only versus explicit installation behavior;
- idempotent managed-section installation and preservation of unrelated text;
- duplicate markers, modified sections, symlink/reparse targets, and detach
  fail-closed behavior;
- doctor state transitions, JSON output, repository-bound probe, and exit
  codes;
- CLI operation when MCP is unavailable;
- no writes to Agent-global files/configuration during `attach` or `auto`;
- all three language documentation counterparts and command examples; and
- the existing workspace quality, Windows runtime, and V1 Oracle gates.

No code, provider file, global configuration, or public installation is changed
until this design and its implementation plan are approved as WI-39.
