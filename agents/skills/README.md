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

The Runtime projection is the source of the current `guideId`, blockers,
evidence freshness, safe actions, and human-decision requirement. A missing or
localized guide is a documentation problem and must not change action
admission. Re-query after a Contract or repository snapshot change because an
old projection cannot authorize a new state.

Each guide has the same compact shape: applicability, authoritative inputs,
operations, success conditions, failure evidence, and continue-or-stop
conditions. Protocol fields and command facts belong to the Runtime, schema,
CLI help, or capability metadata; deeper rationale belongs under
[`docs/reference`](../../docs/reference/README.md).
