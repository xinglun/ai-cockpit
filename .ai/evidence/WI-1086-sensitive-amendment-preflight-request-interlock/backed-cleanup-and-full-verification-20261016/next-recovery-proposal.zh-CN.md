# 后续最小恢复提案（未实施，交 Raydot）

## 真实失败绑定

- 执行 HEAD：b77eb89940c7c8869e31bdb08eecb15b44cc0e70；产品源码仍为 2a5910e967585c3589af382d29143f1b50b0a2c1。
- Contract：sha256:e7245a06dd9fc2a45346800a300183c2e3207a40a71bf579f2c5ec138538387f。
- snapshot：sha256:0840ed40a921b36f42e61d97d8db8eb664882c5239bdd6f9336744cea3918e54。
- 正式失败 attempt：sha256:e571cea63e6020dfcb89cdd72c6d69338a4c8d0c4d00f291f1032276bc6bc797。
- 原始 receipt SHA-256：7bcee0c4e23ddf8f44912ff3bf6b9433b40e9ffa0b24bb652cf5d0e5c835c226。
- 13 planned/executed、0 reused、maxConcurrentProcesses=1；588672ms；12 passed/1 failed；无节点 timeout。CLI、原先失败的 lifecycle_entry、verification 包此次均通过，但它们不替代 repository 的新失败。

## 1. Fixture 范围：仅提出，不修改源码或 Contract

失败测试 pending_retry_survives_a_bounded_contract_amendment 在 recovery_decision.rs:1488 于 amendment 后直接调用 record_human_preflight_review；helper 的治理记录在 :217 被当前有效状态校验拒绝：preflight decision evidence requires a recorded green, yellow, or red preflight result。

最小候选修复是该次 amendment 后先通过原生 preflight_work_item_with_runtime 记录当前 Contract 的真实 preflight，再调用原人审 fixture helper；保留末尾 assert_action_admitted 和原全部断言，不手写 Summary/receipt，不弱化 governance_controls。具体未应用差异见 proposed-recovery-fixture.patch。该调用能否在此 fixture 的实际准入下完成仍未验证，不能把提案当成 GREEN；如果真实准入拒绝，保留 RED 交 Raydot，不利用混绑解锁。

这个测试路径目前不在 WI-1086 Contract 内。提请 Raydot 技术审阅是否通过正常 typed amendment 仅添加 crates/cockpit-repository/tests/recovery_decision.rs，并保持串行边界与原断言。实际修改前核对 expectedContractDigest 和真实准入；不猜测新 Contract、人审 digest 或批准复用结果。当前不新增人审请求，不伪造 confirm_review。

## 2. 执行器缺陷：和 repository 失败分别处理

我新增的保护器没有对 /proc/<pid>/task/<tid>/children 路径缺失（ENOENT）拒绝执行，而把异常当作空子进程集合，误报 ownedProcessCount=0，导致收养的僵尸未回收。路径缺失事实已实测；具体内核/容器实现原因未证实。

原保护进程 PID 805031、startTimeTicks 24465201 已被本任务独立保护器 SIGSTOP，保留为 T。原生验证 PID 805807、startTimeTicks 24465792 已结束为 Z；它及其他收养子进程共1089个僵尸仍属于原保护进程。PID1 名下僵尸维持21316，没有把这批交给 PID1。不能声称它们已经回收，也不能声称原保护器正常结束。

独立保护器从约410秒接管，使用 /proc/*/stat 的父子关系及startTime确认原验证树，保持512MiB两盘保护和885秒开始取消/900秒验证限时，未重启验证、未改断言或门禁。原生验证约597秒时已结束，无资源或时间取消；新正式失败由Runtime自己生成。原保护器执行元数据未完成落盘，原 shell session仍保留，不能将原始监督整体报告为成功。/proc/stat中的末尾字段不用于推断真实退出码；以Runtime正式失败回执和完整stdout/stderr为事实。

后续保护器需要：
1. 子进程可观测性在启动正式验证前 fail closed；不支持 task/children 时使用已核验的 /proc/*/stat 父子图。
2. 实际父进程用 waitpid 回收自身直接收养子进程；已由 Popen 等待的主子进程必须排除，不能在监控线程抢走其退出状态。
3. 原生命令退出后用 waitpid(-1,WNOHANG) 排空已结束的自有子进程，逐项记录；仍存活的自有后代按既定截止时间取消并等待。
4. 新保护器只能回收它自己的子进程，不能从旁系进程回收805031名下的僵尸。原父进程必须获得执行waitpid的恢复能力后才能安全退出。现有环境未提供可调用的在运行Python进程内执行修复接口；没有尝试提权、ptrace注入或更改安全/权限配置。
5. 请环境负责人和Raydot先确定原保护进程的安全回收路径；不得盲目 SIGCONT、终止它、重建工作区或把1089个僵尸交给PID1来掩盖缺口。没有删除该保留进程或扩大主机清理授权。

这份提案不运行新的验证、不自动变更上述运行中进程状态。主机回收问题和新增fixture范围都完成审阅及实际准入后，再安排新的完整验证；本轮失败和更早原始失败必须永久保留，单项通过不能覆盖。

## 当前停止点

带备份的283文件清理已经完成并有远端记录；完整验证真实失败。无CI、Ready、merge、finish/archive、close、共享Runtime安装或其他资源清理。继续前需Raydot审阅两项阻塞；同一政策的批准不由本提案重录。
