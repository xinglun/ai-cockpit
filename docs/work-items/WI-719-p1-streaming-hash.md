---
author: AI Cockpit maintainers
title: "WI-719 — P1 large-file streaming-hash experiment"
description: "Measure and, only if justified, evaluate bounded streaming hashing for large changed files without weakening evidence or fail-closed governance."
audience: [maintainer, reviewer, adopter]
workItemId: WI-719-p1-streaming-hash
status: measured_declined
authority: human:repository-owner
lastVerifiedBy: WI-719-p1-streaming-hash
terminalArchive: .ai/work-items/archive/WI-719-p1-streaming-hash.contract.json
terminalVerification: .ai/evidence/WI-719-p1-streaming-hash.verification.json
terminalFinalization: .ai/decisions/WI-719-p1-streaming-hash.finalize.json
terminalDecision: .ai/decisions/WI-719-p1-streaming-hash.close.json
---

[简体中文](WI-719-p1-streaming-hash.zh-CN.md) · [日本語](WI-719-p1-streaming-hash.ja.md)

# WI-719 — P1 large-file streaming-hash experiment

## Outcome

The streaming-hash candidate was measured and declined. The candidate was
rolled back; production behavior is unchanged. The North Star remains
Calibrated Human-Agent Trust. The decision is evidence-based rather than a
claim that the candidate was unsafe.

## Hypothesis and boundary

The candidate replaced whole-file `fs::read` for changed files with chunked
hashing and a bounded text capture. It preserved the existing raw digest byte
sequence, the `MAX_CHANGE_TEXT_BYTES` capture boundary, and fail-closed read
failure behavior by committing the digest only after a complete read. The
experiment was limited to `crates/cockpit-git`; it did not change caches,
IncrementalMerkle, authorization, evidence binding, recovery, Outcome, or
release acceptance.

## Paired measurement

Baseline and candidate were built from base revision
`caa6ddffcc1847c9a1161e1d8aa414f1acabd11e`, with separately bound Runtime
identities, the same repository snapshot, macOS arm64/10 CPU, 10,331 tracked
files, 80,274,774 tracked bytes, 621 historical Work Items, and one 16 MiB
ordinary-path changed file. Each run retained raw order, first measurement,
one OS-cache warmup, 20 warm samples, nearest-rank percentiles, and Runtime
identity. p99 is unavailable at 20 samples.

| Command | Baseline warm p50/p95 ms | Candidate warm p50/p95 ms | Candidate delta |
| --- | ---: | ---: | ---: |
| status | 2,895.183 / 3,346.259 | 2,965.996 / 3,043.153 | +2.445% / −9.057% |
| inspect | 133.806 / 143.756 | 134.867 / 139.398 | +0.793% / −3.036% |
| doctor | 52.129 / 54.041 | 52.776 / 54.804 | +1.241% / +1.412% |
| observe | 188.036 / 204.221 | 186.403 / 207.658 | −0.869% / +1.684% |

The Contract required at least 5% improvement for both `status` p50 and p95.
The candidate missed p50 and the explicit budgeted gate reported
`budget_exceeded:status:2965.996>2739.035`. The comparator also failed closed
because this host could not provide a trustworthy filesystem type or
comparison key. Runtime-internal read bytes, hashed bytes, Git calls, child
processes, and peak memory remain explicitly unavailable; no unavailable
metric was filled with zero.

Raw evidence is retained in
`.ai/evidence/external/WI-719-p1-streaming-hash.baseline.large-file.json` and
`.ai/evidence/external/WI-719-p1-streaming-hash.candidate.large-file.json`;
the bound decision and gate result are summarized in
`.ai/evidence/external/WI-719-p1-streaming-hash.experiment-summary.json`.

## Correctness and governance

Before rollback, focused candidate tests passed for the large capture boundary
and raw digest bytes. Normalized `inspect`, `status`, `doctor`, and `observe`
outputs, errors, and exit codes matched the source baseline after removing only
Runtime/binary identity fields. The rollback leaves the existing repository and
IncrementalMerkle tests unchanged; they pass on the final tree. No production
optimization is accepted, and no performance benefit is claimed.

## Follow-up

The next performance Work Item should target the measured large-history
`status` bottleneck, not large-file streaming, unless a future isolated
measurement exposes a trustworthy resource or latency benefit. The resident
MCP, polling, parallel-read, coordinator, cache, in-process Git, and PGO
questions remain out of scope here.

## Validation limitation

Documentation acceptance and Work Item status consistency passed. The repository
wide `promote_closed_work_item.py --check-all` portion failed closed because the
parallel WI-717 projection is missing its required regular Markdown file. That
unrelated WI-717 defect is retained in
`.ai/evidence/external/WI-719-p1-streaming-hash.validation-limitation.json` and
was not changed by this Work Item.
