# Independent review evidence

## Review boundary

Read-only review of source commits `b57561dab0893a1e4dba486305cae8ed486c61bd..03ad312412482dc84a0177fef3f70ee0b98266cb`. The reviewer found no Critical findings and assessed the branch as not ready to merge. The findings below are retained as the recovery scope; they are not a GitHub PR review or hosted CI result.

## Important findings

1. Node reuse hashes only caller-provided `input_paths`, not a complete trusted read-set, so source changes outside that list can reuse stale success. Relative executable paths are canonicalized from the Runtime process directory rather than the composition worktree. (`crates/cockpit-verification/src/composition.rs`, original review locations near lines 734 and 813.)
2. Composition does not enforce that the invoking Work Item is the declared integration owner. Required-scenario and compatibility coverage can be satisfied by caller-supplied labels without proving that the command implements the checks. (`crates/cockpit-repository/src/collaboration.rs`, original review locations near lines 975 and 1061.)
3. Killing a process after the invalidation event is written can leave `.lock` permanently present; retry then times out instead of reconciling. (`crates/cockpit-repository/src/coordination_store.rs`, original review locations near lines 190 and 890.)
4. Evidence validation rejects final symlinks but does not contain-check intermediate symlinks, allowing a parent link to escape the registered worktree. (`crates/cockpit-repository/src/collaboration.rs`, original review location near line 1252.)
5. Action selection compares outcome IDs without provider identity, so providers with the same outcome ID can cross-block. (`crates/cockpit-repository/src/collaboration.rs`, original review location near line 842.)
6. The real CLI/process acceptance proves reuse only for identical inputs; it does not mutate an observed input while leaving the JSON unchanged. (`tests/acceptance/cross_wi_coordination_processes.py`, original review location near line 271.)
7. The MCP coordination schema declares `workItemId` twice. The later consumer description overwrites the provider description used by `publish-outcome`, making the action's identity semantics ambiguous. The review rated this Minor; this successor treats it as Important because it affects the public outcome-publication contract. (`crates/cockpit-mcp/src/lib.rs`, original review locations near lines 396 and 401.)

## Not treated as an in-scope finding

The reviewer declined to judge whether direct callers of the exported low-level `cockpit_verification::run_composition` must be admitted through the repository service. The supported CLI/MCP composition routes use `run_admitted_composition`; call sites will be checked before retaining this boundary. If repository call sites bypass admission, this ruling must be revisited.

The reviewer did not rerun the reported Rust tests or hosted CI. Those results remain separately bound evidence and are not attributed to this review.
