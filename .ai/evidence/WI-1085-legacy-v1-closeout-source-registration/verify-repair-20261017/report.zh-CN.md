# WI-1085 修复版本正式验证与审阅交接

Outcome: 🟡 验证通过，交 Raydot 作最终技术审阅；Draft PR #1044 保持 Draft。

本报告由 agent:Codex 编写。最新实现提交为 `007fdcba2e81d01285a694dce57cd78e0f74fcf4`；本次实际本地正式验证与 CI source HEAD 均为 `b5a219eab232695fe2b69102c9de5b3062837ec3`。本次收尾提交只加入 `.ai/` 证据及原生记录，不修改产品、断言、CI workflow 或三语文档。该收尾提交的 SHA 由 Git 提交产生，提交后的 HEAD/CI 状态在 PR 和可见 Outcome 中分别报告，不能把旧 CI 的 SHA 说成新 SHA。

## 实现与基线

- `main` 的 `9e370e1c2c2e2dceda79de62ca4e849ae313624c` 已通过 `ffd1efa7b59e5405c73c4bae5ab26d037bc48866` 合入原工作分支；三语 parity 保留双方内容。Contract 受保护基线仍为 `681df30e2fe8c869b6a41498822818c017dc4fb6`，未手改。
- 本模块现有 scope 中新增四个真实 Windows directory junction/reparse 单测，由既有 Windows `cargo test --locked -p cockpit-repository --lib` 执行。真实 `mklink /J`、reparse 属性、ambient 对照、outside victim、canonical 路径未写入等断言均保留，无静默 skip。
- 修复 CI 暴露的四处 repository Clippy 问题及一处 MCP 等价 let-chain；Windows helper 仅按 Path components 重建本机分隔符，不 canonicalize 测试 alias、不绕过安全断言。最新实现完整 Clippy `-D warnings` exit 0（42.69 秒），fmt check exit 0，详见上一阶段 `verify-synced-20261017/`，不以它们替代本次正式收据。

## 精确材料与审阅绑定

| 绑定 | 当前值 |
| --- | --- |
| repositoryId | `sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b` |
| Work Item | `WI-1085-legacy-v1-closeout-source-registration` |
| Contract | `sha256:7fea3f7d77255e61459ff1c51ac46ce9e9b73ff4a627939ad3b844d96ed093a1` |
| reviewedSourceHead | `007fdcba2e81d01285a694dce57cd78e0f74fcf4` |
| repositorySnapshot | `sha256:9a97106cf5ce0a70f760d4837d554570e1fd7792a3cce29280c32f054a77e21b` |
| sourceSnapshot | `sha256:d769eda16f0a78a672715212f5bb4820b4a443880eb197f5eabcc465e7852871` |
| materialManifest | `sha256:b17c12d60094a36f04cfd9ba4b5895dd7e92c38c5dcb361aa87eef771afc82cf` |
| requestDigest | `sha256:10f95e4ec0a84e26ab0ccd9f240c515217340b0bef949a92f144d3005a6bc663` |
| 原生材料审阅 receiptDigest | `sha256:8e9020154c5e308bfc9296b326a59408bdd3ca41ed22d4d4b4c96bc24c22f226` |

Raydot 的实际决定 `accept_permitted_unknowns` 已按原生流程记录，`reviewerActor=agent:Raydot`，`assurance=self_declared`。请求共有 36 entries、0 Finding、32 项不变。仅接受 `crates/cockpit-repository/src/legacy_closeout_source.rs` 及其同名 integration test 的 `readable_committed_rust_syntax_unknown`；不扩大接受范围，不复用旧 request88/旧收据。实际输入、转达、请求与 Runtime 返回均保存在本目录；完整 scanner/profile/policy 绑定见 `material-request.json`。

## 新源码本地正式验证

