# Bootstrap Work Items

This repository has not installed AI Cockpit. It must not install the V1 template.
Until the Rust runtime can govern itself, every change is recorded in a Markdown
Work Item and reviewed by a human.

Each Work Item uses one branch, one base revision, one change scope, one evidence
bundle, and one outcome. A Work Item cannot claim completion from prose alone.

Required sections:

- Intent and Goal
- Scope and Out of Scope
- Sources and Unknowns
- Acceptance Criteria
- Required Evidence
- Base Revision
- Changed Files
- Verification
- Human Decisions
- Outcome

The canonical English file has `.zh-CN.md` and `.ja.md` semantic equivalents.
When a runtime behavior changes, all three language documents must be updated in
the same Work Item or the change remains incomplete.

## Bootstrap commands

Until WI-16 is complete, use ordinary Rust commands and record their output:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

After the self-governance cutover, V2 commands become the authoritative lifecycle
surface. No V1 `make ai-*` command is used in this repository.

