# WI-1086 六次串行定向诊断

本轮仅执行 Raydot 审阅通过的六次定向诊断，不代表完整 workspace 验证通过。产品与测试源码未变；原断言、人审记录及此前完整失败尝试均保留。

执行源码 HEAD：`fe652eac2367987b82692f54f5ff2d0173e8e2e0`。Contract：`sha256:b217077be18ddb78c731de6e4188e048416e914f63acb89a1b0e95781a325360`；六次准入查询的 snapshot 均为 `sha256:5f85b933a4ccd16836dfc7f1271a87e950154ab5bcb7bf0da816791ac95f88d8`，均允许 `run_verification`，没有新真人门禁。

使用候选 Runtime 1.0.1，绝对路径及 SHA-256 与 `../runtime-candidate-20261016.json` 相同。现有 `/usr/local/bin/docker-init` 为 Tini 0.19.0 / git.de40ad0，SHA-256 `8cc70470bb4b21c25ddca7108c5820fa14d5019b5f1ec7e94bfca65de2479990`。未安装工具、修改系统设置或新增 supervisor。

单次调用保留原 PATH 并前置 Cargo 路径，启用仓库已有 Cargo runner、`CARGO_BUILD_JOBS=1`、`RUST_TEST_THREADS=1`、native `--workers 1` 与 libtest `--test-threads=1`。Tini 使用 `-s -g -vvv -w`；其主子进程为现有 timeout，工作负载期限 890 秒、强制终止余量 10 秒。native 对用于日志重定向的 bash 命令采用默认 300 秒期限，未申请不被 policy 允许的显式 timeout override。

每个目录的 `invocation.json` 保留实际 argv、环境与旧尝试列表；`admission.json` 保留真实原生准入；`process-before.json` / `process-after.json` 为只读进程快照；`cargo.stdout.log` / `cargo.stderr.log` 是不经 native 64 KiB 捕获上限的独立 Cargo 日志；native 与 Tini 的完整输出、实际启动退出码及结果也分别保留。内层 bash 使用 `exec cargo ... >log 2>log`，没有将失败改成成功。

| 目录 | 定向测试 | Cargo 结果 | native/Tini 返回 |
|---|---|---|---|
| 01-cli-calibrated | calibrated_auto_command_reuses_a_persisted_receipt_in_a_second_process | 1 过、0 败，退出 0 | 0 |
| 02-cli-work-item | work_item_detected_command_uses_dynamic_profile_authorized_reuse | 1 过、0 败，退出 0 | 0 |
| 03-mcp-auto | mcp_auto_verify_uses_the_same_profile_authorized_cross_process_service | 1 过、0 败，退出 0 | 0 |
| 04-mcp-receipt-rejection | repository_bound_verify_persists_execution_attempt_when_receipt_recording_fails | 1 过、0 败，退出 0 | 0 |
| 05-mcp-failed-execution | repository_bound_verify_persists_failed_execution_as_non_reusable_attempt | 1 过、0 败，退出 0 | 0 |
| 06-repository-verification-service | 整个 verification_service 目标 | 16 过、1 败，退出 101 | 1 |

本轮明确失败测试为 `linked_worktree_cargo_test_binary_embeds_its_own_manifest_directory`：外层 `verification_service.rs:77` 转述隔离子测试失败，内层在 `verification_service.rs:415` 因 `assert!(run.receipt.passed)` 失败。完整 panic 见 `06-repository-verification-service/cargo.stderr.log`。该断言没有打印内层 Cargo receipt 的 stderr，因此底层失败原因仍未知。本次不能将它归因于 npm、process observer 权限或 fixture preflight 顺序；也不能据新结果回填旧截断日志。

第六次原生失败尝试 ID：`sha256:23f3251051ffcfe707d2d0a19b26c6850c745c410d74984c11906cc58352f68d`，状态 `execution_failed`，其 execution record 保留退出 101、passed=false、timedOut=false。前五次各自生成真实成功尝试，ID 见各目录 result.json 及 round-summary.json。第六项失败后没有再测试，没有全量重跑、PR、合并或关闭。

六次均有真实 Tini subreaper 注册及退出日志；前后 PID 1 僵尸基线均为 1,403，新增僵尸、同一执行会话遗留进程、观察错误均为 0。Tini 记录了合计 27 次孤儿僵尸回收。该结论限于本轮快照及运行日志，不保证任意后代在未来运行中都被回收。已有 PID 1 僵尸未处理。

另保留一个外围 shell 收尾差异：前两次工具报告外围 shell 退出 1，但实际 Cargo、native 和 Tini 主子进程均退出 0；第二次 EXIT 记录也为 1。第三次起使用非 login shell，第三至五次外围返回 0，第六次与实际失败一致返回 1。没有为此重跑测试或修改 shell 配置；收尾差异原因未进一步确认，相关原始返回记录见 host-shell-observation.json。

post-round-native-outcome.json 仍将最后一次成功定向收据投影为 verified/green，仅列出 user_visible_benefit_not_declared。此投影不代表本轮全部通过，未手改 Summary 或修复选择逻辑。验收继续以已保存的完整 workspace 失败尝试和本轮定向失败证据为准。下一步由 Raydot 审阅本次 repository 断言及其内层执行事实；没有提出或启动完整 13 节点重跑。
