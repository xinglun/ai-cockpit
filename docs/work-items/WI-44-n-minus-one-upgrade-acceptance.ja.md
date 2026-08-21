---
author: AI Cockpit maintainers
title: "WI-44 — N-1 adopter アップグレード受入れ"
description: "既存 adopter が明示的な移行後も動作を継続できることを、再現可能な公開後検証で証明する。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: implementation-acceptance
capabilityClaims:
  - n_minus_one_upgrade
keywords: [work-item, release, upgrade, migration, adopter]
---

# WI-44 — N-1 adopter アップグレード受入れ

## 意図と境界

この Work Item は、旧 Runtime と新 Runtime の二つの不変な公開 Release
アーカイブだけを使う公開後受入れハーネスを定義する。ソースのビルド、workspace
binary の実行、Release truth の変更、第二技術スタックの検証は行わない。

Runtime-only upgrade は attach 済み repository を変更しない。Repository Protocol
schema が変わる場合、新 Runtime は `MIGRATION_REQUIRED` を返し、明示的に承認された
migration を待つ。

## 受入れフロー

`tests/release/adopter_upgrade_acceptance.sh` は両アーカイブをダウンロードして検証し、
隔離 Cargo adopter を作成する。旧 Runtime で attach、Work Item と evidence を記録し、
次を確認する。

1. 新 Runtime が旧 schema を検出する。
2. migration plan は read-only で、未承認 apply は fail closed になる。
3. 承認済み migration は Runtime identity 付き receipt を記録し、旧 evidence の bytes を変更しない。
4. 新 Runtime が `COMPATIBLE` となり、Agent doctor、Work Item close、新しい verify を完了する。
5. `acceptance.json`、Runtime identity、隔離証跡、履歴 digest、`SHA256SUMS` を生成する。

公開後受入れに失敗しても `releasePublished: true` を記録する。既に公開した Release を
未公開へ戻したり、失敗した receipt を再利用したりしてはならない。

## 再現

新しい公開 Release が存在してから実行する。

```bash
tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --from-tag v0.1.1 \
  --to-tag v0.2.0 \
  --target aarch64-apple-darwin \
  --output ./release-adopter-upgrade-acceptance
```

公開前の静的チェックは次で実行できる。

```bash
bash tests/release/adopter_upgrade_acceptance_test.sh
```

出力は公開後の受入れ artifact であり、公開前 gate ではない。対象の公開 tag と archive
digest に結び付けて保存する。
