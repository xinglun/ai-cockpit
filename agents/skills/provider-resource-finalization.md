# Provider resource finalization

## Applicability

Load this guide only when the active Contract explicitly binds a Provider
resource such as a reviewed pull request, hosted check, release candidate, or
other external identity. It is not part of the ordinary Work Item route.

## Authoritative inputs

Use the Runtime status and action explanation, the exact Contract resource
identity, the reviewed change identity, provider receipts/manifests, and the
declared cleanup and acceptance evidence. Provider state is evidence, not
authorization; the Runtime decides whether the next action is admitted.

## Operations

1. Confirm the exact resource, repository, Work Item, Contract, and Runtime
   identities before any provider operation.
2. Perform only the cleanup or acceptance operation admitted by the current
   projection. Preserve a retry checkout and identity when a remote step
   fails.
3. Record the provider result in the Runtime's identity-bound receipt path;
   do not hand-edit receipts or archive records.
4. Re-query and validate the receipt before close. Keep implementation,
   provider cleanup, projection, and host Outcome delivery as separate facts.

If a Work Item was archived before its reviewed PR resource was bound, use the
Runtime-aware `work-item finalize-plan` recovery path with the exact Contract,
archive manifest, branch, worktree, provider, and PR identities. Runtime adds
one digest-bound `.ai/decisions/*resource-context.json` record and leaves the
archived Contract and manifest unchanged. Repeating the same handoff is
idempotent; a different identity is rejected. This bounded repair is not a
replacement Work Item and does not authorize cleanup by itself.

## Success conditions

The exact provider resource has current accepted evidence, the Runtime's
finalization gate accepts the receipt, and no provider cleanup remains. The
Outcome distinguishes code completion from resource cleanup and states the
remaining host-display limitation, if any.

## Failure evidence

Preserve the provider response, resource URL or immutable identity, retry
checkout, manifests, receipt digest, and Runtime rejection. Do not delete a
branch, worktree, receipt, or provider resource to make the projection look
clean.

## Continue or stop

Continue only after a fresh identity-bound projection admits the next action.
Stop when provider state is missing, remote delivery fails, evidence is stale
or contradictory, or a human decision is required. Do not treat a merged or
released-looking resource as authorization without the Contract and Runtime
evidence.

## Reference

See [derived artifacts](../../docs/reference/derived-artifacts.md), [CI release
evidence](../../docs/reference/ci-release-evidence.md), and the [command
reference](../../docs/reference/commands.md).
