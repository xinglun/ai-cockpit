---
author: AI Cockpit maintainers
title: "WI-139F — Runtime v0.2.13 release acceptance"
description: "統合済み recovery と adopter acceptance control を immutable public Runtime として公開する。"
audience:
  - maintainer
status: active
authority: repository-local
lastVerifiedBy: pending-release-evidence
workItemId: WI-139F-runtime-v0-2-13
---

# WI-139F — Runtime v0.2.13 release acceptance

この Work Item は現在の統合済み Runtime を `v0.2.13` として公開します。完了には
immutable public Release、public fresh-adopter acceptance、`v0.2.12` からの N-1
upgrade acceptance、この repository での install check が必要です。acceptance は
download した public binary だけを使用し、source build で置き換えてはいけません。

Release receipt は tag、archive digest、binary digest、platform、Runtime identity、
adopter repository identity、isolation manifest、cleanup result、lifecycle evidence
を結び付けます。post-release failure は acceptance failure として記録し、公開済み
Release truth は変更しません。
