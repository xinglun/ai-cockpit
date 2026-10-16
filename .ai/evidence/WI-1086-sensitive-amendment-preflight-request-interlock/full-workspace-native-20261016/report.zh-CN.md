# WI-1086 同环境完整原生验证

本次完整验证真实执行全部 13 个 workspace package 节点，13 通过、0 失败、0 复用、0 超时，耗时 743,167 ms；Cargo 节点、native/Tini 和外围 shell 均退出 0。执行源 HEAD 为 `e84740eb0ecdf0c2bb895cfd8ddc94af4c475e0b`。未修改产品源码、测试断言或验收要求，未再次扩大权限目录。

## 身份与正式证据

- repositoryId：`sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b`
- Work Item：`WI-1086-sensitive-amendment-preflight-request-interlock`
- Contract：`sha256:b217077be18ddb78c731de6e4188e048416e914f63acb89a1b0e95781a325360`
- repositorySnapshot：`sha256:5f85b933a4ccd16836dfc7f1271a87e950154ab5bcb7bf0da816791ac95f88d8`
- Runtime：1.0.1；实际二进制 digest `sha256:0ffdcd20740f22b96621cf06adb5479d83fb435f503a26c18d824d61c4bfe9c4`，来源与 ../runtime-candidate-20261016.json 一致。
- 完整成功尝试：`sha256:2538ee3a9aabafa3b55338fc32a3e7fab2c125cbd2ee366508f6cfc4c0335adc`，状态 `execution_completed`。
- 正式 verification.json 的 createdAt 为 `2026-10-16T06:17:36Z`，receiptDigest 为 `sha256:dfcda2d31913981195bea528b1d078bce0edd4e231a2f3d8ad2675c98e6903c3`。

正式 receipt 与成功尝试内的 receipt 逐项一致；repository/WI/Contract/snapshot/Runtime 绑定一致。13 条 execution record 均为 spawned=true、passed=true、exitCode=0、timedOut=false，节点集合与完整 coverage manifest 一致，nodesPlanned=nodesExecuted=processesSpawned=13，nodesReused=0。正式证据由 Runtime 生成；没有手改 Summary、receipt 或人审记录。

原生 post-status.json 确认 verification=verified、evidenceFreshness=fresh、blockers=[]、humanDecisionRequired=false，允许 finish。post-outcome.json 直接引用上述完整成功尝试，已不是之前的定向测试收据。post-validation.json 的 evidenceClasses=verified，同时 state/intentAlignment=unknown，保留 intent_alignment_missing warning；Outcome 另保留 user_visible_benefit_not_declared。未据测试通过宣称用户收益已证明或 WI 已关闭。

## 执行环境与边界

执行前重新生成并核对 plan.json：source command 仍为 `cargo test --locked --workspace`，13 节点全部 execute、0 reusable，既有 package 超时均为 600 秒。与前一份 plan-only 的 repository/snapshot/Runtime、元数据及 node/program/args/timeout 一致。原生准入与实际 argv、环境、权限及容量记录见 admission.json 和 invocation.json。

只使用之前已获准的 `/home/agent/.cache/ai-cockpit-verify-target` 精确目录的单次写权限。保留原 PATH 并前置 Cargo，Node/npm 均解析到已确认的安装目录；固定 RUSTUP_HOME/CARGO_HOME、隔离 CARGO_TARGET_DIR、CARGO_INCREMENTAL=0、CARGO_BUILD_JOBS=1、RUST_TEST_THREADS=1、RUST_TEST_NOCAPTURE=1 和仓库现有 runner。未安装工具、修改安全设置、替换 HOME 或新增 supervisor。

现有 `/usr/local/bin/docker-init` 为 Tini 0.19.0 / git.de40ad0，SHA-256 `8cc70470bb4b21c25ddca7108c5820fa14d5019b5f1ec7e94bfca65de2479990`，使用 `-s -g -vvv -w`。既有 timeout 给予工作负载 890 秒，终止余量 10 秒。实际运行在该边界内自然退出，没有触发终止路径，不能据此声称超时回收路径经过本轮实测。

执行前磁盘可用约 19 GiB、inode 充足、pids.max=max；16 GiB cgroup 内存占用主要为可回收文件缓存。完整执行前后僵尸基线均为 1,405，新增僵尸及遗留会话进程均为 0。另按“基线存活进程后来变为僵尸”的情况重新核对，仍无新增。Tini 日志包含真实 subreaper 注册、1,401 次孤儿僵尸回收及正常退出。已有 PID 1 僵尸未处理。

前后完整进程快照读取错误均为 0。期间 monitor-01 和 monitor-10 各记录一个 /proc/stat ENOENT；对应目录当时已确认不存在，其余监测无读取错误、无新增 PID 1 僵尸或 Tini fatal。竞态原样保留，未将观察失败替换为空集合。1,405 的本次基线比前轮 1,404 多 1，该变化在两轮完整观测窗口之间，来源未归因，未计入本次测试新增。

## 日志完整性与限制

外部 native/Tini stdout（650,614 字节）与 stderr（86,819 字节）原样保留，哈希见 result.json，退出状态单独保存。captured-node-streams/ 是从原生尝试记录解码的各节点输出。Runtime 既有每流 64 KiB 上限仍适用：repository stdout 标记 stdoutTruncated=true；所有节点 stderr 均未截断，其余节点 stdout 未截断。没有隐瞒该限制，也未将解码副本称为无限长度 Cargo 原始输出。

此前完整失败尝试、全部定向失败及环境修复记录均保留。本次成功来自实际重新执行 13 节点及新的正式收据，不是复用历史或 hosted 结果。

## PR/CI 最小下一步（尚未执行）

GitHub 只读查询确认默认分支 main，目前此 head 分支没有 PR。先由 Raydot 审阅当前差异、本次完整证据及保留的 validation/benefit 未知项；再按 Runtime 当前允许的 finish 流程完成必要收尾，使用同一分支向 main 创建 draft PR，附本次尝试 ID、正式收据、原始失败与恢复证据。随后按仓库既有 CI route 检查最新提交的候选 Runtime 与 gate 结果，不重复用本地定向结果替代 CI。

本轮仅完成验证与结果提交推送，没有执行 finish、创建 PR、手动触发 CI、merge、close 或清理。普通技术审阅由 Raydot 负责；没有重复请求已有 confirm_review 绑定。
