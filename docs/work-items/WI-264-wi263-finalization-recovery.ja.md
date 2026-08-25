---
author: AI Cockpit maintainers
title: "WI-264 — WI-263 finalization recovery"
workItemId: WI-264-wi263-finalization-recovery
description: "immutable な predecessor bytes を書き換えず、merge 済み WI-263 の resource boundary を回復します。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-264-wi263-finalization-recovery
authority: canonical
---

# WI-264 — WI-263 finalization recovery

## Intent

installed Runtime は predecessor head が stale な post-merge finalization
transition を拒否しました。本 Work Item は bounded successor として WI-263
の歴史を保持し、正確な resource cleanup boundary だけを担当します。

## 観測された境界

PR #215 は merge commit
`47c9dd8e7107526f280274a92ccc7399493125cb` で merge 済みで、reviewed feature
head は `ce7af9def1ccf4066eded50f56d1a5b301f1ca8b` です。WI-263 の immutable
pre-merge finalization root は
`bc8f8e655a7616965b06ddacbc0feb0c807e64a0` に bind されたままです。間に
文書修正があるため、Runtime はそれを append-only receipt として扱うことを
正しく拒否しました。

Runtime が生成した
`.ai/decisions/WI-263-wi260-reconciliation.recovery.json` は WI-263 を
supersede しますが、archive、evidence、Outcome、events、finalization root
は書き換えません。

## Acceptance boundary

- WI-263 の historical bytes は byte-identical のままです。
- recovery receipt は predecessor digests と Runtime identity を bind します。
- merge 済み PR head と merge commit を provider fact として記録します。
- valid な Runtime finalization receipt と local postcondition を確認した後
  だけ、正確な branch と worktree を削除します。
- English、Simplified Chinese、Japanese の文書を同期します。

## Verification

- 明示的な `--repo` を付けた installed Runtime の `inspect`、`status`、`doctor`。
- 本 Work Item に bind した Runtime verification。
- provider merge と正確な branch/worktree cleanup の確認。
- `tests/ci/governance_integrity_gate_test.sh`。
- documentation parity と acceptance checks。

## Evidence boundary

この recovery は WI-263 の stale な finalization chain を green にせず、履歴
records も書き換えません。current terminal boundary を確立できるのは、本
Work Item の fresh verification、provider receipt、archive、structured close
decision だけです。
