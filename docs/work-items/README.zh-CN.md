# Bootstrap Work Item

本仓库尚未安装 AI Cockpit，也不能安装 V1 template。在 Rust runtime 能够治理自身
之前，每个变更都记录在 Markdown Work Item 中，并由人审查。

每个 Work Item 使用一个分支、一个 base revision、一个变更 scope、一个 evidence
bundle 和一个 outcome。不能只靠文字声称完成。

必需章节：

- Intent 与 Goal
- Scope 与 Out of Scope
- Sources 与 Unknowns
- Acceptance Criteria
- Required Evidence
- Base Revision
- Changed Files
- Verification
- Human Decisions
- Outcome

英文 canonical 文件必须有 `.zh-CN.md` 和 `.ja.md` 语义等价版本。Runtime 行为变化
时，三个语言文档必须在同一个 Work Item 中同步更新，否则变更不算完成。

WI-16 完成前使用普通 Rust 命令并记录输出；不使用 V1 的 `make ai-*` 命令。

