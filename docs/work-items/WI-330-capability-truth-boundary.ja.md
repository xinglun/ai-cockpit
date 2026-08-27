---
workItemId: WI-330-capability-truth-boundary
status: in_progress
---

# WI-330 — capability-truth boundary calibration

## Intent

固定 source の capability claim、freshness、Capability Truth Matrix の 4 file を Rust
repository と一つずつ比較し、各 file の product boundary を記録します。

## File-level decision

| Pinned source path | Classification | Rust counterpart と決定 |
| --- | --- | --- |
| `docs/reference/capability-claim-authoring.md` | `reference-only` | lexical claim-binding checker は copy しません。Target の capability page と registry は bounded observed fact だけを報告し、文書 metadata は evidence ではありません。 |
| `docs/reference/capability-evidence-freshness.md` | `reference-only` | Work Item receipt freshness は検証しますが、source の Capability Truth row expiry と portable-environment policy は current Runtime の機能ではありません。 |
| `docs/reference/capability-truth-matrix.json` | `reference-only` | source 30-row matrix を Rust wire format や authorization source として copy しません。`capability_truth_registry` は observed fact と明示的な exclusion を request-scoped に報告します。 |
| `docs/reference/capability-truth-matrix.md` | `reference-only` | Target capability/adoption 文書は observed、repository、adopter、provider、enterprise の境界を説明し、source matrix/checker を宣伝しません。 |

## Boundary

この Work Item は比較の境界を閉じるもので、claim checker、row expiry policy、assurance level、
Python/V1 runtime asset の copy は追加しません。将来 Rust-native claim/evidence 機能を追加する
場合は schema、stale handling、multilingual scope、adopter acceptance を human-owned Work Item
で先に定義します。

## Acceptance

- 各 pinned path に classification、counterpart、non-overclaiming reason があること。
- English、Simplified Chinese、Japanese の comparison/parity record が一致すること。
- inventory regression が 4 path をカバーし、件数が変わらないこと。
- source Python script、source matrix JSON、V1 runtime state を copy しないこと。

## Verification

`bash tests/conformance/reference_file_inventory_test.sh`

