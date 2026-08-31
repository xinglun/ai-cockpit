---
author: AI Cockpit maintainers
title: "发布与分发"
description: "面向读者的安装、验证、升级、回滚与 MCP 指南。"
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

# 发布与分发

发布完成后，当前安装基线是公开且绑定身份的 `v0.2.52` Release；在 provider Release
存在之前，应使用上一个公开的 `v0.2.50` archive。预留的 `v0.2.51` tag 是不可变的发布失败尝试
（workflow run `33417057474`）：它是 lightweight tag，没有 provider Release，永远不能复用或作为安装基线。
Homebrew 和手动安装都使用公开 archive
与 manifest；仓库配置仍使用 `cockpit.toml`，安装 runtime 不会在目标仓库创建 `.ai`。
同一套验收 harness 既有发布前 staged-candidate 模式，也有发布后 public-Release 模式；
两者都不会从源码 workspace 获取 Runtime。
此前公开的 `v0.2.50` 基线继续作为历史 evidence 保留，不会复用为当前安装 identity。未公开的 `v0.2.49` 标签（workflow run `33379366308`）作为发布前失败的不可变历史保留，不是安装基线。

`v0.2.46` 标签保留为不可变的发布失败历史（`33330269507`）；它从未创建公开
Release，也不是安装基线。失败根因是打 tag 时尚未执行已关闭 Work Item 的强制文档 promotion。

未公开的 `v0.2.36` 标签保留为不可变的 staged 验收失败历史，不是安装基线。

`v0.2.35` 标签作为失败的发布历史保留（workflow run `33162800569`）；它没有公开
Release 或可安装制品，也不是安装基线。更早的 `v0.2.34` 失败（workflow run
`33155382717`）同样保留为历史。

WI-239 持久化的 provider snapshot 与当前 provider API 都将该 Release 报告为
`immutable: false`。因此其身份是可检测漂移的，而不是 provider 保证不可变：tag、
release manifest、`SHA256SUMS`、archive digest 与发布后 receipt 必须相互一致。

不可变的 `v0.2.30` tag 记录了因 active Work Item 目录缺失导致的发布路由失败；它没有公开
Release，并作为失败历史保留。预留的 `v0.2.24` tag 记录了一次发布前治理质量门失败，不可变的 `v0.2.25` tag 又记录了后续的
source-quality 失败；`v0.2.26` 也记录了后续 source-quality 失败；它们都没有公开 Release，都是不可变历史，不是安装基线。
`v0.2.32` tag 也因 WI-299 修复前的 adopter finalization 绑定缺陷而保留为失败的 staged 发布历史；
它没有公开 Release，不是安装基线。

## CI 质量门与 Runtime shadow 边界

CI 以版本化 `repository_gate_manifest.json` 作为规范 gate 集。类型化 receipt 根据
changed paths、Contract risk 与 workflow stage 选择累加的 `light`、`standard` 或
`strict`。未知、release-owned、高风险、merge 与 release 输入都 fail closed 到
`strict`。runner 校验 Git revisions、Contract 与 manifest digest，再只执行 receipt 中
有序的 gate ID；不能用任意命令替代。

release source quality 始终请求 `strict`。manifest 管理的 Cargo gates 使用逐 package
确定性测试，CI 与 release 都上传 route 和 gate receipts。`.gitattributes` 从 source
archive 排除 `.ai` 与生成目录，同时保留 Cargo 源码和 lockfile。

历史 Runtime shadow 基线是固定的公开 `v0.2.28`；当前 release route 还会验证
`v0.2.52`。`tests/ci/runtime_verify_shadow.sh` receipt 是 standard/strict route 的 **execution
smoke**。它验证公开且绑定身份的 `v0.2.52`，并使用仓库规范 profile。它不宣称 Runtime
全局 T0–T3 route、affected graph 完整性、跨 Work Item 物理执行或每个 Work Item 的
evidence coverage。参考 Makefile orchestration 在本 Rust 仓库中属于
different-by-design，不会复制。Runtime 全局路由与通用 CLI `verify --command` 语义超出
WI-224 的非 `crates/**` scope，明确 deferred。

## 发布候选版本

审查合并 Work Item 且同步默认分支后，通过推送 annotated Git tag 触发发布。使用以下命令；不要使用
`gh release create`，因为它可能在 workflow 验证候选版本之前创建 provider Release 和 lightweight tag：

```bash
git fetch origin main --tags
git tag -a v0.2.52 -m 'ai-cockpit v0.2.52'
test "$(git cat-file -t v0.2.52)" = tag
test "$(git rev-parse v0.2.52^{})" = "$(git rev-parse HEAD)"
git push origin v0.2.52
```

