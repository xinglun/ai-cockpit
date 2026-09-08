---
author: AI Cockpit maintainers
title: 性能优化专项总结(2026-09)
description: 测量并修复了什么、测量后有意不做什么、还有什么尚未测量。
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-651-performance-initiative-synthesis
---

# 性能优化专项总结(2026-09)

本文档为 2026-09 的 AI Cockpit 性能优化专项收尾。目的是把测量并改动了什么、
测量后*有意*没有改动什么(附带该决定的证据)，以及还有什么尚未测量，集中记录
在一处，让未来的贡献者从已验证的事实出发，而不必重新推导测量结果或猜测优先级。

North Star：Calibrated Human-Agent Trust。以下所有改动都保持了治理判定、仓库
隔离、证据有效性和恢复行为；没有任何一项削弱了必需检查，也没有任何性能主张是
在缺少实测前后对比的情况下做出的。

## 已交付的 Work Item

### WI-647 — 基准测试冷/热分组正确性(P0)

`tests/performance/runtime_benchmark.sh` 先对全部计时样本排序再把最小值当作
"cold"，导致第一次进程调用被后续任意一次最快调用悄悄替换。修复为先按原始调用
顺序分组（`tests/performance/runtime_benchmark_stats.py`），并用固定序列测试
（`[120, 20, 22, 21]` → cold 必须为 `120`）加以固定。同时在样本数低于既定下限时
抑制不可靠的 `p50`/`p95` 主张，记录测量环境与测量前身份探测的披露，并修复了
`regression_gate.sh` 仅接受 `int` 的类型检查（此前该 gate 从未接受过本工具产生
的任何真实 float 型 `elapsedMs` 测量值）。未改动任何 Rust 源码。见
`docs/work-items/WI-647-benchmark-cold-warm-grouping.md`。

### WI-648 — status 瓶颈诊断(P0)

使用修复后的工具测量，本仓库上 `status` 耗时约 1.8 秒（`inspect`/`doctor`/
`observe` 均低于 110ms）。通过临时的、从未提交的剖析手段，将根本原因定位到
`historical_finalization_inventory`
（`crates/cockpit-repository/src/lib.rs:3832`）为每一条历史
`.ai/decisions/*.finalize.json` receipt 都调用一次
`resolve_resource_finalization_head`，而该函数本身每次都会重新扫描整个
`.ai/decisions` 目录——成本是 O(decisions 条目数 × 历史 receipt 数)，而非
O(条目数)。仅做诊断，未改动代码。见
`docs/work-items/WI-648-status-bottleneck-diagnosis.md`。

### WI-649 — status 历史扫描修复(P1)

修复了 WI-648 发现的瓶颈：`historical_finalization_inventory` 现在只读取一次
`.ai/decisions`，并把每个 work item 预先分组好的 transition 候选传给新增的
`resolve_resource_finalization_head_with_candidates`，不再逐条重新扫描。原有的
`resolve_resource_finalization_head`（供 `finalize`/`finalize-verify`/
`finalize-recovery-plan` 使用）未改动。通过逐字节验证 `status` JSON 输出完全
一致（383 条历史 receipt，仅 `runtimeDigest` 因二进制变化而不同），以及新增的
回归测试证明不同 work item 之间候选不会混淆。测量结果：两组独立测量中
`status` 的 cold 与 warm 均改善约 20–23%；剩余约 1.24 秒的
`historical_finalization_inventory` 成本（从约 1.7 秒降下来）花在逐条 receipt
的处理上（`closed_finalization_projection_kind`、`archived_contract_digest`、
transition 文件读取），本 Work Item 未涉及。见
`docs/work-items/WI-649-status-history-scan-fix.md`。

### WI-650 — 验证子进程改为阻塞等待(P1)

`execute_captured` 此前用 `child.try_wait()` + `sleep(10ms)` 循环等待每一个
子进程。一个独立的微基准测试（在 CLI 之外测量以消除无关噪声）表明，这为近乎
瞬时完成的命令额外增加了约 11ms 的纯等待（均值 12.19ms 对 1.19ms），对耗时
数秒的命令则没有可测量的差异。仅在 Unix 上修复：子进程 move 进一个专用线程，
由其阻塞在 `child.wait()` 上并通过 channel 汇报，调用方只做一次有界的
`recv_timeout`。Windows 逐字未改动，且明确未经验证（没有可用的 Windows
环境）。通过 `ai-cockpit verify --command true` 做的端到端测量：baseline 约
104–108ms，candidate 约 94–98ms。见
`docs/work-items/WI-650-verification-wait-blocking.md`。

## 评估过但未采纳的候选方案及其证据

### P1 — 大文件流式哈希 / 有界并行读取

