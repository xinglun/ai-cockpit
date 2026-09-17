---
author: AI Cockpit maintainers
title: Verification execution cache policy
description: Cargo 検証の cache を再利用可能かつ制御可能に保つ。
audience: [adopter, contributor, maintainer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-878-verification-target-policy
---

# Verification execution cache policy

Runtime は Cargo verification command の child process を起動する前に実行環境を解決します。

- `CARGO_INCREMENTAL=0` を設定し、bounded verification で価値の低い incremental artifact を無効にします。
- `CARGO_TARGET_DIR` を `$HOME/.cache/ai-cockpit-verify-target`（または platform の同等な user directory）に固定し、checkout ごとの `target` tree ではなく compile 済み dependency を再利用します。
- Cargo 以外の command は宣言された environment を変更しません。

この policy は shell の慣習ではなく Runtime execution boundary です。effective environment は verification observation identity に含まれるため、policy または toolchain の変更が無関係な receipt を暗黙に再利用することはありません。planning は child process 起動前に完了します。

verification session の後は、まず `cargo` または `rustc` の process がないことを確認します。その後 repository-local な incremental directory だけを `rm -rf target/debug/incremental` で削除できます。shared target cache と再利用可能な dependency は残ります。`cargo clean` を通常の cleanup にしないでください。

この policy は cache の場所だけを制御し、verification、authorization、merge、release、human approval の status を付与しません。
