---
author: AI Cockpit maintainers
title: "WI-262 Release version-consistency 清理"
description: "使 post-release version consistency 清理确定性执行并 fail-closed。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
capabilityClaims:
  - release_cleanup
  - release_truth_preservation
  - isolated_release_regression
---

# WI-262：Release version-consistency 清理

## 目标

post-release 的 `tests/release/version_consistency.sh` 会把
`release-manifest.json` 下载到隔离目录。原来的 EXIT trap 只删除 metadata
临时文件，然后尝试执行 `rmdir`；因此成功检查会留下已下载的 manifest，且
清理失败会被静默忽略。

本 Work Item 将清理定义为显式后置条件。成功路径和 manifest 校验失败路径
都会删除隔离下载目录。清理失败时必须输出 fail-closed 结果并标记
`release truth unchanged`；脚本绝不会重写或撤销公开 Release。

## 范围

- `tests/release/version_consistency.sh`
- `tests/release/version_consistency_test.sh`
- 本 Work Item 的三语文档

回归测试使用隔离临时根目录、伪造的 `gh` provider 和注入的清理失败。它证明
成功路径与 manifest 失败路径不留下临时文件，同时注入失败会被看见且不会改变
Release truth。

## 验证

```text
bash -n tests/release/version_consistency.sh
bash tests/release/version_consistency_test.sh
cargo test --locked --workspace
```

测试 wrapper 不构建源码 fallback，也不访问 GitHub。它把伪造 provider 绑定到
workspace version，并断言清理结果。

## 验收边界

清理属于运行卫生，不是发布权限。清理失败必须出现在命令结果和 evidence 中，
但不得把已发布 Release 改写成未发布，也不得修改任何 Release metadata。
