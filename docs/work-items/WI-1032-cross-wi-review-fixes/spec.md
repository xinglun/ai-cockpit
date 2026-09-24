# WI-1032: Cross-WI review corrections

## Purpose and lineage

This is a serial recovery continuation of `WI-1031-cross-wi-acceptance-corrections`, created only because that Work Item was archived before its independent review completed. The predecessor archive, verification evidence, and events remain immutable. This is not a parallel implementation track and does not authorize a second active Work Item.

The implementation must close the Important findings from the independent review of commit range `b57561d..03ad312` and preserve the accepted WI-1031 behavior. It stops before any release, tag mutation, or publication.

## Scope

- Make node reuse depend on a complete, trusted read-set and runtime-observed command, executable, environment, and upstream inputs. If the read-set or executable identity is not reliably observable, execute rather than reuse.
- Resolve relative executables from the isolated composition worktree actually used to run them.
- Admit composition only for the declared integration owner. Required scenarios and compatibility constraints must bind to registered authoritative check identities; caller-supplied labels alone cannot authorize coverage.
- Make registration/invalidation retry recover after an actual process interruption, including a process killed while holding the coordination lock. Keep intermediate state fail-closed and event history append-only.
- Contain every outcome-evidence read to the registered worktree, rejecting symlinked parent components as well as final symlinks.
- Select action dependencies by `(providerWorkItemId, outcomeId)` pairs so same-named outcomes from different providers do not cross-block.
- Extend the real CLI/process acceptance to mutate an observed execution input without changing the composition JSON and prove that the affected process is spawned again.
- Remove the duplicate `workItemId` property from the MCP coordination schema, or otherwise give each action an unambiguous provider/consumer identity contract.
- Preserve ordinary single-WI serial execution and the canonical governance gate.
- Validate that the candidate CLI remains inspectable against the object engineering project (Sentinel) without writing its repository state or changing its lifecycle Runtime.

## Out of scope

- Editing or replacing any WI-1031 archive, evidence, decision, or event bytes.
- Modifying Sentinel source, Contracts, evidence, branches, worktrees, or coordination state.
- Broad redesign of the collaboration protocol, scheduler, or storage layout beyond the verified findings.
- Installing or publishing a new Runtime, creating or moving tags, release preparation, or public artifact acceptance.
- Weakening the canonical CI manifest or accepting caller declarations as verified facts.

## Acceptance criteria

1. An undeclared or incomplete node read-set disables reuse. Changing a source file that the command can read while leaving composition JSON unchanged never yields a zero-process stale reuse. A relative executable is hashed from the exact composition worktree; changing that executable changes node identity.
2. A non-owner Work Item cannot run admitted composition, and no command process starts. A command such as `sh -c true` cannot satisfy required scenarios or compatibility constraints merely by carrying their labels; the Runtime must match the registered authoritative check identity and reject incomplete coverage before spawn.
3. A real process terminated after invalidation persistence but before registration completion can be retried successfully. Exactly one valid impact event remains, the new registration becomes authoritative only after reconciliation, and consumers are blocked during any inconsistent state.
4. Outcome evidence whose final file or any parent directory is a symlink escaping the registered worktree is rejected at registration/publication/inspection boundaries. No outside file bytes are accepted as evidence.
5. When two providers declare the same `outcomeId`, an action consuming one provider's outcome is neither blocked nor admitted based on the other provider's event.
6. CLI acceptance performs an unchanged-input reuse run and a changed-observed-input run with identical composition JSON; the first may spawn zero processes, while the second spawns the affected node and preserves unaffected reusable nodes only when their trusted read-sets are unchanged.
7. The MCP schema exposes one unambiguous `workItemId` contract for each coordination action, and CLI/MCP parity tests assert the intended provider-versus-consumer identity semantics.
8. The existing multi-linked-worktree, multiple-real-process acceptance and the ordinary no-collaboration serial path continue to pass under the repository's canonical Runtime CLI + Cargo/CI gate.
9. A read-only candidate CLI compatibility check against Sentinel succeeds without changing its Git tree, Contract/evidence state, lifecycle Runtime, or coordination store.

## Verification and governance

- Keep installed lifecycle Runtime `0.2.113` fixed for this Work Item. Candidate code owns collaboration validation and candidate process tests; this does not assert that an older Runtime understands or enforces new collaboration fields.
- Use the repository's Runtime CLI + Cargo/CI canonical gate; this repository has no supported Make entrypoint. Keep Sentinel's own Make-based governance untouched.
- Run test-first, one fix at a time. Use real processes and multiple linked worktrees for crash/reuse behavior; test fixtures alone do not prove concurrency.
- Keep queries read-only. Registrations, events, publications, pause/resume, and recovery remain explicit write operations.
- Merge and exact cleanup are in scope under the user's authorization. Stop before release and leave the release decision to the user.
