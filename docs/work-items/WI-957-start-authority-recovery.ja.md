---
author: AI Cockpit maintainers
title: "WI-957 — start authority と typed 検証の検証"
description: "active Work Item state を作成する前に未対応の authority 値を拒否し、typed 必須検証を宣言した identity で実行する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-957-start-authority-recovery
lastVerifiedBy: WI-957-start-authority-recovery
---

[English](WI-957-start-authority-recovery.md) · [简体中文](WI-957-start-authority-recovery.zh-CN.md)

# WI-957 — start authority と typed 検証の検証

Runtime は active Contract または Summary の書き込み前に未対応の authority 値を拒否する。
CLI は typed 必須検証宣言の `check` identity で検証を計画するため、検証
receipt は同名の finish gate を満たせ、無関係な既定 workspace test には
フォールバックしない。
checkpoint 後の追加 amendment がこの evidence を無効化した場合は、現在の
置換検証を一度だけ開始できる。無関係な control failure は引き続き subprocess
開始前に実行を停止する。
