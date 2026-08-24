---
author: AI Cockpit maintainers
title: "WI-257 — post-close promotion recovery"
workItemId: WI-257-post-close-promotion-recovery
description: "failed predecessor を書き換えず、clean な current base から typed post-close documentation promotion を復旧します。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-257-post-close-promotion-recovery
authority: canonical
---

# WI-257 — post-close promotion recovery

WI-257 は current default-branch base から repository-owned post-close documentation
orchestrator を再配信します。WI-256 と close 済み PR #208 は repository 外の immutable
failed-delivery history として保持します。この WI はそれらの `.ai` record を import せず、
repository terminal truth として表現しません。

## Acceptance boundary

- typed plan は repository identity、exact に同期した `origin/main`、approved close、
  sequence-2 finalization、archive/evidence identity、6 個の controlled documentation
  path の exact before/after digest を bind します。
- stale/descendant revision、foreign/malformed identity、duplicate/unknown JSON field、
  symlink/nonregular input/output、dirty/partial projection、unexpected path は write 前に
  fail closed。already-current plan の再 apply は deterministic no-op です。
- isolated bare-origin regression は `HEAD` で `main` を advertise し、clone が
  orchestrator と同じ default-branch identity を使うことを確認します。
- WI-255 の 3 言語 Work Item と 3 言語 reference-parity projection を、immutable な
  `.ai` lifecycle byte を変更せず `Implemented` にします。

## Lifecycle handoff

repository workflow は次の通りです。

```text
close → visible Outcome → post-close plan/apply → check-all → terminal CI
```

WI-257 自体は verified close まで conditional status を維持します。parity ledger は future
archived Contract、verification evidence、finalization chain、close receipt path を示しますが、
Runtime が作成する前にそれらの存在を主張しません。

## References

- [Agent workflow](../reference/agent-workflow.ja.md)
- [Commands](../reference/commands.ja.md)
- [Reference parity](../reference/reference-parity.ja.md)
- [Failed predecessor PR #208](https://github.com/xinglun/ai-cockpit/pull/208)
