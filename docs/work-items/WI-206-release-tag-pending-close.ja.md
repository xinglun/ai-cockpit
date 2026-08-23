---
author: AI Cockpit maintainers
title: "WI-206 — release tag の pending-close governance boundary"
description: "merge 済みであることを証明できる Release tag の公開を許可しつつ、Runtime による close を必須にする。"
audience:
  - maintainer
  - adopter
workItemId: WI-206-release-tag-pending-close
status: in_progress
authority: canonical
lastVerifiedBy: WI-206-release-tag-pending-close
---

# WI-206 — release tag の pending-close governance boundary

v0.2.25 の source-quality gate は、merge 済みだが未 close の current-cycle
Work Item を含む tag を正しく拒否しました。これは、merge 後の finalization
transition に Release の Runtime が必要である一方、Release gate がその Runtime
を install できる前に実行されるという順序の deadlock を示しました。

この Work Item は境界を明確にします。pre-merge finalization receipt が identity-bound
で、記録された PR head が tag commit の ancestor であることを Git が証明できる
場合だけ、Release tag を一時的に `awaiting_merge_close` として投影できます。
公開後の binary で finalization と structured human close を必ず完了します。
通常 branch と証明できない tag は引き続き fail-closed です。

## Acceptance boundary

1. 祖先証明に成功した Release tag だけを `awaiting_merge_close` として受理します。
2. 非祖先、malformed、foreign、通常 branch のケースは引き続き block します。
3. 英語・簡体字中国語・日本語の workflow 文書で release 順序と公開後 close を説明します。
