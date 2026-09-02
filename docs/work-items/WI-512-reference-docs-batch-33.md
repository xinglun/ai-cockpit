---
author: AI Cockpit maintainers
title: "WI-512 — governance reference documentation batch 33"
description: "A bounded, one-by-one comparison of governance and verification-boundary reference pages."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-512-reference-docs-batch-33
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-512-reference-docs-batch-33
terminalArchive: .ai/work-items/archive/WI-512-reference-docs-batch-33.contract.json
terminalVerification: .ai/evidence/WI-512-reference-docs-batch-33.verification.json
terminalFinalization: .ai/decisions/WI-512-reference-docs-batch-33.finalize.json
terminalDecision: .ai/decisions/WI-512-reference-docs-batch-33.close.json
---

[简体中文](WI-512-reference-docs-batch-33.zh-CN.md) · [日本語](WI-512-reference-docs-batch-33.ja.md)

## Goal

Read the pinned local reference pages one by one and record a bounded Rust-native
counterpart for each. This Work Item preserves semantic responsibility while
explicitly refusing source Python, Shell, Make, provider, installer, wizard, or
JSON-wire copying.

## File-by-file decisions

| Pinned source path and digest | Classification | Rust counterpart / non-claim |
| --- | --- | --- |
| `docs/reference/schemas.md` — `4ed6c44bfcfea93300c39fa467170902932e4371d218f09269bed9da26fbf625` | implemented-different-by-design | `docs/reference/schemas.*`, typed Protocol and schema tests; source registries are not wire requirements. |
| `docs/reference/test-architecture.md` — `3c475a84e6b7634c6d98c44af029d6b01aff6a36da5649195d1daa1d52d2a82f` | implemented-different-by-design | tri-language test architecture, dynamic quality route and governance gates; tier is not assurance. |
| `docs/reference/test-weakening-guard.md` — `17824614224f43bde778ab3985d1abc42d6c53ad0b5a5a26d3fc371e25a3ba7c` | implemented-different-by-design | Rust governance signals/regressions and tri-language guard docs; source implementation is not copied. |
| `docs/reference/test-weakening-guard.zh-CN.md` — `9b5b06cc25f0e05443a3b5b9181b3a04c076a9e67c5d8f17c02f2f45412f548e` | implemented-different-by-design | Chinese presentation of the same typed guard boundary; locale does not grant authority. |
| `docs/reference/test-weakening-guard.ja.md` — `0ba5c1fd600990111a1942dcd48e4e4bda5903f6119fc51bbd67f0ffd7702b76` | implemented-different-by-design | Japanese presentation of the same typed guard boundary; locale does not grant authority. |
| `docs/reference/verification-fixture-boundary.md` — `712ecf6a4aed8793464b40ac41cb8b9d19a47663da5c28b61403e14552990f1e` | implemented-different-by-design | tri-language fixture boundary and isolation/adopter manifests; source helper bytes are not copied. |
| `docs/reference/troubleshooting.md` — `57f2415177d9135c506ef9c325dd7dc8bb989ee4801907da173bac5df640dee3` | implemented-different-by-design (WI-504 revalidated) | explicit `--repo` Runtime recovery; provider wizard and toolchain remain external. |
| `docs/reference/troubleshooting.ja.md` — `0addca04e66d0118311cf7a169b8dd060d42b500c265f85496b014300749bbf9` | implemented-different-by-design | Japanese recovery and adapter boundary; source session implementation is not copied. |
| `docs/reference/upgrade.md` — `3ebbb05b52a281c1974dc446e6707fc8cbd5f3fddd2897f6c8bf868133ac92f4` | implemented-different-by-design | shared Runtime upgrade versus explicit repository migration, immutable evidence. |
| `docs/reference/upgrade.ja.md` — `48367289304c82e14a7ace646092b6f115b1c18b4ee01ab96122f41255ec01e9` | implemented-different-by-design | Japanese upgrade/migration presentation; source installer and locale JSON are not copied. |
| `docs/reference/work-item-lifecycle-closure.md` — `91fab8d045cd45eeb616d768e174ba840b5c5f7ba1a0a4a819065515822ad324` | implemented-different-by-design (WI-504 revalidated) | Rust finalize/close, recovery and `ready_on_base`; source Make/Python recovery is not a Rust command. |
| `docs/reference/work-item-lifecycle-closure.ja.md` — `e47bdb794178855b2f4dad4b40fc6d5ee4f150d1c2c58a3f69eb897835593ea3` | implemented-different-by-design | Japanese closure and historical recovery boundary; provider-specific routes remain external. |

The target and adopters inherit the shared external Runtime, isolated repository
context, Contract/evidence/knowledge records, Agent adapter boundary, and human
Outcome handoff. They do not inherit source-specific commands, provider policy,
generated history, or source wire shapes. The two WI-504 paths are revalidated
without duplicating inventory records; the other ten have current WI-512 records.

## Acceptance and verification

- Each row was read at source commit `fde3380f81fea5fd2e288f7a8849f737dc074060`.
- Every row has a classification, counterpart, and explicit non-claim; no
  `migrate-gap` or deferred record is introduced by this batch.
- The tracked inventory remains 5,119 paths and retains its 669 retired-path
  ledger; no source or object repository is changed.
- Tri-language comparison, parity, documentation, inventory, and workspace
  checks are run from the Contract and recorded in the terminal evidence.

This is semantic/documentation parity. It is not source command compatibility,
Python module compatibility, or JSON-wire compatibility.
