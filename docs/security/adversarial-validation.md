# Adversarial validation

The security boundary is fail-closed and evidence-driven. The conformance
corpus is semantic rather than string-based: it compares decision state,
blockers, unknowns, safe actions, required checks, authority, and outcome state.

Runtime boundary tests additionally verify that repository text is treated as
data, Work Item IDs cannot traverse paths, MCP evidence paths stay inside the
repository, verification commands use an allowlist and target cwd, and finish
cannot self-declare completion without a fresh passed receipt.

Any failed or unknown provider result remains non-green. Human authority can
resolve a decision requirement but cannot manufacture a verification receipt.
