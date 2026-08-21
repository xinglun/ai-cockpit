---
author: AI Cockpit maintainers
title: "Reference source parity"
description: "Rust runtime と reference AI Cockpit template の evidence-based 比較。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: wi-41-reference-parity
capabilityClaims:
  - reference_parity
---

# Reference source parity

このページは `xinglun/ai-cockpit` と reference source
`spirex-ds-dev/ai-cockpit-template` の比較を記録します。比較した reference
snapshot は `e5acb67`、Rust runtime の基準は `031f67d` です。

これは境界の監査であり、reference implementation のコピーではありません。
Rust project は独立した V2 runtime であり、V1 の Python module、Makefile helper、
repository state は install しません。

## Parity matrix

| Reference concern | Rust runtime status | Evidence と boundary |
| --- | --- | --- |
| Reader-first の入口と言語切替 | Partial、WI-41 で修正 | root README は相互リンク済み。完全な reader route と内部履歴の整理は WI-42。 |
| Purpose、problem、architecture、capability overview | Implemented | `docs/philosophy*`、`docs/architecture*`、`docs/capabilities*` が Rust runtime と外部責任を説明。 |
| Shared Runtime と request-scoped repository context | Implemented | `docs/architecture/runtime-topology*`、明示的な `--repo`、repository isolation tests。 |
| Repository attach と minimum scaffold | Implemented | `attach`、`.ai/cockpit.toml`、`.ai/project.json`、`.ai/agent-interface.json`、attach tests。 |
| Explicit Agent Discovery / Adapter layer | Implemented | `agent list/install/doctor/repair/detach`、ownership 付き managed section、`.ai/adapters/<provider>.json`。`attach` は Agent file を変更しない。 |
| Work Item lifecycle と governance decision | Implemented | Contract、preflight、verification evidence、archive、close、human decision records。 |
| Bounded verification と fail-closed evidence reuse | Implemented | Runtime identity、snapshot/toolchain/environment binding、receipt store、workspace verification suite。 |
| MCP repository binding | Implemented | repository-bound stdio MCP service と CLI/MCP parity tests。 |
| Public Release と fresh-adopter acceptance | Implemented | WI-40 harness、public `v0.1.1` evidence、post-publication CI job。 |
| Runtime-only upgrade と repository migration | Not yet implemented | versioning docs は境界を示すが `compatibility` / `migrate` command はない。WI-43 で実装。 |
| N-1 old-adopter upgrade acceptance | Not yet implemented | fresh-adopter acceptance はあるが、old-adopter matrix は WI-44。 |
| Reference installer、Makefile、V1 helper scripts | Intentionally not copied | Rust binary を配布し、install/provider configuration と repository state を分離。 |
| Reference の historical Work Item と internal progress plan | Product capability ではない | WI-42 で reader route から内部履歴を外し、archive evidence は Git で監査可能に保つ。 |

## 完了している範囲

Rust implementation は reference product の主要な user-visible boundary を
満たしています。1 つの Runtime が複数の独立した repository を治理し、repository
state は分離され、Agent discovery は明示的かつ ownership 付きで、decision は
evidence に bind され、public Release acceptance は繰り返し実行できます。

現在の project は意図的に `cockpit.toml` を TOML のまま保持します。reference
template の JSON project/profile record は必要な場所で Rust Protocol files として
表現されます。`cockpit.toml` を JSON に変更することは parity の対象ではありません。

## 次の Work Item

- WI-42：reader-first の documentation home、内部進捗の削除、三言語 route の正規化。
- WI-43：明示的な Runtime compatibility gate と versioned repository migration
  protocol を追加し、historical evidence は書き換えない。
- WI-44：N-1 old-adopter upgrade を Release evidence matrix に追加。

これらは documentation navigation、runtime/repository behavior、release acceptance
という異なる authority surface を変更するため、別々の Work Item とします。
