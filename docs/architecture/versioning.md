---
author: AI Cockpit maintainers
title: "Versioning"
description: "Runtime and Repository Protocol version identity and migration boundary."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - versioning
---

# Versioning

Runtime version and Repository Protocol version are independent.

```text
ai-cockpit --version
0.1.0

repository:
protocol_version = 1
```

The CLI version identifies the executable package. Protocol version identifies
the repository storage contract. Runtime version, runtime digest, and protocol
version are exposed together on identity-bearing surfaces such as `inspect`,
`doctor`, MCP `initialize`, and verification evidence; `--version` alone is a
short package-version command and does not promise the full identity envelope.

A Runtime upgrade may add capabilities while continuing to support Protocol 1.
Only a Protocol 1 → Protocol 2 change is a repository migration. Historical Work
Items retain the Project Profile digest and protocol version used at their decision
boundary. A major migration is a separately reviewed Work Item that preserves old evidence.
