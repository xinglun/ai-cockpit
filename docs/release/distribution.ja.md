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

現在の installation baseline は公開済みで identity-bound な `v0.2.39` Release です。Homebrew と manual install は
public archive と manifest を使い、Repository configuration は `cockpit.toml` のままです。runtime の install は
対象 repository に `.ai` を作成しません。同じ acceptance harness に publication 前の staged-candidate mode と publication 後の public-Release mode があり、どちらも source workspace から Runtime を取得しません。

未公開の `v0.2.36` tag は staged acceptance failure の immutable な履歴として保持し、installation baseline にはしません。

`v0.2.35` tag は失敗した publication の履歴として保持します（workflow run `33162800569`）。public Release や installable artifact はなく、installation baseline ではありません。先行する `v0.2.34` の失敗（workflow run `33155382717`）も同様に保持します。

WI-239 に保存した provider snapshot と現在の provider API は、この Release を
`immutable: false` と報告します。従って identity は drift-detectable ですが provider-immutable
ではありません。tag、release manifest、`SHA256SUMS`、archive digest、post-release receipt
が一貫していなければなりません。

immutable な `v0.2.30` tag は active Work Item directory がない clean-batch で release route が失敗した事実を記録し、公開 Release はありません。失敗履歴として保持します。予約済みの `v0.2.24` tag は公開前 governance gate failure を記録し、immutable な `v0.2.25` tag も後続の
source-quality failure を記録しています。どちらにも公開 Release はありません。これらは immutable history であり
 installation baseline ではなく、`v0.2.24`、`v0.2.25`、`v0.2.26` は公開されていない immutable な失敗履歴です。
`v0.2.32` tag も WI-299 前の adopter finalization binding defect による staged 公開失敗履歴として保持され、公開 Release はなく installation baseline ではありません。

## CI quality gate と Runtime shadow の境界

CI は versioned `repository_gate_manifest.json` を canonical gate set とします。型付き
receipt は changed paths、Contract risk、workflow stage から累積的な `light`、
`standard`、`strict` coverage を選択します。unknown、release-owned、high-risk、merge、
release inputs は `strict` へ fail closed します。runner は Git revisions、Contract と
manifest digest を検証し、receipt の順序付き gate ID だけを実行します。任意 command
で置換できません。

release source quality は常に `strict` を要求します。manifest-owned Cargo gates は
deterministic package-by-package tests を使い、CI と release は route/gate receipts を
upload します。`.gitattributes` は source archive から `.ai` と generated roots を除外し、
Cargo sources と lockfile を保持します。

過去の Runtime shadow baseline は pinned public `v0.2.28` であり、現在の release route は
`v0.2.39` も検証します。`tests/ci/runtime_verify_shadow.sh` receipt は standard/strict route の **execution smoke**
です。identity-bound public `v0.2.39` を検証し、repository の canonical profile を実行します。
Runtime-global T0–T3 route、affected graph completeness、cross-Work-Item physical execution、
Work Item ごとの evidence coverage は claim しません。reference Makefile orchestration は
この Rust repository では different-by-design で copy しません。Runtime-global routing と
generic CLI `verify --command` semantics は WI-224 の non-`crates/**` scope 外として deferred
です。

## 開始前

公開済みの identity-bound Release、対象 repository path、OS に合う archive が必要です。Homebrew install
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

同じ公開済み GitHub Release から archive、`release-manifest.json`、`SHA256SUMS` を取得します。
v0.2.39 の checksum file は全十個の archive/SBOM を対象にするため、download した archive だけを検証します。

```bash
archive="ai-cockpit-v0.2.39-aarch64-apple-darwin.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
gh attestation verify "$archive" --repo xinglun/ai-cockpit
```

Release 公開後は GitHub CLI で正確な 3 ファイルを取得することもできます。

```bash
archive="ai-cockpit-v0.2.39-aarch64-apple-darwin.tar.gz"
gh release download v0.2.39 --repo xinglun/ai-cockpit \
  --pattern "$archive" --pattern release-manifest.json --pattern SHA256SUMS
```

