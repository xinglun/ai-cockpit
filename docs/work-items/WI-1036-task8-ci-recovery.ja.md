---
author: AI Cockpit maintainers
workItemId: WI-1036-task8-ci-recovery
title: Task 8 CI クローズアウト
description: PR #997 で観測された3種類の CI failure だけを修正し、必須 gate を維持して Task 8 を merge・cleanup し、release 前に停止する。
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1036-task8-ci-recovery
---

[English](WI-1036-task8-ci-recovery.md) · [简体中文](WI-1036-task8-ci-recovery.zh-CN.md)

# WI-1036 — Task 8 CI クローズアウト計画

これは Task 8 の最終かつ範囲を限定した CI クローズアウトである。[PR #997](https://github.com/xinglun/ai-cockpit/pull/997) の [CI run 36363705256](https://github.com/xinglun/ai-cockpit/actions/runs/36363705256) に記録された failure のみを扱う。`cockpit-verification` の composition test 7件で process file descriptor を検査できなかったこと、merge 後の projection 前に predecessor Work Item の条件付き status が残り `docs_closed_work_item_promotion` が失敗したこと、`ci_manifest_regression` が期待された missing-receipt 診断に到達しなかったことが対象である。

Process observer の原因はまだ確認されていない。実装は temporary worktree を所有する可能性のある process を検査できない場合、意図的に fail closed する。この振る舞いを変更する前に、実際の所有権の曖昧さか、無関係な process／実行 scheduling との相互作用かを特定する。

## 範囲を限定した実施計画

1. 失敗した CI run、log、現在の PR head を保持する。Runtime が identity と freshness の完全一致を確認した場合のみ既存 receipt を再利用し、現行 Runtime が要求しない限り archive 済みの14-node hosted verification を再実行しない。
2. 報告された composition failure に対する小さな failing regression を追加する。process ownership の観測問題を診断してから最小限の安全な修正を行う。所有者不明を、稼働中 worktree の cleanup や再利用の許可にしてはならない。
3. lifecycle stage ごとの documentation promotion test を追加する。merge 前 CI では merge 後 projection を要求せず、同期済み main での必須 promotion は引き続き実行し、test で保護する。
4. missing-receipt negative case を隔離し、現在の repository の無関係な documentation state が期待する診断を隠さないようにする。不正 receipt と誤った receipt の検査も維持する。
5. 新しい Runtime preflight の後、この Work Item 内で admission された対象 test だけを直列実行する。canonical PR CI で必須 gate 全体を検証し、gate の skip や弱体化は行わない。
6. 修正 PR を Task 8 branch に統合し、PR #997 の正確な head で必須 check の成功を確認してから PR #997 を merge し、正確な local Work Item cleanup を完了する。
7. Task 8 を close したら直ちに Task 9 へ進む。release、tag、publication の前で停止する。

## レビュー可能な変更単位

- composition ownership／scheduling の修正と対象を絞った Rust regression。
- lifecycle stage に応じた promotion eligibility と、merge 後も promotion が必須であることの regression。
- 決定的な missing-receipt manifest regression。
- 本 Work Item の三言語 plan／status projection。

新しい協調機能、汎用 guide の再設計、無関係な Task 8 修正、Task 9 の移行は含めない。現在は進行中であり、この plan は修正や検証の成功を主張しない。