workflow 会拒绝 lightweight tag、已存在的 provider Release，或 peeled commit 不是已审查 source commit 的 tag。
发布失败后 tag 永久保留；下一个候选版本必须递增一个 patch 版本。

## 开始前

你需要一个已发布且绑定身份的 Release、目标 repository 路径，以及与操作系统匹配的 archive。Homebrew
安装需要已安装 Homebrew；macOS/Linux 手动校验使用 `shasum` 和 `awk`，Windows 使用 PowerShell。
`gh attestation verify` 是可选的额外 provenance 校验。

## macOS 主安装方式

在维护的 Homebrew tap 可用后，从已发布的 release line 安装 Formula：

```bash
brew install xinglun/tap/ai-cockpit
ai-cockpit --version
brew test xinglun/tap/ai-cockpit
```

升级和卸载：

```bash
brew update
brew upgrade xinglun/tap/ai-cockpit
brew uninstall ai-cockpit
brew untap xinglun/tap                 # 可选
```

当前 Formula 只支持 macOS ARM64 和 Intel；Linuxbrew 不属于支持路径。

## 验证 Release 制品

从同一个已发布 GitHub Release 下载 archive、`release-manifest.json` 和 `SHA256SUMS`。
v0.2.52 的校验文件覆盖全部十个 archive/SBOM，因此只校验实际下载的 archive：

```bash
archive="ai-cockpit-v0.2.52-aarch64-apple-darwin.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
gh attestation verify "$archive" --repo xinglun/ai-cockpit
```

如果 Release 已存在，也可以使用 GitHub CLI 下载准确的三个文件：

```bash
archive="ai-cockpit-v0.2.52-aarch64-apple-darwin.tar.gz"
gh release download v0.2.52 --repo xinglun/ai-cockpit \
  --pattern "$archive" --pattern release-manifest.json --pattern SHA256SUMS
```

文件名、target、checksum、manifest 和 attestation subject 必须一致。单独的上传或
semantic tag 不能证明安装完整。
CLI 和 MCP 的 `verify` JSON 会输出 `runtimeVersion` 与 `runtimeDigest` 这两个 Runtime identity fact。
发布后 acceptance harness（不是 Core 自身）必须在接受 Release evidence 前，将它们绑定到公开下载的 binary；
在 harness 之外使用这些 JSON 时，比较责任属于调用者。

### 后续 candidate 的制品绑定 SBOM 策略

失败的 staged v0.2.32 没有可供 adopter 使用的公开资产，其失败记录保持不可变，不会被改写为成功
Release。v0.2.52 发布后，公开 bytes 才成为不可变事实；其 `SHA256SUMS` 覆盖五个 archive 与五个
按 target 命名的 SBOM，且每个 target SBOM 都绑定对应的 archive 与 executable。

使用 WI-241 边界构建的 release candidate 遵循更严格的契约。每个按 target 命名的 SPDX 2.3
文档保留 dependency scan，并增加一个 release-archive Package 与一个 release-binary File。
`DOCUMENT DESCRIBES` 该 Package，Package `CONTAINS` 该 File；两个节点都携带从实际 staged
archive 与其中 executable member 计算的非零 SHA-256。错误的 target、version、文件名、digest、
节点数量或关系会在 candidate 聚合前失败。source dependency scan 或 SBOM 文件名本身绝不等于
adopter acceptance。

封闭的公开资产集合是五个 archive、五个 target SBOM、`release-manifest.json`、
`ai-cockpit.rb` 与 `SHA256SUMS`。manifest 绑定十个 target 制品；`SHA256SUMS` 按稳定文件名顺序，
准确一次绑定这十个文件以及 manifest 与 Formula（它不能校验自身）。最终 provenance subject
集合覆盖同样的十三个公开文件。额外的 build-named SBOM、其他 orphan publishable 文件、重复或
缺失的 checksum entry、digest 不匹配都会 fail closed。现有 staged/public adopter acceptance
与 attestation gates 仍位于该验证之后。

## 发布后 adopter 验收

发布前，`staged_adopter_acceptance` 把下载的 candidate archive、manifest 与 checksums
绑定到 source `HEAD`，执行规范 adopter lifecycle、isolation checks 与 cleanup proof。
独立的 `staged_adopter_upgrade_acceptance` 使用上一公开 Release 升级到该 staged target。
publish 依赖这两个 job。其 receipt 记录 `stagedCandidate: true` 和
`releasePublished: false`，不会改写 provider Release truth。

