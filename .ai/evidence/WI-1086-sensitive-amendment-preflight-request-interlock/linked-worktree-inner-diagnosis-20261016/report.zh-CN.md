# linked-worktree 唯一剩余失败的定位与环境修复

本轮源码 HEAD 为 `fc5d0759d05f6ff79593ac9a970291b983bc8c0d`。只复现 `linked_worktree_cargo_test_binary_embeds_its_own_manifest_directory`，未重复此前已绿的 CLI/MCP 或 recovery_decision，也未执行完整 workspace 测试。没有修改产品源码、测试源码、断言、Contract 或真人决策。

## 已读事实

`verification_service.rs:63` 的隔离子进程主动移除 `CARGO_TARGET_DIR`；`execution_context.rs:966` 起的原生 fallback 按当前 HOME 与 checkout hash 选择 `~/.cache/ai-cockpit-verify-target/<hash>`。本环境 HOME 为 `/home/agent`，其普通沙箱视图只读。此前失败 fixture 的普通与 linked checkout 均有正确 Cargo.toml、Cargo.lock 和 src/lib.rs，未保存可读取的内层 receipt，fallback 目录也不存在。单凭这些事实尚不能定根因，因此使用已安装的 `/usr/bin/strace` 跟踪同一精确测试的 write 系统调用。

## 实际证据

07-exact-strace 中，内层 Cargo PID 279645 的 stderr 写入明确为：

```text
error: failed to create directory `/home/agent/.cache/ai-cockpit-verify-target`
Caused by:
  Read-only file system (os error 30)
```

原始字节来自 `07-exact-strace/inner-write-syscalls.log:167` 至后续各次 stderr write；该测试失败，Cargo 退出 101、native/Tini 返回 1，原生尝试为 `sha256:3c68c71edcc32107f63dd0de085c48651e3afd28f3406ed68f3f171d40173918`。strace 只作诊断，不是新增 supervisor。

08-exact-cache-write 首先请求只给尚不存在的 `/home/agent/.cache/ai-cockpit-verify-target` 增加单次写权限；测试仍在相同断言失败，目录之后仍不存在。该次未启用内层 trace，不另行声称其底层错误文本已被捕获；失败尝试 `sha256:758e33a450b7cee6ade85b7c980ec80fa6fa505b00e54a148e601bea47009fe5` 保留。

随后使用受支持的单次文件系统权限请求，给予现有父缓存目录 `/home/agent/.cache` 写权限，仅执行 `mkdir -p -- /home/agent/.cache/ai-cockpit-verify-target`。命令退出 0，实际目录为 agent 所有、0700。该准备动作没有改系统设置或 HOME，也未开放测试访问整个父缓存目录。

09-exact-cache-ready 对测试命令仅授予已存在的 `/home/agent/.cache/ai-cockpit-verify-target` 写权限，其余环境与原断言不变。测试 1 过、0 败；Cargo、native、Tini、外围 shell 均退出 0，成功尝试为 `sha256:ea4afe64d2fcfd7914dd3224144da09337df6d6bde57d6cfad6244888b7b7ccc`。内层 write trace 同时捕获普通与 linked checkout 各自正确的 MANIFEST_PATH，原有业务断言确实执行并通过。

三个精确运行均重新取得原生 run_verification 准入，使用已验证的现有 Tini、修正 PATH、仓库 runner 与单线程参数。每次进程快照均为 1,404 → 1,404 个僵尸，观察错误、新增僵尸及遗留会话进程均为 0。该基线比前轮 1,403 多 1；两轮之间新增的来源未被这些快照归因，不能将其算入某次测试，也未处理已有 PID 1 僵尸。

本轮确认并消除的是沙箱缓存写权限问题。不是 manifest 路径断言缺陷；无需 scope amendment 或源码修复。所有原始失败、日志、退出状态与成功记录保留。成功仍限于这个精确测试；完整 13 节点验收尚未执行，旧 status 投影不替代完整验收。

## 下一次完整验收计划（尚未执行）

同一候选 Runtime 1.0.1 及原生 `verify --repo /workspace/ai-cockpit --work-item WI-1086-sensitive-amendment-preflight-request-interlock --workers 1`。保留现有 PATH 并前置 Cargo，保持固定 RUSTUP_HOME/CARGO_HOME、隔离 CARGO_TARGET_DIR、CARGO_INCREMENTAL=0、CARGO_BUILD_JOBS=1、RUST_TEST_THREADS=1 与仓库 runner；加入 RUST_TEST_NOCAPTURE=1 使 panic 不再依赖大段父测试 stdout 捕获。

完整命令仍由现有 Tini `-s -g -vvv -w` 包围既有 timeout（890 秒、10 秒终止余量），仅授予上述现有缓存目录的单次写权限。采用原生默认每 Cargo package 节点 600 秒，不使用被 policy 拒绝的显式 900 秒 override，不用 shell workspace 单节点替换原生 13 节点 manifest。

已生成真实 `full-workspace-plan-only.json`：13 planned、13 to execute、0 reusable；顶层 verification processesSpawned=0，coverage manifest 单独记录 1 次 cargo metadata 规划进程。source command 仍是 `cargo test --locked --workspace`。这份文件是规划结果，不是执行收据。

执行前刷新原生准入并记录新进程基线；执行时保留 native/Tini 原始日志、完整尝试收据及每节点退出/超时/截断标志。Runtime 仍有每流 64 KiB 捕获上限，本计划不声称它提供无限长度 Cargo 输出；任何失败或关键日志截断应如实停止并定位。执行后核对全部 13 节点确实执行、无复用替代、验收通过及进程差异，再进入后续评审。本轮没有启动该完整执行、PR、合并或关闭。
