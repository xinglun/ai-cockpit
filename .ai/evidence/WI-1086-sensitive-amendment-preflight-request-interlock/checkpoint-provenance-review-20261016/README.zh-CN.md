# checkpoint 来源与 count fallback：静态补查，尚未执行

用户要求的负例已逐项核对真实源码。两处风险尚未经新测试执行，不能称为实际 RED，也不能据此最终验收 WI-1086。

1. current_amendment_review_request_available 校验 before_edit 与最新 revalidation 的 schema/recorded/repository/WI 等，但最终只比较 originalBeforeEditContractHash == before_edit.contractHash。两个字段同时缺失时均是 serde_json::Value::Null，相等；两个字段同时伪造为相同合法 digest 或相同字符串也相等。应先做真实 CLI/MCP RED，再核实完整原生 journal 中首个 checkpointed transition 与 before_edit 根来源的绑定；仅加非空/格式检查不足以拒绝两个合法但同时伪造的 digest。历史多次 amendment 也要覆盖，不能把最新 previousContractDigest 错当原 before_edit。

2. helper 要求 checkpointCount=1；但 status_projection 移除 stale/material fallback 的条件也依赖这个可变 count==1。将真实 post-checkpoint amendment 的 count 改成 0/2 时，helper 拒绝而 material_projection_unavailable 仍可能提供 run_preflight。负例须先证明真实 material unavailable 条件被触发，再检查 CLI/MCP status/actions 和 preflight 的零写入。修复方向需以认证的 journal checkpointed 事实为边界，不因篡改 Summary count 跳到另一 fallback，也不误伤真实 pre-checkpoint 流程。

原有单字段 tamper 和 14 类 provenance 测试覆盖不到这些组合。negative-cases.planned.json 列出 5 个计划用例与正向约束，全部 not_run；source-observations.json 绑定当前源码字节和行号。原始完整失败和 focused 复跑均继续分开保留。

当前 typed amend 后的 Runtime 需要当前 Contract 真人决定，run_verification 未准入；按 AGENTS.md 暂停添加测试源码、执行测试与产品修复。没有修改生产或测试文件，没有安装新 Runtime，没有借上述疑似漏洞运行真实 WI。解除当前真实准入与容量条件后，先最小 RED，再在既有授权源码路径内修复并做完整回归。
