---
author: AI Cockpit maintainers
title: "WI-793 — v0.2.90 release recovery"
description: "WI-792 の retry evidence を保持し、最新 default branch から v0.2.90 を再 delivery する。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: explicit-user-authorized-release-recovery
workItemId: WI-793-release-v0-2-90-recovery
lastVerifiedBy: WI-793-release-v0-2-90-recovery
---

[English](WI-793-release-v0-2-90-recovery.md) · [简体中文](WI-793-release-v0-2-90-recovery.zh-CN.md)

# WI-793 — v0.2.90 release recovery

## Historical status

WI-793 は保持された歴史的 predecessor です。v0.2.90 Release と adopter evidence が利用可能になった後、Runtime は WI-794 を後継とする `supersede` decision を記録しました。archive、verification、retry、supersede の各 record は immutable のままで、現在の release closure は WI-794 が担当します。

## Recovery boundary

WI-793 は WI-792 に厳密に bind された successor です。WI-792 の retry receipt は
immutable evidence として predecessor branch に保持します。同じ future-dated
timestamp により古い stale candidate が現在の retry 以降に並び、Runtime が消費を
拒否したためです。本 successor は最新の同期済み `origin/main` から開始し、
predecessor bytes を書き換えません。

## Intent と release boundary

A–E trust review の統合後に v0.2.90 を公開します。Release は reviewed PR と同期済み
default branch から作成し、新しい annotated tag を使い、download した public artifact
だけを acceptance の根拠にします。Rust/Runtime behavior、tag、governance history、
release gate は変更・再利用・弱体化しません。

## Release-note categories

- Communication fixes: Outcome reason、finalization recovery action、CLI/MCP、summary/full、
  tri-language の human semantics を検証します。
- Validation coverage: A–E counterexample、chat history なしの handoff、historical
  compatibility、controlled cleanup、deterministic concurrent-change path を evidence にします。
- Architecture consistency: Outcome input は bounded で revalidated な ObservationContext
  boundary 内で組み立て、pure rendering は I/O-free に保ちます。
- Performance diagnostics: main-path timing と byte/hash/Git/process counter は macOS/Linux
  filesystem path を含みます。診断であり、未検証の speedup は主張せず、rejected optimization
  の結論も維持します。

## Acceptance

- Cargo metadata、lockfile、三言語 projection、reference metadata が v0.2.90 identity に一致する。
- reviewed PR が merge 前に hosted route、quality、behavioral、Windows、task completion を通過する。
- annotated tag と public Release manifest が merged main commit に bind し、artifact/checksum/SBOM/attestation が通過する。
- public install と N-1 upgrade は isolated root の downloaded artifact で行い、cleanup を証明する。
- successor の archive、finalization、close、exact cleanup、main 同期を検証する。

## Verification

- `bash tests/release/version_consistency.sh --repo <repo>`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `cargo test --locked --workspace`
- `bash tests/release/version_consistency.sh --repo <repo> --post-release --repository xinglun/ai-cockpit --tag v0.2.90`
- public Release に対する `tests/release/adopter_acceptance.sh` と `tests/release/adopter_upgrade_acceptance.sh`
