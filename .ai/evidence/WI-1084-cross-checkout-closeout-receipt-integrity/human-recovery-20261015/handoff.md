# WI-1084：真人恢复已记录，新 Contract 正式验证通过，待 Raydot 材料决定

## 实际批准与新绑定

Ray 的直接回复为“我本人 Ray 批准此绑定的 retry，并选择 confirm_review”，对应 questionItemId `["request_user_input_async","call_283d0f39bf3b4f62a228739c48b0c4bc",0]`。批准请求位于提交 `bdcf6eca604dd81659370133ca40d5d5e8fcf48c`，raw SHA-256 `c6e552ac1904da3226d3ecaba6f47c6c04882db9b5a794f570adecdec045b931`。`actual-human-approval.json` 保留原回复与范围；记录时间是本代理捕获时间，不冒称人类提供的时间。

支持的 `work-item recover` 已实际接受同一 WI retry。新 preflight 真实生成后，逐项核对批准的条件未变化，再以支持的 controls 写入 Ray 的 confirm_review：

- Contract：`sha256:6cdd832a40672ea8b73aad731b22a5ec1a6dc6e3737c7046b442695274253159`
- 新 preflightDecision：`sha256:e1770338bfc0b1bac50fc0f4b3040c5e58fbc9d82256b3d8c550f897a93c944f`
- repositorySnapshot：`sha256:8a7631eab8bbec1e4f20e359ee54a942a93a94dcbe4ec96d1f412628e71e2c5e`
- amendment journal：`sha256:23503ecd09559ac30da58efb7f1285c5ec503a4717c5eb7832827e7035020473`
- profile：`sha256:8282c7d0f7780af966e90e681f972e9c5a18be5892e95f4832c600590c0aa4b0`

没有沿用旧 preflight。`conditional-preflight-binding-check.json` 保留条件检查。一次本地输入准备错误及 controls 缺文件拒绝原样留存；修正读取 committed wrapper 的 receipt 成员后，独立的 controls 调用 exit0。没有手写生成记录或改源码。

## 新 Contract 的真实本地正式验证

当前 Runtime 在 policy review 后先准入 run_verification；材料 record 当时尚未准入，因此按实际顺序恢复正式验证。源码执行 head `bdcf6eca604dd81659370133ca40d5d5e8fcf48c`；源码、测试、文档与此前接受的 source head `07f005f793ded957cba8a6d676e3d0aef05630bd` 相同。实际 Runtime 二进制 SHA-256 `5307ba5324d397e0fb971aa8b926ba8176336a0a6e76dfae7bd3a6e5cd11fa38`，Rust/Cargo1.98.1，命令、路径、环境和逐节点结果均保留。

`verify --repo <当前工作树> --work-item WI-1084-cross-checkout-closeout-receipt-integrity --workers 1` 实际13/13通过，13 spawned、0 reused、maxConcurrentProcesses1、0 spawn failures、0超时。外层耗时600.657072秒，Native597100ms；外层900秒上限和正常600秒逐节点限制均未改变。repository节点333353ms。repository stdout按Runtime的64KiB边界截断；其stderr及每节点pass/exit/timeout元数据完整，不能声称所有stdout完整。

- 新 receiptDigest：`sha256:d175a92572fd3d6e3033633aeb913e1183adcd56fc5ccf099a0713e4901734e4`
- 新 canonical raw SHA-256：`db9b1fdead824834440be232b41ae965f127015cdd644c235a02f1c2abb29314`
- verificationEvidence canonical digest：`sha256:0cdfdf39c5e126f669dbba017ebf338e273fc52e2a470d651e6098dfeacdcb41`
- 成功 attempt：`.verification-attempt.241cd804880a8c50f72004e90cfa5cc54b4f5f64a6be775b13e9b712bab0d6ab.json`

见 `new-formal-result.json`、`new-contract.verification.receipt.json`、原始stdout/stderr和execution元数据。新旧Contract的正式结果明确分开。

## 当前准入与所缺输入

正式验证后 status 为 checkpointed/count1、governance green、blocking=false、blockers=[]、humanDecisionRequired=false、verified/fresh；validate为verified，Findings=[]。Runtime准入record_material_review_decision和finish。原failedGate的恢复由Runtime处理，没有手动清除。

**尚缺 Contract 指定的 agent:Raydot 的实际材料决定、rationale 和 residualRisk。** 已接受技术方案/profile不在这里被冒充为新请求的签署决定。`raydot-review-ready.json` 是精确待评审包（不是批准）；其 requestDigest `sha256:2c4fe9aa518c83150415f8512f50d054415a42d92714c3b679c3e1829f1ee501`。唯一reviewable项仍为`crates/cockpit-repository/tests/closeout_recovery.rs`，cause `readable_committed_rust_syntax_unknown`，blob `sha256:943f497c0153b7bda27239e02cbeed7fe7949a72cb403847972c70e5c0c0ae0a`，hunk `sha256:38b599cb53fe461263ca1534e01eb357f2e5c06c081e58ba03856705ffb536e0`。Finding0，无新增scope或authority变化。此普通技术输入不要求重复向Ray索取已经记录的真人批准。

Raydot提供真实决定后，应先刷新当前status及plan、核对绑定和准入，再通过typed material-review record记录；仅允许消解许可的effective语法Unknown，原始scanner Unknown必须保留。然后查询status/validate，再按真实门禁finish/archive；未记录材料决定时不重复已知会拒绝的finish。记录检查点可能影响准入，提交后仍须实际重新查询，不能假定回执仍fresh。此阶段没有调用finish/archive，没有归档digest。

## 历史、空间与CI

原322项清单中321项原地逐字节不变；唯一由Runtime正常替换的active canonical验证回执，旧字节完整保留在`historical-original-contract.verification.json`（raw `3e50cfff0224aaf40694b75be5d315364dcdf37ee1a0ba15913cd05ca9a19554`）及不可变Git提交7f723bfb。旧失败attempt、成功attempt、原finish拒绝、Contract历史均未改写。

每30秒记录空间与inode采样；采样最低workspace2862608384字节、tmp2056056832字节，无资源停止。结束workspace约2.67GiB、tmp约2.70GiB。未做额外清理。PID1仍tail，历史僵尸样本如实记录；具体旧限制触发原因仍未证实。

执行head bdcf6eca的自动CI38726687202最后只读观察：route、v1-behavioral-oracle、quality成功，Windows仍运行。这不是完整CI验收，后续新记录head的必需检查也必须单独读取。没有手动重跑旧CI。

## 人类Outcome

Outcome: 🟡 新Contract正式验证通过，待Raydot材料决定。Finding0；待处理输入1项，材料及用户收益Unknown2项。Native的“🟢 已声明的验证通过”仅说明已声明验证；真人policy review已真实记录，但没有close批准。使用量覆盖和host显示确认仍未知，未补造收益。PR1040保持Draft；没有Ready、merge、真人close、分支/worktree删除、新WI、Task9或其他资源操作。下一步为上述精确材料技术决定，再正常finish/archive。
