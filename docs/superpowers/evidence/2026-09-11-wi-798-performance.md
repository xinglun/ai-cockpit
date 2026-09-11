# WI-798 performance evidence — 2026-09-11

This record is local evidence for `WI-798-collaboration-observability-performance`.
It is not a release or Work Item completion receipt.

## Frozen identity

- Integration base: `c72254c2ae6bc712139fc755698694653ae004ba`
- Worktree branch: `codex/wi-798-collab-observability-performance`
- Runtime version: `0.2.90`
- Current integration binary digest: `sha256:520188172dbc5ddbb70990315f431ce3c596429667874d8b6a14d012cc89cd48`
- Host: macOS 26.6.2 arm64; Rust/Cargo 1.94.1; Python 3.14.4
- Object project: `/Users/sei-rinn/dev/workspace_rust/sentinel`
- Object project source: `develop` at `6516bc281cf882e3dbce3b6cbf48287c0fda0cde`; source checkout remained clean
- Object project repository identity: `sha256:77b40b1960f2a5251724cfa3b5591e58f15e1439153e8e6cb2610232a709f350`

## P0 isolation scanner

Fixture: 160 regular files in 20 directories, 3 symlinks (valid, dangling,
and escaping), with the output outside the fixture. The old Shell helper and
the Rust scanner produced the same 183-line manifest with digest
`2cd0590a228d00532b5d339defbae50eadccc95a5ad54d99cb140b532d519ea1`.

| implementation | runs (seconds) | p50 | trace lines | selected external-command patterns |
|---|---:|---:|---:|---:|
| Shell helper | 3.63, 3.90, 3.41 | 3.63 | 4,325 | 871 |
| Rust scanner | 0.05, 0.01, 0.01 | 0.01 | 7 | 1 |

The representative scanner median is 99.7% lower and the three-run maximum
is 98.7% lower. This is a scanner-phase result, not a claim about the full
release critical path. The official isolation manifest regression suite also
passed with `AI_COCKPIT_ISOLATION_BIN` bound to the current Rust binary.

## 100-warm-sample measurements

Each scenario ran 100 valid warm samples for each of `inspect`, `status`,
`doctor`, `observe`, and `diagnose`. The benchmark retained first samples and
raw order. Each scenario accounted for 513 external benchmark processes:
505 measured command processes, 5 warmups, 3 probes, and 4 metadata Git
processes. Runtime-internal child-process counters remain explicitly
unavailable where the current boundary cannot expose them.

Cockpit samples were collected from the current repository in two runs:

| run | inspect p50/p95 ms | status p50/p95 ms | doctor p50/p95 ms | observe p50/p95 ms | diagnose p50/p95 ms |
|---|---:|---:|---:|---:|---:|
| baseline | 62.866 / 65.228 | 84.674 / 92.004 | 13.515 / 15.680 | 63.669 / 65.704 | 63.601 / 65.160 |
| optimized candidate after revert | 72.013 / 78.392 | 100.947 / 112.261 | 15.570 / 16.905 | 74.888 / 78.843 | 74.063 / 79.718 |

These samples do not prove a benefit for the candidate optimization; the
candidate was therefore not retained as a performance claim. Raw reports:

- `/var/folders/3z/qt8pjxx568xbg3d10zks40sr0000gp/T/tmp.QmQmuu294I/baseline-current-latest.json`
- `/var/folders/3z/qt8pjxx568xbg3d10zks40sr0000gp/T/tmp.QmQmuu294I/optimized-after-revert.json`

The verified object-project runs used three clean, single-file-change, and
cross-module-change clones from the same Sentinel commit:

| scenario | inspect p50/p95 | status p50/p95 | doctor p50/p95 | observe p50/p95 | diagnose p50/p95 |
|---|---:|---:|---:|---:|---:|
| many-files-clean | 94.396 / 104.372 | 149.091 / 157.153 | 50.740 / 51.704 | 113.652 / 115.617 | 95.981 / 97.688 |
| single-file-change | 108.558 / 111.071 | 162.144 / 164.877 | 51.450 / 52.407 | 127.273 / 130.674 | 109.617 / 112.483 |
| multi-file-change | 110.166 / 113.986 | 163.821 / 167.369 | 51.517 / 55.645 | 127.909 / 131.773 | 110.209 / 114.696 |

Raw reports:

- `/var/folders/3z/qt8pjxx568xbg3d10zks40sr0000gp/T/wi798-sentinel-final.PrgjEz/many-files-clean.json`
- `/var/folders/3z/qt8pjxx568xbg3d10zks40sr0000gp/T/wi798-sentinel-final.PrgjEz/single-file-change.json`
- `/var/folders/3z/qt8pjxx568xbg3d10zks40sr0000gp/T/wi798-sentinel-final.PrgjEz/multi-file-change.json`

These object-project runs measure actual repository-bound CLI cost. Sentinel
had no active Work Item, so they are not presented as a Sentinel Work Item
lifecycle measurement.

## Recovery and remaining acceptance boundary

- The phase-resume regression proved Prepare is reused after a later injected
  failure, without repeating the Prepare copy; failed output and cleanup were
  reported. The current implementation intentionally re-runs candidate/public
  acceptance and close in fresh roots, and the real publish receipt is not yet
  persisted from the publish job into public acceptance.
- Local Rust, Shell/Python static policy, workflow convergence, release
  acceptance, resume, and full workspace tests were run during integration.
- The complete release critical-path timing, hosted PR checks, merge, new
  immutable Release, downloaded public install/upgrade acceptance, Work Item
  close, and post-close cleanup are not established by this record.

