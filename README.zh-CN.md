# AI Cockpit

AI Cockpit 是面向 AI 辅助工程的新 Rust Repository Governance Runtime。
它是单一 binary、CLI-first，并提供本地 MCP adapter 和版本化 Repository Protocol。

本仓库不是 V1 的升级、迁移或 Rust 移植。V1 模板只作为规格来源、行为 Oracle、
conformance corpus 来源和历史证据参考。不会把 runtime 代码、Python 模块、
Makefile.ai、安装器或 runtime schemas 复制到对象工程。

Northbound 是 MCP 与 CLI，Southbound 是 Repository Protocol。Rust 治理核心必须
独立于 adapter，也必须独立于应用代码。

开始贡献前请先阅读[文档导航](docs/README.zh-CN.md)。

