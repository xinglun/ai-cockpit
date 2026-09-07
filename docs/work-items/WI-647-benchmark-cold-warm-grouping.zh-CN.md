---
author: AI Cockpit maintainers
title: WI-647 — 基准测试冷/热分组正确性
description: 在任何运行时优化之前，先修复开发性能测量工具中的冷/热样本误分类问题。
workItemId: WI-647-benchmark-cold-warm-grouping
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-647-benchmark-cold-warm-grouping
terminalArchive: .ai/work-items/archive/WI-647-benchmark-cold-warm-grouping.contract.json
terminalVerification: .ai/evidence/WI-647-benchmark-cold-warm-grouping.verification.json
terminalFinalization: .ai/decisions/WI-647-benchmark-cold-warm-grouping.finalize.json
terminalDecision: .ai/decisions/WI-647-benchmark-cold-warm-grouping.close.json
---

# WI-647 — 基准测试冷/热分组正确性

本 Work Item 是 AI Cockpit 性能优化专项的 P0：在尝试任何运行时优化之前，先让开发用性能
测量工具 (`tests/performance/runtime_benchmark.sh`) 变得可信。本次改动只涉及测量工具本身，
不改变治理判定、证据语义或必需的 verification 图谱。

## 发现的缺陷

`runtime_benchmark.sh`（由 commit `acb3c386`、WI-402 引入）先对采集到的全部样本排序，
再把最小值当作 "cold" 报告：

```python
values.sort()
warm = values[1:]
...
{"name": f"{name}.cold", "elapsedMs": round(values[0], 3), "iterations": 1}
```

先排序再分组，会把第一次进程调用悄悄替换成后续任意一次最快的调用。以固定序列
`[120, 20, 22, 21]`（第一次慢、之后都快）为例，旧代码会把 `20` 报告为 cold，把
`[21, 22, 120]` 报告为 warm —— 真正的首次调用（`120`）没有被记为 cold，而是被
混入了 warm 样本集合。WI-402 自身的报告直接采信了该脚本的输出，从未质疑过排序顺序。

## 修复内容

- `tests/performance/runtime_benchmark_stats.py`（新增）：纯函数
  `summarize(name, raw_ms, prior_probe_calls)`。`raw_ms[0]` 始终作为 cold 报告；
  `raw_ms[1:]` 按原始调用顺序保留为 warm 样本（`rawMs`）；分位数计算只使用另行
  排序的副本。
- `tests/performance/runtime_benchmark_stats_test.py`（新增）：固定序列
  `[120, 20, 22, 21]`（cold 必须为 `120`）、一个反向异常值序列，以及下述可靠性下限的验证。
- 当 warm 样本数低于既定可靠性下限（`MIN_SAMPLES_FOR_P50=5`、
  `MIN_SAMPLES_FOR_P95=20`）时，`p50Ms`/`p95Ms` 会附带明确的 `insufficient_samples`
  原因被抑制，不会用过少的样本冒称分位数。报告的 `elapsedMs` 会回退到已观测到的
  最差样本，即使分位数不可靠，budget gate 依然能保持 fail-closed。
- 每次采集现在都会记录 `environment` 区块（硬件、操作系统、文件系统、仓库
  head/branch/dirty 状态、被跟踪文件数）以及 `preMeasurementProcessInvocations`
  列表。由于 `--version`/`inspect`/`status` 的身份探测在任何样本被测量之前就已运行，
  `.cold` 样本只是第一次被 *测量* 的调用，并非真正的 OS 冷缓存调用；脚本现在会
  明确说明这一点，而不是暗示缓存是全新的。
- `measurementModel` 记录本工具只测量独立 CLI 进程的延迟，不测量常驻 MCP 会话的延迟。
- `tests/performance/regression_gate.sh` 此前要求 `elapsedMs`/`maxElapsedMs` 必须是
  Python 的 `int`。而 `runtime_benchmark.sh` 的真实输出向来都是四舍五入后的 float，
  因此任何真实测量结果对任何 budget 文件都会以 `sample_malformed` 失败 —— 该 gate
  从未真正接受过真实测量证据。现已修复为接受 `int` 或 `float`（不含 `bool`），
  `iterations` 仍保持严格的 `int`。

## 不在本次范围内（推迟到后续 P0/P1 Work Item）

- 8 种场景的夹具矩阵（大小规模的干净仓库、单/多/大文件修改、大量历史 Work Item、
  并发 verification、常驻 MCP 重复查询）。
- 阶段级别的耗时拆解（git 查询/文件读取+哈希/证据校验/调度+子进程执行/outcome 投影）
  以及资源指标（读取/哈希字节数、git 调用次数、启动进程数、缓存失效原因、峰值内存）
  在 `status`/`doctor`/`observe` 上的暴露 —— 目前只有 `inspect` 会报告
  `filesRead`/`filesHashed`/`gitCalls`，没有任何命令报告进程数或峰值内存。
- 一个允许显式绑定的 baseline/candidate Runtime 身份不同、但仍验证各自证据完整性的
  开发专用比较器。
- 常驻 MCP 会话延迟测量。
- 任何 P1–P3 运行时优化（request-scoped 去重、`IncrementalMerkle`、
  `PhysicalSingleFlightCoordinator`、轮询等待的改动、常驻缓存、PGO）。

## 验证

`cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets
--all-features -- -D warnings`、`cargo test --locked --workspace` 均无变化地通过
（未改动任何 Rust 源码）。`python3 tests/performance/runtime_benchmark_stats_test.py`
与 `bash tests/performance/regression_gate_test.sh` 均通过。针对已安装的 v0.2.87
二进制在本仓库运行了 `runtime_benchmark.sh`，并将其 JSON 输出与真实的（float）
budget 文件一起送入 `regression_gate.sh`，在通过与预算超限两种情形下都确认了该
gate 现在既能消费真实测量证据，又能在真正出现回归时保持 fail-closed。

### 本地测量（参考信息）

2026-09-07，macOS arm64（Darwin 25.6.0，arm64，10 个逻辑 CPU），针对本仓库
（已归档 580 个 Work Item，追踪文件 9614 个，HEAD `1623ee5a` 处于干净状态），
以 6 次迭代记录如下（括号外为新分组，旧脚本的错误分组见正文说明）：
`inspect.cold` 98.951 ms、`status.cold` 1887.792 ms、`doctor.cold` 50.172 ms、
`observe.cold` 119.701 ms。`status` 明显比 `inspect`/`doctor`/`observe` 更昂贵，
这是留给下一个 P0 瓶颈排序 Work Item 的原始证据，并非本 Work Item 已对其做了优化的声明。
以上均为本地进程延迟观测，不构成 provider 或 enterprise 层面的保证。
