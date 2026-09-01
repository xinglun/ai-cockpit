---
author: AI Cockpit maintainers
title: "WI-469 — reference ledger parity 順序 recovery"
description: "不変の WI-468 配信を復旧し、新しい verification evidence より前に parity 投影を登録します。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-469-reference-ledger-parity-order-recovery
predecessorWorkItemId: WI-468-reference-ledger-parity-promotion
status: implemented
authority: authorized
lastVerifiedBy: WI-469-reference-ledger-parity-order-recovery
terminalArchive: .ai/work-items/archive/WI-469-reference-ledger-parity-order-recovery.contract.json
terminalVerification: .ai/evidence/WI-469-reference-ledger-parity-order-recovery.verification.json
terminalFinalization: .ai/decisions/WI-469-reference-ledger-parity-order-recovery.finalize.json
terminalDecision: .ai/decisions/WI-469-reference-ledger-parity-order-recovery.close.json
---

# WI-469 — reference ledger parity 順序 recovery

## Intent と境界

WI-469 は不変の WI-468 に対する明示的な recovery successor です。前置の
archive/evidence bytes を保持しながら、hosted governance gate が拒否した
documentation projection の順序を修正します。

固定したローカル reference は `/Users/sei-rinn/dev/workspace_python/ai-cockpit-template` です。
semantic comparison のみに使用し、reference Runtime、Python module、repository state はコピーしません。

## Scope

- 3 言語の comparison page で manifest 派生の current snapshot を一致させます。
- 3 言語 page の WI-467 と WI-468 projection を `recovered` にします。
- WI-469 verification evidence を作成する前に、3 つの parity ledger へ WI-469 row を登録し、
  terminal record paths を明示します。
- predecessor records を immutable のまま保持し、recovery lineage を残します。
- count、status、language、history-order drift に対して documentation/conformance gate を fail-closed にします。

## Acceptance

1. 英語・中国語・日本語の WI-467/WI-468 document projection が `recovered` で一致し、recovery evidence に bind されること。
2. 3 つの parity ledger の WI-469 row が、その verification evidence の Git history 登場より前に存在すること。
3. manifest-derived snapshot と reader routes が regression gate を通り、意図的な drift は fail-closed になること。
4. predecessor archive、evidence、recovery、historical source bytes を書き換えないこと。

## Verification

- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `bash tests/docs/parity_status_check.sh`
- Contract に宣言した workspace quality gate

## Recovery boundary

WI-468 の CI rejection は決定論的な ordering defect でした。terminal parity row が verification evidence の後に初めて追加されたためです。
この successor は row を先に登録してから fresh verification を実行します。predecessor は historical/recovered のまま保持し、書き換えたり current success に再分類したりしません。