维护者可以在 Release 发布后重复执行公开 binary 验收基线。
不可变的 `v0.2.36` tag 当前记录了一次 staged 验收失败，没有公开 Release，也没有 adopter 基线。
仓库保留的 WI-239 receipt 仍是历史 v0.2.31 基线。后续成功 Release 必须先持久化自己的公开 binary
receipt，才能被描述为 adopter 基线；仅有 hosted job artifact 不构成仓库持久化基线。

持久化 adopter acceptance 基线：`aarch64-apple-darwin`（WI-419，公开的
`v0.2.44`；binary digest 为
`sha256:69d28c970c2b89534e63cb685c6cc02a2f135d3067b6a84feaabce2adce1d5e5`）。
完整 receipt 保存在 `.ai/evidence/WI-419-release-v0-2-44-adopter-acceptance/`。
WI-416 作为不可变的历史 v0.2.43 基线保留，较早的 WI-239 receipt 仍是历史
v0.2.31 evidence；不会用 hosted job artifact 替代仓库持久化基线。
GitHub Actions run `32696048024` 仍单独作为 `x86_64-unknown-linux-gnu` 的 hosted Linux
验收 evidence 保留，不是本次单 target 的持久化基线。

```bash
tests/release/adopter_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --tag v0.2.52 \
  --target aarch64-apple-darwin \
  --output ./release-adopter-acceptance
```

harness 只下载指定的公开 Release，按 SHA-256 固定解压后的 binary，创建隔离的 Cargo adopter，执行
attach/profile/Agent doctor，保持 `first-adopter-smoke` 为 `not_ready`，验证 Work Item lifecycle 和 evidence reuse，
并生成 `acceptance.json` 与 `SHA256SUMS`。它不会使用 workspace 或本地 Runtime binary。发布后验收失败时仍记录
`releasePublished: true` 和 `adopterAcceptance: failed`，不会重写已发布的 Release。第二技术栈覆盖属于后续独立 Work Item。

这份发布后 receipt 的 lifecycle close 必须是完整的结构化 Human Decision。harness 要求记录 actor、authority source、reason、evidence reference、policy reference、决定时间和 resume condition；它会把常规且非符号链接的 `.ai/decisions/<work-item>.close.json` 复制到验收 artifact，并生成包含 adopter `repositoryId`、Work Item ID、决定摘要和校验结果的 binding record。缺失、foreign、字段不完整或 identity 不匹配的 close receipt 都会 fail closed；不会把已发布的 Release 改写成未发布。

在旧 Work Item 和新 Work Item close 之前，harness 都必须执行 Runtime 的资源收尾边界：`finalize-plan` 在 verification 之前绑定 fixture 的 branch/worktree context。归档之后，harness 提交 fixture branch 上的归档记录，将其 fast-forward 到幸存的 control worktree，删除精确的 fixture branch 与 worktree，然后用 `disposition: deleted` 的 `finalize` 与 `finalize-verify` 记录结果。这不是装饰步骤；保留资源时 `close` 必须 fail closed。

在验收 receipt 输出最终确定后，成功、失败和中断路径都会只清理经过校验的临时 `run_root`。
`cleanup.json` 以及 `acceptance.json` 中的 `cleanupState` / `cleanupError` 记录清理结果。清理失败时必须
fail closed：进程以非零状态结束，receipt 变为 `adopterAcceptance: failed`，但 `releasePublished` 保持
`true`，不会把已发布的 Release 改写为未发布。
target 与 platform 始终显式记录；选择 Linux x86_64 target 时也遵循同一验收基线。

N-1 harness 只有在升级验收和清理都通过时才返回零；未设置的退出状态不会被当作成功。

隔离 receipt 包含文件、目录、symlink、metadata 与 digest 的 typed before/after manifest。HOME 和
XDG_CONFIG_HOME 是禁止写入的 root；TMPDIR 与 CARGO_HOME 明确分类为允许 Runtime 写入的 root，相关写入
会被记录，不会被误认为全局配置写入。

公开和 N-1 harness 都会在进入隔离环境前解析宿主机的 `RUSTUP_HOME` 与 active toolchain，显式传入
`RUSTUP_TOOLCHAIN`；无法解析时拒绝隐式下载 Rust toolchain。

为避免发布后遗漏配置或文档版本，release workflow 会从 Cargo metadata 推导当前版本，并运行
`tests/release/version_consistency.sh`。source check 会校验三种语言的入口和当前 archive 示例，
post-release check 会校验公开 Release 的 manifest 与 asset 名称。历史 N-1 引用会明确保留，不会被误认为当前基线。