- 原生 attempt：`sha256:65fc3b515bee5d59944b2b9da3f26a40fd66425ec175ed025b41154dd5dba832`。
- canonical receiptDigest：`sha256:7cc40bfb22a726cbff4e98551c56951f7a283db0e41774d9c44992c2b82b695c`；canonical receipt 与 attempt receipt 相等。
- 实际执行 **13/13 通过，0 失败、0 复用、0 超时**；tool exit 0，单 worker、最大实际并发 1。总耗时 **876.971 秒**（执行 874.846 秒）。记录时刻 UTC `2026-10-16T18:44:14Z`，JST `2026-10-17 03:44:14`。
- 隔离 target；Tini subreaper + timeout TERM 890 秒/kill-after 10 秒，整体上限 900 秒。无旧 Python supervisor。正式运行前 13 包 no-run 准备成功但不计作测试。
- 正式前后可用空间 6,304,100,352 → 6,286,438,400 字节，保留 2 GiB 下限。旧 zombie 身份 1442 → 1442、无增减；本次 Tini root 已消失、session 无存活进程、观察到 1582 次回收、新增 OOM kill 0。真实观察错误不解释为空集合。
- Runtime 实际 binary SHA256 `0ffdcd20740f22b96621cf06adb5479d83fb435f503a26c18d824d61c4bfe9c4`，来自固定源码 `a12ee2ff4413ac261560339c9ed7af59dde0f701` 在本环境构建；不声称与旧安装 Runtime 原字节连续。

节点及各自退出码/超时/捕获限制见 `formal-result.json`。repository stdout 原生 `stdoutTruncated=true`，保留该事实，不重建未知尾部。55 条精选 Linux 成功行均来自实际保留的 process bytes，见 `scenarios.json`；它们不是独立签名单测收据。

## 实际 CI 与 Windows

