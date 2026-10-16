# WI-1086 完整验证、CI 与容量执行计划

> **For agentic workers:** 按现有委托由实施者使用 superpowers:executing-plans 逐项执行，交 Raydot 技术审阅。本次仅提交计划；未启动本计划中的验证、构建、CI 或清理。

**Goal:** 为已审阅源码取得当前环境的真实完整验证回执，再取得精确 PR head 的完整 CI，保留全部旧失败。
**Architecture:** 原工作树、原隔离 target、现有安装 Runtime；单次完整正式验证后保存回执，再以 Draft PR 触发现有工作流。复用 Cargo 产物，完整重新执行 Runtime 计划中的节点。
**Tech Stack:** Rust/Cargo 1.98.1、安装 Runtime v1.0.1、Python subreaper、GitHub Actions。
**Spec:** 当前 Contract、`../checkpoint-negative-fix-20261016/current-stage-v3.zh-CN.md`，以及 Ray 2026-10-16“先提交完整验证及 CI 的最短执行路径和容量计划；不盲目大构建，不扩大删除，完整通过前不关闭”的指示。

## 绑定与事实

| 项目 | 当前观察 |
|---|---|
| Work Item | WI-1086-sensitive-amendment-preflight-request-interlock |
| 已获 Raydot 定向技术审阅的源码 | `2a5910e967585c3589af382d29143f1b50b0a2c1` |
| 本次只读起点（证据提交） | `3609f24cb854f359fe127d1cb218e6e368bccffc`，干净；crates/Cargo/tests/workflow 相对上述源码无差异 |
| 原始基线 | `681df30e2fe8c869b6a41498822818c017dc4fb6` |
| Contract | `sha256:e7245a06dd9fc2a45346800a300183c2e3207a40a71bf579f2c5ec138538387f` |
| 当前 repositorySnapshot | `sha256:0840ed40a921b36f42e61d97d8db8eb664882c5239bdd6f9336744cea3918e54` |
| repositoryId | `sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b` |
| 安装 Runtime | `/tmp/wi1084-closeout-execution-retained-b1487517/ai-cockpit`；SHA-256 `5307ba5324d397e0fb971aa8b926ba8176336a0a6e76dfae7bd3a6e5cd11fa38` |
| 现有候选 CLI | 原隔离 target 下 `debug/ai-cockpit`；SHA-256 `3da828ca22763d620f2b3d4e7121f9400bb02854bbdd1b1bb1f6f3ff0868688b`；52,132,416 bytes |
| 工具链 | rustc 1.98.1 (48a229cea 2026-09-01)；cargo 1.98.1 (797e8a9bc 2026-08-05) |
| Runtime 状态 | checkpointed / yellow / verification not_ready；admission allowed；humanDecisionRequired=false；safeActions 含 run_verification |
| 验证声明及只读计划 | `cargo test --locked --workspace`；13 节点全部执行，nodesReused=0；各节点原有限时 600 秒 |
| 已有定向结果 | 207 项通过及 fmt 通过；不能据此声称完整验证通过 |
| 远端 PR / CI | GitHub GET：该精确分支所有状态 PR 为 []；起点 head 的 workflow_runs 为 0 |
| 清理 | 已批准的 279 文件因占用观察不完整全部跳过，实际删除 0、释放 0；本计划无新增删除 |

当前政策确认沿用正常追加的人审 receipt（原始 SHA-256 `3f1f7cec2c558fb7a21eb47116782d48ec0fb88257fa0e920fc6df156793102d`）。不重绑旧 receipt，不重复要求 Ray 批准同一政策。Runtime Outcome 原文仍显示 Human decision: Not recorded；不能把政策审阅解释为关闭审批。

## Global Constraints

- 只做 WI-1086；WI-1085、WI-1087、旧 control/review 的未提交内容与所有历史证据保留。
- 不修改 Contract、断言、门禁、parity、workflow；不安装候选为共享 Runtime。
- 单构建任务、单测试线程、单 Runtime worker；不与其他 WI 的构建、共享源码或生命周期操作并行。
- 每次执行前在同一执行器内读取真实准入，拒绝即不启动验证；旧未准入观察不能作为证明。
- 本地执行总上限 900 秒，含取消及回收；原生节点 600 秒保持不变。
- 不创建冷 target，不另做 release/no-run/预热构建；不删除、移动或替换旧工作树来掩盖冲突。
- 当前执行范围止于完整验证与 CI 回报；未完整通过前不关闭，也不在本次计划提交中 finish/archive、Ready、merge 或 close。

## Review Focus

1. 同一源码但执行环境不同会改变 command digest：最终运行前按精确环境重取计划，不复用本次发现用的默认环境 digest。
2. Cargo 复用产物不等于 Runtime 复用回执：13 个节点都要实际执行；任何失败、缺项、超时或未知均不能通过。
3. 同时链接、测试临时文件与日志可能增加空间：只读文件大小不能证明峰值，使用保守预算和停止线。
4. 原完整验证的 procfs errno13 原因仍未证实：保留失败，使用仓库现有 process-observer runner 串行协调；不修改权限或减断言。
5. CI 的 source head、GitHub tested merge 与 hosted Runtime 不是同一事实：分别核验绑定，不能替代本地正式回执。

