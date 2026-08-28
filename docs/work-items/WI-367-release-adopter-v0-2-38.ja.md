---
author: AI Cockpit maintainers
title: "WI-367 — v0.2.38 public Release adopter acceptance"
workItemId: WI-367-release-adopter-v0-2-38
description: "不変な公開 v0.2.38 artifact を隔離された新しい adopter repository で受け入れ、再現可能な evidence baseline を保持する。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-367-release-adopter-v0-2-38
terminalArchive: .ai/work-items/archive/WI-367-release-adopter-v0-2-38.contract.json
terminalVerification: .ai/evidence/WI-367-release-adopter-v0-2-38.verification.json
terminalFinalization: .ai/decisions/WI-367-release-adopter-v0-2-38.finalize.json
terminalDecision: .ai/decisions/WI-367-release-adopter-v0-2-38.close.json
capabilityClaims: [release_distribution, adopter_acceptance, repository_isolation]
---

# WI-367 — v0.2.38 public Release adopter acceptance

[English](WI-367-release-adopter-v0-2-38.md) · [简体中文](WI-367-release-adopter-v0-2-38.zh-CN.md)

## Intent

不変な公開 v0.2.38 Release binary だけを使って新しい adopter repository を治理し、Release acceptance と isolation boundary を証明し、将来の Release 用に再現可能な baseline を残します。

## Scope と boundary

- source、workspace binary、`cargo build`/`cargo run` fallback を使わずに
  public adopter acceptance と upgrade harness を実行します。
- Runtime identity、Release metadata、evidence reuse、Work Item lifecycle、
  isolation manifest、cleanup proof を保持します。
- manifest helper が macOS Bash 3.2 と Linux Bash の両方で動作することを確認します。

Runtime implementation、CI workflow policy、global Agent/MCP configuration、
historical evidence の書換え、adopter business code は本 WI の範囲外です。

## Acceptance

1. 公開 v0.2.38 Release metadata、archive digest、binary digest を記録し相互に一致させます。
2. 新しい adopter repository を download 済み v0.2.38 binary だけで attach・治理します。
3. repository identity、Work Item lifecycle、evidence reuse、close decision を Runtime identity とともに記録します。
4. HOME と XDG_CONFIG_HOME は変更せず、Runtime write root は隔離し、temporary run root は成功時も失敗時も削除します。
5. source checkout、workspace binary、Cargo fallback は使いません。
6. acceptance receipt と checksum は再現可能で、後続 Release の baseline に利用できます。
7. public harness は macOS Bash 3.2 と Linux Bash で manifest deadlock なく完了します。

## Evidence と結果

- 公開 Release：[v0.2.38](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.38)
- Release workflow：[33195494850](https://github.com/xinglun/ai-cockpit/actions/runs/33195494850)
- Acceptance evidence：`.ai/evidence/WI-367-release-adopter-v0-2-38/acceptance.json`
- Runtime verification：`.ai/evidence/WI-367-release-adopter-v0-2-38.verification.json`
- Isolation/cleanup evidence：`.ai/evidence/WI-367-release-adopter-v0-2-38/isolation.json` と `cleanup.json`

公開 workflow と local immutable-artifact run は pass しました。初回 run で判明した
macOS portability defect は `tests/release/isolation_manifest.sh` で修正し、regression test でカバーしています。
