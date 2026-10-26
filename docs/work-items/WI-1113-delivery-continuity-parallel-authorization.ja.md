---
author: AI Cockpit maintainers
title: "WI-1113-delivery-continuity-parallel-authorization — scope を限定した delivery continuity と並行 Work Item authorization"
description: "Windows archive 修正を引き継ぎ、receipt 移行、明示的な一回限りの scope authorization、並行 Work Item 調整、有限 retry と中断復旧を扱います。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1113-delivery-continuity-parallel-authorization
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1113-delivery-continuity-parallel-authorization
---

# WI-1113 — Delivery continuity と並行 authorization

この Work Item は WI-1112 の native successor です。fixed Runtime は WI-1112 を `replaced/not_verified` として保持し、元の evidence も保存しました。fixed Runtime は 2026-10-26T21:14:40Z に WI-1113 の staged activation を記録しました。現在有効な Contract は `sha256:98fa4e72be9f32e399e1b7e5c05df89e8092e4461b3a5df0d96b8e36b5e9736d`、`baseRevision` は `b4adab2e55119d50bb72ec29f3fdfd2693d14504` です。predecessor recovery HEAD `5a942f93acaf0a514a291e39674465feddfa69e7` は別の identity です。現在の Contract に対する正確な native owner confirmation は保留中です。実装・正式検証の記録はなく、必須 25 scenario はすべて未検証です。

## scope と境界

- WI-1112 の Windows local-Git path と `.ai` artifact の厳密な byte 保護、digest 検証、1 byte tamper 拒否を維持します。
- 検証済み recovery lineage から導出する read-only delivery-continuity projection を追加します。predecessor の local archive、current successor、external check、Runtime が admit した次 action を分けます。archive や PR の作成だけで delivery 完了とはみなしません。
- append-only material-profile transition を追加します。旧 receipt を historic Contract、profile、request、manifest、合法な同一 Work Item の amendment chain に対して検証し、旧 receipt と pointer history を保持して新しい request を開きます。この transition 自体は新しい承認を与えず、Unknown を Clean にしません。未対応・foreign・malformed、または変更された material は引き続き block します。
- 移行できない receipt を amendment 前に検出する lint を追加します。
- 明示的に列挙した同一 repository の Work Item に限る exact-scope authorization manifest、独立 child grant/status、member ごとの atomic start quota、変更のない check に対する bounded retry、中断復旧、正確な partial result を追加します。Work Item 間の authority 継承、暗黙 member、共有 counter、任意 command retry、失敗履歴削除は行いません。
- CLI/MCP の動作を一致させ、英語・日本語・簡体字中国語で説明します。CI workflow、assertion、global Git 設定、main への merge、release、production 有効化、fixed Runtime の置換、別の正確な判断なしの close は対象外です。

## 検証計画

Contract は必須 25 scenario と 19 verification command を宣言しています。Windows path/byte 整合性、正しい／不正な successor lineage、archive と delivery の区別、exact material reference と delta、append-only transition の拒否例、amendment 前 lint、明示的 batch membership と独立 grant、atomic quota、bounded retry と不確実な実行結果、partial result、CLI/MCP parity、scope/target/expiry を対象にします。Runtime が実証を記録するまで全 scenario は `unverified` です。hosted PR run は自動 trigger・入口失敗・rerun を含め合計 5 回まで。local check は各 900 秒、1 worker、同一条件の同一 check に対する追加試行 2 回までです。


## PR 比較と base

WI-1113 の PR は #1075 の未変更の head から WI-1111 branch に向かうため、WI-1112 から継承した変更と WI-1113 の変更を含む累積差分になります。#1075 の merge を前提にせず、#1074/#1075 は開いたまま保持します。新しい PR は累積 diff と各変更の由来を審査し、純粋な WI-1113 差分とは表現しません。Contract の predecessor recovery identity、通常 staged activation が Runtime によって設定する baseRevision、provider 比較 base は別々に記録します。

## 現在の状態

typed plan は累積 PR 比較と package 単位の検証計画を記録しています。WI-1113 は現在 `implementation_active` ですが、正確な native owner confirmation は保留中で、PR/resource context は未 binding です。このページは現在の native status を示し、実装・検証・hosted CI・user-visible benefit を主張しません。
