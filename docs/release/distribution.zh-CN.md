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

当前安装基线是公开且不可变的 `v0.2.16` Release。Homebrew 和手动安装都使用公开 archive
与 manifest；仓库配置仍使用 `cockpit.toml`，安装 runtime 不会在目标仓库创建 `.ai`。
维护者可以使用发布后的 adopter 验收 harness；它不是发布前 gate，也不是 Runtime 命令。

## 开始前

你需要一个已发布且不可变的 Release、目标 repository 路径，以及与操作系统匹配的 archive。Homebrew
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

从同一个不可变 GitHub Release 下载 archive、`release-manifest.json` 和 `SHA256SUMS`。
校验文件覆盖全部十个 archive/SBOM，因此只校验实际下载的 archive：

```bash
archive="ai-cockpit-v0.2.16-aarch64-apple-darwin.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
gh attestation verify "$archive" --repo xinglun/ai-cockpit
```

如果 Release 已存在，也可以使用 GitHub CLI 下载准确的三个文件：

```bash
archive="ai-cockpit-v0.2.16-aarch64-apple-darwin.tar.gz"
gh release download v0.2.16 --repo xinglun/ai-cockpit \
  --pattern "$archive" --pattern release-manifest.json --pattern SHA256SUMS
```

文件名、target、checksum、manifest 和 attestation subject 必须一致。单独的上传或
semantic tag 不能证明安装完整。
CLI 和 MCP 的 `verify` JSON 会输出 `runtimeVersion` 与 `runtimeDigest` 这两个 Runtime identity fact。
发布后 acceptance harness（不是 Core 自身）必须在接受 Release evidence 前，将它们绑定到公开下载的 binary；
在 harness 之外使用这些 JSON 时，比较责任属于调用者。

## 发布后 adopter 验收

维护者可以在 Release 发布后重复执行公开 binary 验收基线：

**v0.2.16 的完整 adopter acceptance 基线为 `x86_64-unknown-linux-gnu`。**
Release workflow 对其他四个已发布 target 提供 build 和 smoke evidence；除非另有独立验收记录，
不能宣称它们完成了完整 adopter lifecycle。

```bash
tests/release/adopter_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --tag v0.2.16 \
  --target x86_64-unknown-linux-gnu \
  --output ./release-adopter-acceptance
```

harness 只下载指定的公开 Release，按 SHA-256 固定解压后的 binary，创建隔离的 Cargo adopter，执行
attach/profile/Agent doctor，保持 `first-adopter-smoke` 为 `not_ready`，验证 Work Item lifecycle 和 evidence reuse，
并生成 `acceptance.json` 与 `SHA256SUMS`。它不会使用 workspace 或本地 Runtime binary。发布后验收失败时仍记录
`releasePublished: true` 和 `adopterAcceptance: failed`，不会重写已发布的 Release。第二技术栈覆盖属于后续独立 Work Item。

这份发布后 receipt 的 lifecycle close 必须是完整的结构化 Human Decision。harness 要求记录 actor、authority source、reason、evidence reference、policy reference、决定时间和 resume condition；它会把常规且非符号链接的 `.ai/decisions/<work-item>.close.json` 复制到验收 artifact，并生成包含 adopter `repositoryId`、Work Item ID、决定摘要和校验结果的 binding record。缺失、foreign、字段不完整或 identity 不匹配的 close receipt 都会 fail closed；不会把已发布的 Release 改写成未发布。

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

发生 schema 变化的基线是历史上的 v0.1.1 到 v0.2.0 迁移。v0.2.16 是保持同一
schema 的 patch Release；其 N-1 run 仍使用同一个 harness，在确认 compatibility 后记录
`migrationState: not_required`。当前 N-1 run 使用紧邻的上一个公开 Release 与当前 Runtime，例如：

```bash
tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --from-tag v0.2.12 \
  --to-tag v0.2.16 \
  --target x86_64-unknown-linux-gnu \
  --output ./release-adopter-upgrade-acceptance
```

它证明旧 adopter 检测、审查门控迁移、历史字节保持、继续运行以及隔离的
repository/runtime identity。这是发布后 evidence，不能用源码构建替代，也不能改写
Release truth。迁移验收 artifact 与 adopter 安装路径分开维护。

历史 v0.1.1 到 v0.2.0 的 schema 迁移 evidence 仍保留在归档中。由于 v0.2.0 Runtime
早于相邻迁移链 receipt 字段，当前 harness 不重新运行这个历史 pair。

发布 workflow 会在发布和 publication handoff 之后，用独立的
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
archive="ai-cockpit-v0.2.16-${target}.tar.gz"
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
$archive = "ai-cockpit-v0.2.16-x86_64-pc-windows-msvc.zip"
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

该 fallback 适用于当前已发布的不可变 `v0.2.16` tag。
发布完成后，workspace 含多个 package，必须显式选择 `cockpit-cli`：

```bash
cargo install --git https://github.com/xinglun/ai-cockpit.git --tag v0.2.16 --locked --root "$HOME/.local" --bin ai-cockpit cockpit-cli
"$HOME/.local/bin/ai-cockpit" --version
cargo uninstall --root "$HOME/.local" cockpit-cli
```

## 回滚

回滚时下载并验证指定的不可变历史 Release archive，再手动替换 binary。无版本号的
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
