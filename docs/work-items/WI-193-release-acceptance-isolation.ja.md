---
author: AI Cockpit maintainers
title: "WI-193 — Release acceptance isolation hardening"
description: "adopter harness の cleanup、source manifest、allowed-root symlink containment を fail closed にします。"
audience:
  - maintainer
  - reviewer
workItemId: WI-193-release-acceptance-isolation
status: historical
authority: canonical
lastVerifiedBy: WI-195-governance-recovery-gate
---

# WI-193 — Release acceptance isolation hardening

WI-193 は immutable な履歴 predecessor です。正しい公開 Runtime context で lifecycle receipt を更新できなかったため、predecessor は red/blocked のまま保持し、green completion とは表示しません。現在の bounded delivery は WI-195 で継続します。

WI-193 は、両 adopter harness が一時 run root を作成する前に EXIT cleanup
handler を登録します。このため toolchain 解決または setup の失敗でも checksum
付き `acceptance.json` と `cleanup.json` receipt を生成し、run root を残しません。

source isolation は、すべての tracked/untracked source path と ignored content を
含む全 `.ai` entry の deterministic typed manifest を比較します。宣言済み output
subtree だけを除外し、output ancestor directory の metadata を正規化するため、
source checkout 内への evidence 書き込みを mutation と誤判定しません。TMPDIR と
CARGO_HOME の manifest は symlink metadata、literal target、resolved target を保持し、
各 allowed root の外側を指す target を拒否します。

commit 済みの v0.2.23 public adopter と v0.2.22 → v0.2.23 N-1 receipt は、どちらも
`aarch64-apple-darwin` を明記します。Linux x86_64 は Release workflow の CI
coverage であり、2 番目の完全な adopter evidence target ではありません。この Work
Item は published Release、tag、historical evidence、Runtime Core、crates、CI parity
file を書き換えません。immutable recovery receipt は
[WI-193 recovery](../../.ai/decisions/WI-193-release-acceptance-isolation.recovery.json) です。

[English](WI-193-release-acceptance-isolation.md) ·
[简体中文](WI-193-release-acceptance-isolation.zh-CN.md)
