# AI Cockpit

AI Cockpit は、AI 支援開発のための新しい Rust 製 Repository Governance Runtime
です。単一バイナリ、CLI-first、ローカル MCP adapter、バージョン付き Repository
Protocol を提供します。

このリポジトリは V1 のアップグレード、移行、または Rust port ではありません。
V1 template は仕様ソース、behavioral oracle、conformance corpus のソース、
過去の evidence 参照としてのみ使います。runtime code、Python module、
Makefile.ai、installer、runtime schema を対象 repository にコピーしません。

Northbound は MCP と CLI、Southbound は Repository Protocol です。Rust の
governance core は adapter と application code のどちらからも独立します。

貢献する前に[ドキュメントマップ](docs/README.ja.md)を読んでください。