## 1. 已完成的只读准备

- [x] 读取实际 status、validate 和人类 Outcome；未更改治理记录。
- [x] 默认环境及拟运行环境各作一次 `verify --plan-only`，仅规划所需 cargo metadata；均为 13 节点、0 复用。
- [x] 盘点原 target、磁盘、inode、进程、工具链、两个 CLI 哈希、CI 工作流与远端 PR/run。
- [x] 原始输出、命令、时间、元数据和字节 manifest 随本计划保存。

validate 仍为 unknown：`intent_alignment_missing`。status 仍有 `user_visible_benefit_not_declared`、`verification_evidence_missing`。这些事实不阻断当前 Runtime 已允许的验证；完整验证成功也不会自动补齐它们。

## 2. 容量预算与启动条件（提请 Raydot 审阅）

资源观察：2026-10-16T02:44:53.732512+00:00（日本时间 11:44:53）。

| 项目 | 实测 / 建议 |
|---|---:|
| /workspace 可用 | 681,738,240 bytes = 650.156 MiB（约 682 MB） |
| /workspace 可用 inode | 1,930,173 |
| 原隔离 target 实际占用（按 device/inode 去重） | 2,484,903,936 bytes = 2.314 GiB |
| target 路径逐文件统计中的硬链接重复量 | 75,948,032 bytes，不重复算容量 |
| 最大现有 executable / workspace rlib | 约 52 MB / 65 MB |
| /tmp 可用 | 2,293,882,880 bytes；不同 device，不能给 /workspace 增容，也不足以复制完整 target |
| 建议 warm run 启动线 | /workspace available >= 1,073,741,824 bytes（1 GiB） |
| 距建议启动线缺口 | 392,003,584 bytes（约 374 MiB） |
| 运行停止线 | /workspace available <= 536,870,912 bytes（512 MiB） |
| 新日志/元数据/取消收尾预留 | 67,108,864 bytes（64 MiB） |
| 从启动线到停止线、扣除预留后的产物增长预算 | 469,762,048 bytes（448 MiB） |

以上是建议的保守运行预算，不是实测峰值保证，也不是修改测试或 Runtime 门禁。已有产物可降低重建成本，但旧完整验证没有峰值磁盘记录，不能证明当前 650 MiB 能完成全程。

- [ ] 在当前工作盘获得上述余量后，立即重测磁盘、inode、活动构建与 target 一致性。当前容量结论为 **不启动**。请 Raydot 协调当前工作盘容量；不把未执行的清理计入预算。
- [ ] 所有新输出仍在原 WI 证据目录和原 target；不自动退到 /tmp、其他 target 或新工作树。
- [ ] 每 100ms 观察工作盘与 /tmp 可用量、日志增长及自有子进程。到工作盘停止线、/tmp <=512 MiB、日志预算耗尽或写盘错误时，终止本次自有验证进程树并保留 partial/failed/unknown。
- [ ] 启动前重新确认无其他 Cargo/rustc，运行时避免并发 Runtime 查询及其他 WI 测试。PID1=tail、PPID1 僵尸 21,316 为既有环境缺陷；使用 subreaper，扫描自身所有 task 的 children 并回收自有后代，记录前后值，不批量 kill。
- [ ] 不达启动线或资源停止后，不自动重复完整构建；将实际容量、失败和缺口交 Raydot。若只能更换环境，先另行解决保全/恢复与 Runtime 准入，不将另一环境事实写成这里的结果。

## 3. 本地完整正式验证：一次原生调用

**执行器要求：** 将下面环境及 argv 交给有准入检查、资源观察、超时和 subreaper 的同一执行器；不可裸跑。现有 focused runner 只针对 cargo test，不能直接声称已支持正式 verify。执行阶段需先准备该轻量封装，保存其原始字节；不修改产品源码。

环境：
```text
HOME=/tmp/wi1084-runtime-home-20261015
RUSTUP_HOME=/workspace/.rustup
CARGO_HOME=/workspace/.cargo
PATH=/workspace/.cargo/bin:<原 PATH>
CARGO_BUILD_JOBS=1
CARGO_INCREMENTAL=0
RUST_TEST_THREADS=1
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_TARGET_DIR=/workspace/ai-cockpit-wi1086-sensitive-amendment-preflight-request-interlock/target/wi1086-sensitive-amendment-preflight-request-interlock
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/workspace/ai-cockpit-wi1086-sensitive-amendment-preflight-request-interlock/tests/ci/run_process_observer_test_runner.sh
```