CI 和 release workflow 中的 action 都固定到完整 commit SHA；其中基于 Node 的 action 使用官方稳定的
Node24-compatible 基线。`tests/release/action_runtime_policy.sh` 会同时检查两个 workflow，发现旧 ref、未固定
ref 或缺少必需 action 时 fail closed。今后更新 action runtime 时，必须同步更新该 policy 与本节说明。

### 历史 N-1 schema 迁移验收

发生 schema 变化的基线是历史上的 v0.1.1 到 v0.2.0 迁移。v0.2.52 是保持同一
schema 的 patch Release；其 N-1 run 仍使用同一个 harness，在确认 compatibility 后记录
`migrationState: not_required`。当前 N-1 run 使用紧邻的上一个公开 Release 与当前 Runtime，例如：

```bash
tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --from-tag v0.2.50 \
  --to-tag v0.2.52 \
  --target aarch64-apple-darwin \
  --output ./release-adopter-upgrade-acceptance
```

它证明旧 adopter 检测、审查门控迁移、历史字节保持、继续运行以及隔离的
repository/runtime identity。这是发布后 evidence，不能用源码构建替代，也不能改写
Release truth。迁移验收 artifact 与 adopter 安装路径分开维护。

历史 v0.1.1 到 v0.2.0 的 schema 迁移 evidence 仍保留在归档中。由于 v0.2.0 Runtime
早于相邻迁移链 receipt 字段，当前 harness 不重新运行这个历史 pair。

发布前 staged N-1 job 使用公开 N-1 archive 与 staged candidate archive，不使用源码构建
或任意 verification command。发布后，release workflow 会在 publication handoff 之后用独立的
`adopter_upgrade_acceptance` job 执行这个 harness。对于 tag push，workflow 通过 provider
API 解析紧邻的上一个已发布 semantic Release。第一个公开 Release 会写入带 checksum 的
`adopterAcceptance: not_applicable` receipt。维护者也可以手动触发 workflow，提供
`from_tag`、`to_tag` 和可选的 `target`；手动触发只消费已经发布的 artifact，永远不会发布
Release。即使验收失败，job 也会上传 `acceptance.json`、各步骤 JSON/stderr、两个 Runtime
identity 记录和 `SHA256SUMS`。

## 手动 archive 安装

macOS/Linux 用户下载对应的 `.tar.gz` 和 `SHA256SUMS`，选择准确的 Rust target，校验 archive，
再将 `ai-cockpit` 放入 `$HOME/.local/bin`：

```bash
target="aarch64-apple-darwin" # 选择与机器匹配的 target
archive="ai-cockpit-v0.2.52-${target}.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
mkdir -p "$HOME/.local/bin"
tar -xzf "$archive"
install -m 0755 ai-cockpit "$HOME/.local/bin/ai-cockpit"
case ":$PATH:" in
  *:"$HOME/.local/bin":*) ;;
  *) echo "请先将 $HOME/.local/bin 加入 PATH" >&2; exit 1 ;;
esac
"$HOME/.local/bin/ai-cockpit" --version
```

Windows 用户下载 `.zip` 和 `SHA256SUMS`，比较准确 checksum，解压到用户 bin 目录，并将该目录加入用户 `PATH`：

```powershell
$archive = "ai-cockpit-v0.2.52-x86_64-pc-windows-msvc.zip"
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

## Rust 开发者 fallback

该 fallback 适用于当前已发布且绑定身份的 `v0.2.52` tag。
发布完成后，workspace 含多个 package，必须显式选择 `cockpit-cli`：

```bash
cargo install --git https://github.com/xinglun/ai-cockpit.git --tag v0.2.52 --locked --root "$HOME/.local" --bin ai-cockpit cockpit-cli
"$HOME/.local/bin/ai-cockpit" --version
cargo uninstall --root "$HOME/.local" cockpit-cli
```

## 回滚

回滚时下载指定的历史 Release archive，验证其 manifest 与 digest 后再手动替换 binary。无版本号的
Homebrew Formula 始终跟踪当前 release，不是回滚选择器。

## MCP 与 repository attach

从已安装 runtime 启动本地 MCP adapter，并显式绑定 repository：

```bash
ai-cockpit mcp --repo /path/to/attached-repository
```

安装和 repository attach 是两个独立操作。审阅目标 Work Item 后再显式执行：

```bash
ai-cockpit attach --repo /path/to/repository
```

需要面向人的 MCP 结果时，使用明确 `workItemId` 和可选 `language` 调用 `work_item_outcome`。其文本 content
与 CLI 使用相同的 human handoff；`work_item_get` 仍是原始机器查询。

MCP client 配置示例（不同 client 的配置键可能不同）：

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

关键契约是 installed binary、`mcp` 子命令和显式 repository path。安装本身不会 attach 或修改 repository。
