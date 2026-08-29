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
可运行 `regression_gate.sh <baseline.json> <candidate.json>`，拒绝缺失样本、零迭代、
identity 不一致和预算回归。该 gate 只消费已采集 evidence，不会构建源码 fallback。

Verification scheduler 还支持每个命令的 resource weight 和显式 resource budget。
weight 为零或超过预算时 fail-closed；依赖顺序、受保护节点和 receipt reuse 语义不变。
Repository context 和 Runtime session 都是 request-scoped，不创建进程级 current repository。

WI-395 的 Rust 原生优化移除了聚合 Work Item status 的重复 snapshot，在已有 Git 索引读取中捕获
source-tree 摘要，以一次受限查询解析远端默认元数据，并避免观察阶段反复递归排序。优化保持
request-scoped 和 identity-bound，不创建全局 repository cache，也不复制参考源安装流程。
