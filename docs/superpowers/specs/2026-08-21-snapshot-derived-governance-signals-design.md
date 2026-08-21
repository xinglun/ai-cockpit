# Snapshot-Derived Governance Signals Design

## Problem

The repository decision boundary currently passes `false` for untrusted
material, test weakening, and coverage weakening. The Core can govern those
facts, but CLI, MCP, finish, archive, and close never derive them from current
repository reality. Calling Git from three new checkers would violate the V2
`snapshot once → all gates reuse` architecture.

## Architecture

`cockpit-git::RepositorySnapshot` gains process-local `ChangeEvidence` facts.
The existing four Git calls remain the only Git observation boundary. The diff
call returns a zero-context patch rather than only name-status output; the
status call remains authoritative for changed paths and change kinds. Changed
file bytes already read for hashing are reused for bounded text inspection.

Change evidence records path, change kind, added and removed lines, bounded
after-text, and an explicit text/binary/too-large/deleted/unavailable state.
The inspection limit is 262,144 bytes per changed file. Evidence fields use
`serde(skip)` on `RepositorySnapshot`: CLI/MCP never serialize source text, and
the existing diff digest still binds the exact content.

`cockpit-repository::derive_governance_signals` is a deterministic, IO-free
consumer of the immutable snapshot. It detects strong repository instruction
injection, destructive test weakening, and coverage-policy weakening. A
relevant binary, oversized, or unavailable change produces an explicit unknown
rather than a false clear result.

`governance_decision_for_contract` is the sole adapter into `GovernanceInput`.
It supplies the three detected booleans and inspection unknowns, so CLI, MCP,
finish, archive, and close reuse identical semantics without rescanning Git.

## Detection Boundary

- Untrusted material requires both instruction-override language and a
  destructive, bypass, execution, credential, or exfiltration term. Repository
  prose remains data and cannot grant authority.
- Test weakening detects deleted test files, added skip/disable markers,
  material assertion removal, explicit success bypasses, and destructive test
  requests in changed text.
- Coverage weakening detects lowered `fail_under`/`threshold`/`minimum`, new
  `omit`/`exclude` entries, and removed `source`/`source_pkgs` values in known
  coverage configuration files.
- Ordinary test additions and refactors that preserve assertion strength do
  not produce weakening findings.

## Acceptance

1. No production governance signal remains hard-coded to `false`.
2. One immutable snapshot supplies every signal without increasing the four
   Git calls.
3. Security-test deletion is Red; coverage lowering and repository prompt
   injection are Yellow and require review.
4. Uninspectable relevant material is Yellow/Unknown, never silently clear.
5. Snapshot serialization and MCP observation do not expose raw change text.
6. CLI and MCP use the same central signal derivation.
