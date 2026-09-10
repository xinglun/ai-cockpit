---
author: AI Cockpit maintainers
title: "WI-792 — v0.2.90 Release"
description: "Trust diagnostics と governance documentation の統合受入れ後に次の patch を公開する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-release
workItemId: WI-792-release-v0-2-90
lastVerifiedBy: WI-792-release-v0-2-90
---

[English](WI-792-release-v0-2-90.md) · [简体中文](WI-792-release-v0-2-90.zh-CN.md)

# WI-792 — v0.2.90 Release

## Intent と境界

A–E trust review の統合受入れ後に、strategy-approved な次の patch を公開する。
Release は reviewed PR と同期済み default branch から作り、新しい annotated tag
を使い、download した public artifact だけを adopter acceptance の根拠にする。
この Work Item は Rust/Runtime behavior、過去の evidence、tag の再利用、Release
gate の弱体化を変更しない。

## Release note の分類

- Communication fix: Outcome reason projection が実際の governance gap を区別し、
  finalization recovery action が具体的な条件を保持する。CLI/MCP、summary/full、
  三言語の協調チェックは human semantics を検証する。
- Validation coverage: A–E counterexample、chat history なしの handoff、historical
  compatibility、controlled cleanup、deterministic concurrent-change path を実行可能
  な check に結び付ける。
- Architecture consistency: Outcome input は bounded で再検証可能な
  ObservationContext boundary 内で組み立て、pure rendering は I/O-free のままにする。
- Performance diagnostics: main path の stage timing と byte/hash/Git/process counter、
  macOS/Linux filesystem path を提供する。これは診断計測であり、未検証の速度向上は
  主張しない。以前 rejected となった optimization の結論も維持する。

## Acceptance

- Cargo metadata、lockfile、現在の三言語 release/version projection、reference
  metadata が一つの package identity に一致する。
- reviewed PR は merge 前に hosted quality、route、behavioral、Windows、task
  completion check を通過する。
- annotated tag と public Release manifest が merged main commit に結び付き、全
  artifact、checksum、SBOM、attestation の gate が通過する。
- public install と N-1 upgrade は isolated root の download artifact で実行し、
  cleanup を証明する。workspace build は Release artifact の代替ではない。
- Work Item の archive、finalization、close、exact branch/worktree cleanup、main
  同期を検証する。

## Verification

- `bash tests/release/version_consistency.sh --repo <repo>`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `cargo test --locked --workspace`
- `bash tests/release/version_consistency.sh --repo <repo> --post-release --repository xinglun/ai-cockpit --tag v0.2.90`
- public Release に対する `tests/release/adopter_acceptance.sh` と `tests/release/adopter_upgrade_acceptance.sh`
