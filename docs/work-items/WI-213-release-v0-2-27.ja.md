---
author: AI Cockpit maintainers
title: "WI-213 — v0.2.27 immutable release と adopter acceptance"
description: "merge 済み main から v0.2.27 を公開し、installed Runtime で public artifact を検証する。"
audience:
  - maintainer
  - adopter
  - reviewer
workItemId: WI-213-release-v0-2-27
status: current
authority: canonical
lastVerifiedBy: WI-213-release-v0-2-27
---

# WI-213 — v0.2.27 immutable release と adopter acceptance

この Work Item は v0.2.26 の source-quality failure 後の最初の public
Release を公開します。v0.2.26 tag は immutable な失敗履歴として保持し、
書き換えたり再利用したりしません。v0.2.27 は merge 済み PR #160 から作成し、
公開 download artifact だけで受入れます。

## Acceptance

1. Cargo version、lockfile、release document、tri-language parity、release
   workflow policy が v0.2.27 に一致する。
2. tag は merge 済み PR #160 の descendant にだけ作成し、source、tag、manifest、
   checksum、provenance gate を通過する。
3. public adopter と N-1 acceptance は immutable な download artifact だけを使い、
   source checkout、workspace binary、local `target` fallback を禁止する。
4. acceptance receipt は repository/runtime identity、isolation manifest、cleanup、
   evidence reuse、localized Outcome を bind する。
5. 公開後は installed v0.2.27 Runtime で WI-212 の post-merge finalization
   transition、`finalize-verify`、structured close を完了する。

## Out of scope

Reference source の逐 file parity は次の batch です。この Work Item は無関係な
Runtime feature や user-global Agent/MCP configuration を変更しません。

## Evidence boundary

公開 Release、download archive、manifest、checksum、attestation、adopter receipt は
immutable external evidence です。公開後に失敗した場合は
`releasePublished: true` と `adopterAcceptance: failed` を記録し、Release truth を書き換えません。
