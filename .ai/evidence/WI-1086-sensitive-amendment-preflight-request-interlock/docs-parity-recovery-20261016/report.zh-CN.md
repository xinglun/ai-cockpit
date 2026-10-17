# WI-1086 三语 prearchive parity 登记修正

Outcome: 🟡 新快照原生验证通过；已缩短派生日志路径，等待新 head 的 Windows CI 验证

## 授权与实施

Raydot 在本会话技术审阅通过精确三行 patch，并授权必要验证、提交和推送同一 Draft PR。只改三语 reference-parity 的 WI-1086 一行：条件状态保留进行中，archive/finalize/close 明确为计划引用。三个路径尚不存在；当前 verification 真实存在。未修改 validator、验收标准、Contract，也不手改任何 Runtime 生成状态。未 Ready/merge/close/清理。

文档提交：4838c6bd4a0cc7f42fbdb805e416007389718d9c。Contract：sha256:b217077be18ddb78c731de6e4188e048416e914f63acb89a1b0e95781a325360。新 source snapshot：sha256:0baea8ae0baf946dd633725e95abe63aa43617ed6705d20bca991ab2b59f1cbe。

## 文档 checker

实际执行 `bash tests/docs/parity_status_check.sh /workspace/ai-cockpit <report>`：补回既有 Cargo PATH 后 exit=0，输出 governance integrity gate passed: 914 work items。首次没有 Cargo PATH 的执行 exit=1、未生成 gate report，真实 traceback 单独保存在 docs-check-missing-cargo.*，没有把它记为 checker 判定失败或成功。

## 原生准入与计划

新快照 status 为 finish_ready、旧 verification stale、run_preflight admitted。原生 preflight exit=0 后 run_verification admitted，无 blocker、humanDecisionRequired=false。coordination inspect 是空 store，无 registration/generation 可供 environment-drift request 绑定，因此不虚构 expectedGeneration、不新建 coordination 注册；沿用已确认 Runtime/工具链/目录及权限。

ProfileAuthorized 的 native plan：13 planned / 13 execute / 0 reuse / 0 processes spawned，全部 reason=reuse_not_configured。按原生计划只执行一次完整 workspace 验证。没有为复用或省时更改 profile、Contract 或验收标准。无需 retry receipt；Runtime 直接允许 finish_ready 刷新。

## 执行与回收

沿用既有 PATH（prepend Cargo，保留 Node/npm）、RUSTUP_HOME/CARGO_HOME、单独 CARGO_TARGET_DIR、jobs=1、tests=1、已有 flock runner。Tini 0.19.0 子收割器包裹 timeout 890s + TERM 后 kill-after 10s；native package timeout=600。仅沿用 `/home/agent/.cache/ai-cockpit-verify-target` 精确写权限；没有新 supervisor、没有清理 PID1 僵尸或改变 HOME。

执行前基线 1408 僵尸，readErrors=[]。monitor-01 有一个已退出 PID 的 ENOENT，后续确认该 /proc/PID 不存在并单独保留 followup；没有把观测错误当作空子进程集合。结束结果见下文；原始观测与读取竞争跟进均保留。

## 保留的历史结果

旧 head 96cc38f39ae45db716aac65171009a087cb96bd4 的 run 38861581726 已 completed/failure：route、v1-behavioral-oracle、windows-runtime success；quality failure，docs_governance_integrity 三条 invalid_prearchive_parity_registration，依赖 gate blocked。原始 artifacts 与 run 元数据保存在 old-head-ci-*。这些结果不归属后续新 head。

旧成功完整 immutable attempt 2538ee3a... 保留。当前 fresh 状态曾执行一次 verify --plan-only 被准入拒绝，processesSpawned=0，却原生写入 a3479d9e... precondition_rejected attempt；已保留，不能称为通过或已运行的测试。

## 仍保留的不确定性

user_visible_benefit_not_declared；intent_alignment_missing 警告；原有失败 attempt 投影不取代旧 canonical 的缺陷另行保留，不扩大本次三行修正。测试回收的 timeout 分支尚未演练。新 head 自然 CI 待推送后记录。

## 本次完整执行结果

