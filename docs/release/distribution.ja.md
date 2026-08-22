---
author: AI Cockpit maintainers
title: "Release と配布"
description: "reader-first の installation、verification、upgrade、rollback、MCP guide。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
keywords: [ai-cockpit, installation, release, homebrew, mcp]
---

# Release と配布

現在の installation baseline は公開済みで immutable な `v0.2.9` Release です。Homebrew と manual install は
public archive と manifest を使い、Repository configuration は `cockpit.toml` のままです。runtime の install は
対象 repository に `.ai` を作成しません。Maintainer は post-release adopter acceptance harness を実行できますが、pre-release gate や Runtime command ではありません。

## 開始前

公開済みの immutable Release、対象 repository path、OS に合う archive が必要です。Homebrew install
には Homebrew、macOS/Linux の manual verification には `shasum` と `awk`、Windows には PowerShell
を使います。`gh attestation verify` は追加の provenance check として任意です。

## macOS の primary install

maintained Homebrew tap が利用可能になった後、公開済み release line の Formula を install します。

```bash
brew install xinglun/tap/ai-cockpit
ai-cockpit --version
brew test xinglun/tap/ai-cockpit
```

Upgrade と uninstall：

```bash
brew update
brew upgrade xinglun/tap/ai-cockpit
brew uninstall ai-cockpit
brew untap xinglun/tap                 # optional
```

現在の Formula は macOS ARM64 と Intel のみを対象とします。Linuxbrew は supported path
ではありません。

## Release artifact の verify

同じ immutable GitHub Release から archive、`release-manifest.json`、`SHA256SUMS` を取得します。
checksum file は全十個の archive/SBOM を対象にするため、download した archive だけを検証します。

```bash
archive="ai-cockpit-v0.2.9-aarch64-apple-darwin.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
gh attestation verify "$archive" --repo xinglun/ai-cockpit
```

Release 公開後は GitHub CLI で正確な 3 ファイルを取得することもできます。

```bash
archive="ai-cockpit-v0.2.9-aarch64-apple-darwin.tar.gz"
gh release download v0.2.9 --repo xinglun/ai-cockpit \
  --pattern "$archive" --pattern release-manifest.json --pattern SHA256SUMS
```

Filename、target、checksum、manifest、attestation subject は一致しなければなりません。
Upload や semantic tag だけでは install の完了 evidence になりません。
CLI と MCP の `verify` JSON は `runtimeVersion` と `runtimeDigest` という Runtime identity fact を返します。
公開後の acceptance harness（Core 自体ではありません）が Release evidence を受け入れる前に、公開 download binary の
identity と結び付けます。harness 外で JSON を使う場合の比較責任は caller にあります。

## Post-release adopter acceptance

Maintainer は Release 公開後に public binary acceptance baseline を再実行できます。

**v0.2.9 の完全な adopter acceptance baseline は `x86_64-unknown-linux-gnu` です。**
Release workflow は他の 4 target に build と smoke evidence を提供しますが、別の acceptance run が記録されない限り、
full adopter lifecycle の完了とは主張しません。

### 過去の N-1 schema migration 受入れ

schema が変わった基準は、過去の v0.1.1 から v0.2.0 への migration です。
v0.2.9 は同じ schema の patch Release ですが、N-1 run は同じ harness を使い、compatibility
を確認した後に `migrationState: not_required` を記録します。current N-1 run は直前の
public Release と current Runtime、例えば次のように実行します。

```bash
tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --from-tag v0.2.8 \
  --to-tag v0.2.9 \
  --target x86_64-unknown-linux-gnu \
  --output ./release-adopter-upgrade-acceptance
```

旧 adopter の検出、レビュー承認付き migration、履歴 bytes の保持、継続動作、
repository/runtime identity の隔離を検証する公開後 evidence である。source build で
代用したり Release truth を書き換えたりしてはならない。
Migration acceptance artifact は adopter の installation path とは分離して管理します。

過去の v0.1.1 から v0.2.0 への schema migration evidence は archive に保持されますが、
v0.2.0 Runtime は隣接 chain receipt field より前のため current harness でその pair は再実行しません。

release workflow は publication と publication handoff の後に、独立した
`adopter_upgrade_acceptance` job でこの harness を実行します。tag push の場合は provider
API から直前の published semantic Release を解決します。最初の public Release では checksum
付き receipt に `adopterAcceptance: not_applicable` を記録します。maintainer は `from_tag`、
`to_tag`、任意の `target` を指定して workflow を手動実行することもできます。manual dispatch
は既に公開された artifact だけを消費し、Release を publish しません。失敗時も
`acceptance.json`、step ごとの JSON/stderr、両方の Runtime identity、`SHA256SUMS` を upload
します。

```bash
tests/release/adopter_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --tag v0.2.9 \
  --target x86_64-unknown-linux-gnu \
  --output ./release-adopter-acceptance
```

Harness は指定した public Release だけを download し、展開した binary を SHA-256 で pin します。isolated Cargo adopter を作成し、
attach/profile/Agent doctor、`first-adopter-smoke` の `not_ready`、Work Item lifecycle、evidence reuse を検証し、`acceptance.json` と
`SHA256SUMS` を出力します。workspace や local Runtime binary は使いません。post-release acceptance が失敗しても
`releasePublished: true` と `adopterAcceptance: failed` を記録し、公開済み Release を書き換えません。second technology stack は別の Work Item です。

