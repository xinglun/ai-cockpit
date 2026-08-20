# Product Boundary

## Identity

AI Cockpit is a Repository Governance Layer for AI-assisted engineering. Its
North Star is calibrated human-agent trust and its core rule is evidence over
self-declaration.

The governed chain is:

```text
Evidence → Governance Decision → Human Control
```

## In scope

- deterministic repository observation;
- bounded Work Item contracts;
- scope, authority, evidence, and lifecycle decisions;
- fail-closed verification planning and evidence reuse;
- repository-local facts, decisions, evidence, and knowledge projections;
- CLI and read/verify MCP adapters.

## Explicitly out of scope

AI Cockpit is not an Agent Runtime, Workflow Engine, Security Sandbox, general
prompt-injection detector, identity provider, compliance certificate, or
replacement for human review. Provider identity, branch protection, production
isolation, signing, SBOM generation, provenance, and enterprise policy remain
external evidence or adopter responsibility.

## Architecture constraints

- Runtime root is never inferred from the binary path.
- Runtime code is never copied into an adopter repository.
- Repository Protocol version is independent from Runtime version.
- MCP and CLI call the same application services; neither owns governance rules.
- A human decision can resolve a workflow question, but cannot turn an unverified
  check into a pass.

