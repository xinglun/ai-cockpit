# Verification semantics

AI Cockpit records two independent questions for verification:

| Dimension | Meaning |
| --- | --- |
| `VerificationTier` (`T0`–`T3`) | How strong or authoritative the verification procedure must be. |
| `EvidenceAssurance` | Where the resulting evidence came from: `SelfDeclared`, `RepositoryVerified`, `ProviderVerified`, or `EnterpriseVerified`. |

`T3` does not mean `ProviderVerified` or `EnterpriseVerified`. It only says
that the requirement calls for authoritative verification. The assurance level
is determined by the evidence actually bound to the result.

A `VerificationRequirement` records the required tier, required assurance,
reason, and references to the policy, stage, and protected gate that caused the
requirement. The Runtime does not infer policy from the tier and does not
silently upgrade evidence assurance. An unmet requirement remains visible as a
governance gap and cannot become green through presentation.

Generated implementation approaches are repository-local evidence. When a
Work Item is archived, the approach is moved with the contract, summary,
outcome, events, reports, and parallel intelligence sidecar; it must never
remain as an orphan under the active directory. An active repository-local
parallel slot is also an explicit archive blocker and must be released first.

The wire schema is strict (`verification semantics schemaVersion: 1`):
`schemaVersion` is required; unknown fields, unknown tier values, and unknown
assurance values fail closed. Existing
`AssuranceLevel` consumers remain wire-compatible; new code should use the
`EvidenceAssurance` name.

Implementation evidence: `crates/cockpit-protocol/src/lib.rs`,
`crates/cockpit-verification/src/lib.rs`, and
`crates/cockpit-repository/src/lib.rs`.
