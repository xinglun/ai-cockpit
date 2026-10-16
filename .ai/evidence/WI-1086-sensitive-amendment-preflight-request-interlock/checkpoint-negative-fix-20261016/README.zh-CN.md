# checkpoint 组合来源与 material fallback 回归

本检查点仅加入两文件三处已批准的 fixture preflight 顺序插入，以及现有 contract_amendment_processes.rs 的 10 条 CLI/MCP 负例。所有原断言保留，生产代码尚未修改。

新用例：两个 checkpoint source 字段同时缺失、相同伪造合法 digest、相同非 digest 字符串，以及真实非祖先 HEAD 触发 material unavailable 时 count=0/2。每个负例先检查未篡改的 native checkpoint 可生成请求、material fallback 确实出现，再断言拒绝和 Summary/Contract/journal/decision 零写入。

red-combined-checkpoint-provenance 是在外层准入断言失败后错误启动的未准入观察，10 项失败，原输出完整保留，不能作为正式 Runtime 回执或已准入 RED。guard-denies-uncommitted-material 证明新执行器在同一状态拒绝启动 Cargo。此提交后还需在真实准入允许的状态重跑 10 条取得可用 RED，再修复范围内生产代码。原 13 节点全量失败回执保持不变。

所有命令单构建任务/测试线程、隔离现有 target、900 秒上限；空间低于 256 MiB 只停止该命令自己的进程组。没有清理不明文件或安装 Runtime。
