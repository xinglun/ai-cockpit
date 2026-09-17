# WI-881 Interface Discovery Design

## Status

Approved for implementation under `WI-881-interface-discovery`. This is a
narrow interface-discovery trial inspired by BAML commit
`6ed9c768a56b7bc626b09846122ff5074aa903cf`; it does not import BAML or copy
its runtime.

## Problem and boundary

The `work-item outcome` surface currently has facts in several places: the
Clap command definition, the MCP JSON schema, the protocol types, and human
reference pages. The existing Agent first-start document is already embedded
with `include_str!` and installed through an ownership record. The problem is
discoverability and drift between these representations, not lifecycle
behavior or the Outcome model.

This WI therefore keeps the existing `capability show` discovery entry and
adds one explicit `work-item-outcome` surface. It will not add a new approval
gate, change `work-item outcome` defaults, alter archive delivery, scan Work
Item history, or run verification merely to describe an interface.

## Design

### One structured fact projection

`cockpit-protocol` owns a versioned `InterfaceDescription` projection for the
trial. It contains the surface name, description schema version, Runtime
version, and per-surface parameters for CLI and MCP. Each parameter records
its stable name, wire type, requiredness, default, enum values, aliases, and a
short factual description. Shared facts such as `view = summary|full` and
`delivery = false` are constructed once and projected into both surfaces.

The projection is pure and deterministic: it uses compile-time Runtime
version metadata, no clock, network, repository traversal, or subprocess. It
does not infer authorization, recovery, or lifecycle constraints from Rust
types. Those rules remain in the existing authoritative guidance.

### Existing discovery entry points

CLI keeps `ai-cockpit capability show --repo <path>` unchanged when no surface
is supplied. With `--surface work-item-outcome`, it emits the structured JSON
description or a deterministic Markdown fragment. This branch returns before
repository compatibility/observation work, so description is read-only and
cheap.

MCP keeps `capability_show` unchanged with no arguments. Its optional
`surface`, `format`, and `language` arguments request the same structured
description or localized Markdown. The handler calls the same protocol
projection; it does not maintain a second lifecycle or recovery rule.

### Human documentation

The three command reference pages receive a clearly delimited generated
interface-facts region. The renderer produces the region from the structured
projection; surrounding examples, rationale, lifecycle rules, and capability
limits remain human-authored. A deterministic parity test renders each
language twice and checks the committed region, so changing a shared fact
requires one source change plus the generated-region refresh rather than
three hand-maintained tables.

Agent first-start guidance will point to the discovery command and state that
it is descriptive, not authorization. Because the installed adapter embeds
the same first-start document, no new adapter format or ownership mechanism is
introduced. Existing doctor/install tests remain the authority for current,
owned-old, modified, and unknown-ownership states; if those behaviors already
pass, this WI records the no-code-change reason rather than duplicating them.

## Data flow

```text
protocol InterfaceDescription
        ├── CLI capability show (JSON / Markdown)
        ├── MCP capability_show (JSON / Markdown)
        ├── parity tests against Clap and MCP validation
        └── generated facts region in commands.{md,zh-CN.md,ja.md}
```

The actual `work-item outcome` execution path continues to use its existing
parser, renderer, guarded observation, delivery, and receipt functions. Tests
compare those runtime definitions with the description; the description does
not replace execution or grant permission.

## Error and compatibility behavior

Unknown surfaces or formats fail with a bounded argument error and do not
write repository state. Existing no-argument capability output is byte- and
schema-compatible. The description schema is additive and versioned; a
consumer that does not understand the surface can continue using the existing
capability registry. Localization affects labels and prose only, never the
parameter names, values, defaults, or governance facts.

## Verification

Focused tests will cover CLI parser parity, MCP schema/validation parity,
deterministic JSON/Markdown generation, generated-region protection, and
zero subprocess/repository-history work for discovery. Existing Agent doctor
and install tests will be run unchanged plus a first-start discovery assertion
where needed. Workspace verification uses the Contract's isolated target
policy with `CARGO_INCREMENTAL=0`.

## Explicit non-goals

- No BAML dependency, DSL, or external network access.
- No lifecycle, authorization, verification, receipt, or Outcome delivery
  changes.
- No full documentation migration or automatic translation of human prose.
- No changes to Sentinel or any object repository.
- No Rust toolchain/dependency upgrade and no release in this WI.
