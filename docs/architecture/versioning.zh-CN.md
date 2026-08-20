# 版本策略

Runtime version 与 Repository Protocol version 独立。

```text
ai-cockpit 2.3.5
supports repositoryProtocol = 1

repository:
protocol_version = 1
```

Runtime 升级可以增加能力，同时继续支持 Protocol 1。只有 Protocol 1 → Protocol 2
才属于 repository migration。Runtime 启动时必须报告两个版本和 Runtime digest。

Protocol 兼容性必须显式处理：不支持的 major protocol 是 Red；当前 Runtime 缺少
可选能力则是 Yellow，并给出安全动作。历史 Work Item 永远保留决策边界使用的
Project Profile digest 和 protocol version。

