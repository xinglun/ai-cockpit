# Self-Governance Cutover Readiness Implementation Plan

> Execute mutating steps only after the named human decisions in WI-30.

1. Review and integrate WI-25 through WI-29; record the resulting revision.
2. Re-run workspace tests, static gates, and the locked V1 Oracle on a clean
   worktree.
3. Build the exact `ai-cockpit` binary and record version plus SHA-256 digest.
4. Obtain explicit human approval for repository attach.
5. Run `ai-cockpit attach --repo <repository-root>` and inspect the complete
   diff; reject any write outside `.ai/` or any copied runtime implementation.
6. Run `status` and `doctor`; review the proposed project profile.
7. Obtain an explicit human profile decision and record it through the runtime.
8. Exercise the first governed Work Item through start, preflight, verify,
   finish, archive, and human close without changing product behavior.
9. Update Bootstrap documentation only after all cutover evidence is green.
10. Keep commit, push, hosted CI, tag, and release as separately authorized
    operations.
