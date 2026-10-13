# WI-1079 本地验证失败与定点诊断

日期：2026-10-13。此记录补充当前分支上的本地正式验证失败与一个定点诊断；不替换、不改写 Runtime 生成的回执，也不把单节点成功提升为完整验收。

## 绑定身份

- 仓库：`xinglun/ai-cockpit`
- 分支：`codex/wi1079-post-merge-lifecycle-layout`
- 验证时源码 HEAD：`a8f2388a2b9ec028bf0848f1856084ae52cecc1d`
- repository ID：`sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b`
- Contract：`sha256:b3980367ee51ea5d63fe8b4f69df3929b67fd1b7ebd7d1a60003722bd424772e`
- execution scope：`sha256:fde9cd0d9edc6f409418e5682ce037d2947c5dd3e84e593ca78a82891c53fc24`
- 本地验证快照：`sha256:042362055af08a2119705cb5a3b77804a58df2d28fe9152fcc1d8fdb20d39b35`
- material manifest：`sha256:41ff30216af1f3eaf399314752d66dc10f6812c3f419b3c170b96b8d46ba60f4`
- material review receipt：`sha256:735975c44ba7bdf88932a8b02b352223fa0fcce8aa3e8c22ee73002091e16e50`
- formal 验证 CLI 候选：`/workspace/.home/.cache/ai-cockpit-verify-target/wi1079-candidate-a8f2388/debug/ai-cockpit`，SHA-256 `6b3e1bfdf4d0d550c16782fef8aa3df1e9e95cda54e2839315577e24849b5753`。
- 已安装治理 Runtime：版本 `1.0.1`，SHA-256 `b2a43b0a1f7c460f5f4d4969d02b02c4f27d45152b1027d46e68d458a9ec63fd`。该 Runtime 用于之后的状态与 Outcome 查询；正式候选回执自身记录 Runtime SHA `6b3e…5753`。
- 历史本地候选 `78642ed…cad82` 与 hosted 候选 `84c101e4…24ffd` 仍是独立历史身份，不参与本次回执。此前候选构建 manifest 的说明称它未担任正式 `verify` CLI；但实际命令调用了上述候选路径，且两份 Runtime 回执都记录其完整 SHA `6b3…5753`，故本记录以实际命令和回执绑定为准。

## 正式 14 节点运行

执行的是 Contract 声明的完整计划：`cargo test --locked --workspace` 分成 13 个 Cargo package 节点，再加 `bash tests/workflow/resource_finalization_policy_test.sh`。使用 serial worker 和 900 秒外层 watchdog；回执记录 14/14 节点执行、并发峰值 1、执行耗时 260531 ms。

- 通过 11/14：10 个 Cargo package 与 Bash 策略检查。
- 失败 3/14：`cockpit-cli`、`cockpit-repository`、`cockpit-verification`。
- 原始正式失败回执：[verification-attempt.09d58…b43f.json](WI-1079-v1-0-1-post-merge-lifecycle-layout.verification-attempt.09d58dda27a30337625e886145f91fa4d0ef38c6e1bdd4f64ff1a6f87be3b43f.json)。SHA-256：`398d15b5ec464233c5fa525c4a35d9f0f2d4ac27506561f4ea73534758e12355`。Attempt ID：`sha256:09d58dda27a30337625e886145f91fa4d0ef38c6e1bdd4f64ff1a6f87be3b43f`。

当时 repository package 中 4 个 lifecycle fixture 都因缺少 PR #1034 head 对象 `d18be4f8624d08f91fe1c961dfed287cfe9543df` 无法执行 `git merge`。之后只 fetch 了该 PR head 到只读诊断 ref，没有切换或改写当前分支源码。

## 定点诊断

对象拉取后，只用候选 Runtime 执行一个代表性 fixture：

```text
cargo test --locked --package cockpit-repository --lib resource_lifecycle::post_merge_sync_tests::post_merge_layout_same_green_preflight_still_requires_valid_material
```

这个单节点检查通过，耗时 17962 ms。原始回执：[verification-attempt.60ff5e…b8ab.json](WI-1079-v1-0-1-post-merge-lifecycle-layout.verification-attempt.60ff5ee321630825a2f7726099aad851431c745846d4cfbd423aaf4512c8b8ab.json)。SHA-256：`91c67f52529f2080f5aad0332770631fed78696bd25ef1a5e2dea4484a4afe15`。Attempt ID：`sha256:60ff5ee321630825a2f7726099aad851431c745846d4cfbd423aaf4512c8b8ab`。它只覆盖一个 test，不覆盖完整 repository package，更不覆盖 14 节点 Contract 计划。

