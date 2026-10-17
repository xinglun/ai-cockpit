# WI-1086 已批准两文件扩围与真实准入

Raydot 技术批准由当前用户消息传达。仅正常 typed amend 新增 scope_amendment.rs 与 lifecycle_entry.rs 两个测试文件，目标为三处测试 fixture 的 current preflight 顺序，不放宽任何原断言。sequence=2，完整原始 stdout/stderr/meta 与前后记录在此保存。两个文件尚未修改。

amend 后实际 safeActions 含 run_preflight，因此只运行正常 preflight 取得真实当前请求。preflight 返回 needs_human_confirmation；随后 status 为 needs_human_decision，run_verification 不再允许。依 AGENTS.md 与 Runtime 的实际准入，暂停 fixture 实施、添加测试源码和测试执行；没有把技术批准伪造成 confirm_review，也没有利用旧 Summary 字段混绑。精确新请求在 pending-current-contract-review.request.json，交 Raydot 协调，尚未记录决定。

现有旧 Contract 的真人 receipt 字节与 Summary 内原决定完整保留，新旧不混用。原完整验证失败 attempt 也原样保留，不以单项通过替代。当前未运行新构建、未清理缓存、未最终验收。

漂移检查 CLI 首次错误使用 --id，parser 在参数解析即拒绝；原错误保留。当前支持流程需要 registration generation；正常 coordination inspect 返回 registrations=[]，故未伪造 generation 或注册/共享事件，继续串行。typed amend 自身捕获真实 environmentObservationDigest。