Filename、target、checksum、manifest、attestation subject は一致しなければなりません。
Upload や semantic tag だけでは install の完了 evidence になりません。
CLI と MCP の `verify` JSON は `runtimeVersion` と `runtimeDigest` という Runtime identity fact を返します。
公開後の acceptance harness（Core 自体ではありません）が Release evidence を受け入れる前に、公開 download binary の
identity と結び付けます。harness 外で JSON を使う場合の比較責任は caller にあります。

### 以降の candidate に対する artifact-bound SBOM policy

失敗した staged v0.2.32 には adopter が使える公開 asset がありません。その失敗履歴は immutable
なまま保持し、成功した Release として再標識しません。v0.2.39 の公開後は bytes が immutable
となり、五つの archive と五つの target-named SBOM を `SHA256SUMS` が対象にします。

WI-241 boundary で build する release candidate には、より厳格な contract を適用します。
各 target-named SPDX 2.3 document は dependency scan を保持し、一つの release-archive
Package と一つの release-binary File を追加します。`DOCUMENT DESCRIBES` は Package を指し、
Package は File を `CONTAINS` します。両 node は実際の staged archive と executable member
から計算した nonzero SHA-256 を保持します。target、version、filename、digest、node cardinality、
relationship のいずれかが違えば candidate aggregation 前に失敗します。source dependency scan
や SBOM filename だけを adopter acceptance として扱うことはできません。

closed public inventory は五つの archive、五つの target SBOM、`release-manifest.json`、
`ai-cockpit.rb`、`SHA256SUMS` です。manifest は十個の target artifact を binding し、
`SHA256SUMS` はその十個と manifest と Formula を stable filename order で一度ずつ binding
します（自身は checksum できません）。final provenance subject set は同じ十三個の public file
を対象にします。追加の build-named SBOM、その他の orphan publishable file、checksum entry の
duplicate/missing、digest mismatch は fail closed です。既存の staged/public adopter acceptance
と attestation gate はこの validation の downstream に残ります。

## Post-release adopter acceptance

publication 前に `staged_adopter_acceptance` は download 済み candidate archive、manifest、
checksums を source `HEAD` に bind し、canonical adopter lifecycle、isolation checks、cleanup
proof を実行します。別の `staged_adopter_upgrade_acceptance` は直前の public Release から
staged target への upgrade を行います。publish は両 job に依存します。receipt は
`stagedCandidate: true` と `releasePublished: false` を記録し、provider Release truth を
書き換えません。

Maintainer は Release 公開後に public binary acceptance baseline を再実行できます。
immutable な `v0.2.36` tag は現在 staged acceptance failure を記録しており、公開 Release と adopter
baseline はありません。repository に保持された WI-239 receipt は historical v0.2.31 baseline として残ります。
今後の成功した Release は、adopter baseline と呼ぶ前に自身の public-binary receipt を永続化する必要があります。
hosted job artifact だけでは repository-persisted baseline になりません。

永続化された adopter acceptance baseline: `aarch64-apple-darwin`（WI-239、公開
`v0.2.31`；provider metadata は `immutable: false` を記録）。GitHub Actions run
`32696048024` は `x86_64-unknown-linux-gnu` の hosted Linux acceptance evidence としてのみ保持し、
永続化された single-target baseline ではありません。

### 過去の N-1 schema migration 受入れ

schema が変わった基準は、過去の v0.1.1 から v0.2.0 への migration です。
v0.2.39 は同じ schema の patch Release ですが、N-1 run は同じ harness を使い、compatibility
を確認した後に `migrationState: not_required` を記録します。current N-1 run は直前の
public Release と current Runtime、例えば次のように実行します。

```bash
tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --from-tag v0.2.33 \
  --to-tag v0.2.39 \
  --target aarch64-apple-darwin \
  --output ./release-adopter-upgrade-acceptance
```

旧 adopter の検出、レビュー承認付き migration、履歴 bytes の保持、継続動作、
repository/runtime identity の隔離を検証する公開後 evidence である。source build で
代用したり Release truth を書き換えたりしてはならない。
Migration acceptance artifact は adopter の installation path とは分離して管理します。