## Rustup 设置的只读因果诊断

- Rustup：`1.29.1 (d95a37b6a 2026-08-13)`；active rustc：`1.98.1 (48a229cea 2026-09-01)`。
- `/workspace/.rustup/settings.toml` 的非敏感键名：`version`、`default_host_tuple`、`default_toolchain`、`profile`；另有 `[overrides]` 节。未在此复制设置值，未改动该文件。
- 当前解析器 `parse_rustup_default_toolchain` 只接受 `default_toolchain`、`profile`、`version`；其他键命中 `_ => return None`。`observe_runtime_toolchain_digest` 通过 `parse_rustup_default_toolchain(&settings)?` 传播这个 `None`，令 `observe_composition_identity` 无法产生完整身份。回执中的具体运行时拒绝原因是 `runtime could not observe a complete composition identity; reuse disabled`，`reuseDecision.kind` 为 `unknown`。解析函数没有更细的用户可见错误码；`default_host_tuple` 是触发该兜底拒绝的实际键。
- `cockpit-verification` 失败节点的首个相关测试 `complete_owned_execution_survives_external_unknown_and_retries_fresh_noop` 期望 spawned 1、实际为 0；同节点另有 `source_or_toolchain_changed` 与 `command_failed` 预期不符，随后测试锁 poison 造成额外失败。该测试调用同一 composition 身份观察路径，因此与缺少 Rustup toolchain identity 一致。
- `cockpit-cli` 的 `cli_and_mcp_composition_share_exact_runtime_supervisor_and_outcomes` 也记录了相同的 `reuseDecision.kind=unknown` 与上述身份原因；MCP attempt 进一步因 `composition_cleanup_deferred` 失败，清理记录是读取 `/proc/<pid>/cwd` 返回 EACCES、进程身份仍可观察。EACCES 是额外的进程清理观察结果，不等同于 Rustup 键解析拒绝；本地普通同 UID 子进程的 cwd 探针可读，因此没有证据表明整个 `/proc` 都不可读。
- 因果判断基于当前源码调用链和两份原始回执；没有重跑这两个失败节点，也没有声称修复。`crates/cockpit-verification/src/composition.rs` 不在本记录对应的原始 WI-1079 Contract scope，本次未改源码或断言。父协调已批准后续通过 Runtime 支持的正式 typed amendment 窄扩此文件及必要回归测试范围；该 amendment 尚未执行，本记录仍绑定原 Contract digest。

## Runtime 状态与停止边界

候选 Runtime 在单节点诊断后写出的当前 `verification.json` 标记 `passed=true`，候选状态也曾显示绿色；这不代表完整计划通过。已安装治理 Runtime `b2a4…63fd` 对相同 Work Item 判定 `evidence_stale`、`verification=not_ready`，安全下一步为 `run_verification`。完整失败回执仍保留，`finish`、`archive`、PR close/merge 均未执行；PR #1035 继续保持 draft。

## 进程观察基线

本轮诊断开始前与只读检查结束后：PID1 均为 `tail`；PPID1 zombie 均为 318，其中 git zombie 310；cgroup `pids.current` 均为 355、`pids.max` 为 `max`；未发现活动 Cargo、rustc 或 ai-cockpit 运行进程。当前观测只涵盖本环境可见的 PID namespace 和采样时刻；PID1 不回收这些子进程，故 PPID1 计数不能代表宿主机全局，也不证明历史触发限制。没有批量 kill、删除旧环境或更改系统配置。

恢复材料原提交已将当前 PR head 推进到 `a8f2388…cc1d`。本诊断提交新增原始失败与定点回执及此说明；它们描述的是 `a8f2388…cc1d` 对应源码快照，不表示后续证据提交的新 HEAD 已通过正式验证。CI 对本诊断提交的门禁需单独查询。

推送前敏感检查覆盖本说明和两份 JSON（包括 hex 编码的 stdout/stderr 解码内容）：未检出私钥标记、GitHub token、AWS access key、Bearer 凭据或高置信度 secret assignment 匹配。
