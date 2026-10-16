# 本轮 Outcome 与 Raydot 交接

Outcome: 🟡 带备份清理完成；完整正式验证失败，暂停后续 CI/收尾。

## 完成与精确绑定

清理：Ray 对 sha256:5881b11f87fd77b5d97a0e528732687a44bc825248aeb8223dea6a5b026a860e 的283文件及披露的不可读引用风险明确批准。全部原字节先备份并读回核验，再删除283、跳过0。653排除文件和候选CLI、安装Runtime、source/.ai/历史证据均保持。/tmp/wi1085-approved-build-preservation-20261016-nyk7nr4c 原字节备份继续保留，不是持久备份，不自动删除。实际工作盘free差额+488,099,840 bytes，清理后达到1GiB+64MiB预算。完整结果在 backed-cleanup-execution-v2-20261016/cleanup-result.json。前次因调用沙箱临时PID变化拒绝的0删除尝试仍保留，不覆盖。

正式验证执行head b77eb89940c7c8869e31bdb08eecb15b44cc0e70，产品字节与已审阅源码2a5910e967585c3589af382d29143f1b50b0a2c1相同。安装Runtime SHA-256 5307ba5324d397e0fb971aa8b926ba8176336a0a6e76dfae7bd3a6e5cd11fa38，Contract e7245a06dd9fc2a45346800a300183c2e3207a40a71bf579f2c5ec138538387f，snapshot0840ed40a921b36f42e61d97d8db8eb664882c5239bdd6f9336744cea3918e54。

真实新失败attempt e571cea63e6020dfcb89cdd72c6d69338a4c8d0c4d00f291f1032276bc6bc797，原件SHA-256 7bcee0c4e23ddf8f44912ff3bf6b9433b40e9ffa0b24bb652cf5d0e5c835c226。13节点全部执行，0复用，最大并发1，588.672秒；12通过、repository失败，无节点超时。完整原始stdout/stderr、节点输出和正式Runtime失败原件全部保留。更早旧Contract的3469bd4b…失败原件仍保持d83bb2da…；不以本轮12通过覆盖旧失败，不声称存在成功canonical verification.json。

唯一测试失败 pending_retry_survives_a_bounded_contract_amendment：recovery_decision.rs:1488 amendment 后直接调用人审fixture，:217被有效preflight状态校验拒绝。此次CLI、lifecycle_entry及verification包通过，不等于完整通过。没有追加单项复跑或新的完整重试。

## 两项停止原因

1. 该fixture新增路径 crates/cockpit-repository/tests/recovery_decision.rs 不在现有Contract范围。已准备最小未应用patch：先记录当前Contract的原生preflight，再调用原fixture人审helper；保留所有原断言及收据完整性校验。提请Raydot技术审阅并决定是否正常typed amend；不改parity或validator、不写实际confirm_review。提案并未运行或证明GREEN。
2. 我新增的保护器对 /proc/<pid>/task/<tid>/children 缺失未fail closed，误报无子进程。原父进程805031/startTimeTicks24465201被暂停T，1089个自有僵尸（含已结束原生验证805807/startTimeTicks24465792）仍属于它；PID1名下僵尸保持21316。独立stat父子图保护器接管原生验证的容量/时间保护且没有重启验证，但原父进程尚未执行waitpid回收。其原execution-meta未落盘，原exec session1926仍保留；原监督进程未正常结束，不能报告宿主清理成功。

原生Runtime已生成真实失败回执，旁系保护器没有调用waitpid回收另一父进程的权限/能力，不能把“保留”说成“回收”。需要环境负责人提供能使原父进程执行waitpid的受支持恢复途径，技术审阅具体绑定见 next-recovery-proposal.zh-CN.md。当前没有可用的运行中Python远程执行/调试通道；未尝试提权、注入或系统安全/权限修改。不得盲目恢复或终止805031导致这批进程交给不回收的PID1。

## 当前状态和风险

Runtime仍checkpointed / verification not_ready；当前人审需求false、run_verification仍allowed，但真实失败、fixture范围和宿主回收故障尚未解决，当前不重试。原政策人审receipt保持3f1f7cec…字节不变；本轮没有要求Ray重复确认同一政策，没有记录新confirm_review。

工作盘约1.16GB，/tmp约1.80GB，最终精确值见final-preservation-and-resources.json。321.3秒时/tmp采样仅541,863,936 bytes，距512MiB线仅4,993,024 bytes；原保护器完整最小值尚未从其内存取回，不能用稀疏采样冒充全程最小值。下次验证前还应重新审阅/tmp容量预算，不删除保全备份凑空间。

两项明确工程阻塞；Runtime另保留收益声明、意图对齐、成功验证缺失等治理未知。清理后的容量改善已实测；修复用户体验收益仍未完成验收。

未执行CI、Ready、merge、finish/archive、close、共享Runtime安装、备份删除或其他资源清理；WI-1085业务及其他WI继续暂停。后续仅按Raydot的技术审阅和真实准入继续。
