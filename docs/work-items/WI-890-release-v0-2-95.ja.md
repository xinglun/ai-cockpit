---
author: AI Cockpit maintainers
workItemId: WI-890-release-v0-2-95
title: 四方向収束後の governed v0.2.95 release
description: recovered/replaced された v0.2.95 公開経路を保持し、不変の失敗候補を公開済みとは主張しない。
audience: [maintainer, reviewer, adopter]
status: recovered
authority: explicit-user-authorization
lastVerifiedBy: WI-890-release-v0-2-95
---

# WI-890 — 四方向収束後の governed v0.2.95 release

この Work Item は、Outcome 配信修正、四方向収束、Issue #851、BAML に着想を
得た interface-discovery trial、Rust/toolchain 更新が完了した後の最後の公開
経路である。review 済み candidate の merge、immutable tag と Release の作成、
download した成果物の release acceptance harness 合格が揃って初めて release
完了とする。

これは recovered/replaced された過去の公開経路である。不変の候補履歴は保持
するが、v0.2.95 は公開されていない。残りの公開 evidence は WI-892 が引き継ぎ、
この記録を provider Release や download 済み成果物の成功証明として扱わない。

## Acceptance boundary

- v0.2.93 を N-1 baseline として保持し、review 済みで同期した main commit から
  v0.2.95 だけを公開する。
- isolated root で package、checksum、manifest、Release asset、download 済みの
  fresh-install/N-1 upgrade acceptance を検証する。
- 既知の unknown を明示する。現行 performance evidence は noise を超える退化を
  示さないが development-cycle 改善は証明していない。host を設定しない場合の
  default host display confirmation も unknown のままである。
- object repository の main branch を変更せず、製品 scope を追加せず、生成した
  report をユーザー対話への表示証明として扱わない。

## Verification plan

provider を変更する前に、宣言した format、governance、documentation、package、
candidate、release acceptance check を完了する。`CARGO_INCREMENTAL=0` の共有
verification target を使い、command、exit code、log、artifact digest、environment
identity、cleanup evidence を記録する。acceptance が失敗した場合は evidence を
保持し、同じ tag の再公開で隠さない。

## Publication and post-release evidence

review 済み merge と hosted checks の green 後、repository の release workflow
で immutable な v0.2.95 tag/Release を公開する。`gh release download` で公開 asset
を取得し、workspace build ではなくその download に対して acceptance harness を
実行する。finalize と close の前に、install/upgrade 状態、release link、checksum と
manifest の identity、temporary root の正確な cleanup を記録する。
