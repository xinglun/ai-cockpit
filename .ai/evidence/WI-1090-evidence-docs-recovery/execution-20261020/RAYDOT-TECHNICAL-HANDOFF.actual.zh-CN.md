# WI1090 文档恢复：实际验证与技术边界

Outcome: 🟡 验证未就绪；本项保持 checkpointed，等待 agent:Raydot 技术裁决。

Ray 的真实 confirm_review 已原生记录：actor human:Ray、recorder agent:Codex，Contract `sha256:f263774f07df8a5b88f6fc538538d6d5d249552f0fcc934374b281d07d0519b9`，原确认 preflight `sha256:af4dc69ed22faa7e02174ae06b68449716e14dbbaa679ed55939f219c27cc3f6`。随后实际预检返回 human_decision_recorded，checkpoint1 成功。没有记录材料残余风险接受。

源码/三语登记提交 `28ae748c3d2a25f143e3113f16a0b505dc53b22e` 已推送，并严格先于首次正式验证。恰好八项 R100 迁移、两个 locator 文件、六份既有投影与三份自身页面完成。实际审计核对了两份原 manifest、27份前项原生记录、两份 Rust 与其余19项原路径/字节，均符合批准约束。原 manifest 的逻辑键需经显式 locator 解析。

正式验证绑定：

- source HEAD：`28ae748c3d2a25f143e3113f16a0b505dc53b22e`；base/main：`b8ec2690976ab4c554aa9f5b2aa7fcca715156ca`。
- Runtime：1.0.1，`sha256:b5cbd8570df5d3cb2004ae517fe88df6136c66d92fc1913070d18f2c8f67f257`。
- 实际刷新 preflight：`sha256:3798c12ae4ef8b0546e2fb066e7488ac89fd62c3af888ea5e7782365795011eb`；snapshot：`sha256:7b1e1bae297aa899271021bc6f1737ef0af14848f01c85bb18cd97c9c8ab421c`；materialManifest：`sha256:64bfbce55119ab94eaa7b287ae50a470844961ed4b54b148e1e8bbb1f80824c3`。
- 计划2，实际执行2，复用0，最大并发1，无超时。`documentation_acceptance.sh` exit0；`parity_status_check.sh` exit1。总耗时约208秒。
- 原生失败 attempt：`.ai/evidence/WI-1090-evidence-docs-recovery.verification-attempt.b5699af612d2135b5a1b16acd028099dcaed804099699bc63f926b67c591c9d2.json`。完成验证收据未生成。
- 初次过早 plan-only 的零进程拒绝 attempt `da4a9b77502c7d97389ddcf44464be7cf2f6cd0e22469746672df601adc9223e` 保留；刷新准入后 plan-only 成功，随后才实际执行两项检查。

实际 gate 只有两项 error：前项 `WI-1090-authority-source-material-review-replacement` 在中文、日文 parity 的 `stale_prearchive_parity_registration`。完整 gate 已识别前项的原生选择记录，归类 awaiting_successor_close；本项归类 prearchive_parity_registered。没有短 ID 歧义、缺少本项页面或登记路径的错误。

只读诊断已证明：现有 `recovery_successor_owns_parity_projection` 返回 false。Contract.scope 包含三份 parity 文件，但原生 Summary.changedPaths 仍只有三项历史 activation 路径。现有 gate 要求两份记录都包含全部三条路径，才把真实的后补翻译登记作为历史恢复处理。原生 Summary 的下一处可用写入位于成功验证记录流程；失败验证不能提升为完成证据。当前 controls 接口不支持 changedPaths。该自举循环是技术判断，证据及源码位置保存在 `bootstrap-diagnosis.actual.agent-recorded.json`。

当前原生 status：humanDecisionRequired=false，blockers=[]，允许 run_verification/record_governance_controls 等；未允许 finish/archive。材料扫描 Unknown 继续保留。原生中文 Outcome 已读取并保留，显示“验证尚未就绪 / 已建立检查点”。该 Outcome 的人工决定文字显示“未记录”，与已存在的原生 confirm_review 收据及 preflight human_decision_recorded 有展示差异；保留原文，不因此伪造或重复人工决定。

请 Raydot 裁决受支持的原生 source-projection 恢复途径，或另批有明确测试的最小 gate/Runtime 修复范围。现行 Contract 明确禁止修改测试、gate policy 或 Rust 实现，因此本轮不能自行扩展到这些文件。当前没有新的原生真人决定请求，不能生成或复用一个新的 confirm_review。未重复无变化检查，未手改 Summary、历史登记、原生收据或保护历史。

连接检查 2026-10-20T01:15:53Z 实际成功执行 shell、Runtime1.0.1、GitHub ls-remote；本地与远端均为28ae748c，main 为b8ec。本轮验证进程及观察到的子进程均已退出；磁盘剩余约4.7 GiB。完整外部日志：`/workspace/scratch/wi1090-confirmed-execution-20261020/native-two.attempt1.*`；完整 gate JSON 已保存为该目录的 `native-two.governance-integrity-full.actual.json`。当前必要日志与定向诊断随本分支推送；不重启环境、不重跑已有检查。

自身 finish/archive、独立 Draft PR 和新 CI 尚未执行。原 PR1050 保持原样，未 merge/close/删除/发布；Sentinel 与真实消费者均未操作，未组成 WI1089+WI1090 最终候选。
