# WI-1078 Retained stage — 20261015

- Protection baseline captured; original records and six worktrees protected.
- Three actual HTTP 200 Provider reads captured with headers and exact raw bodies.
- Three Runtime evidence observe calls succeeded; selectors are actual returned values.
- Existing legacy imports unchanged; no lifecycle finalization recorded yet.
- Scope excludes Deleted, cleanup, close, tests and publication outside the existing recovery ref.

- Ruling: first capture collection fractions exceed second-resolution boundAt; preserve all first receipts and metadata. Fresh second capture plus explicit clock-order wait produced valid ordered timestamps without edits or gate changes. Root will bind only second-capture selectors.

- First finalize rejected before persistence: currentMainRevision must bind sync merge M1, while the newest tip is separately proved to contain M1. Original failed input/output retained.
- Corrected only that binding and actual invocation timestamp; Native finalize accepted schema-v4 Retained root, exit 0.
- finalize-verify returned expected CleanupPending with Retained only; no Deleted sequence1 exists and no tests were run.
