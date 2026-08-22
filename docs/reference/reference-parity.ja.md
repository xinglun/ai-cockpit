---
author: AI Cockpit maintainers
title: "Reference source parity"
description: "Rust runtime と reference AI Cockpit template の evidence-based 比較。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_parity
---

# Reference source parity

このページは Rust runtime と reference AI Cockpit product の capability boundary を
比較します。adopter と reviewer のための product reference であり、実装履歴は reader
route に含めません。

## Parity matrix

| Reference concern | Rust runtime status | Evidence と boundary |
| --- | --- | --- |
| Reader-first の入口と言語切替 | Implemented | root README は相互リンクし、adopter と maintainer の導線を分離している。 |
| Purpose、problem、architecture、capability overview | Implemented | `docs/philosophy*`、`docs/architecture*`、`docs/capabilities*` が Rust runtime と外部責任を説明。 |
| Shared Runtime と request-scoped repository context | Implemented | `docs/architecture/runtime-topology*`、明示的な `--repo`、repository isolation tests。 |
| Repository attach と minimum scaffold | Implemented | `attach`、`.ai/cockpit.toml`、`.ai/project.json`、`.ai/agent-interface.json`、attach tests。 |
| Explicit Agent Discovery / Adapter layer | Implemented | `agent list/install/doctor/repair/detach`、ownership 付き managed section、`.ai/adapters/<provider>.json`。`attach` は Agent file を変更しない。 |
| Work Item lifecycle と governance decision | Implemented | Contract、preflight、verification evidence、archive、close、human decision records。 |
| Bounded verification と fail-closed evidence reuse | Implemented | Runtime identity、snapshot/toolchain/environment binding、receipt store、workspace verification suite。 |
| MCP repository binding | Implemented | repository-bound stdio MCP service と CLI/MCP parity tests。 |
| Public Release と fresh-adopter acceptance | Implemented | 公開 binary harness、Release evidence、post-publication CI job を提供します。 |
| Runtime-only upgrade と repository migration | Implemented | `compatibility`、`migrate plan`、承認済み `migrate apply` が履歴 evidence を保持し Runtime identity を bind する。 |
| N-1 old-adopter upgrade acceptance | Public-artifact harness available | harness は旧 schema 検出、approval gate、履歴保持、継続動作を検証します。各 Release workflow でこの gate を自動化するかは明示的に定義します。 |
| Installation と provider configuration | External boundary | Rust binary を 1 つ配布し、install/provider configuration と repository state を分離。 |

## 完了している範囲

Rust implementation は現在の user-visible boundary を満たします。1 つの Runtime が
複数の独立した repository を治理し、repository state は分離され、Agent discovery は
明示的かつ ownership 付きで、decision は evidence に bind され、public Release
acceptance は繰り返し実行できます。Partial または Available と記載した項目は、
隠れた parity 完了宣言ではなく、明示された follow-up boundary です。

現在の project は意図的に `cockpit.toml` を TOML のまま保持します。reference
template の JSON project/profile record は必要な場所で Rust Protocol files として
表現されます。`cockpit.toml` を JSON に変更することは parity の対象ではありません。

## 現在の境界

Reader route、Runtime migration boundary、公開 artifact acceptance harness は実装・文書化済みである。
今後の変更でも、共有 Runtime upgrade、明示的 repository migration、repository-local
evidence の分離を維持する。