[CI run 38946515596](https://github.com/xinglun/ai-cockpit/actions/runs/38946515596) 已终态 success，route、quality、v1-behavioral-oracle、windows-runtime 四 job 全部 success。source HEAD 为上述 `b5a219ea…`；实际测试 PR merge `a408ad078eb973e5254806254e44ba0d8102baae`，parents 为 `9e370e1c…` / `b5a219ea…`。GitHub merge tree 与本地 b5 tree 相等：`bb8cb5a5427ad8e0649dabfd555c7feb9f37df8c`。

- strict repository gates 共 42：**40 passed、2 not_applicable、0 reused**。不写成 42 个实际执行通过；N/A 原因保留在原始 gates JSON。
- workspace package coverage **13/13 实际覆盖、无遗漏**，由 coverage runner 执行；`executedByHostedRuntime=[]`，`verificationReceiptState=not_provided`。Hosted CI 与本地原生 Work Item receipt 分开。
- Windows job `117128843526`，repository `--lib` **136 passed / 0 failed / 0 ignored / 0 filtered out，856.20 秒**。随后 RPC/cognitive 步骤也成功。

四个本模块新增用例的真实成功行见 `ci-windows.log` 和 `windows-case-result.json`：

| 用例尾名 | 日志行 | UTC 成功时刻 |
| --- | ---: | --- |
| `publication_rejects_windows_reparse_parent_before_any_write` | 1376 | 18:29:39.1336756 |
| `history_reads_reject_windows_reparse_parent` | 1387 | 18:29:42.2984668 |
| `obligation_enumeration_fails_closed_on_windows_reparse_directories` | 1388 | 18:29:42.3645164 |
| `pinned_repository_root_rejects_windows_reparse_alias` | 1389 | 18:29:42.3887344 |

Windows revision-binding artifact `12068620575` 的发布 ZIP SHA256 为 `7cce766c416d81ed7fda278565130887baf64712968103238db3a78029ba149b`；ZIP entry 原字节保留为 `ci/windows-revision-binding.json`。其他 artifacts 的发布 ZIP SHA、成员字节及 SHA 见 `ci/artifact-origins.json`。GitHub app 返回的 job log 是已解码字符串，保存的日志不能冒称连接器解码之前的原始 transport bytes。

## A6、场景、意图和收益分别说明

- 11 个声明场景按有界测试及源码检查形成当前 agent:Codex 证据映射，17 条 acceptance evidence 已通过原生 controls 刷新。`scenarios.json`、`criteria.json`、`a6-matrix.json` 标明最终技术验收由 Raydot 判断。
- A6 的 Linux 实际执行证据包括路径/大小/枚举预算、source 身份、duplicate-key、no-replace victim、parent replacement、FIFO 与依赖义务等。精确 inventory absolute/traversal/duplicate path 形状的部分结论来自 `a6-source-guards.json` 的源码检查，**不声称这些形状另有动态执行用例**。
- Windows 四例支持 **directory junction/reparse 子边界**。其他 Windows reparse tag、末端 file symlink 变体为 **not_executed**；其他模块通过不替代本项证据，也不把四例扩大成穷尽保证。
- 意图 `resolved`：隔离 fixtures 和当前源码支持完整旧来源的现时显式登记、proof loss 阻断、合法 Retained→Deleted 追加及真实历史保留。它不替代最终验收。
- **用户可见收益 unknown**：降低错误信任 closed legacy / 错误释放 scope 的风险是推断，没有真实用户节时、采用率或生产事故减少数据。没有登记真实 WI-1004；全部本地锚点丢失/整体回滚仍不可识别，三语文档保留该界限。

## 当前原生状态与授权边界

`final-status.json`：checkpointed，verification verified，evidence fresh，blockers 0，humanDecisionRequired false；有效 unknown 仅 `user_visible_benefit_not_declared`。raw scanner Unknown 及其有限审阅收据仍可见。

`final-validation.json`：state/scenarioCoverage/acceptanceEvidence/evidenceClasses 为 verified，intentAlignment resolved，findings 0。Runtime 的 evidence projection 与绿色 Outcome 不是人工最终批准；`finish` 虽出现在 safeActions，本轮仍只交审。

原生 `outcome.zh-CN.txt` 已生成并返回。宿主是否单独显示该文件未知；最终消息提供独立可见 Outcome。原有真人决定保留，不伪造或复用到新绑定；本轮没有新的原生真人决定门禁。

剩余步骤是 Raydot 审阅本次完整材料与有界 A6，再由有权人明确决定后续生命周期。**不执行 Ready、merge、archive、close、额外清理或删除**；没有新实现/测试失败阻塞。

## 字节、历史和收尾诊断

- 旧失败、旧 head `9e1e4786` 的真实 13/13、旧 CI Windows 133/3、Clippy 失败和全部 attempt 均保留，不作为修复源码通过证据。`prior-native-byte-manifest.json` 与当前正式结果证明旧 attempt bytes 未变；旧 canonical 已另存历史副本，当前 canonical 由 Runtime 正常更新。
- Native executionRecords stdout/stderr 实为十六进制编码。当前 `streams/` 保存严格 hex-decoded process bytes，`streams.json` 保存长度/SHA/原生路径/截断事实并逐项 round-trip。最初序列化十六进制字符串保留在 `streams-hex/`；旧已提交 `verify-synced-20261017/streams/` 的序列化字符串原字节及哈希未改。详见 `stream-decoding-note.json`。这不改变实际测试结果。
- controls 输入中曾误把 repositorySnapshot9a 称为 SourceSnapshot9a。旧输入/返回保留，描述通过原生 controls 更正为 repositorySnapshot9a / sourceSnapshotd769，未改生成记录。
- 一次只读进程观察调用将 `--help` 解释为输出文件名，短暂生成仓库根目录非 `.ai` JSON，Runtime 正确报告 stale。该观察原字节移入本目录后，重新查询恢复原 snapshot/fresh；未跑 preflight 或测试，未绕过门禁。中间失败状态与原因见 `observer-output-relocation.json`。
- 除用户之前精确批准删除的旧 WI1086 debug 编译目录外，没有追加清理。旧环境、主 worktree 独有 WI1086 cleanup receipt、stash 及临时 Git 历史均保留。
- `byte-manifest.json` 是本次目录与关键原生记录的字节清单，排除自身以避免自引用。Report/assessment 是 agent 说明；原生 canonical、attempt、材料收据及 Summary 没有手改。
