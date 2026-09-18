---
author: AI Cockpit maintainers
workItemId: WI-892-release-v0-2-97
title: 不変の v0.2.96 候補拒否後の v0.2.97 リリース
description: ワークスペースのバージョン識別子を修正し、修復済み候補を公開する。
audience: [maintainer, reviewer, adopter]
status: implemented
authority: user:release-after-all-work-items
lastVerifiedBy: WI-892-release-v0-2-97
terminalArchive: .ai/work-items/archive/WI-892-release-v0-2-97.contract.json
terminalVerification: .ai/evidence/WI-892-release-v0-2-97.verification.json
terminalFinalization: .ai/decisions/WI-892-release-v0-2-97.finalize.json
terminalDecision: .ai/decisions/WI-892-release-v0-2-97.close.json
---

# WI-892 — 不変の v0.2.96 候補拒否後の v0.2.97 リリース

v0.2.96 タグは、ソースワークスペースが 0.2.95 のままだったため、不変の
失敗候補として保持する。本 Work Item ではワークスペースの識別子を修正し、
失敗タグを移動・削除せず次の未使用バージョンを公開する。

## 受入れ境界

- ワークスペースと生成成果物を 0.2.97 に統一する。
- v0.2.96 を不変のまま保持し、aggregate の失敗証拠を記録する。
- レビュー済みチェック、manifest/checksum/SBOM の binding、および隔離された
  ルートでのダウンロード成果物 fresh-install と v0.2.93 upgrade が通過した後だけ公開する。
- 対象リポジトリと Outcome/HCI/performance の範囲は変更しない。

## 検証計画

最初に format と version check を実行し、宣言済み workspace 検証、release workflow、
ダウンロード成果物の acceptance を続ける。失敗 run の記録を保持し、タグを移動せず、
無関係な Work Item を再実行しない。
