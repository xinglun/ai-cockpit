# 文档导航

英文文档是面向机器的术语规范；中文和日文文档必须保持语义等价，不能只是摘要。

## 从这里开始

- [产品边界](architecture/product-boundary.zh-CN.md)
- [Runtime 拓扑](architecture/runtime-topology.zh-CN.md)
- [版本策略](architecture/versioning.zh-CN.md)
- [Bootstrap Work Item 规则](work-items/README.zh-CN.md)
- [Repository Protocol v1](protocol/v1/specification.zh-CN.md)
- [性能验收](../tests/performance/README.zh-CN.md)
- [实测性能基线](performance/baseline.zh-CN.md)
- [发布与分发](release/distribution.zh-CN.md)
- [对抗性验证](security/adversarial-validation.zh-CN.md)
- [Work Item 路线](work-items/WI-03.zh-CN.md)

## 开发顺序

1. 冻结语义和协议。
2. 构建纯治理核心。
3. 对 repository 一次观察，并复用 immutable snapshot。
4. 加入 verification、生命周期写入、knowledge、attach 和 MCP。
5. 证明 conformance、性能、对抗行为和 thin-repository 使用。

WI-03 至 WI-24 记录当前实现状态。仍为 partial 的条目在证据门禁完成前不会进入 GA。

在 Rust runtime 能够治理自身之前，开发使用 `docs/work-items` 中的 Markdown
bootstrap 规则。本仓库永远不会安装 V1。
