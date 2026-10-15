# WI-1084 单次关闭／集成／清理批准请求草稿 v3（来源已更正、当前CI全绿）

状态：仅供 Raydot 技术审阅，尚未向 Ray 提交，尚未获真人批准。当前 Git head 保持 `b148751722994816d5ed5fc49edf75b72580a82c`，工作树干净；未提交、推送或执行任何真人步骤。已实际核验当前head的CI38731302895为completed/success、4/4；该结果不替代实际真人批准或后续C的检查。

完整请求：`WI-1084-final-human-approval.request.v3.json`，原始 SHA-256：`sha256:82a4fdb4ef4af4e010c96e9e5c17b55061777208f9c003ba453c0ce1bac16bdb`。

## 待 Raydot 审阅后发送的单次确认文本

请 Ray 本人确认是否批准完整请求 `sha256:82a4fdb4ef4af4e010c96e9e5c17b55061777208f9c003ba453c0ce1bac16bdb`：仅在 WI-1084 当前最终 head `b148751722994816d5ed5fc49edf75b72580a82c` 的 CI `38731302895` 完整成功且 Raydot 接受收尾后，对归档 `sha256:d781bacebb53b33d600e70bd354dec291b5296d4110eb15009741df1b9df781e`（Contract `sha256:6cdd832a40672ea8b73aad731b22a5ec1a6dc6e3737c7046b442695274253159`）记录本人明确的 ordinary close；只追加关闭证据、必要批准材料和本 WI 六份标准终态文档投影。

该受限追加会产生新 head C；只有 C 的完整必需检查及 Raydot 审阅通过后，才将 PR #1040 从 Draft 转为 Ready，并以 expected_head_sha=C 正常 merge commit 合并。成功合并、正常 fetch 和默认远端同步后，只删除对应远端/本地分支 `codex/wi1084-closeout-receipt-integrity` 及干净工作树 `/workspace/ai-cockpit-wi1084-closeout-receipt-integrity`。

本请求还包括：保留精确 Runtime 的独立副本，创建并保留新的独立 control；实际删除后通过 Runtime 记录 ordinary-cleanup，并仅在现有权限和分支保护正常允许时，以一个受限、非强制的 main 快进治理提交 R 持久保存本 WI cleanup 回执及必要执行证据，再核验其必需检查。保护规则不允许或 main 已变化时立即停止并保留实际回执，不擅自开新 PR/分支/WI、绕过保护或改用强推。

全部其他分支、旧 control、未提交内容、WI-1078 资源、历史证据、Library 恢复包和旧环境保留。Sentinel 本地候选复验单独进行，不是本请求的准备前置条件。任何绑定、门禁、资源身份或权限不符均停止，不修改源码、断言、门禁或冻结历史，不执行 force/reset/clean/batch prune。

此次 WI 关闭仅认可已声明的新 origin/v2 完整性范围。我已知旧无 origin v1 删除唯一 receipt 仍可能显示 closed/blocking=false，完整旧 v1 原地重试仍返回 closeout_recovery_receipt_conflict；此次批准不宣称旧 v1 问题解决，不授权迁移或伪造 origin。**#1004 继续 OPEN**，PR、提交与合并元数据不得包含自动关闭它的措辞或操作。Mac 回报属于固定候选的外部现场证据，云端未独立读取其原始报告/清单/日志；Deleted摘要已由追加更正评论6191899624提供完整64hex并核对字符数，旧评论和旧草稿保留；不再要求补正。

建议回复：**我本人 Ray 批准该完整绑定请求，知悉旧 v1 残余及 #1004 保持 OPEN，并同意上述 CI 与复核前提。**

## 给 Raydot 的执行路线审阅点

1. 当前 Contract 无 resourceContext，采用 ordinary close 与 ordinary-cleanup；不补造 provider finalization。close 的 Runtime-owned cleanup binding 捕获 b148 的真实身份，后续不手改成 C。
2. b148 的成功 CI 不替代 C、实际 merge M、治理追加 R 的必需检查；各 SHA 必须由真实操作生成，逐阶段核验。C 的许可范围只含关闭回执、请求/执行材料及六处确定性文档投影，没有实现代码或测试变化。
3. 旧本地 main 被脏 control 占用。新 control 拟使用 `/workspace/ai-cockpit-control-wi1084-final-b1487517`，同 common-dir `/workspace/ai-cockpit/.git`，在实际 M 上 detached；不会挪动旧 main 或宣称全仓 ready_on_base。该 control 和 Runtime 副本保留，不纳入本次删除对象。
4. Runtime 副本拟保留于 `/tmp/wi1084-closeout-execution-retained-b1487517/ai-cockpit`，原始 SHA-256 必须为 `5307ba5324d397e0fb971aa8b926ba8176336a0a6e76dfae7bd3a6e5cd11fa38`。目标路径目前不存在；此准备阶段没有创建 control 或复制二进制。
5. **请 Raydot 在向 Ray 提交前审阅请求中的 postCleanupReceiptPersistenceRequestedForRaydotReview。** 清理观察发生于合并之后，所以其持久化 R 是额外的受限治理提交。请求将正常 main 快进追加的精确 A-only 范围显式纳入授权；它不包含保护绕过或额外对象。如果现行流程必须另 PR，应先重整这一段请求，不能把含糊范围交给 Ray。
6. 当前跟踪树约421.1 MB，按909.3 MB估计新 control 的保守峰值；Runtime副本52.1 MB，无新构建。每步前后重新检查空间/inode，监控不授予新缓存清理权限。

所有精确摘要、路径、当前保护对象与脏文件原字节摘要见 JSON。真人 actor/authoritySource/decidedAt 留空；本草稿不是有效 close receipt。


## 本版补充来源与边界

- Mac 回报：https://github.com/xinglun/ai-cockpit/issues/1004#issuecomment-6191800813 ，固定 b148、Mac arm64 CLI SHA-256 d5017faf…29c43；repository/CLI各6/6，与云端Native13/13及Linux Runtime5307…fa38分开记录。
- 新v2恢复/幂等/删除篡改fail-closed、真实Retained→Deleted获外部现场支持；旧无origin v1的唯一receipt丢失保护及原地升级没有获得解决。
- PR标题/正文及分支提交说明未发现自动关闭#1004措辞；已在PR顶部明确OPEN和有界v2范围，未修改Issue状态、源码或head。
- 旧v1方案见 `legacy-v1-followup-proposal.md`：先只读盘点可信原件，再评审显式、版本化、追加观察/预期锚；无独立来源不猜测历史。当前不写代码、不建WI、不迁移。
- seq2旧评论59hex文字保留为历史；当前完整值为84afc073492b30366068e15f74a860606969150ae15076b1a351115916804bac（64hex），已通过追加更正记录，不再等待补正。
- 原始草稿及其014755…0946摘要继续保留，但它从未构成真人批准；旧v2亦保留，只有本v3为当前待审稿，待Raydot定稿后提交一次真人请求。


## 当前CI及分工

CI38731302895精确绑定b148：四任务成功，quality40通过/2不适用，Windows114/114、library688.29秒。源码/Contract/归档不变，未重跑旧CI。旧v1歧义从最初994a065a Contract就存在并明确保留，不是本轮新增实现范围。

AI Cockpit由Raydot直接协调；Sentinel仅由Ray转发联系。云端不操作或直接联系本地Sentinel；旧v1只记录后续需求，不开迁移实现。
