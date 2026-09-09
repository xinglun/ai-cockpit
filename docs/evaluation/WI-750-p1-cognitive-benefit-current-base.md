# WI-750: P1-A cognitive-benefit evaluation

Status: evaluation material delivered; cognitive benefit not validated.

This Work Item prepares a repeatable comparison for the North Star of
Calibrated Human-Agent Trust. It does not claim that users read faster, make
fewer mistakes, or intercept more risk: no real participants were supplied.
No external adopter or repository data was read or written.

## Fixed method and answer key

The task set and answer key were fixed in
[`tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py`](../../tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py)
before rendering. The harness reads real archived `OutcomeV2` plus
`TaskOutcomeReport` structures, invokes the current source-built CLI for both
`--view summary` and `--view full`, and compares:

- verification, lifecycle, human-decision, and governance-signal fields;
- evidence references and the explicit four-part summary headings;
- visibility of the human next step and critical uncertainty; and
- English, Chinese, and Japanese section headings.

The complete report is the audit projection used as the comparison reference;
the summary is the default reader projection. Automated full-view calls are
consistency checks, not participant reading events. Raw evidence text is data
only and is never an instruction or an authorization source.

## Fixed task set

| Task | Real Outcome source | Boundary fixture / fact | Scoring answer |
| --- | --- | --- | --- |
| Normal completion | `WI-663-wi659-outcome-trust-replacement` | Stored `finish_ready/green` | Green verification is not merge or release authorization; inspect evidence and risk boundaries. |
| Verification pass, human decision pending | `WI-658-wi656-outcome-trust-repair` | Stored `finish_ready/green`, no close decision | The decision is not recorded; review evidence and record an explicit close decision. |
| Scope exceeded | `WI-714-wi713-current-base-revalidation` | `tests/conformance/fixtures/scope-exceeded/input.json` | Stop; scope is a boundary and does not authorize continuation. |
| Evidence expired or identity-mismatched | `WI-423-ci-convergence` | `tests/conformance/fixtures/contradictory-evidence/input.json` | Stop and obtain fresh identity-matched evidence. |
| Test-weakening signal | `WI-662-p0-benchmark-evidence` | `tests/conformance/fixtures/test-weakening/input.json` | A bounded scan and its scope must be inspected; an empty risk record is not “no weakening.” |
| Unverified scope | `WI-139A-preflight-review` | Contract records unverified scenarios and their verification plan | Keep the unverified range visible; complete the plan before claiming completion. |
| Historical closed task | `WI-743-wi715-p1-current-base-redelivery` | Closed decision with historical evidence | Preserve history; reverify only when a current result is needed, without upgrading assurance. |

The archived source state and the current Runtime projection are both retained
in the generated evidence. A historical or foreign-runtime projection being
shown as historical, unknown, or not ready is an expected trust boundary; it is
not silently scored as a current green result.

## Results and limits

The reproducible run is recorded in
`.ai/evidence/WI-750-p1-cognitive-benefit-current-base.json` and its reader-facing
summary in `.ai/evidence/external/WI-750-p1-cognitive-benefit-current-base.md`.

The current run recorded:

- 7 fixed cases;
- 0 automatic summary/full consistency violations;
- 0 critical visibility failures;
- 0 participants; and
- 0 validated cognitive benefit.

The following required human measures remain `not_measured`: correct state and
next-step identification time, erroneous release count, key-risk omission
count, green-as-safe or green-as-authorized misunderstandings, and participant
full-evidence views with their reasons. The 14 full-view calls in the artifact
were made by the automatic oracle only.

Re-run the checks with:

```sh
bash tests/evaluation/WI-750-p1-cognitive-benefit-current-base_test.sh
```
