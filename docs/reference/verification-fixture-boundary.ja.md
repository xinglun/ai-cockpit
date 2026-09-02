---
author: AI Cockpit maintainers
title: Verification fixture の境界
description: 隔離 repository fixture の内容と、それが証明できないもの。
audience: [contributor, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/verification-fixture-boundary.md
lastVerifiedBy: WI-512-reference-docs-batch-33
---

# Verification fixture の境界

[English](verification-fixture-boundary.md) · [简体中文](verification-fixture-boundary.zh-CN.md) · [日本語](verification-fixture-boundary.ja.md)

Repository test は temporary copy で Rust Runtime を実行できます。fixture に含めるのは source
と repository-local Protocol input だけで、呼び出し元の Runtime state は含めません。明示的に
必要と宣言しない限り Git metadata、worktree、virtual environment、Cargo/build output、language
tool cache は除外します。

保持された Work Item history を縮小のためにコピーせず、fixture setup が source checkout を
削除することもありません。fixture result は local test evidence であり、provider、hosted CI、
adopter、production、enterprise evidence ではありません。Release/adopter claim には各自の
immutable artifact と isolation receipt が必要です。
