# M6 Agent-copy storage locator

Owner: `WI-1090-evidence-docs-recovery`. These eight files are byte-preserving relocations of Agent-owned copies from frozen Git source `cba3ea1661749ebbf1929bc523c5b86ff10eb7e7`. They are not newly generated native receipts. Two copies with the same digest remain distinct files.

`path-map.actual.json` preserves the exact approved index. Its `newPathProposal` values are the approved storage destinations; the execution audit establishes their actual state. To resolve a logical key from either original M6 manifest, find the unique entry whose `logicalM6Path` equals that key and read its `newPathProposal` relative to the repository root. Verify `bytes`, `rawSha256`, and the source `gitBlob`. For every unmapped key, continue to use the original `m6-execution-20261019/` path. Never fall back silently or treat the moved old physical path as present.

Both original manifests retain their original bytes and logical keys. Historical internal receipt references are unchanged; use the explicit locator for these copies or the immutable source commit for the original physical layout. The complete external fixtures and evidence remain preserved in `/workspace/scratch/wi1090-m6-execution-20261019/`; this machine-local retention is not a hosted artifact guarantee.

Rollback requires reversing exactly the eight indexed moves without overwriting an existing path. This README and index grant no lifecycle, human, material-risk, consumer, Sentinel, release, or Runtime-adoption authority.