過去の v0.1.1 から v0.2.0 への schema migration evidence は archive に保持されますが、
v0.2.0 Runtime は隣接 chain receipt field より前のため current harness でその pair は再実行しません。

publication 前の staged N-1 job は public N-1 archive と staged candidate archive を使い、
source build や任意 verification command に置換しません。publication 後、release workflow は
publication handoff の後に独立した
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
  --tag v0.2.39 \
  --target aarch64-apple-darwin \
  --output ./release-adopter-acceptance
```

Harness は指定した public Release だけを download し、展開した binary を SHA-256 で pin します。isolated Cargo adopter を作成し、
attach/profile/Agent doctor、`first-adopter-smoke` の `not_ready`、Work Item lifecycle、evidence reuse を検証し、`acceptance.json` と
`SHA256SUMS` を出力します。workspace や local Runtime binary は使いません。post-release acceptance が失敗しても
`releasePublished: true` と `adopterAcceptance: failed` を記録し、公開済み Release を書き換えません。second technology stack は別の Work Item です。

この post-release receipt の lifecycle close は完全な structured Human Decision でなければなりません。harness は actor、authority source、reason、evidence reference、policy reference、決定時刻、resume condition を要求します。通常ファイルかつ symlink ではない `.ai/decisions/<work-item>.close.json` を acceptance artifact にコピーし、adopter の `repositoryId`、Work Item ID、decision digest、検証結果を含む binding record を生成します。close receipt の欠落、foreign、必須項目不足、identity 不一致は fail closed となり、公開済み Release を未公開へ書き戻すことはありません。

old Work Item と new Work Item の close 前に、harness は Runtime の resource-finalization boundary も実行します。`finalize-plan` が verification 前に fixture の branch/worktree context を bind します。archive 後、harness は fixture branch の archive 記録を commit し、存続する control worktree へ fast-forward した後、対象 branch と worktree を削除し、`disposition: deleted` の `finalize` と `finalize-verify` を記録します。これは表示だけの手順ではなく、resource が retain された場合に `close` が fail closed になる実際の lifecycle 要件です。

receipt の出力を確定した後、success、failure、interrupt のすべての経路で、検証済みの一時 `run_root` だけを削除します。
`cleanup.json` と `acceptance.json` の `cleanupState` / `cleanupError` が結果を記録します。cleanup failure は
fail closed とし、プロセスは non-zero で終了して receipt は `adopterAcceptance: failed` になりますが、
`releasePublished` は true のまま維持し、公開済み Release を未公開に書き戻しません。
target と platform は明示的に保持し、target として Linux x86_64 を選んだ場合も同じ基準で検証します。

N-1 harness は upgrade acceptance と cleanup の両方が成功した場合だけ zero を返し、未設定の exit status を成功として扱いません。

isolation receipt には file、directory、symlink、metadata、digest の typed before/after manifest を含めます。
HOME と XDG_CONFIG_HOME は write forbidden root、TMPDIR と CARGO_HOME は allowed Runtime-write root として
明示的に分類され、global configuration write と取り違えないよう記録します。

公開と N-1 の両 harness は隔離環境に入る前に host の `RUSTUP_HOME` と active toolchain を解決し、
`RUSTUP_TOOLCHAIN` を明示的に渡します。解決できない場合は暗黙の Rust toolchain download を拒否します。

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
archive="ai-cockpit-v0.2.39-${target}.tar.gz"
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
$archive = "ai-cockpit-v0.2.39-x86_64-pc-windows-msvc.zip"
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

この fallback は現在公開済みの identity-bound な `v0.2.39` tag で利用できます。Workspace は複数 package を含むため `cockpit-cli` を明示します。

```bash
cargo install --git https://github.com/xinglun/ai-cockpit.git --tag v0.2.39 --locked --root "$HOME/.local" --bin ai-cockpit cockpit-cli
"$HOME/.local/bin/ai-cockpit" --version
cargo uninstall --root "$HOME/.local" cockpit-cli
```

## Rollback

Rollback では、名前付きの過去 Release archive を取得し、manifest と digest を verify してから binary を
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
