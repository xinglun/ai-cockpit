# AI Cockpit task guide index

Task guides explain an admitted operation. They do not determine repository
state, invent authorization, or maintain a second action allow-list. Read the
Runtime `status` projection first and load only the guide selected by its
current facts.

| Guide ID | Load when | Do not load for | Runtime query |
| --- | --- | --- | --- |
| `ordinary-work-item` | Ordinary implementation, declared verification, archive, and local cleanup | A real failed or stale verification, bound Provider resource, or explicit release/upgrade acceptance | `ai-cockpit status --repo <repository>` |
| `verification-failure-recovery` | A verification failure, timeout, stale receipt, or invalid evidence is present | A normal success path without such evidence | `status` plus the failed verification/evidence identity |
| `provider-resource-finalization` | The Contract explicitly binds an external Provider resource | A no-resource Work Item | `status` plus the resource identity and action explanation |
| `release-upgrade-acceptance` | The Contract explicitly authorizes release or upgrade acceptance | Ordinary development or a release merely mentioned in background text | `status` plus the immutable artifact identity |

## Selection rules

Runtime status selects the current `guideId`, blockers, freshness, safe
actions, and human-decision requirement. A missing/localized guide is a docs
defect, not an admission change. Re-query after Contract or snapshot changes;
an old projection cannot admit a new state.

Guides explain admitted operations; Runtime/schema/CLI/capability metadata own
protocol facts. Use the ordinary route for Contract amendments and environment
drift; see [`docs/reference`](../../docs/reference/README.md) for semantics.
