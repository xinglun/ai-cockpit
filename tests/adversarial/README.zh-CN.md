# 对抗性验证面

v2 conformance corpus 包含 15 个语义案例，每种语言（英文、日文、中文）每个案例有五条 wording variant。
crate 集成测试要求所有 variant 产生相同的规范治理决定。Manifest 会绑定 RAI-01 到 RAI-12 的状态，
因此 `not_proven` 和 `partial` 边界不会被误当成通过。

conformance corpus 与 crate 集成测试覆盖范围越界、破坏性权限、缺失/过期/矛盾证据、
不支持的完成声明、仓库提示注入、恶意删除、跨 Work Item 证据、未知 provider 结果、
测试/覆盖率削弱、归档恢复、MCP 路径约束和验证工作目录约束。