未采纳。在本仓库上运行 `ai-cockpit inspect --repo`，无论仓库有 9614 个被跟踪
文件，报告的都是 `filesRead: 2, filesHashed: 2`——WI-395 此前的优化已经让这一步
是 request-scoped 且与仓库大小无关的，而非全树遍历。本仓库中最大的被跟踪文件约
2.1MB（`tests/conformance/reference_file_inventory.json`），若干
adopter-acceptance 清单文件约 1.3–1.6MB。这个量级的整文件读取在本地
SSD/APFS 上是亚毫秒级的。本仓库没有任何实测证据表明文件数量或文件大小是
P0/P1 工作中发现的任何延迟的驱动因素——唯一被测量到的瓶颈（`status` 的约 1.8
秒）在于目录条目的重复扫描和逐条 receipt 的解析逻辑，而不是文件 I/O 的量。
现在实现流式哈希或有界并行读取，只会在没有实测收益的情况下增加并发控制的
复杂度（有界文件句柄、确定性的输出顺序）。只有当未来某个仓库或场景（例如尚未
构建的 P0"大文件修改"场景）的测量显示出不同结果时，才应该重新考虑。

### P2 — 请求/验证去重(`PhysicalSingleFlightCoordinator`)

未接入。直接查阅源码
（`crates/cockpit-verification/src/lib.rs:1473`）确认
`PhysicalSingleFlightCoordinator` 除了自己的测试文件
（`crates/cockpit-verification/tests/physical_execution.rs`）之外，在
`cockpit-cli`、`cockpit-mcp`、`cockpit-agent`、`cockpit-core` 中都没有调用
者——它已被实现和测试，但没有接入任何命令路径。按照本专项自身的验收标准，
接入它需要完整的执行身份匹配、按 Work Item 的授权与证据绑定验证，以及对
必须保持 fresh 的检查的显式 fail-closed 行为——这是一次带有真实仓库隔离与
授权风险的、不小的改动。本专项的任何测量都没有捕捉到具体的并发重复成本：
一次 3 路并发的 `status`（一个只读、与验证无关的命令）运行显示的 wall time
符合 I/O 竞争的特征，而非 verification 执行去重的机会；也没有测量过针对
同一 Work Item/命令的并发 *verification* 场景。缺少这样的测量，现在接入该
协调器将是一次投机性的架构改动，而非实测驱动的优化。重新考虑此事的具体前提，
是先构建并测量尚未构建的 P0 场景"多个并发验证请求"（同一仓库、Work Item、
命令），并记录下重复执行的次数。

### P2 — `IncrementalMerkle` 缓存可信度

未采纳，目前也不构成风险。直接查阅源码
（`crates/cockpit-git/src/lib.rs:18`）确认 `IncrementalMerkle` 除了自己的
测试文件（`crates/cockpit-git/tests/snapshot.rs`）之外没有调用者，没有接入
`GitRepository::snapshot()` 或任何其他生产路径。由于目前生产环境中没有任何
地方信任这个类型缓存的摘要，本专项要求核实的那些缓存有效性风险（恢复了
mtime 的同长度编辑、文件替换/删除/重命名/类型变化、仓库根变化、读取期间的
并发修改、watcher 事件丢失）今天都不适用——没有任何受保护的判定依赖它的
输出。如果未来某个 Work Item 要接入它，必须先建立这些有效性保证（或者在
无法证明时明确重新读取或返回 unknown），然后才能做任何性能上的主张；本文档
有意不预先授权这种接入。

### P3 — 常驻 MCP 仓库绑定缓存、进程内 Git、PGO

未采纳。每个 P3 候选都以"只有先前测量证明确有必要"为前提。P0–P2 阶段的任何
测量都没有指向这三者中任何一个能解决的瓶颈：已确认的瓶颈
（`historical_finalization_inventory` 的目录重扫）是作为纯算法改动
（WI-649）被修复的，并不需要常驻缓存、不同的 Git 实现，或 profile-guided
编译。现在推进其中任何一项都是没有目标的优化，这正是本专项自身的基本原则
（先测量、不假设倍数）明确排除的做法。

## 已知缺口(未被悄悄丢弃)

以下 P0 范围内的项目在本专项中未完成，留待未来以测量优先方式推进的 Work
Item：

- 8 种场景的夹具矩阵（大小规模的干净仓库、单/多/大文件修改、大量历史 Work
  Item、并发 verification、常驻 MCP 反复查询）——只有"大量历史 Work Item"这一
  场景通过本仓库自身的真实历史得到了实质性的验证。
- 在 `status`/`doctor`/`observe`/`diagnose`/`work-item status` 上暴露阶段级别的
  资源指标（读取/哈希字节数、git 调用次数、启动进程数、缓存失效原因、峰值
  内存）——目前只有 `inspect` 暴露 `filesRead`/`filesHashed`/`gitCalls`。
- 常驻 MCP 会话延迟测量——修复后的工具（WI-647）明确只测量独立 CLI 进程的
  延迟。
- 一个允许显式绑定的 baseline/candidate Runtime 身份不同、但仍验证各自证据
  完整性的开发专用比较器——`regression_gate.sh` 仍然要求 baseline 与
  candidate 的 `runtimeVersion`/`runtimeDigest` 完全一致，这对发布验收而言是
  正确的，但对比较开发身份绑定不同的构建而言仍是一个未解决的缺口。

以上每一项都是可以独立立项的正当 Work Item；之所以没有在这里实现，是因为
都还没有被测量过，这与本专项"先测量再修改"的纪律是一致的。
