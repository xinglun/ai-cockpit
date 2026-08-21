---
author: AI Cockpit maintainers
title: "Enterprise Deployment Boundary"
description: "What the shared Runtime and repository-local Protocol provide, and what an enterprise must supply externally."
audience:
  - adopter
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - enterprise_deployment_boundary
---

# Enterprise deployment boundary

Install one shared `ai-cockpit` Runtime per machine or toolchain. Attach each
repository explicitly; repository identity, contracts, evidence, knowledge,
and adapters remain isolated under that repository's `.ai/` directory.

AI Cockpit provides typed contracts, bounded verification, fail-closed reuse,
repository-local records, and exportable audit events. It does not provide a
global identity provider, OS sandbox, branch protection, production change
control, secret manager, enterprise SIEM, WORM retention, signature service,
SBOM generator, provenance authority, or organization-wide approval directory.

Adopters must bind those external controls to Work Items through delegated
evidence and must define classification, retention, disposal, and export rules
for their data. A local green decision is not proof that an external control
was satisfied unless the external evidence is present, valid, and bound.
