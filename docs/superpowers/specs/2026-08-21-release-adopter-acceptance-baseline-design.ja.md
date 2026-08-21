# Release Adopter Acceptance Baseline 設計

## 目標

immutable な public Release artifact だけを使う、再現可能な post-release adopter acceptance
baseline を確立します。公開 GitHub Release binary が、source fallback と global Agent write なしで、
新しい adopter repository をゼロから governance できるかを証明します。

## 境界

最初は `tests/release/adopter_acceptance.sh` と release 後の GitHub Actions job だけを追加し、
`ai-cockpit acceptance` Runtime command は追加しません。script は acceptance harness であり、
adopter 向けの governance capability ではありません。

script は Runtime binary を得るための `cargo build`、`cargo run`、workspace binary、local `target`
binary を絶対に使いません。public Release archive を download し、manifest/checksum を検証し、
固定した absolute path の binary だけを呼び出します。Cargo は temporary adopter の通常の test に
だけ使えます。

## 入出力

```text
tests/release/adopter_acceptance.sh \
  --repository OWNER/REPOSITORY \
  --tag vX.Y.Z \
  --target TARGET \
  --output DIRECTORY
```

CI では全引数を明示し、missing/ambiguous input は fail closed にします。output には raw JSON
evidence、`acceptance.json`、全 evidence（自身を除く）を bind する `SHA256SUMS` を保存します。
summary は `releasePublished`、`adopterAcceptance`、各 step、repositoryId、runtime identity、
timestamp、failure reason を含みます。

post-release step が失敗しても `releasePublished: true` を保持し、`adopterAcceptance: failed` だけを
記録します。既存の Release truth は変更しません。

## Runtime identity と flow

`runtime.json` は tag、version、archiveDigest、binaryDigest、platform、archive、downloadSource、
releaseUrl、`releasePublished: true` を固定します。doctor、inspect、Work Item verification の
`runtimeVersion/runtimeDigest` が download artifact と一致することを検証します。

初期 Cargo adopter を commit してから attach、profile confirm、Agent list/install/doctor、
`first-adopter-smoke` の `not_ready` skeleton、完全な Work Item lifecycle を実行します。同一の
isolated environment で verify を二回実行し、最初は実行、次は receipt reuse と zero spawn を要求します。
最後に source checkout に `.ai/` がなく、isolated HOME/XDG の前後が同じであることを証明します。

`adopter_acceptance` job は tag push、`publish`、`publish_handoff` の後だけ実行し、candidate artifact
ではなく public Release から binary を取得します。失敗しても acceptance artifact を upload します。

## 範囲外

Runtime CLI command、Repository Protocol、global provider config、第二の technology stack は変更しません。
Node/npm adopter は別 Work Item とします。
