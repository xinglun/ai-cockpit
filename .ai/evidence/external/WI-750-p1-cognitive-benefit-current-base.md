# WI-750-p1-cognitive-benefit-current-base

This is a repository-local evaluation artifact, not a real user study.
No participants or external adopter data were supplied; cognitive benefit remains unvalidated.

## Fixed answer key and method

The seven task categories and answer key were fixed in the evaluation script before rendering.
For each real archived OutcomeV2 record, the current Runtime rendered both `summary` and `full`.
The automatic oracle compares verification, lifecycle, human-decision, governance-signal, evidence-reference, and critical-uncertainty facts.
It does not count its own full-view calls as user behavior.

## Results

- Cases: 7
- Automatic consistency violations: 0
- Critical visibility failures: 0
- Correct state/next-step time: not measured (no participants).
- Erroneous release count: not measured (no participants).
- Key-risk omission count: not measured as user behavior; automatic visibility checks are reported above.
- Green-as-safe or green-as-authorized misunderstandings: not measured (no participants).
- Full evidence views: 14 automated calls, all for consistency checking; not user behavior.

## Per-case evidence

| Case | Source Outcome | Stored state | Current summary verification | Current summary next-step evidence |
| --- | --- | --- | --- | --- |
| `normal-completion` | `WI-663-wi659-outcome-trust-replacement` | `finish_ready/green` | Verification not ready | yes |
| `verified-pending-human-decision` | `WI-658-wi656-outcome-trust-repair` | `finish_ready/green` | Verification not ready | yes |
| `scope-exceeded` | `WI-714-wi713-current-base-revalidation` | `blocked/red` | Superseded historical item | yes |
| `evidence-expired-or-identity-mismatch` | `WI-423-ci-convergence` | `blocked/red` | Superseded historical item | yes |
| `test-weakening-signal` | `WI-662-p0-benchmark-evidence` | `finish_ready/green` | Verification not ready | yes |
| `unverified-scope` | `WI-139A-preflight-review` | `finish_ready/green` | Verification status unknown | yes |
| `historical-closed-task` | `WI-743-wi715-p1-current-base-redelivery` | `finish_ready/green` | Verification not ready | yes |

## Limits

The source records are repository evidence, not participant responses. Historical records are intentionally reported as historical or otherwise not current when the current Runtime cannot revalidate them. This artifact therefore demonstrates repeatability and cross-view consistency only; it does not claim a reduction in human reading time or an observed safety benefit.

Reproduce with:

```sh
bash tests/evaluation/WI-750-p1-cognitive-benefit-current-base_test.sh
```
