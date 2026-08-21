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

WI-34 定义安装契约，但不声称首个公开 Release 或 `xinglun/homebrew-tap` 已经存在。
仓库配置仍使用 `cockpit.toml`；安装 runtime 不会在目标仓库创建 `.ai`。

## 开始前

你需要一个已发布且不可变的 Release、目标 repository 路径，以及与操作系统匹配的 archive。Homebrew
安装需要已安装 Homebrew；macOS/Linux 手动校验使用 `shasum` 和 `awk`，Windows 使用 PowerShell。
`gh attestation verify` 是可选的额外 provenance 校验。

## macOS 主安装方式

在 WI-35 发布首个经过验证的 Release 并合并 Formula 后：

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

当前 Formula 只支持 macOS ARM64 和 Intel；Linuxbrew 不属于 WI-34 支持路径。

## 验证 Release 制品

从同一个不可变 GitHub Release 下载 archive、`release-manifest.json` 和 `SHA256SUMS`。
校验文件覆盖全部十个 archive/SBOM，因此只校验实际下载的 archive：

```bash
archive="ai-cockpit-v0.1.1-aarch64-apple-darwin.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
gh attestation verify "$archive" --repo xinglun/ai-cockpit
```

如果 Release 已存在，也可以使用 GitHub CLI 下载准确的三个文件：

```bash
archive="ai-cockpit-v0.1.1-aarch64-apple-darwin.tar.gz"
gh release download v0.1.1 --repo xinglun/ai-cockpit \
  --pattern "$archive" --pattern release-manifest.json --pattern SHA256SUMS
```

文件名、target、checksum、manifest 和 attestation subject 必须一致。单独的上传或
semantic tag 不能证明安装完整。

## 手动 archive 安装

macOS/Linux 用户下载对应的 `.tar.gz` 和 `SHA256SUMS`，选择准确的 Rust target，校验 archive，
再将 `ai-cockpit` 放入 `$HOME/.local/bin`：

```bash
target="aarch64-apple-darwin" # 选择与机器匹配的 target
archive="ai-cockpit-v0.1.1-${target}.tar.gz"
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
$archive = "ai-cockpit-v0.1.1-x86_64-pc-windows-msvc.zip"
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

该 fallback 适用于当前已发布的不可变 `v0.1.1` tag。
发布完成后，workspace 含多个 package，必须显式选择 `cockpit-cli`：

```bash
cargo install --git https://github.com/xinglun/ai-cockpit.git --tag v0.1.1 --locked --root "$HOME/.local" --bin ai-cockpit cockpit-cli
"$HOME/.local/bin/ai-cockpit" --version
cargo uninstall --root "$HOME/.local" cockpit-cli
```

## 回滚

回滚时下载并验证指定的不可变历史 Release archive，再手动替换 binary。无版本号的
Homebrew Formula 始终跟踪当前 release，不是回滚选择器。

## MCP 与 repository attach

从已安装 runtime 启动本地 MCP adapter：

```bash
ai-cockpit mcp
ai-cockpit mcp --repo /path/to/attached-repository
```

安装和 repository attach 是两个独立操作。审阅目标 Work Item 后再显式执行：

```bash
ai-cockpit attach --repo /path/to/repository
```

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
