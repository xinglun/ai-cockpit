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

WI-34 は installation contract を定義しますが、最初の public Release や
`xinglun/homebrew-tap` が既に存在すると主張しません。Repository configuration は
`cockpit.toml` のままで、runtime の install は対象 repository に `.ai` を作成しません。

## 開始前

公開済みの immutable Release、対象 repository path、OS に合う archive が必要です。Homebrew install
には Homebrew、macOS/Linux の manual verification には `shasum` と `awk`、Windows には PowerShell
を使います。`gh attestation verify` は追加の provenance check として任意です。

## macOS の primary install

WI-35 が検証済みの最初の Release を公開し Formula を merge した後に実行します。

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

現在の Formula は macOS ARM64 と Intel のみを対象とします。Linuxbrew は WI-34 の
supported path ではありません。

## Release artifact の verify

同じ immutable GitHub Release から archive、`release-manifest.json`、`SHA256SUMS` を取得します。
checksum file は全十個の archive/SBOM を対象にするため、download した archive だけを検証します。

```bash
archive="ai-cockpit-v0.1.0-aarch64-apple-darwin.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
gh attestation verify "$archive" --repo xinglun/ai-cockpit
```

Release 公開後は GitHub CLI で正確な 3 ファイルを取得することもできます。

```bash
archive="ai-cockpit-v0.1.0-aarch64-apple-darwin.tar.gz"
gh release download v0.1.0 --repo xinglun/ai-cockpit \
  --pattern "$archive" --pattern release-manifest.json --pattern SHA256SUMS
```

Filename、target、checksum、manifest、attestation subject は一致しなければなりません。
Upload や semantic tag だけでは install の完了 evidence になりません。

## Manual archive install

macOS/Linux では対応する `.tar.gz` と `SHA256SUMS` を download し、Rust target を選び、archive を
verify してから `ai-cockpit` を `$HOME/.local/bin` に置きます。

```bash
target="aarch64-apple-darwin" # machine に合う target を選ぶ
archive="ai-cockpit-v0.1.0-${target}.tar.gz"
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
$archive = "ai-cockpit-v0.1.0-x86_64-pc-windows-msvc.zip"
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

この fallback は WI-35 が immutable な `v0.1.0` tag を公開した後だけ利用できます。現在の source remote
にはまだその tag がありません。公開後も Workspace は複数 package を含むため `cockpit-cli` を明示します。

```bash
cargo install --git https://github.com/xinglun/ai-cockpit.git --tag v0.1.0 --locked --root "$HOME/.local" --bin ai-cockpit cockpit-cli
"$HOME/.local/bin/ai-cockpit" --version
cargo uninstall --root "$HOME/.local" cockpit-cli
```

## Rollback

Rollback では、名前付きの immutable な過去 Release archive を verify してから binary を
手動で置き換えます。Version を持たない Homebrew Formula は current release を追跡するため、
rollback selector ではありません。

## MCP と repository attach

Installed runtime から local MCP adapter を起動します。

```bash
ai-cockpit mcp
ai-cockpit mcp --repo /path/to/attached-repository
```

Install と repository attach は別操作です。対象 Work Item を review してから明示的に実行します。

```bash
ai-cockpit attach --repo /path/to/repository
```

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
