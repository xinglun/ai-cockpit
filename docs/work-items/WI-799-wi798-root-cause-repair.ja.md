---
author: AI Cockpit maintainers
title: "WI-799 — WI-798 root-cause repair"
description: "公開 CI とリリースを再開する前に、観測された協調、比較差分、scope、ガバナンス、復旧ループの根因を修正します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-root-cause-repair-and-release
workItemId: WI-799-wi798-root-cause-repair
lastVerifiedBy: WI-799-wi798-root-cause-repair
predecessorWorkItemId: WI-798-collaboration-observability-performance
recoveryDecision: .ai/decisions/WI-798-collaboration-observability-performance.recovery.7f58122c0b99798e61e08bed7f3f3c3ccf8dd4da8dd8dffebd1b4f7be68fd96c.json
---

[English](WI-799-wi798-root-cause-repair.md) · [简体中文](WI-799-wi798-root-cause-repair.zh-CN.md)

# WI-799 — WI-798 root-cause repair

## Recovery boundary

WI-799 は WI-798 の bounded successor です。WI-798 は不変の履歴証拠として
保持し、この Work Item は実装とリリース経路の再検証で発見した根因だけを
修正します。predecessor の記録を書き換えず、技術的な retry を無関係な
新しい Work Item に変換しません。

## Root-cause boundary

対象は、構造化された CLI failure report、Contract amendment 後の再検証、
governance と parity 登録、clean worktree における committed PR comparison
facts、共有 Contract scope glob の意味論です。comparison gate は Contract
baseline、CI comparison baseline、実際の HEAD diff を別々に bind し、Runtime
所有の governance path を Contract に明示します。

## Acceptance

- 高コストな検証より前に、順序、scope、stale evidence、登録不備を検出します。
- verification failure は構造化 evidence と non-zero exit を返し、failure を完了 evidence として扱いません。
- hosted 相当の clean checkout が committed comparison diff、変更テスト、governance record を観測します。
- Contract scope pattern と evaluator の意味論を一致させ、filename glob が directory boundary を越えない回帰を持ちます。
- English、Simplified Chinese、日本語の文書と parity row が同じ predecessor、evidence、将来の terminal lifecycle を bind します。
- provider CI または release を再開する前に、reviewed branch の strict local route が通ります。

## Verification boundary

- current Contract: `.ai/work-items/active/WI-799-wi798-root-cause-repair.contract.json`
- verification: `.ai/evidence/WI-799-wi798-root-cause-repair.verification.json`
- planned terminal archive: `.ai/work-items/archive/WI-799-wi798-root-cause-repair.contract.json`
- planned terminal verification: `.ai/evidence/WI-799-wi798-root-cause-repair.verification.json`
- planned finalization: `.ai/decisions/WI-799-wi798-root-cause-repair.finalize.json`
- planned close: `.ai/decisions/WI-799-wi798-root-cause-repair.close.json`

3 言語のページと parity row は同じ事実を保持します。provider CI、review、
merge、publication、公開 artifact acceptance はそれぞれの evidence が得られる
まで pending であり、完了とは宣言しません。
