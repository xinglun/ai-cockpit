# WI-1076 diagnostic integration snapshot

Date: 2026-10-11 UTC

This is a disposable diagnostic composition. It is not an AI Cockpit Runtime verification receipt and does not replace the preserved formal result.

## Inputs

- Repository: `xinglun/ai-cockpit`
- Main snapshot: `095e185b710b0c0016c655d3792f1b3438a62bac`
- WI-1076 source snapshot: `c601541cadb798aeef320c37d0ec36481e7165e9`
- WI-1070 approved documentation source: `b2402611267b5a8c65c15e834e462ea74d0b42ad` (parent `e8c5875aac423cfe3ea1a72052c768c6eba48643`)
- WI-1076 implementation under test: `c928907b44c4464f40cf5ac47089c32fa682a7fe`

The fixture began from exact WI-1076 source snapshot `c601541…`. It overlaid only six WI-1070 documentation files from `b240261…`: the three WI-1068 stable-release work-item pages and the three reference parity pages. For the parity pages, only the WI-1068 and WI-1070 rows from the approved documentation change were applied; the existing WI-1076 row was preserved. The two WI-1076 governance implementation/test files were copied at the committed `c928907…` content. No WI-1070 `.ai` governance state, other WI-1070 files, or other 45 source files were included. The resulting fixture had exactly eight modified paths.

## Paths and SHA-256

- `docs/work-items/WI-1068-v1-0-1-stable-release.md` — `d802626785371dedac49ef9b27730de5f638ef06e038317cef4286c987eb0771`
- `docs/work-items/WI-1068-v1-0-1-stable-release.zh-CN.md` — `8bc7f4c5c5a3dd4907c66fd61c414f93664ce90a3d5001216dcc40df22e1db6c`
- `docs/work-items/WI-1068-v1-0-1-stable-release.ja.md` — `49ad4561b67167e921ebdc7a7a00f2a6235070375fa776ce20ff5c94617efa29`
- `docs/reference/reference-parity.md` — `e7b0c47f8af0428a8afd89183365b3dae1dfd7ccfc5a4fa97d4e39c3122d33e0`
- `docs/reference/reference-parity.zh-CN.md` — `b8a660e60cc98c7622e2f27100f99885442ca350275fdf87ca225438f201069b`
- `docs/reference/reference-parity.ja.md` — `0f0a45767df6b2f0c9187b825ca6464ea99e588d44dcd98aed142cc1c8e4da65`
- `tests/ci/governance_integrity_gate.py` — `8b9a0bd18dfc51fa5f1cd011c57f0284ad4a9f9370645991b2c28ccafbed8c62`
- `tests/ci/governance_integrity_gate_test.sh` — `0726afd33efe76d6d4ea4e5a66eb765cd5508179ca84e2967f76a54140258ca2`

## Diagnostic commands

All commands exited 0 in the composed fixture:

- `bash tests/docs/work_item_status_consistency_test.sh`
- `PATH=/workspace/.cloud-tools/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH RUSTC=/workspace/.cloud-tools/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin/rustc RUSTDOC=/workspace/.cloud-tools/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin/rustdoc bash tests/docs/parity_status_check.sh`
- Same Rust environment with `bash tests/ci/governance_integrity_gate_test.sh`

The exact output and exit codes are in `WI-1076-snapshot-gates-20261011.log`.

## Formal Runtime result retained separately

The formal WI-1076 Runtime attempt `1e01298e816a220f9ecdf3fe95c485169d785ac2e42139c2307dfd35b8abdcc6` remains 2/3, with the status-consistency node blocked by the main snapshot's approved WI-1070 documentation dependency. This composed fixture is diagnostic evidence only; it does not make that Runtime attempt green and must not be described as a Runtime receipt.

## Work Item evidence ownership and exact source boundary

- WI-1076 Contract: `sha256:8e66ad0fe684897115710684f5b7bd026d07fabbed2a537c2a3e339316f3e5a1`; branch `codex/wi1076-governance-recovered-predecessor-20261011`; branch head `c928907b44c4464f40cf5ac47089c32fa682a7fe`. The repair commit changes only `tests/ci/governance_integrity_gate.py` and `tests/ci/governance_integrity_gate_test.sh`. WI-1076 owns those paths and its own projection/parity files and Runtime records.
- WI-1070 current branch head: `00a79fa629434ba2de194393798d1ec957bbcbc2`; selected documentation commit: `b2402611267b5a8c65c15e834e462ea74d0b42ad`. The six selected paths are the three `docs/work-items/WI-1068-v1-0-1-stable-release.*.md` pages and the three `docs/reference/reference-parity*.md` pages listed above. They are within the WI-1070 Contract at its current branch head. The diagnostic composition excludes the two WI-1070 `.ai` changes in `b240261…`, later WI-1070 diagnostic/evidence commits, and every other WI-1070 branch path.
- The failed Runtime attempts remain separate and immutable under `.ai/evidence/`; attempt `5f391c61…` is `execution_failed` with 0/3 nodes passed; attempt `1e01298e…` is `execution_failed` with 2/3 nodes passed (the status-consistency node failed on the main snapshot). The earlier `bec58b49…` precondition-rejected record remains in the branch history. The active WI-1076 Summary is preserved as generated; no attempt was reclassified as passed.

## Existing cross-Work-Item route checked read-only

The repository's `agents/skills/ordinary-work-item.md`, `docs/reference/agent-workflow.md`, and `docs/reference/cross-wi-coordination.md` define the supported route: each Work Item keeps its own Contract, branch, worktree, and PR; cross-Work-Item composition additionally needs compatible declarations, linked worktrees in one Git common directory, registration, a current slot lease, and fresh Runtime admission. `work-item coordination inspect` in this repository returned empty registrations, events, recoveries, requests, blockers, and unknowns. The disposable snapshot above was assembled manually and is not a Runtime composition or a candidate PR.

The existing selective-integration precedent is `docs/superpowers/plans/2026-10-03-v1-0-1-bounded-hotfix.md` (Task 1): port only reviewed source paths, exclude unrelated active governance records, and verify the exact integrated head. No PR or merge was created in this work.
