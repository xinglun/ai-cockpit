# WI-116：merge 後の archive close evidence

## 目的

レビュー済み branch の merge 後も明示的な `archive → close` lifecycle を完了
できるようにする。archive 済み Work Item を close するときは immutable な
verification evidence、archive manifest、outcome binding、repository identity、
Runtime identity を検証し、現在の Git snapshot が変わっただけで stale と判定
しない。

WI-115 はすでに merge 済みで archive bytes が immutable なので、安全に同じ
Work Item へ修正を追加できない。このため successor として扱う。

## 範囲

- close の governance gate は archive evidence の検証を使う。
- active/finish/archive gate は引き続き current snapshot に bind する。
- 改ざんまたは identity 不一致の evidence、archive manifest、repository、Work Item、
  foreign Runtime は fail closed のままにする。
- archive 後の merge commit と structured close の回帰テストを追加する。
- immutable archive-manifest 境界を三言語の文書に記載する。

## 受け入れ条件

- 正しい archive 済み Work Item が merge commit 後に close できる。
- 改ざんまたは identity 不一致の evidence は拒否される。
- 既存の lifecycle と archive-integrity tests が通過する。
- Runtime は structured human decision を生成し、archive bytes を書き換えない。

## 状態

Runtime verify、archive、close が完了するまで進行中。
