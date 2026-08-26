---
author: AI Cockpit maintainers
title: "WI-308 — reference evidence、trust、rollback-corruption batch 04 retry"
workItemId: WI-308-reference-file-comparison-batch-04-retry
description: "4 つの pinned reference file を比較し、Rust-native/adopter 向け parity boundary を記録します。"
audience:
  - maintainer
  - reviewer
status: in progress
lastVerifiedBy: WI-308-reference-file-comparison-batch-04-retry
authority: canonical
---

# WI-308 — reference evidence、trust、rollback-corruption batch 04 retry

## Intent と goal

この Work Item は pinned source commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` の 4 file、demo GIF、
`docs/case-study-ai-rollback-corruption.md`、`docs/concepts/evidence-governance.md`、
`docs/concepts/trust-layer.md` を一つずつ比較します。adopter に対する evidence-backed
な file-level parity を記録しますが、source の Python/Make/installer 実装や binary asset
は copy しません。

## File decision

| Reference file | Classification | Target evidence / boundary |
| --- | --- | --- |
| `docs/assets/ai-cockpit-demo.gif` | reference-only | GIF89a、800x435、587,945 bytes、SHA-256 `88838de7221dc859efde7e8e87913d0a23a21466195647ded60612adbad1f795`。visual reference のみで binary は copy しません。 |
| `docs/case-study-ai-rollback-corruption.md` | implemented-different-by-design | 三言語 adversarial-validation と typed Contract/scope check が unauthorized path、無関係な変更、controlled recovery を扱います。仮想 case であり auto-rollback や merge approval は主張しません。 |
| `docs/concepts/evidence-governance.md` | implemented-different-by-design | Enterprise governance、Outcome/evidence docs、typed Protocol/Repository record が Evidence → Governance Decision → Human Control を投影します。provider evidence は delegated です。 |
| `docs/concepts/trust-layer.md` | implemented-different-by-design | Product boundary、philosophy、enterprise governance、capability truth が calibrated trust、fail-closed unknown、human control、non-goals を定義します。 |

これは semantic responsibility parity であり source wire/byte compatibility ではありません。
Contract value と evidence は authored fact のまま保持し、prose を proof とせず、local evidence
を provider/enterprise assurance に黙って昇格させません。

## Successor と recovery boundary

この実装は当初 WI-306 と reviewed PR #268 に記録されましたが、その archived delivery は
merge されませんでした。WI-307 により default branch の parity projection が変わった後、
旧 PR を更新すると archived Contract/base の書き換え、または archive 後の branch conflict
解決が必要になります。そのため WI-306 は immutable な historical provider evidence として
保持し、この successor は current remote `main` から fresh Contract で同じ bounded file
comparison を再実施します。旧 PR を current success/failure として扱ったり revive したり
しません。

## Scope

- Reference inventory generator、generated ledger、regression assertion を更新します。
- English、中文、日本語の comparison/parity/work-item docs を同期します。
- 三言語 adversarial-validation に rollback-corruption boundary を追加します。
- 明示的な repository context で installed shared Runtime を検証します。adopter も同じ
  semantics を継承しますが、`.ai/` state は repository ごとに分離します。

## Out of scope

Rust production code、新 command/governance semantics、release/adopter/CI、global Agent/MCP
configuration、source Python/Make/installer、reference GIF または binary copy、immutable
historical evidence/archive bytes は対象外です。

## Acceptance と verification

1. 4 つの pinned file を読み、個別に分類し、GIF の digest/type/dimensions/size を記録します。
2. 三言語 adversarial-validation が scope violation、無関係な変更、completed-work rollback
   risk を Rust-native evidence boundary と過度な security claim なしで説明します。
3. Evidence Governance/Trust Layer から enterprise governance、Outcome/evidence、product
   boundary、philosophy、capability truth への reader route を明示します。
4. inventory、comparison、parity、Work Item を同期し、WI-308 を deferred に残さず
   `migrate-gap` を追加しません。
5. installed `ai-cockpit` を全て明示的な `--repo` で使い、preflight、checkpoint、verify、
   finish、archive、reviewed PR、merge、finalization verification、close、exact cleanup を
   完了します。最終 human Outcome は中国語で可視化します。

必須 check は `cargo test --locked --workspace` と、Runtime/CI が定める reference inventory、
documentation、governance-integrity、release-quality check です。
