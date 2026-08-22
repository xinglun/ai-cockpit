---
author: AI Cockpit maintainers
title: "WI-155 — CI/release gate の収束"
description: "release test を deterministic に保ち、Phase 1 Runtime shadow を execution smoke として定義します。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-155-ci-release-gate-convergence
workItemId: WI-155-ci-release-gate-convergence
---

# WI-155 — CI/release gate の収束

WI-155 は release の source-quality gate を、CI と同じ deterministic な package-by-package Cargo test strategy に揃えます。
各 package は `--test-threads=1` で実行し、個別 test binary 内の verifier worker cap による parallel coverage は維持します。

Runtime shadow は Phase 1 の **execution smoke** であることを文書と static check により明示します。immutable な public binary が
repository-bound verification command を一つ実行できることを検証しますが、policy route/planner、affected graph の完全性、
Work Item 間の physical execution、Work Item ごとの evidence receipt coverage は主張しません。この境界は既存の Cargo および
release gate を削除・置換するものではありません。

Evidence: `.ai/evidence/WI-155-ci-release-gate-convergence.verification.json`。
Decision: `.ai/decisions/WI-155-ci-release-gate-convergence.close.json`。
