---
author: AI Cockpit maintainers
title: "WI-149——構造化された Release adopter decision"
description: "post-release adopter acceptance を完全な repository-bound Human Decision receipt に結び付ける。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-149-release-decision-acceptance
workItemId: WI-149-release-decision-acceptance
---

# WI-149——構造化された Release adopter decision

post-release adopter acceptance と N-1 upgrade acceptance は、各 Work Item を完全な
structured Human Decision で close しなければなりません。harness は immutable Release
binary に actor、authority source、reason、evidence reference、policy reference、決定時刻、
resume condition を渡します。

close 後、harness は `.ai/decisions/<work-item>.close.json` が通常ファイルかつ symlink では
ないことを要求し、Work Item、closed state、confirmed decision、structured fields を検証して
acceptance artifact にコピーします。binding record には adopter の `repositoryId`、Work Item
ID、decision digest、検証結果を記録します。decision evidence の欠落または不一致は fail closed
となり、公開済み Release truth を変更しません。

static wrapper は structured close、copy、validation の境界を検証します。三言語の Release
distribution guide は同じ acceptance contract を説明します。Runtime Core と CLI semantics は
この Work Item の範囲外です。

Evidence: `.ai/evidence/WI-149-release-decision-acceptance.verification.json`。
Close decision: `.ai/decisions/WI-149-release-decision-acceptance.close.json`。
