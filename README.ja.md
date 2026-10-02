---
author: AI Cockpit maintainers
title: "AI Cockpit"
description: "AI coding agent のための repository governance：明示的な scope、verification evidence、監査可能な human decision。Rust 製で CLI と Model Context Protocol（MCP）interface を提供します。"
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - repository_governance_layer
---

# AI Cockpit

[English](README.md) | [中文](README.zh-CN.md)

AI coding agent のための repository governance です。scope を明示し、verification evidence をレビュー可能にし、human decision を別に記録します。Rust 製の CLI と、Model Context Protocol（MCP）用の local adapter を提供します。

## Agent が「完了」と言ったとき

変更は宣言した scope 内ですか。必要な check は現在の repository で実行されましたか。approval はレビュー中の変更に対して有効ですか。

作業開始前に、人が [Work Item Contract](docs/getting-started/first-work-item.ja.md) を記録します。タスク、agent が変更できる範囲、完了の確認方法、結果を approval できる人を明記します。verification はそこで定めた check と repository state の evidence を記録します。人の決定は明示され、監査できます。check に合格しても作業の approval にはなりません。

各 repository は専用の `.ai/` directory に governance state を保存します。

通常の repository-only 変更は `start --prepare` から始めます。Contract を記録します。preflight は作業と repository が開始可能かを確認します。人の判断が不要なら、編集前の repository snapshot（checkpoint）も保存します。

実装後、`verify` が Contract で指定された check の evidence を記録します。`finish` は evidence が現在の repository snapshot と一致するかを確認し、Work Item Outcome を記録します。`archive` は Work Item の記録を保存します。

verification と audit evidence を保持します。active Work Item の Runtime next action に従ってください。

ローカル branch または worktree のみを使う作業では、`close` が判断を記録してからローカルの cleanup を行います。PR、branch、worktree を provider が管理する場合は、`close` の前に、宣言済みの cleanup を完了して検証します。

対象 resource が宣言されていない場合は、Runtime が示す `close` 手順に従います。provider 操作は追加しません。証拠の唯一のコピーを保持する worktree は削除しないでください。詳細は[Agent workflow reference](docs/reference/agent-workflow.ja.md)を参照してください。

## 実際の初回利用を試す

Apple Silicon macOS では、次の command で公開済み stable v1.0.0 を install し、archive の SHA-256 を検証できます。curl、shasum、tar、install が必要です。

~~~bash
set -eu
asset=ai-cockpit-v1.0.0-aarch64-apple-darwin.tar.gz
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
curl -fL "https://github.com/xinglun/ai-cockpit/releases/download/v1.0.0/$asset" -o "$asset"
printf '%s  %s\n' 3af024699ffdc14e095273945d55507a950c6c115ae1ea8f228ef5425fb7b4f3 "$asset" | shasum -a 256 -c -
tar -xzf "$asset" ai-cockpit
mkdir -p "$HOME/.local/bin"
install -m 0755 ai-cockpit "$HOME/.local/bin/ai-cockpit"
export PATH="$HOME/.local/bin:$PATH"
ai-cockpit --version
~~~

新しい terminal でも使う場合は、$HOME/.local/bin を shell の起動時 PATH に追加してください。

この command は Apple Silicon macOS 専用です。Linux ARM64（GNU）、Linux x86_64（GNU）、Windows x86_64 の stable artifact があります。正確な filename と checksum は[配布ガイド](docs/release/distribution.ja.md)を参照してください。install 後、操作権限のある repository を指定します。POSIX shell では次を実行します。

~~~bash
repo=/path/to/repository
ai-cockpit --version
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
~~~

Windows PowerShell では repository path を指定して、同じ初回利用手順を実行します。

~~~powershell
$repo = "C:\path\to\repository"
ai-cockpit --version
ai-cockpit inspect --repo $repo
ai-cockpit attach --repo $repo
ai-cockpit status --repo $repo
ai-cockpit doctor --repo $repo
~~~

**inspect** は repository facts を読み取ります。**attach** は governance state を保存する `.ai/` directory を repository 内に初期化します。Agent instructions の install や global MCP settings の変更は行いません。**status** は repository state と Runtime 互換性を要約します。**doctor** は attach 状態、protocol version、Runtime 互換性を確認します。実際の出力を確認してください。install や attach だけでは work の approval や verification を意味しません。[最初の Work Item walkthrough](docs/getting-started/first-work-item.ja.md)に全体の手順があります。

## Stable 版と optional prerelease

既定では stable v1.0.0 を使います。[Release page](https://github.com/xinglun/ai-cockpit/releases/tag/v1.0.0)に Apple Silicon macOS、Linux ARM64 GNU、Linux x86_64 GNU、Windows x86_64 の artifact があります。v1.0.0 には Intel macOS、Linux musl、Windows ARM64 向け Release archive はありません。

macOS ARM64 v1.0.1-rc.1 は独立試用向けの optional prerelease です。既定の install 先ではありません。この prerelease の正式な release acceptance check はまだ完了していません。[prerelease page](https://github.com/xinglun/ai-cockpit/releases/tag/v1.0.1-rc.1)。

## 境界

AI Cockpit は scope、宣言済み verification、evidence freshness、human decision を記録します。production sandbox は提供せず、branch protection も設定しません。外部 provider の identity も証明しません。人間による review や security policy を置き換えるものではありません。check の記録は、その check と repository state の evidence です。普遍的な安全性や性能向上を証明するものではありません。

## 続けて読む

- [Getting started](docs/getting-started/README.ja.md)
- [Capabilities and boundaries](docs/capabilities.ja.md)
- [Release と配布](docs/release/distribution.ja.md)
- [Agent workflow reference](docs/reference/agent-workflow.ja.md)
- [Contributing](CONTRIBUTING.md)