- 原生声明 `cargo test --locked --workspace`：13 planned / 13 executed / 13 passed / 0 reused / 0 timed out；实际 13 个 project process，elapsedMs=752940，外层 shell/Tini/native exit=0。
- immutable attempt：`sha256:b33bd7be27c53407a22472b08b3b90769bcc6dbc0d35acc7127537c6b6b9d261`。formal receiptDigest：`sha256:c3e1b50fb7ea44c3cc3c8f6dc0a18f9db67c07aede36f25ef4332bc9aa886004`，createdAt `2026-10-16T07:11:52Z`。canonical 内嵌 receipt 与该 attempt 完全相同。
- post status：finish_ready、verification=verified、fresh、blockers=[]、humanDecisionRequired=false，safeActions 恢复 archive_when_reviewed 等；本会话没有归档授权，未执行该动作。
- 当前 source snapshot、canonical、原生 Outcome/task-report 均绑定新 0baea8ae...；原生更新 Summary、Outcome、task-report，未手改。post-binding.json 已逐项断言绑定一致，无需额外 finish 或 retry。
- 最终 before/after 读取完整（readErrors=[]）；僵尸 1408→1408，无新僵尸（包括基线活进程变为 Z 的检查）、无残留同 session 进程，Tini 正常退出并记录 1401 次 adopted zombie reap。既有 PID1 僵尸未清理。
- 运行中 monitor-01、05、07 共 5 个 ENOENT 条目；每个随后确认 /proc/PID 不存在，原始错误与跟进保留；未把错误当空子进程集合。
- 当前 repository node stdoutTruncated=true（原生64KiB捕获限制）；其余 stdout 和全部 stderr 未截断。外层 native/Tini 完整日志不恢复已截断 Cargo 字节。成功正常退出没有演练 timeout 清理。
- post 文档 checker 再次通过全部 914 工作项，exit=0。validation evidenceClasses=verified，但 state=unknown、intent_alignment_missing warning 仍保留；status user_visible_benefit_not_declared 仍保留。acceptanceResults 仍是标准列表，不能称为独立验收收据。

## 技术实施裁定

- 使用已批准三行修正和现有普通 finish_ready 刷新机制；没有新 scope amendment、retry 收据或额外测试工具项目。
- 用户已明确授权必要验证和同一分支推送；本次没有等待额外技术批准，也不把 Raydot 技术审阅写成人类 Runtime confirm_review。
- 本地验证通过仅证明此绑定下的声明检查；未实证用户收益或平台整体安全性，CI 独立等待新 head 实际结果。

## head 1ef519f8 的自然 CI 与路径修正

run 38865914036 已 completed/failure：route、v1-behavioral-oracle、quality 成功，windows-runtime 失败。quality 的实际 strict 路由 contractPath=null；治理 findings=[]，WI-1086 为 prearchive_parity_registered；format/clippy/workspace_package_tests 成功，coverage runner 实际执行13包，omitted=[]，verificationReceiptState=not_provided（不是 hosted native receipt）。

Windows 的 directory-handle、preflight-lock、governance-controls-lock 步骤均成功；windows_invalid_regular_binary_fails_without_fallback_or_artifact_writes 在 cognitive_benefit.rs:164 的 fixture clone checkout 前置条件失败。日志明确列出本次新增26个 captured-node-streams 路径 Filename too long，尚未进入二进制行为断言；后续 native-path/read-only parity 两步骤 skipped。这是本次派生证据包装引入的缺陷，不能归为环境随机失败。

最小修正：只重命名26个派生 stdout/stderr 日志，改为较浅 streams/<package>.out.log 或 .err.log；相对路径最大198→127字符。stream-paths.json 保留每项旧路径、新路径、字节数和SHA256，并逐项验证前后字节相同。原提交历史与失败日志保留；原生 canonical、immutable attempts、Contract、Summary、Outcome 均不因本次重命名而手改。测试源码、断言、Git配置、workflow未更改。

实际 Windows 验证仍需下一新 head 的自然 CI，Linux 上的长度检查和字节哈希不能冒充 Windows checkout 通过。若 Runtime 当前仍 verified/fresh，沿用其正式准入而不无谓重跑本地13节点；新 head CI 仍须独立完整观察。

路径修正后实际 Runtime status 仍 finish_ready / verified / fresh，source snapshot 与 verificationEvidence digest 不变，无 blocker 或新真人门禁；因此未重复调用 verify 或 preflight。真实文档 checker 再次 passed，914项。26个新路径的字节还逐项与不可变 b33bd7be... attempt 中的原生 stdout/stderr hex capture 完全比对一致（stream-byte-check.json）。
