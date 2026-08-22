# WI-118 — Release toolchain と cleanup fail-closed の修正

## 目的

公開 adopter と N-1 の Release acceptance を隔離環境で決定的かつ fail closed にする。
WI-117 のレビュー指摘を扱うが、WI-117 の過去の archive evidence は書き換えない。

## 範囲

- 両方の Release harness が隔離 root に入る前に host の `RUSTUP_HOME` と active toolchain を明示的に bind する。
- cleanup failure は non-zero で終了し、`adopterAcceptance` を failed にする。ただし `releasePublished: true` は維持する。
- static regression を追加し、三言語の Release 文書を同期する。

## 範囲外

Runtime protocol semantics、global Rust installation、公開済み Release の変更。

## 受入れ

1. toolchain identity がない場合、公開と N-1 の harness は暗黙の rustup download を拒否する。
2. cleanup failure は `cleanup.json` と `acceptance.json` に記録され、process は失敗し passed receipt を残さない。
3. 公開後の cleanup failure が immutable な Release truth を変更しない。
4. static harness test と三言語の Release 文書チェックがすべて成功する。
