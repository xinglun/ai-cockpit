---
author: AI Cockpit maintainers
title: Governance profiles
description: Risk-based quality routing for Light, Standard, and Strict Work Items.
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-346-reference-governance-profiles-status
capabilityClaims:
  - risk_based_quality_routing
---

# Governance profiles

[English](governance-profiles.md) · [简体中文](governance-profiles.zh-CN.md) · [日本語](governance-profiles.ja.md)

AI Cockpit chooses a quality route from the repository facts, the Work Item
Contract, the stage, and the applicable policy. The route is proportional to
risk: `light < standard < strict`. Mixed changes use the highest applicable
route, and unknown or empty path evidence never lowers the route.

This page describes verification intensity. It is not an assurance claim and
it does not replace human authority.

## The three profiles

| Profile | Typical change | Target route |
| --- | --- | --- |
| `light` | Documentation, comments, non-executable examples, formatting-only changes | Focused quality checks |
| `standard` | Ordinary source, tests, bug fixes, and small refactors | Project verification plus any explicitly declared impact evidence |
| `strict` | Governance, CI, installer, security, dependency, destructive/public API, migration, calibration, or evidence-schema changes | Full repository and supply-chain checks |

`release` is an operation class, not a fourth profile. A release-owned
operation can add release-preflight, artifact, checksum, SBOM, provenance, and
adopter checks to the strict floor. A non-release strict change does not acquire
the release graph merely because it is strict.

## Profile effect and assurance

The route keeps these dimensions separate:

- `VerificationTier` (`T0`–`T3`) states how strong the verification must be.
- `EvidenceAssurance` (`SelfDeclared`, `RepositoryVerified`,
  `ProviderVerified`, `EnterpriseVerified`) states who or what can vouch for
  the evidence.
- Cost and reuse observations describe resource use. They are advisory and
  cannot lower a required route or turn unknown evidence green.

`T3` does not mean `ProviderVerified`, and `strict` does not mean
`EnterpriseVerified`. A tier or assurance requirement must be traceable to
Organization Policy, Project Policy, Release Policy, a protected gate, or an
explicit human-owned Contract. The planner may propose an escalation; it must
not hide policy inside a plan.

The reference template's static reference-impact scanner is not a Rust Runtime
capability in this release. The operation-time evaluator checks declared
operation, target, scope, authority, freshness, trust, and impact facts, but it
does not infer callers, dynamic references, external consumers, or monitoring
dependencies. A Standard route therefore does not silently claim that a
delete/rename/deprecation is safe: when such impact is relevant, the Contract
must declare the required evidence or the result remains `unknown`/human
review. See [operation-time policy re-evaluation](operation-time-policy-reevaluation.md)
and the [reference parity boundary](reference-parity.md).

Every route keeps the same mandatory control floor: scope, trust, lifecycle,
and evidence integrity. Optional heavy or cost-related checks are not
authorization or security switches. Unknown profiles, malformed policy,
unsafe paths, invalid bases, incomplete overrides, or removal of a mandatory
control fail closed.

## How a route is selected

The repository-bound route is evaluated before the command that it protects:

```text
repository snapshot + Contract + stage/policy
                 ↓
        `ai-cockpit gate --repo <path> --contract <file>`
                 ↓
      declared verification command / hosted gate
```

The route considers committed, staged, unstaged, and untracked paths relative
to the Contract base. The resulting receipt binds the repository, Work Item,
base/snapshot, selected profile, verification tier, assurance requirement,
reasons, and gate identity. The receipt is evidence of routing; it is not an
authorization token.

An explicit profile may raise the automatic result but may not lower it. A
downgrade requires an expiring, Work-Item-scoped human override with approval
evidence, reason, acknowledged risk, and a list of checks not run. It cannot
become a permanent exception.

## Session and repository boundaries

Quality report writers use a non-blocking lock local to the worktree. A second
invocation in the same worktree fails closed; separate worktrees remain
parallelizable. The shared Runtime has no current project or global active Work
Item. Every adopter repository passes an explicit `--repo`, and its Contract,
evidence, and adapter records remain private to that repository.

The source template's `make ai-cockpit-quality` and Python router are useful
conformance references, not commands that this Rust repository must copy. The
target's supported surfaces are the installed Runtime, explicit repository
context, typed Contract/verification records, and the repository's declared
CI gate. Local results are not silently promoted to hosted or enterprise
assurance.

## Safe reading of a result

Read the human handoff with
`ai-cockpit work-item outcome --repo <path> --id <work-item>`. A green route
means the named evidence can be reviewed; it does not authorize merge, release,
publication, or a security claim. Yellow means evidence or a decision is
incomplete. Red means a required control failed or the context is invalid and
work must stop. See [How to read Cockpit status](how-to-read-cockpit-status.md)
for the person-facing reading order.
