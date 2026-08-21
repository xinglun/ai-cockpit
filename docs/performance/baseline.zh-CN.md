# 性能基线（本地证据）

使用 `cargo test -p cockpit-cli --test performance -- --nocapture`，于 2026-08-21 在开发
工作区采集：

| 面 | 夹具 | 结果 |
| --- | --- | --- |
| `status` 温热启动 | 12 个样本 | 中位 2 ms |
| repository observation（增量缓存命中） | 200 个生成文件，读取/哈希 406 个文件 | 38 ms |
| knowledge 无关查询 | 10,000 条记录 | 访问历史记录 0 条 |

本次 status 目标（<50 ms）和增量 observation 目标（<100 ms）均达成。首次未缓存扫描会单独
测量；验收目标适用于增量缓存命中路径。这些数字是本机证据，不是普遍保证。
