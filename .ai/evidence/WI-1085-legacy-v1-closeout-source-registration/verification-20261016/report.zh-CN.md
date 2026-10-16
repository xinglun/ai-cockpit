# WI-1085 正式验证真实失败报告

Outcome: 🟡 正式验证失败，等待空间恢复与 typed scope amendment。

## 已记录的审阅

Raydot 的 accept_permitted_unknowns 已通过原生流程记录，reviewerActor=agent:Raydot、assurance=self_declared。requestDigest 为 sha256:94bdbf788b5bc75f54766d9cff37830f166b57cf6aa5ef629df84525bc4728d1；仅接受 repository 的 legacy_closeout_source.rs 源码和对应测试两份 readable_committed_rust_syntax_unknown。原生 receiptDigest 为 sha256:e093474e78b36185793f67fa74adf5583486a2c7d4d731bac19cf36294e61ad8，Contract 和材料绑定未改变。此决定不是正式验收。

## 实际执行

本轮源码 HEAD fe205fa70646114d35dd7f618cd840a21a7b035a；隔离 target、单 worker、单 Cargo build job、RUST_TEST_THREADS=1，使用仓库已有 observer runner 与 Tini subreaper。外层 timeout 890 秒，TERM 后最多 10 秒 KILL；package 上限 600 秒。编译预热 --no-run 实际成功，但不算测试通过。

原生 attempt a1f426f3b6e44021d6cc094ba17f251f84ef8c276b4dbcd5f0f47c67268e655b 于 2026-10-16T12:10:10Z 记录 execution_failed：13 节点实际执行，10 通过、3 失败、0 复用、0 超时，最大并发 1，原生 elapsedMs=521796，工具 exit=1。未生成成功 canonical verification receipt。

- MCP：material_review_plan_tool_uses_shared_read_only_repository_service 在 material_review.rs:88 断言实际工具数 33、期望 31。新增两个原生 legacy source 工具已在 TOOL_NAMES 中声明为 33。这是实际断言失败，不能归因为磁盘满。
- repository：编译 status_projection 测试时链接器 signal 7 / Bus error。磁盘耗尽是环境相关怀疑，链接器消息本身没有证明完整因果。
- verification：创建 .fingerprint 目录明确失败：No space left on device (os error 28)。后续只读 df 与 statvfs 均记录 /workspace 可用 0 字节，默认沙箱也无法启动。

静态还发现 tests/usage.rs:25 同样期望 31；本轮 MCP 在更早的 material_review 失败后停止，没有执行到它，不把此静态发现算成第四个执行失败。两份测试均在当前 scope 外。建议向 Raydot 提交最小 typed amendment：只增加 crates/cockpit-mcp/tests/material_review.rs 和 crates/cockpit-mcp/tests/usage.rs，并精确同步工具数 31→33，保持其余断言。尚未应用，不修改产品或断言。

## 进程、空间与原字节

正式运行前后僵尸均为 1419；按 PID+startTimeTicks 对比无新增、无丢失。Tini root 829050 已消失，同 session 无存活进程，已采样的 owned live/zombie 均无残留，未见活跃 cargo/rustc。Tini 日志记录 main child 正常返回 1、No child to wait，并保留其收养僵尸回收日志。过程快照不能枚举全部历史后代，瞬时读失败只在后续确认 PID 消失时标记 resolved。详见 process-recovery.json 与原始观测。

26 份 native captured streams 从 attempt 的 hex 字段逐字节解码到短路径 streams/cockpit-*.{out,err}.log；streams.json 记录每份长度、SHA256、截断标记和真实原生历史字段引用。本轮所有 captured streams 的截断标记均为 false。原生 attempt、旧失败证据、原始 stdout/stderr 均不覆盖。

唯一清理候选已报用户：/workspace/ai-cockpit/target/wi1086-sensitive-amendment-preflight-request-interlock/debug/，占用 11,885,486,080 字节；仅是旧编译生成目录，不包括同级 release/ai-cockpit。当前批准待定，未执行删除，未动旧环境、证据、独有 cleanup receipt 或其他 target。

## 当前准入与下一步

失败后 native status 保留 humanDecisionRequired=false、safeActions 包含 run_verification，canonical verification missing；material review receipt 仍为 e093474e…。validate 保留 11 required_scenario_unverified 与 intentAlignment=unresolved，无 Findings。acceptanceEvidence=verified 是映射形状，不代表验收通过。原生 Outcome 原文一并保存，未手改 Summary。

需先恢复空间，再按 native typed amendment 与当前人审门槛处理两份测试；材料 requestDigest 一旦变化必须报告新旧绑定并重新审阅。此报告和失败收据先提交推送供 Raydot 审阅，不重新运行旧 head，不发 PR、Ready、合并、归档、关闭或清理。

宿主实际显示此 Outcome 的状态未知；本报告是已生成证据，聊天中另行显式交付。
