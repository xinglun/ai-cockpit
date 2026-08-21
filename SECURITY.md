# Security policy

AI Cockpit provides repository-bound governance, bounded verification, and
auditable evidence. It is not a sandbox, identity provider, immutable audit
ledger, or compliance certification. Organizations remain responsible for
branch protection, production isolation, provider identity, signing, SBOM,
provenance, and retention controls that are outside the Runtime.

## Supported versions

| Version line | Support |
| --- | --- |
| Latest `0.x` release | Security fixes and release-blocking regressions while the line is current |
| Older `0.x` releases | Best effort only; upgrade to the latest release |

Because the project is pre-1.0, compatibility and support boundaries are
declared in each release. A repository Protocol migration is separate from a
Runtime upgrade and must not rewrite historical evidence.

## Reporting a vulnerability

Please use a private GitHub Security Advisory for this repository when
available. Include the affected release or commit, platform, reproduction
steps, impact, and whether any repository or evidence data was exposed. Do not
post credentials, customer data, or an exploit payload in a public issue. If
private advisory intake is unavailable, open an issue requesting a private
maintainer contact and share only a redacted description.

Maintainers will acknowledge a report, reproduce it in an isolated repository,
classify its release impact, and publish a fix or mitigation with a clear
supported-version statement. A report does not authorize changing immutable
release assets or deleting historical evidence.

## Security patch policy

Security patches must preserve fail-closed behavior, add a deterministic
regression test, update the threat model or boundary documentation when the
trust boundary changes, and pass the same hosted quality, oracle, and Windows
runtime checks as a normal release. Emergency patches may shorten the release
window, but may not skip required verification or human decision records.

For deployment limits and threat assumptions, see:

- [Threat model](docs/security/threat-model.md)
- [Enterprise deployment boundary](docs/security/enterprise-deployment-boundary.md)
- [Vulnerability reporting policy](docs/security/vulnerability-reporting.md)
- [Enterprise governance boundary](docs/security/enterprise-governance.md)

AI Cockpit supports enterprise compliance controls and evidence handoff; it is
not an ISO 27001, SOC 2, or other organizational certification.
