# v1.0.1 Bounded Hotfix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Deliver WI-1052 and #1004 repairs on current main, with honest Mac arm64 prerelease acceptance.

**Architecture:** Selectively port product changes from the frozen candidate into the clean WI-1055 branch. Keep historical WI records immutable, and separate real test/CI evidence from Runtime lifecycle status.

**Tech Stack:** Rust workspace, Cargo, AI Cockpit Runtime CLI, GitHub Actions, macOS arm64.

**Spec:** `docs/work-items/WI-1055-v1-0-1-bounded-hotfix.md`

## Global Constraints

- Base `78ae7240aac016f005fcf4ced61fcb251ea20eb1`; old candidate `befdbd11d60af5c01d40ec62497aa9c8b46ba0ce` is reference only.
- Do not copy WI-1053 active governance files or claim WI-1055 Runtime admission.
- Do not overwrite `v1.0.1-rc.1`; release only a fresh unoccupied version.

## Review Focus

- A pending sensitive amendment: request can be generated, verification cannot spawn before authentic review.
- Tampered or mismatched historical closeout: recovery changes no destination bytes.
- Interrupted recovery: rollback removes only files installed by that attempt.
- Sentinel formatter after recovery: exact scope first; legitimate historical files must not become formatter product edits, while unrelated dirty files still block.
- New head after main integration: no old candidate test receipt is reused as current proof.

---

### Task 1: Selective source integration

**Files:** `crates/cockpit-repository/src/lib.rs`, `crates/cockpit-repository/src/lifecycle.rs`, CLI/MCP entrypoints and their focused tests.

- [ ] Map each selected hunk to WI-1052 or #1004; exclude all old active WI records.
- [ ] Port WI-1052 code and negative process-spawn test; run `cargo test --locked -p cockpit-cli --test contract_amendment_processes`.
- [ ] Port #1004 planner/recovery, CLI/MCP adapters, and closeout tests; run the three `closeout_recovery` test targets.
- [ ] Add the downstream formatter-after-recovery regression only if exact-scope Sentinel replay shows a remaining product defect.
- [ ] Commit separately reviewable WI-1052 and #1004 changes.

### Task 2: Documentation and exact-head verification

**Files:** `agents/skills/ordinary-work-item.md`, `docs/reference/{commands,agent-workflow,reference-parity}*.md`, current WI documentation.

- [ ] Update command and workflow guidance in English, Japanese, and Chinese to match the selected code.
- [ ] Run focused tests, `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, and `cargo test --locked --workspace` against the new source head.
- [ ] Obtain independent review and PR CI bound to the exact pushed commit; resolve actual failures without weakening gates.

### Task 3: Mac prerelease acceptance

**Files:** `Cargo.toml`, `Cargo.lock`, release notes and immutable build assets.

- [ ] Recheck that `v1.0.1-rc.2` tag and Release are unused; if reserved, select the next free semantic prerelease.
- [ ] Build Mac arm64 asset from the exact merged head, verify downloaded checksum, install, and run `doctor`.
- [ ] Publish only the proven prerelease and disclose the bootstrap exception, Mac-only proof, unresolved governance debt, and Task9 deferral.
- [ ] Preserve release, CI, install, rollback, and non-admitted WI status evidence for later reconciliation.
