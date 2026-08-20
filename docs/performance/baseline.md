# Performance baseline (local evidence)

Captured with `cargo test -p cockpit-cli --test performance -- --nocapture` on
2026-08-21 in the development workspace:

| Surface | Fixture | Result |
| --- | --- | --- |
| `status` warm startup | 12 samples | median 2 ms |
| repository observation (incremental cache hit) | 200 generated files, 406 files read/hashed | 26 ms |
| knowledge unrelated query | 10,000 records | 0 historical records accessed |

The status target (<50 ms) and the incremental observation target (<100 ms) are
met in this run. The first uncached scan is intentionally measured separately;
the acceptance target applies to the incremental cache-hit path. The numbers
are evidence for this machine, not a universal guarantee.
