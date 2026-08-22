# WI-116：合并后的归档关闭证据

## 目标

允许经过审查的分支合并后继续完成显式的 `archive → close` 生命周期。
关闭已归档 Work Item 时，必须校验不可变的 verification evidence、archive
manifest、outcome 绑定、repository identity 和 Runtime identity；不能仅因当前
Git snapshot 改变就把证据判定为 stale。

这是 WI-115 的 successor：WI-115 已经合并，归档 bytes 不可变，不能安全地把修复
追加到原 Work Item。

## 范围

- close governance gate 使用归档证据路径。
- active/finish/archive gate 继续绑定当前 snapshot。
- 被篡改或 identity 不匹配的 evidence、archive manifest、repository、Work Item
  和 foreign Runtime 继续 fail closed。
- 增加“归档后合并再结构化 close”的回归测试。
- 三种支持语言的文档说明 immutable archive-manifest 边界。

## 验收

- 有效的归档 Work Item 在合并提交后仍可 close。
- 被篡改或 identity 不匹配的证据仍被拒绝。
- 既有 lifecycle 和 archive-integrity 测试保持通过。
- Runtime 生成结构化人工决定，不重写归档 bytes。

## 状态

Runtime verify、archive、close 完成前为进行中。
