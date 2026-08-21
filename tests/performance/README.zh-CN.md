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
