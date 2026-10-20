---
author: AI Cockpit maintainers
title: "WI-1090 — 証拠の保存場所と文書の復旧"
description: "証拠の保存場所と文書の復旧"
audience:
  - maintainer
  - reviewer
authority: explicit-user-authorization
workItemId: WI-1090-evidence-docs-recovery
status: in_progress
lastVerifiedBy: WI-1090-evidence-docs-recovery
---

# WI-1090-evidence-docs-recovery — 証拠の保存場所と文書の復旧

本項は PR1050 の実際の Windows checkout と文書の失敗を修復する、ネイティブで選択された後継である。範囲は八つの Agent コピーのバイト保持移動、二つの locator ファイル、六つの前項文書投影と本項の三言語ページに限定する。Rust、テスト、workflow、グローバル設定、前項の生成履歴は変更しない。他の十九の長いパス候補は元のパスとバイト列を保持する。

前項 `WI-1090-authority-source-material-review-replacement` はアーカイブ済みで、七つのネイティブ検証は成功し、ホスト CI `39344530319` は失敗した。close は保留中である。追記のみの選択記録は `.ai/decisions/WI-1090-authority-source-material-review-replacement.recovery.json`。元 M6 manifest は論理キーを保持し、移動した八つのコピーは `.ai/evidence/WI-1090-authority-source-material-review-replacement/m6-short-20261020/path-map.actual.json` で明示的に解決し、バイト列、ハッシュ、Git blob を照合する。

本ページと三つの完全 ID parity 行は自身の最初の検証 receipt より前にソースへ登録してコミットする。必須検証は `bash tests/docs/documentation_acceptance.sh` と `bash tests/docs/parity_status_check.sh`、および実際の移動整合性監査である。現在の結果とライフサイクルはネイティブ証拠を参照し、登録だけで成功や配信完了を主張しない。

自身の通常の checkpoint、検証、controls、finish、archive は Runtime の実際の許可がある場合に限る。活動中 Contract `.ai/work-items/active/WI-1090-evidence-docs-recovery.contract.json` とアーカイブ Contract `.ai/work-items/archive/WI-1090-evidence-docs-recovery.contract.json` はライフサイクル上の代替位置であり、両方の存在を主張しない。自身の検証は `.ai/evidence/WI-1090-evidence-docs-recovery.verification.json`、将来の finalize と close は `.ai/decisions/WI-1090-evidence-docs-recovery.finalize.json` と `.ai/decisions/WI-1090-evidence-docs-recovery.close.json`。

両 Work Item の close、マージ、ブランチ/作業ツリー/Provider 削除、リリース/tag、Runtime 採用は許可されていない。PR1050 を保持し、独立した Draft PR と実際の CI は別の配信証拠とする。材料スキャンの Unknown と self_declared の保証上の制限を保持する。本項は WI1089 と WI1090 を組み合わせず、Sentinel を操作せず、実際の consumer 復旧を証明しない。
