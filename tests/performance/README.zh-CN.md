# 性能验收夹具

运行 `cargo test -p cockpit-cli --test performance -- --nocapture`，测量温热
`status` 启动和中型仓库观察。测试输出会记录样本数、中位启动时间、读取文件数和
观察耗时。

knowledge crate 还包含 10,000 条记录的无关依赖查询：断言
`historical records accessed = 0`。有界验证回执记录
`nodesPlanned`、`nodesExecuted`、`nodesReused`、`gitCalls`、`filesRead`、
`filesHashed`、`processesSpawned` 和 `elapsedMs`。

<50 ms 的 status 与 <100 ms 的增量观察是发布目标，不是无证据的声明。发布门禁必须
把目标平台的实际 benchmark 输出附加到证据包。

Runtime 提供带 identity 的 `PerformanceBaseline` 记录，必须包含
`runtimeVersion`、`runtimeDigest`、`repositoryId`、采集时间、样本和明确预算。
旧 schema 1 夹具使用 `regression_gate.sh <baseline.json> <candidate.json>`，拒绝缺失样本、
零迭代、identity 不一致和预算回归。该 gate 只消费已采集 evidence，不会构建源码 fallback。
schema 2 是由 `p0_regression_gate.sh` 单独检查的 P0 evidence 契约：baseline 和 candidate 的
Runtime 版本/摘要可以不同，但 repository identity、repository snapshot、证据完整性和可比较环境必须绑定。
预算必须明确指定 `warm.p50Ms`、`warm.p95Ms` 或 `warm.p99Ms`；不可靠分位数直接导致 gate 失败，不能回退到
`elapsedMs` 或其他分位数。

Verification scheduler 还支持每个命令的 resource weight 和显式 resource budget。
weight 为零或超过预算时 fail-closed；依赖顺序、受保护节点和 receipt reuse 语义不变。
Repository context 和 Runtime session 都是 request-scoped，不创建进程级 current repository。

WI-395 的 Rust 原生优化移除了聚合 Work Item status 的重复 snapshot，在已有 Git 索引读取中捕获
source-tree 摘要，以一次受限查询解析远端默认元数据，并避免观察阶段反复递归排序。优化保持
request-scoped 和 identity-bound，不创建全局 repository cache，也不复制参考源安装流程。

便携脚本 `runtime_benchmark.sh <binary> <repo> <output.json> [iterations]
[work-item-id] [budgets.json]` 输出 schema 2 的端到端进程墙钟证据。它保留采样顺序，区分首次测量的
独立 CLI 进程、一次有界 OS 缓存预热后的独立 CLI 进程，并明确标记常驻 MCP 未测量。每条记录保留原始
样本、预热次数、样本数、分位数方法、Runtime/仓库身份、仓库状态、数据规模、场景矩阵状态、阶段边界、
缓存失效原因和资源指标。Runtime 或平台无法可靠提供的实际读取字节、哈希字节、Git 调用数、子进程数和
峰值内存会标记为不可用，绝不填零。脚本要求外部可执行普通文件，记录 Runtime 报告的身份和文件 SHA-256，
原子写出结果，绝不构建或回退到源码。单次迭代 sanity run 不得用于 p95/p99 声明；发布 gate 必须再用
经过明确审查的 budget 文件调用 `p0_regression_gate.sh`；旧 schema 1 夹具继续调用 `regression_gate.sh`。

P0 场景矩阵包含小型/大量文件的干净仓库、单文件/多文件/大文件修改、大量历史 Work Item、多个并发验证
请求以及常驻 MCP 重复查询。一次脚本调用绑定所提供的仓库，未选择的场景报告为 `not_measured`，不会
伪造结果。只有仓库事实证明场景形状时才会标记为已测量：`small-clean` 要求干净且最多 100 个 tracked
文件，`many-files-clean` 要求干净且至少 1,000 个 tracked 文件，`single-file-change`/`multi-file-change`
要求恰好一个/至少两个变更路径，`large-file-change` 要求存在至少 1 MiB 的变更文件，
`many-historical-wi` 要求至少 100 个归档 Work Item。便携脚本不执行并发请求或常驻 MCP 传输，因此这两个
场景继续明确记录为 `not_measured`。
