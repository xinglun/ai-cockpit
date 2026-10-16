# 给 Sentinel #1004 的修复与复核事实草案

状态：**草案，未发送**。供 Raydot 控制线程按用户授权发送；不自动关闭 Issue、不代表已合并/发布或已在真实 Sentinel 执行来源登记。

---

针对 [原候选验收](https://github.com/xinglun/ai-cockpit/issues/1004#issuecomment-6191800813)、[摘要补正](https://github.com/xinglun/ai-cockpit/issues/1004#issuecomment-6191899624) 和 [四份 v1 盘点](https://github.com/xinglun/ai-cockpit/issues/1004#issuecomment-6192236257)，WI-1085 已补齐**完整旧 v1 的显式现时来源登记能力**。当前实现最新源码为 `007fdcba2e81d01285a694dce57cd78e0f74fcf4`，见 [Draft PR #1044](https://github.com/xinglun/ai-cockpit/pull/1044)；当前仍未合并、未发布。

## 改动及已取得证据

- 可对完整、可读且可验证的旧 v1 原 bytes、适用 payload 与真实依赖显式登记；当前 registrar 仅声明 present-day `explicit_attestation`，不重新认证历史 producer，不把旧验证说成当前重跑。
- 新增不可变 requirement/registration 双记录；保留首次准确 bytes/binding，支持受约束中断恢复及幂等重试。登记义务存活时，必需证据丢失/损坏将撤销 closed、保留 readiness 阻断，并拒绝重叠 scope；合法 Retained→Deleted 为同 WI lock 下追加。
- 没有自动迁移未登记旧 v1，没有重写旧 archive/close/verification/provider 或更正评论。原 v2 与普通 close 读兼容保留。
- 云端修复版本实际原生 workspace 正式验证 **13/13、0复用/超时、876.971秒**；attempt `65fc3b51…` / receipt `7cc40bfb…`，Raydot 已独立核验并通过最终技术审阅。
- [CI38946515596](https://github.com/xinglun/ai-cockpit/actions/runs/38946515596) 四 job 成功，source `b5a219eab232695fe2b69102c9de5b3062837ec3`、tested merge `a408ad078eb973e5254806254e44ba0d8102baae`。Windows 本模块四个 directory-junction/reparse 用例实际通过，`--lib` 为 136/0/0，856.20 秒；strict gates 40 passed + 2 not_applicable。
- 当前证据头 `e2d32e20410e34a1ad73e39e087daaa614689fb0` 的 [CI38951742695](https://github.com/xinglun/ai-cockpit/actions/runs/38951742695) 也已四 job success，tested merge `b5af84c1c219c8ff7160ac2aa342a358829d7a7b`；本模块四例再次有实际成功行，Windows --lib 136/0/0、762.31秒。随后原生 finish/archive 已完成，当前等待真人 close，PR 仍 Draft；最新状态见同目录 `report.zh-CN.md`。这些结果不扩大成真实 Sentinel 已验收。

## 请复核的候选边界

请先固定届时明确提供的候选 source SHA，在独立可丢弃副本构建候选 Runtime，记录实际 binary SHA256 与 `inspect.runtimeDigest`。云端治理 Runtime 的 binary hash **不是该候选新实现 binary 的替代证明**。本轮云端未改 Sentinel 原工作区，也未登记真实 #1004 历史；实际目标登记须另有逐 WI 的明确授权与完整来源。

建议复核：

1. 对完整旧 v1 独立来源，确认 plan 只读、精确 repository/WI/Contract/close/provider 与 inventory 绑定，缺来源为 provenance_unknown；登记保留旧 producer、历史 bytes 和 historical_evidence_not_revalidated。
2. 在可丢弃副本验证登记幂等、requirement-only/registration-only 精确恢复，拒绝新 binding/损坏 survivor/覆盖冲突。
3. 登记义务存在时，分别移除或篡改旧 v1 receipt、适用 payload、archive/Contract/close/verification/provider/dependency 或双记录一侧，确认 closed 撤销、readiness 保留以及 overlapping scope 阻断；不把命令 exit 0 等同于 status 非阻塞。
4. 验证合法 Retained→Deleted 追加、commit 后回报失败重试返回原 bytes，原来源 checkout 离线后正常已登记查询仍可用；不篡改真实历史。
5. 保留并再次检查已知边界：**所有本地义务、来源与独立锚点同时丢失或整体回滚仍不可识别**。四个 Windows case 仅覆盖声明的 directory-junction/reparse 子边界，不是所有 reparse tag/末端 file symlink 的穷尽证明。

你在盘点评论中列出的四份旧 v1 与 43 个历史文件，是 Sentinel 侧既有观察；本轮云端没有在 Sentinel 上重新核对或迁移：

- `probe-eligibility-conversion-observation`
- `probe-historical-canonical-observation-integrity`
- `signal-context-high-information-evidence-integrity`
- `signal-context-read-model-rustfmt-drift`

请保留原 receipt、43 个历史文件、Mac 候选验收日志与 manifest；Deleted digest 以已补正评论为准。本轮不把能力实现或有限 fixture 通过记成四个真实 WI 已受保护，也不把预期风险下降说成已测量用户收益。

**#1004 继续 OPEN**，等待候选版本与真实来源场景的后续复核结论，再由有权人决定验收与 Issue 状态。
