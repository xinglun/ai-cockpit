# Adversarial validation surface

The v2 conformance corpus contains 15 semantic cases and five wording variants
per language (English, Japanese, and Chinese). The crate integration test
requires every variant to produce the same canonical governance decision.
Named RAI-01 through RAI-12 statuses are bound in the manifest so
`not_proven` and `partial` boundaries cannot be mistaken for passes.

The conformance corpus and crate integration tests exercise scope escape,
destructive authority, missing/stale/contradictory evidence, unsupported
completion, repository prompt injection, malicious deletion, cross-work-item
evidence, unknown provider results, test/coverage weakening, archive recovery,
MCP path containment, and verification working-directory containment.
