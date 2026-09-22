# Knowledge query measurement

`tests/performance/knowledge_query_benchmark.sh` measures the real
`ai-cockpit knowledge query` CLI against an isolated detached worktree of the
supplied repository. The source revision is recorded, and the temporary
worktree is removed after the capture. The caller's checkout is not used as a
write target.

The first sample is cold: the derived `.ai/knowledge` directory is removed
before the command. Subsequent samples are independent CLI processes against
the reused projection. The output retains every raw sample and reports
diagnostic-only p50/p95/p99 availability; a percentile is `null` when its
declared minimum sample count is not met. Use at least 100 warm samples for
release-grade percentile evidence.

Each CLI payload contains additive `metrics.knowledgeQuery` fields:

- `projectionMs` covers cache validation and, when necessary, projection
  materialization.
- `queryMs` covers indexed candidate materialization and result filtering.
- `candidateRecordAccessCount` is the number of indexed candidates accessed;
  it is not a claim about total memory or serialization cost.

For a reused projection, the benchmark reports `cacheValidationMs` separately.
For a created or rebuilt projection it reports `null` with
`projection_not_reused`, so a cold build is never mislabeled as cache
validation. Resident memory and read bytes are captured from the host's
`/usr/bin/time` format when available; otherwise the result contains an
explicit unavailable reason.

Example:

```sh
tests/performance/knowledge_query_benchmark.sh \
  /path/to/ai-cockpit \
  /path/to/repository \
  /tmp/knowledge-query.json \
  100
```

The query index short-circuits when any explicitly supplied topic, component,
state, or Work Item value has no index entry. This preserves conjunctive exact
filter semantics while avoiding materialization of candidates that another
filter would later discard.
