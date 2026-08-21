# 发布版 Adopter 验收基线设计

## 目标

建立可重复的发布后 adopter 验收基线，只使用不可变的公开 Release 制品。
基线要回答：一个公开 GitHub Release 二进制，能否在没有源码 fallback、没有全局 Agent 写入的情况下，从零建立并治理全新的 adopter repository。

## 边界

第一阶段只增加版本化脚本 `tests/release/adopter_acceptance.sh` 和 GitHub
Actions 发布后 job，不增加 `ai-cockpit acceptance` Runtime 命令。脚本是验收
harness，不是 adopter 可调用的治理能力。

脚本严禁使用 `cargo build`、`cargo run`、workspace binary 或本地 `target`
二进制来取得 AI Cockpit。它必须从公开 Release 下载指定 archive，校验 manifest
和 checksum，解压一个固定 binary，并始终调用该绝对路径。Cargo 仅允许在临时
adopter 中执行其普通测试命令。

## 输入与输出

```text
tests/release/adopter_acceptance.sh \
  --repository OWNER/REPOSITORY \
  --tag vX.Y.Z \
  --target TARGET \
  --output DIRECTORY
```

CI 中四个参数都显式提供；缺失或有歧义时 fail closed。输出目录包含原始 JSON
证据、`acceptance.json` 总摘要和覆盖所有证据文件的 `SHA256SUMS`（不包含自身）。
摘要固定记录 `releasePublished`、`adopterAcceptance`、每一步状态、repositoryId、
runtime identity、时间和失败原因。

发布后失败时必须保留 `releasePublished: true`，只将 `adopterAcceptance` 记录为
`failed`；脚本不能修改或重新解释已经发布的 Release。

## Runtime identity

`runtime.json` 固定保存 `tag`、`version`、`archiveDigest`、`binaryDigest`、
`platform`、`archive`、`downloadSource`、`releaseUrl` 和
`releasePublished: true`。脚本要把 doctor、inspect 和 Work Item verification
证据中的 `runtimeVersion/runtimeDigest` 与下载制品对应起来。

## 验收流程

创建初始 Cargo adopter 并先提交，再执行 attach、profile confirm、Agent list/install/doctor、
`first-adopter-smoke` 的 `not_ready` contract 骨架和完整 Work Item lifecycle。随后在相同
隔离环境中执行两次 verify，要求第一次真实执行、第二次复用 receipt 且不 spawn。最后证明
源码 checkout 没有 `.ai/`，隔离 HOME/XDG 前后没有变化，并生成总摘要和 checksum。

`adopter_acceptance` job 仅在 tag push、`publish` 和 `publish_handoff` 之后运行，直接从公开
Release 下载 binary，不使用 candidate artifact。失败时仍上传完整验收目录。

## 范围外

不增加 Runtime CLI 命令、不改变 Repository Protocol、不写全局 provider 配置、不测试第二种
技术栈，也不把发布后验收伪装成发布前 gate。Node/npm adopter 另立 Work Item。
