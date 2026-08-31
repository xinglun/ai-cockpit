---
author: AI Cockpit maintainers
title: "WI-455 — v0.2.52 annotated tag release recovery"
workItemId: WI-455-release-v0-2-52-annotated-tag
description: "レビュー済み annotated tag と immutable な公開 artifact だけで次の patch を公開する。"
audience: [maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-455-release-v0-2-52-annotated-tag
---

# WI-455 — v0.2.52 annotated tag release recovery

この Work Item は immutable な v0.2.51 lightweight-tag 公開失敗後の次の patch release を準備します。
失敗履歴を保持し、annotated tag の再現可能な検査を追加し、provider Release の作成をレビュー済み workflow 内に限定します。
adopter repository は操作しません。

[English](WI-455-release-v0-2-52-annotated-tag.md) · [简体中文](WI-455-release-v0-2-52-annotated-tag.zh-CN.md)

## Sources

- `docs/release/distribution.*.md`
- `docs/architecture/release-distribution.*.md`
- `.github/workflows/release.yml`
- `tests/release/annotated_tag_identity.sh`
- 失敗した v0.2.51 workflow run `33417057474`

## Acceptance

- workspace metadata と三言語 release documentation が v0.2.52 を示し、v0.2.51 の履歴を改変しない。
- lightweight tag は拒否し、annotated tag は peel してレビュー済み commit に bind する。
- annotated tag の push を使い、provider Release を事前作成しないことを明記する。
- strict release gate、公開 artifact の checksum/SBOM/provenance、staged/public adopter acceptance を source fallback なしで通過する。
- 公開 binary は checksum 検証後だけ install し、現在の repository の Runtime health を確認する。

## Verification

- `tests/release/annotated_tag_identity.sh`
- `tests/release/version_consistency_test.sh`
- `tests/release/workflow_policy.sh .github/workflows/release.yml`
- strict `quality_route.py` + `run_repository_gates.py`
- `cargo test --locked --workspace`