receipt の出力を確定した後、success、failure、interrupt のすべての経路で、検証済みの一時 `run_root` だけを削除します。
`cleanup.json` と `acceptance.json` の `cleanupState` / `cleanupError` が結果を記録し、cleanup failure が acceptance truth を変更することはありません。
target と platform は明示的に保持し、target として Linux x86_64 を選んだ場合も同じ基準で検証します。

isolation receipt には file、directory、symlink、metadata、digest の typed before/after manifest を含めます。
HOME と XDG_CONFIG_HOME は write forbidden root、TMPDIR と CARGO_HOME は allowed Runtime-write root として
明示的に分類され、global configuration write と取り違えないよう記録します。

Release 後に configuration や documentation の version が取り残されないように、release workflow は
Cargo metadata から current version を解決し、`tests/release/version_consistency.sh` を実行します。
source check は三言語の route と archive example を検証し、post-release check は公開 Release の manifest
と asset name を検証します。過去の N-1 reference は明示的に保持され、current baseline と混同されません。

CI と release workflow の action はすべて full commit SHA に pin しています。Node を使う action は公式の
stable Node24-compatible baseline を使い、`tests/release/action_runtime_policy.sh` が両 workflow の stale、
unpinned、missing ref を fail closed で検査します。将来 action runtime を更新するときは、この policy と
この release note を同時に更新します。

## Manual archive install

macOS/Linux では対応する `.tar.gz` と `SHA256SUMS` を download し、Rust target を選び、archive を
verify してから `ai-cockpit` を `$HOME/.local/bin` に置きます。

```bash
target="aarch64-apple-darwin" # machine に合う target を選ぶ
archive="ai-cockpit-v0.2.9-${target}.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
mkdir -p "$HOME/.local/bin"
tar -xzf "$archive"
install -m 0755 ai-cockpit "$HOME/.local/bin/ai-cockpit"
case ":$PATH:" in
  *:"$HOME/.local/bin":*) ;;
  *) echo "$HOME/.local/bin を PATH に追加してから ai-cockpit を使ってください" >&2; exit 1 ;;
esac
"$HOME/.local/bin/ai-cockpit" --version
```

Windows では `.zip` と `SHA256SUMS` を download し、checksum を比較してから user bin directory に展開し、
その directory を user `PATH` に追加します。

```powershell
$archive = "ai-cockpit-v0.2.9-x86_64-pc-windows-msvc.zip"
$expected = Get-Content .\SHA256SUMS |
  Where-Object { ($_ -split '\s+')[1] -eq $archive } |
  ForEach-Object { ($_ -split '\s+')[0].ToLowerInvariant() }
$actual = (Get-FileHash .\$archive -Algorithm SHA256).Hash.ToLowerInvariant()
if ([string]::IsNullOrWhiteSpace($expected) -or $actual -ne $expected) { throw "Archive checksum mismatch" }
$destination = Join-Path $env:USERPROFILE "bin"
New-Item -ItemType Directory -Force -Path $destination | Out-Null
Expand-Archive .\$archive $destination -Force
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$userPath = if ([string]::IsNullOrEmpty($userPath)) { "" } else { $userPath }
if (($userPath -split ';') -notcontains $destination) {
  [Environment]::SetEnvironmentVariable("Path", ($userPath.TrimEnd(';') + ";" + $destination), "User")
}
$env:Path = "$destination;$env:Path"
& "$destination\ai-cockpit.exe" --version
```

## Rust developer fallback

この fallback は現在公開済みの immutable な `v0.2.9` tag で利用できます。Workspace は複数 package を含むため `cockpit-cli` を明示します。

```bash
cargo install --git https://github.com/xinglun/ai-cockpit.git --tag v0.2.9 --locked --root "$HOME/.local" --bin ai-cockpit cockpit-cli
"$HOME/.local/bin/ai-cockpit" --version
cargo uninstall --root "$HOME/.local" cockpit-cli
```

## Rollback

Rollback では、名前付きの immutable な過去 Release archive を verify してから binary を
手動で置き換えます。Version を持たない Homebrew Formula は current release を追跡するため、
rollback selector ではありません。

## MCP と repository attach

Installed runtime から repository を明示して local MCP adapter を起動します。

```bash
ai-cockpit mcp --repo /path/to/attached-repository
```

Install と repository attach は別操作です。対象 Work Item を review してから明示的に実行します。

```bash
ai-cockpit attach --repo /path/to/repository
```

人間向けの MCP result が必要な場合は、明示的な `workItemId` と任意の `language` で `work_item_outcome` を
呼び出します。text content は CLI と同じ human handoff であり、`work_item_get` は raw machine lookup のままです。

MCP client configuration の例です（client ごとに configuration key は異なります）。

```json
{
  "mcpServers": {
    "ai-cockpit": {
      "command": "ai-cockpit",
      "args": ["mcp", "--repo", "/path/to/attached-repository"]
    }
  }
}
```

重要な contract は installed binary、`mcp` subcommand、明示的な repository path です。Install 自体は
attach や repository mutation を行いません。
