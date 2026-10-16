# WI-1085 空间与最小 scope 提案补充

Outcome: 🟡 单一候选清理清单与两份测试草案已准备；未核准、未执行清理，未应用 amendment，未修改测试、产品或验收断言。

## 复用现有清单

原清单提交 f168de42038091303801219ed3b8888afce768c8，路径 `.ai/evidence/WI-1085-legacy-v1-closeout-source-registration/verification-20261016/native-result.json` 的 `storageRecovery`，人读报告为同目录 `report.zh-CN.md`。本次仅补该清单原先缺少的目录构成、链接/挂载、占用、证据排除和 typed 请求，不重复验证。

## 唯一清理候选

完整路径：`/workspace/ai-cockpit/target/wi1086-sensitive-amendment-preflight-request-interlock/debug/`。

精确占用：11,885,486,080 字节（约 11.07 GiB）。481 个目录、2,533 个普通文件，0 符号链接、0 内部挂载、0 跨设备条目、0 遍历错误。完整逐项 device/inode/size/allocated/link 清单见 cleanup-tree-inventory.json。容量按 device+inode 去重，旧 Cargo 程序与 deps 硬链接的分组归属受遍历顺序影响；总量与原 du 一致，不保证 overlay 实际回收恰好等于该总量。

构成：deps 11,548,336,128 字节；build 130,732,032；.fingerprint 7,806,976；根 ai-cockpit 160,763,904；根 cockpit-release 37,830,656；其余目录/依赖路径/锁 16,384 字节。上述是单候选内部构成，未扩大删除清单。

可从已推送源码重建编译产物：已记录最后一次 WI-1086 全工作区执行 source HEAD 4838c6bd4a0cc7f42fbdb805e416007389718d9c，其为当前 merged main 的祖先。来源证据在 main 的 `.ai/evidence/WI-1086-sensitive-amendment-preflight-request-interlock/docs-parity-recovery-20261016/native-verification/invocation.json`，Rust/Cargo 1.98.1。缓存包含多个历史提交的构建输出，不能保证逐个历史二进制或时间戳字节重现；无二进制哈希连续性声明。保留的实际 Runtime 位于 release 而非此候选，源 a12ee2ff4413ac261560339c9ed7af59dde0f701，实际 SHA256=0ffdcd20740f22b96621cf06adb5479d83fb435f503a26c18d824d61c4bfe9c4。

## 占用检查与限制

2026-10-16T12:24:13Z 扫描 7 个存活进程的 exe/cwd/root、fd 和 maps，可读范围内匹配 0；1419 个已有僵尸另计。PID 200 dockerd、PID 251 containerd 为 root 基础服务，其上述 /proc 项读取均被权限拒绝，合计 10 个未解决读取错误。**全局占用状态为 occupancy_unknown，不能声称完整排除占用。** 名称和 UID 不能替代不可读项。详见 cleanup-occupancy.json 与 restricted-process-identities.json。今后如获清理授权，行动前仍须重新核对边界和占用；本次没有绕过权限或删除。

## 唯一证据与保护对象

protected-path-check.json 确认候选无 .ai、原生 receipt、captured-streams 或原始调用日志目录；保护树逐项 inode 检查无与候选共享普通文件、无读取错误。明确保留：实际 release/ai-cockpit、两个检出的 .ai、全部 /tmp 原始日志及临时 Git 结果、当前 WI-1085 target、工具链和共享 fixture cache。候选无 symlink 或内部 mount 可指向这些对象。

WI-1086 唯一清理收据完整路径：`/workspace/ai-cockpit/.ai/decisions/WI-1086-sensitive-amendment-preflight-request-interlock.cleanup.000001.6b9cbf0d4d659d2e7a5f235a6d63ae77c2f02da9dc2d064bf419824179fc99c8.json`，仍保留原字节，SHA256=7304e582f78f5a49ec751f01f5497a3a310a66c5621fa3353813869c961a6a8d。编译缓存的精确历史字节不保证重现，这一限制单独保留。

## 最小 typed scope amendment 草案

当前 repositoryId sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b；WI-1085-legacy-v1-closeout-source-registration；expectedContractDigest=sha256:eae15cbbdbc36606635db999976611d3c036260296d2b5b24b9afa183ae03bd5。

`two-tests-scope.DRAFT.request.json` 按现行 ContractAmendmentRequest 形状：schemaVersion=1，独立 changeId，两个 /scope add，然后仅替换 /concurrencyBoundary 为原值加这两条 implementationPaths；其余 concurrency 字段逐值保留。仅增加：

- crates/cockpit-mcp/tests/material_review.rs
- crates/cockpit-mcp/tests/usage.rs

`two-tests-counts.DRAFT.patch` 仅展示各一行 tools.len() 31→33，其他断言全部保留；未应用。material_review.rs:88 为本轮实际失败，usage.rs:25 为未执行到的静态旧计数。现有 rpc.rs:426 已精确断言 33，作为对应参照。未调用原生 amend，未运行 prospective Contract 验证，未生成新 Contract、人审或材料决定。

## 两工具对应

- work_item_legacy_closeout_source_plan：TOOL_NAMES 行31；schema 行727；分派行2120，调用共享 cockpit_repository::plan_legacy_closeout_source（MCP行2133，repository 定义行1014），只读 exact plan。
- work_item_legacy_closeout_source_record：TOOL_NAMES 行32；schema 行731；同一分派，调用共享 cockpit_repository::record_legacy_closeout_source（MCP行2141，repository 定义行1361），显式记录 exact plan。

两工具使总数 31→33；未删除工具或放宽 schema/验收。新增工具具体请求/返回验证继续由现有 legacy source 测试保留。

## 后续边界

Raydot 先审阅候选、占用 unknown 与两文件提案。空间恢复后，先核对远端 f168de42 及其后续纯证据树，再以正常流程对齐原分支、索引与已保留未跟踪证据，不使用 reset 丢弃记录。之后只有在原生准入、typed amendment 和其新绑定所需真实审阅均满足后，才应用两行变更并验证。Bus error 只记录实际 signal 7，不定性为磁盘所致；磁盘满仅由 verification 节点 os error 28 及实际容量观测确认。
