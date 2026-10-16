# 已准入 RED 与最小候选修复

源码 0069033a46afeab1374107cc9bce079b51c24151 在真实 admission=allowed 后重跑 checkpoint_：10/10 出现预期“必须拒绝但实际接受”的断言失败，包含全部 CLI/MCP 成对来源与 count 0/2 material fallback。此为已准入定向 RED；不是 workspace 完整正式回执。原未准入观察仍单独保留，绝不替代该轮。

三处 fixture 的 insert-only 初试暴露后续多余 preflight 已不准入（CLI1失败、repository50/52）：现将原完整 preflight/断言块移动到确认前，移除仅本次添加的重复调用；两文件的全部原 assert 宏数保持，与源码8ea965d8一致，没有删除或放宽原断言。

当前候选产品修复只涉及原已授权 contract_amendment.rs 和 status_projection.rs：两个来源字段都绑定 native journal 第一个 checkpointed transition 的 previousContractDigest；material/stale fallback 的 post-checkpoint 边界使用 native tail.checkpointed，篡改 Summary count 不能绕开。另在现有双 amendment 链测试中追加未篡改正向断言，防止错用最新 transition 代替最初 before_edit 来源。

本候选提交时 GREEN 尚待后续真实准入后执行，不声称修复通过。空间仍约 683 MB，只采用现有隔离 target 的最小构建测试，不默认可完整重建。
