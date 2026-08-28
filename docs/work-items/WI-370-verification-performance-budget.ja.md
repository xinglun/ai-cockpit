---
author: AI Cockpit maintainers
title: "WI-370 — Verification performance budget と exact reuse"
description: "ガバナンスを弱めず、動的で identity-bound な再利用により重複検証の遅延を減らす。"
workItemId: WI-370-verification-performance-budget
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-370-verification-performance-budget
capabilityClaims: [verification_performance, evidence_integrity]
---

# WI-370 — Verification performance budget と exact reuse

[English](WI-370-verification-performance-budget.md) · [简体中文](WI-370-verification-performance-budget.zh-CN.md)

## Intent と boundary

この Work Item は current repository と adopter repository の重複検証遅延を減らします。
検出された Work Item command は profile-authorized な dynamic path を利用できますが、
完全に identity-bound な receipt だけを再利用します。明示的な custom command は fresh のままです。
repository snapshot、Contract、scope、command、stage、runner、Runtime、profile、toolchain、
dependency、policy のいずれかが変われば、新しい実行または policy による escalation を行います。

required / protected governance check は決して skip せず、unknown impact は timing や cache によって
Green になりません。Rust Runtime は共有された一つの installation のままで、adopter は同じ選択規則を
継承しつつ evidence と repository identity を分離します。

## Verification と acceptance

- selection は executed、reused、escalated、denied と安定した理由を記録します。
- reuse は repository、profile、Runtime、command、scope、stage、runner、base、toolchain、dependency、
  policy context に bind されます。
- reused result は現在の Work Item 用の新しい evidence を作り、別の Work Item を認可できません。
- current project と公開 adopter の測定は cold/warm elapsed と Runtime/repository identity を保持します。
- 三言語の reference 文書は、性能がコスト最適化だけであり、verification truth や required gate を弱めない
  ことを説明します。

archive Contract と verification evidence が machine-readable な authority です。本ページは読者向けの
Work Item projection であり、provider merge と cleanup の検証後にだけ terminal link を追加します。
