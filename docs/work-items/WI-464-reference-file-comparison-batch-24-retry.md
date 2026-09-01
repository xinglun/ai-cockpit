# WI-464 Reference File Comparison Batch 24 — Recovery Retry

## Intent

Preserve the immutable WI-464 delivery attempt and complete the same bounded
comparison with a real provider context. This recovery Work Item does not
expand the source-to-target scope or copy reference implementation bytes.

## Source and boundary

- Reference repository: `/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`
- Pinned source commit: `fde3380f81fea5fd2e288f7a8849f737dc074060`
- Predecessor: `WI-464-reference-file-comparison-batch-24`
- Recovery reason: the predecessor bound a placeholder PR URL before a real
  provider PR existed; its evidence remains immutable and is not rewritten.

## Compared paths

| Reference path | Rust-side result |
| --- | --- |
| `.github/workflows/compatibility.yml` | Implemented differently by design; Rust CI uses its own pinned action and platform policy. |
| `.github/workflows/release.yml` | Implemented differently by design; Rust release manifests, SBOM, provenance, checksums, and adopter harnesses provide the release boundary. |
| `.github/workflows/smoke.yml` | Implemented differently by design; Rust lifecycle and release/adopter checks replace the source Make bridge. |
| `Makefile` | Implemented differently by design; the Rust CLI, Cargo checks, and repository gate manifest are the supported interface. |

No Rust omission was found. Source-only Python/Make/installer behavior remains
explicitly out of scope.

## Delivery rule

The actual reviewed PR must exist before `finalize-plan` binds its URL. The
retry must then run preflight, checkpoint, verification, finish, archive,
finalization, and close serially. The predecessor recovery receipt and all
retry evidence remain append-only and repository-bound.

## Verification

```text
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060
python3 tests/conformance/reference_inventory_docs_test.py
```
