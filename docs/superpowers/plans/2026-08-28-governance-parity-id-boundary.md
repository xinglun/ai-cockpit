---
author: Ray
title: Governance parity ID boundary implementation plan
description: 非 canonical Work Item ID が reference parity の強制検証を誤って発火させないための有界な CI gate 修正計画。
key: governance-parity-id-boundary-plan
---

# Governance parity ID boundary 実装計画

## 目的

`governance_integrity_gate.py` が、canonical な `WI-*` Work Item と、runtime/test 用の任意識別子を同じ reference parity 投影対象として扱わないようにする。canonical Work Item の parity 検証は維持し、既存の archived bytes と pending-close backlog は変更しない。

## 対象範囲

- `tests/ci/governance_integrity_gate.py`
- `tests/ci/governance_integrity_gate_test.sh`
- 本計画書

## 対象外

- archived Work Item、既存の reference parity 文書、PR/merge/close 操作
- pending-close backlog の清理と Runtime の一般 lifecycle policy
- provider、execution authority、production business logic

## 実装手順

1. 回帰テストを先に追加し、非 canonical ID の archived fixture が `missing_parity_entry` で失敗する現状を確認する。
2. archive parity projection の対象判定を canonical `WI-*` ID または明示的 parity row に限定する。
3. canonical ID の parity row 欠落が引き続き fail closed であることを確認する。
4. valid fixture、非 canonical fixture、canonical parity 欠落 fixture を gate test で検証する。
5. `make quality` と Contract の required verification を実行し、Summary/evidence を Runtime で更新する。

## 検証方針

- 正常系: 既存 valid fixture が通過する。
- 境界系: 非 canonical archive ID が parity registry の行を要求しない。
- 拒否系: canonical `WI-*` archive ID の parity row 欠落は従来どおり拒否する。
- 不変条件: `upstream`/reference parity の既存データ、archived Work Item bytes、lifecycle gate の一般挙動を変更しない。
