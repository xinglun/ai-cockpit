# WI-1086 正常建项里程碑

Raydot 已通过 f49a001e 的 v2 技术方案及独立最小 scope，批准正常建项、RED 和最小实施；此处记录其技术授权，不冒充 Ray 本人的 confirm_review。

编号检查：本地记录、正常 fetch 后的 main、远端对应分支及 GitHub issue 查询均未发现 WI-1086 占用。新分支与 linked worktree 位于原 Git common-dir；原 WI-1085 完整保留。显式 fetch main 仍为 681df30e2fe8c869b6a41498822818c017dc4fb6。

Runtime 原生 start 成功，精确冲突为空，既有活跃/历史资源作为原生 advisory 保留。先用受支持 boundary 及 declare 绑定串行边界：parallelizable=false、conflictsWith=WI-1085，dependsOn=[]。修复输入为已批准 main 与只读旧复现；WI-1085 是未来消费者，不能反向依赖其先完成，不伪造消费者记录、coordination registration 或 lease。

初始 boundary 因两个文件名中部 glob 不受支持而退出1，stdout为空且未改 Contract；保留原错误和输入。将 boundary 的生成路径收窄为受支持目录前缀及精确 canonical 文件后，原生命令成功。scope、目标和政策不扩大。

初始 preflight 退出0，yellow/verification_pending，blockers=[]、无 humanDecisionRequest。status 把 yellow 且无 decisionEvidence 标为 needs_human_decision，但已批准的正常 checkpoint 有独立 Runtime 治理检查；原生 checkpoint 实际成功并创建唯一 before-edit，未跳过任何检查或写确认。这是实施授权边界的真实回执，不是测试或验证通过。

checkpoint 后才添加新 WI 自身三语 in_progress 文档及各账本唯一一行；旧 WI-1085 账本仍未动。当前 source 有新未提交文档时，material_review_projection_unavailable 保留，后续 preflight 退出0但带该 blocker，正式验证未准入。先提交源文档及这些原始证据，再刷新 Runtime；不把 before-edit 许可说成正式验证已获准。

下一步：核实提交后的当前材料投影和 Runtime 准入；若仍为真实实施/验证阻断则交 Raydot 判断，不自行修改门禁。只有准入允许后才执行测试/产品修改。未 build、未运行 RED 或正式验证；未操作 Sentinel、安装候选 Runtime、创建 PR、合并、关闭或清理。
