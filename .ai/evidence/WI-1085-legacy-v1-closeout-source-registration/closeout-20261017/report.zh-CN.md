# WI-1085 原生 finish/archive 与真人门禁交接

Outcome: 🟡 已完成原生 finish/archive，等待真人 close 决定及 PR 集成授权。

## 已完成与当前状态

- 实现仍固定 `007fdcba2e81d01285a694dce57cd78e0f74fcf4`；本轮实际已通过 CI 的证据头为 `e2d32e20410e34a1ad73e39e087daaa614689fb0`。本轮没有修改产品、断言、CI 或三语文档，没有重新运行正式测试。
- Raydot 的最终技术审阅已实际通过，精确绑定 attempt65fc / canonical7cc、材料 request10f / receipt8e，55 条 Linux 引用及四个 Windows case。通过原生 `controls` 为17条 acceptance evidence 追加真实审阅 relay，未改写真人 `decisionEvidence`。
- 当前头 [CI38951742695](https://github.com/xinglun/ai-cockpit/actions/runs/38951742695) 已 completed/success，四个 job 全部成功。等待其终态后才执行生命周期写入。
- 原生 `finish` 成功：`2026-10-16T19:28:39.577922800Z`，状态 finish_ready；随后刷新准入得到 archive_when_reviewed，再原生 `archive` 成功：`2026-10-16T19:29:05Z`。
- 最新 status：**archived、verification verified、evidence fresh**；validator state/scenarioCoverage/acceptanceEvidence/evidenceClasses verified，intentAlignment resolved，findings 0。
- 唯一 blocker：`archived_work_item_pending_close`；safeActions 为 close_after_review、record_usage、refresh_status。Status 的 `humanDecisionRequired=false` 属于 preflight/amendment 投影，**不等于已经作出 close 真人决定**；close CLI 明确要求 `--human-decision`，本轮授权没有覆盖该决定。

## 精确绑定与停止点

[close-authority-binding.json](close-authority-binding.json) 是一次完整的代理准备绑定包，不是已提交的真人请求回执或批准：decision/actor/decidedAt/authoritySource 仍为空，未调用 close。

| 字段 | 值 |
| --- | --- |
| repositoryId | `sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b` |
| Work Item | `WI-1085-legacy-v1-closeout-source-registration` |
| Contract | `sha256:7fea3f7d77255e61459ff1c51ac46ce9e9b73ff4a627939ad3b844d96ed093a1` |
| repositorySnapshot | `sha256:9a97106cf5ce0a70f760d4837d554570e1fd7792a3cce29280c32f054a77e21b` |
| sourceSnapshot | `sha256:d769eda16f0a78a672715212f5bb4820b4a443880eb197f5eabcc465e7852871` |
| requestDigest | `sha256:10f95e4ec0a84e26ab0ccd9f240c515217340b0bef949a92f144d3005a6bc663` |
| materialManifest | `sha256:b17c12d60094a36f04cfd9ba4b5895dd7e92c38c5dcb361aa87eef771afc82cf` |
| material review receipt | `sha256:8e9020154c5e308bfc9296b326a59408bdd3ca41ed22d4d4b4c96bc24c22f226` |
| native attempt | `sha256:65fc3b515bee5d59944b2b9da3f26a40fd66425ec175ed025b41154dd5dba832` |
| native canonical receipt | `sha256:7cc40bfb22a726cbff4e98551c56951f7a283db0e41774d9c44992c2b82b695c` |
| archive manifest raw SHA256 | `sha256:d0ccf2dd20e277c1b6563437154e56e04d8e355447eb40370f07a04059c704a8` |

Archive manifest 逐项绑定六份原生 artifact 的准确 bytes/SHA，已核对 6/6；Contract 字节保持，Summary/Outcome/事件/报告的生成与 active→archive 引用处理全部由 Runtime 执行，没有手改。正式 canonical 原 bytes SHA256 仍为 `ae39c0275aa25afef028e9b444d1b43a6346e923e1493e7513672e1378a57d19`。旧验证及失败历史继续保留；上一阶段“等待 Raydot”评估是当时记录，本目录的真实最终审阅 relay 是后续结论。

下一步由控制线程向真人呈现该精确 close 绑定并取得实际决定；执行时重新核对绑定。Ready、merge、删除及 Issue close 都另有授权边界，不能从技术通过、safeActions 或绿色验证推导。PR #1044 继续 Draft。工作分支/worktree 与旧环境均保留，本轮没有 cleanup。

## 当前头 CI 的真实证据

- source `e2d32e20410e34a1ad73e39e087daaa614689fb0`；实际 tested PR merge `b5af84c1c219c8ff7160ac2aa342a358829d7a7b`；base `9e370e1c2c2e2dceda79de62ca4e849ae313624c`。tested merge tree 与 e2 source tree 相等，见 `ci/tested-commit.json`。
- strict gates **40 passed + 2 not_applicable、0 reused**；workspace coverage runner 实际13/13，无遗漏；hosted native Work Item receipt 为 not_provided。单独的本地原生正式 receipt 仍是 13/13、0复用/超时、876971ms 的65fc/7cc，未被 hosted CI 代替或重跑。
- Windows job `117146452068` 完整成功；repository --lib **136 passed、0 failed、0 ignored、0 filtered out，762.31 秒**。四个本模块新增 case 实际 `... ok` 行均位于 `19:10:13Z`，精确全名/行号见 `windows-case-result.json` 与 `ci-windows.log`。
- Windows revision-binding artifact `12070382109` 发布 ZIP SHA256 `abdc9ca32cdeca2485c578050435b2f4f58b51aa4091d9c877c360d1f21d74aa`。四个 artifacts 的 ZIP SHA 与提取成员 SHA 见 `ci/artifact-origins.json`，原成员字节保留。
- job log 保存的是官方连接器返回已解码字符串的 UTF-8 bytes，不声称是连接器解码之前 transport bytes。当前头之后本次正常 lifecycle/evidence 提交若触发自然 CI，会单独报告它的 head/run 状态，不把 e2 通过说成新 SHA 通过，不做纯证据循环推送。

## 已接受的有界结论及未知

Raydot 最终接受声明范围内 A6；Linux55条实际引用、Windows四个 directory-junction/reparse case 与显式静态 guards 支持该有界映射。其他 Windows reparse tag、末端 file symlink 未执行，不宣称穷尽覆盖；其他模块通过不替代本模块证据。意图 resolved；用户可见收益 unknown，风险下降是推断。真实 WI1004 没有登记；所有本地锚点一同丢失/整体回滚仍不可识别。

## Outcome 交付

`archive.json` 包含完整原生 Outcome、returned segments 与 delivery report，`outcome.zh-CN.txt` 是后续完整原生查询。实际 hostDeliveryMode 为 full_handoff_only，hostDisplayConfirmation unknown，deliveryReport.complete=true 但 host acceptance/display unknown；不能说用户已经看见或批准了归档消息。最终可见交接另行提供本轮黄色 Outcome。

## 执行环境与观察边界

控制线程报告过执行环境断连通知。本轮随后真实执行的 filesystem、git、Runtime status 和 GitHub API 均有返回，实际 HEAD/Runtime hash/Contract/snapshot 不变，原生继续准入；这些观测不证明宿主或会话从未中断，也没有冒称环境恢复流程已执行。

第一个 GH watcher 实际 exit1，HTTP401 Bad credentials；新发起的只读 gh api 成功，第二个 watcher exit0，终态另外由 API 核对。未修改全局凭据、未登录或重装。正式运行结束时旧 zombie1442→1442 的记录不改；此后另观察到一个 PPID1 `git` zombie（PID2056860/startTicks5141612），归属和原因未确认，见 `disconnect-observation.json`。没有因此另建工具项目或扩大清理，也没有新本地测试。最终进程/磁盘读数在 `handoff-processes.json`，观察完整性与真实错误保留。

## Sentinel #1004 草案

[sentinel-1004-reply.DRAFT.zh-CN.md](sentinel-1004-reply.DRAFT.zh-CN.md) 已准备，**未发送**，交控制线程按用户授权发送。草案引用原验收、Deleted digest 补正与四份 v1/43文件盘点，清楚区分历史用户观察、当前云端 fixtures、完整真实来源登记尚未执行和候选复核建议；Issue 保持 OPEN，不自动关闭。它不把治理 Runtime 0ffd 的 hash 当作候选新实现 Runtime hash。
