---
author: AI Cockpit maintainers
workItemId: WI-1056-retry-amend-preflight
title: Recovery retry と Contract amendment の preflight 候補修正
description: Runtime start の元の拒否と pending review request 生成に関するローカル候補テストを保持します。
audience: [maintainer, reviewer]
status: in-progress
authority: user-authorized-bounded-governance-repair-exception
lastVerifiedBy: focused-local-tests-only
---

[English](WI-1056-retry-amend-preflight.md) · [简体中文](WI-1056-retry-amend-preflight.zh-CN.md)

# WI-1056 有界ガバナンス修正の例外

この WI は `codex/wi1056-retry-amend-preflight` で `origin/main` の
`a5bd06bf2932072eaef1ce1c33edcd8068b9d39b` から開始しました。正式な
`work-item new` は `not_ready` skeleton のみを作成し、`start` は archive 済みの
WI-1042 と WI-1043 の scope conflict で拒否されました。以前ユーザーが認めた有界な
ガバナンス修正例外により、独立 branch で候補テストとコード修正だけを進めます。
Runtime が admit、verify、close したとは主張しません。

修正対象は、有効な `recoveryRetryPending=true`、sensitive Contract amendment、
`preflightState=not_run` が同時に起きる self-lock に限ります。新しい CLI テストは
修正前に `run_preflight` が欠けるため RED、最小の status projection 修正後に GREEN
となりました。新しい pending request が現在の repository、WI、Contract、snapshot
に bind し、人の decision が記録されず、retry marker が残り、verification command
が起動しないことを確認しました。通常の amendment テストと repository library
テストも通過しましたが、これらはローカル候補テストであり、WI の Runtime verification
receipt ではありません。独立 review と残りの lifecycle は未完了です。

共有 Runtime を置き換えず、別途実環境で確認しました。Task 9 の WI-1049 では正式な
amendment を一回行い、既存の A1 `development_cycle_cost` の二つの path を維持し、
A2 `runtime_benchmark_scenarios` の Rust 実装／テストの二つの path と対象 acceptance
のみを追加しました。B `runtime_benchmark_stats` はこの batch に含めていません。続く
一回の正式 preflight は Contract
`sha256:7de2a941e89fbeb2cef36f5a9c016a580f76ea71420a55aaafcdcba6572866df`
と repository snapshot
`sha256:fe207ccca471f47df7b27a61adf09901e9df622100c68d45730fc941dbd86cdd`
に対して `needs_human_confirmation` を返しました。retry は pending のままで、人の
decision は記録されず、Task 9 の source 作業も再開していません。追加回帰テストでは、
誤った repository、amendment 前の Contract、誤った snapshot の review evidence が
decision を書き込まず拒否されることも確認します。これは request 生成の観測であり、
人の承認や WI-1056 の正式な admission ではありません。
