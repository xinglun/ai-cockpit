---
author: AI Cockpit maintainers
title: "WI-1116-ci-parity-windows-recovery — Windows test cfg と governance parity の復旧"
description: "PR1077 の CI コンパイルと正確な governance projection を最小限修復します。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1116-ci-parity-windows-recovery
status: implemented
authority: native-contract
lastVerifiedBy: WI-1116-ci-parity-windows-recovery
---

## 現在の受入れ状態

この限定Unix wrapper／parity修正は source cc7da2d410efe05d0d0c49d68d784b9c3c6e1939 の実際のローカル6宣言で受け入れられ close 済みです。復旧後継 WI-1117 は進行中で、その結果を本項に加算しません。CI run41376207420 のWindows boundariesは成功、repository-libraryのroot harnessは253成功／1失敗（evidence_store optional_directory_tests）で、最初のResult／errnoは不明です。完全なWindows／全CI成功ではありません。Outcome ENAMETOOLONG、旧118/1、後続Mac/HCI義務を残します。

- `.ai/work-items/archive/WI-1116-ci-parity-windows-recovery.contract.json`
- `.ai/work-items/archive/WI-1116-ci-parity-windows-recovery.archive.json`
- `.ai/evidence/WI-1116-ci-parity-windows-recovery.verification.json`
- `.ai/decisions/WI-1116-ci-parity-windows-recovery.close.json`
- `.ai/decisions/WI-1116-ci-parity-windows-recovery.recovery.json`

## 保存した当初計画（現在の受入れ条件ではありません）

# WI-1116-ci-parity-windows-recovery — Windows test cfg と governance parity の復旧

本ページは作成時に承認された計画を記録します。現在の authority、evidence、完了状態は native Contract と Runtime が決定し、事前登録 path は実行 receipt ではありません。

## 最小変更

- 既存の有限 Unix test compatibility wrapper のみを `cfg(all(test, unix))` に限定します。二つの呼出元は既に Unix test module 内です。production cleanup、Windows test 選択、`-D warnings` は維持します。
- 三言語の WI-1114 parity 行を実在する復旧系列、ローカル検証、close evidence に合わせます。本 successor に必須の三言語ページと正確な prearchive 登録を追加します。
- PR #1077 上で native WI-1114 → WI-1115 → WI-1116 復旧系列を継続し、closed predecessor と失敗履歴を書き換えません。

## 検証と境界

Contract は六つのローカル check を宣言します：parity gate と回帰、repository collaboration test、repository all-targets strict Clippy、workspace fmt check、canonical repository prechecks。material review は毎回の完全な新 request に結び付け、確認済みの readable committed Rust syntax Unknown 機構だけを使用します。Linux 成功は Windows 成功の証拠ではありません。実際の hosted quality と変更しない二つの Windows suite は PR merge 条件として残ります。

本計画ページは検証、archive、close、merge、公開の完了を主張しません。以前の未解明な 118/1 失敗と Outcome 表示の ENAMETOOLONG リスクを保持します。Mac 実機／完全な子孫監督と必要な HCI North Star 修正は後続の独立 WI に属し、その後に release／Sentinel 受入れを行います。credential、security permission、default Runtime、cache cleanup の変更は含みません。
