# WI-118 — 发布 toolchain 与清理 fail-closed 修正

## 目标

让公开 adopter 与 N-1 发布验收在隔离环境中确定性执行并 fail closed。
本 Work Item 处理 WI-117 审查发现，但不回写 WI-117 的历史归档证据。

## 范围

- 两个发布 harness 在进入隔离 root 前显式绑定宿主 `RUSTUP_HOME` 与 active toolchain。
- 清理失败时以非零状态结束并将 `adopterAcceptance` 标为 failed，同时保持 `releasePublished: true`。
- 增加静态回归并同步三语发布文档。

## 不在范围内

Runtime protocol 语义、全局 Rust 安装、以及对已发布 Release 的修改。

## 验收

1. 缺少 toolchain identity 时，公开与 N-1 harness 都拒绝隐式 rustup 下载。
2. 清理失败同时记录在 `cleanup.json` 与 `acceptance.json`，进程失败且不能生成 passed receipt。
3. 发布后的清理失败不改变不可变的发布事实。
4. 静态 harness 测试与三语发布文档检查全部通过。
