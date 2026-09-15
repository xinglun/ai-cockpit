---
author: AI Cockpit maintainers
title: "WI-845 — historical close projection"
description: "非正規な過去の close を書き換えず、有効な append-only successor recovery を投影する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-845-historical-close-projection
lastVerifiedBy: WI-845-historical-close-projection
---

[English](WI-845-historical-close-projection.md) · [简体中文](WI-845-historical-close-projection.zh-CN.md)

# WI-845 — historical close projection

## Intent and boundary

この Work Item は、現在の規約に合わない過去の close decision を持つ
archived Work Item についても、有効な append-only successor recovery を
Runtime の status projection に反映できるようにする。predecessor の元バイト列
は変更せず、repository-bound recovery、successor、archive、close の結び付きを
すべて検証できた場合だけ投影を解決する。Protocol schema、CLI surface、release
および adopter の動作、過去記録の書き換え、広範な branch cleanup は対象外である。

## Recovery boundary

同じ範囲の不具合はこの Work Item を amend して再検証する。異なる scope、
authority、base、独立した変更、安全に範囲内で修正できない場合、immutable な
failed delivery、または明示的な人の指示の場合だけ successor を使う。無効、
不完全、foreign、stale、または改ざんされた recovery evidence は blocking の
まま残り、通常の closed projection にはならない。

## Acceptance

- 有効な repository-bound successor recovery と terminal successor がある場合、
  predecessor は recovered かつ nonblocking として投影され、predecessor の
  close bytes は書き換えられない。
- 未 close、不一致、malformed、foreign、または改ざんされた successor recovery
  は blocking のままで、close validation を迂回しない。
- Rust の focused regression test が正の投影と fail-closed の負のケースを証明する。
- 実装は部分的な状態書き込みを行わず、元の historical close bytes を保持する。
- PR は必要な pre-archive の三言語 Work Item ページと、この Work Item の一意な
  parity 行を含み、repository quality gate が高コストな検証前に変更を確認できる。

## Verification

- `cargo test --locked -p cockpit-repository --test status_projection`
- `cargo test --locked -p cockpit-repository --test recovery_decision`
- `cargo clippy --locked -p cockpit-repository --all-targets --all-features -- -D warnings`
- `cargo fmt --all -- --check`
- `git diff --check`