- [ ] 冻结源码为 2a5910e9 对应产品字节；实际执行 HEAD 必须记录为届时证据提交，另列 source commit。工作树须干净。
- [ ] 在同一执行器用安装 Runtime、显式 `--repo` 重查 `work-item status --id <WI> --json` 与 `work-item validate --id <WI> --json`。准入 allowed、humanDecisionRequired=false、safeActions 含 run_verification 才可继续。若要求 run_preflight，则只按真实准入执行并重查；缺权、矛盾、待人审时停止，不能利用混绑恢复。
- [ ] 同环境执行下面规划 argv 并保存新 command digests：
```text
/tmp/wi1084-closeout-execution-retained-b1487517/ai-cockpit verify
  --repo /workspace/ai-cockpit-wi1086-sensitive-amendment-preflight-request-interlock
  --work-item WI-1086-sensitive-amendment-preflight-request-interlock
  --stage task --workers 1 --plan-only
```
- [ ] 确认声明仍是 workspace 全量、13 节点全执行、原生 timeout=600，没有新增 skip/filter 或 receipt reuse。
- [ ] 启动同一 argv，**仅移除 `--plan-only`**。主进程最多 885 秒，预留 15 秒给自有进程树取消/回收，使本地总调用不超 900 秒。到时记录 timeout，不延长或拆开来冒充一次成功。
- [ ] 原始 stdout/stderr、退出码、执行/源码 HEAD、git diff、全部绑定、环境、rustc/cargo、CLI 路径与运行前后 SHA-256、磁盘最小值、时间和子进程回收写入新独立目录。
- [ ] 原生正式成功回执必须由 Runtime 生成；随后只读核验当前 Contract/snapshot/repository/runtime、passed=true、完整 13/13、无失败/timeout/缺项，并重查 status 的新鲜度。命令 exit 0 或 stdout 单独不能证明通过。

旧正式 attempt `sha256:3469bd4b3664e817ac4f86a4e4ba892f7c12083ab397e19ad1eb40e30fe9603e` 原样保留，raw SHA-256 `d83bb2da7b87d3b9730a132f7f5c9585c3012845c8ec8daed1bf272978cf72d5`。其旧源码 8ea965d8、旧 Contract、610.901 秒、10 通过/3 失败继续是历史事实。单项重跑不能覆盖它。若新完整验证仍失败，保留新 attempt 并按恢复指南回报；不继续关闭。

## 4. CI：保存本地回执后只走一次现有 PR 工作流

- [ ] 本地完整成功后，将必要原始回执及最小文档按字节 manifest、敏感检查、正常 commit/push 保存；不把未证明的收益或 intent alignment 写成已完成。生成文件仍仅由 Runtime 写。
- [ ] 冻结该精确 PR head，记录与审阅源码 2a5910e9 的产品差异为空；在原分支创建 **Draft PR** 指向 main（当前无既有 PR，不建立第二分支或重建工作树）。不 Ready、不合并。
- [ ] 由 `.github/workflows/ci.yml` 的 pull_request 事件自然触发；该 workflow 只接受 main push 和 PR 事件，普通工作分支 push 不触发。无 workflow_dispatch、无借用历史 green。
- [ ] 等待同一精确 head 的原工作流：`route`、`quality`、`windows-runtime`、`v1-behavioral-oracle`。动态路由按真实 Contract 与 diff 选择；不强制 light、不跳 gate，实际适用/不适用数量由新报告决定。
- [ ] route 现有 release CLI 只构建一次并交 quality 使用；quality 正常做 immutable shadow、candidate hosted Runtime verification、Contract-aware gate、repository gates 和 workspace package coverage。Windows 和 oracle 沿用原工作流。此处不在本地重建整套 release/legacy CI 产物。
- [ ] 记录 run/attempt/jobs 的状态、结论、实际耗时；核验 `ci-quality-revision-binding` 与 Windows revision binding 的 source head、event base、tested merge，分别保留 hosted Runtime/Contract/snapshot/digest。
- [ ] 取回必要 JSON/日志并校验摘要；Actions 默认仅保留 7 天，不能只交临时页面。保存质量路由、repository/Contract gate、workspace coverage、hosted orchestration/receipt 和诊断。CI hosted 成功单独报告，不能代替本地正式回执。
- [ ] 收集完 CI 原件后，若需再追加治理材料则记录新 head 及其实际必需检查，不把旧 head 全绿沿用为新 head 全绿；只在真实失败或新 head 必需时重跑。

## 5. 通过标准与交接

- [ ] 本地当前绑定完整 13/13 正式回执通过；所有原失败、RED 与未准入观察仍可查。
- [ ] PR 精确 head 的四个 job（含真实动态适用性）均满足原工作流；source/tested merge/hosted/local 分开。
- [ ] 当前 Runtime status/validate、剩余治理未知、磁盘和进程基线复查后，提交给 Raydot；优先逐个 WI，不推进 WI-1087 实施。
- [ ] 完整验证成功只是进入后续收尾检查的前提；仍按实际 Runtime 条件处理 intent alignment、收益、材料审阅等缺口。关闭、合并和清理不由本计划自动授权。

**最短路径：** 当前工作盘补足安全余量 → 原 target 上一次原生完整验证（900秒总限）→ 保存回执并冻结 head → 原分支 Draft PR 一次现有完整 CI → 精确绑定交 Raydot。当前停止点是容量与计划审阅；本轮没有开始构建、完整验证、CI 或清理。
