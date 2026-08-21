# First Public Release and Homebrew Tap Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute WI-35 to publish one identity-bound `v0.1.0` Release and open a protected Homebrew tap PR, with every hosted and installation receipt tied to the accepted source commit.

**Architecture:** Use the already accepted local release candidate and its tag-push-only workflow. Hosted jobs produce the candidate, gates, attestation, provider Release, and post-publication handoff; an external verifier consumes the handoff to create a tap PR. No job writes the tap default branch.

**Tech Stack:** GitHub Actions, GitHub Releases, `cockpit-release`, Cargo, SHA-256, SPDX SBOMs, GitHub build provenance, Homebrew Formula, and shell/PowerShell smoke scripts.

**Spec:** `docs/superpowers/specs/2026-08-21-first-public-release-homebrew-tap-bootstrap-design.md`

## Global Constraints

- Use only the archived WI-34 candidate commit; a dirty or unbound tree is invalid.
- Publish only a new immutable `v0.1.0` tag push; manual dispatch may not publish.
- Keep `cockpit.toml` as TOML and keep this development checkout without `.ai/`.
- Publish only the manifest, checksum file, Formula, five archives, and five SBOMs.
- Treat provider Release, tap, credentials, PR, merge, and public install as external writes requiring explicit authorization.
- A correction to source or workflow invalidates the candidate and opens a separate Work Item.

---

### Task 1: Confirm authority and external permissions

**Files:**
- Read: `docs/work-items/WI-35.md`
- Read: `docs/work-items/WI-34.md`
- Read: `.github/workflows/release.yml`

- [ ] **Step 1: Confirm the exact archived source commit and version**

Run:

```bash
git status --short
git rev-parse HEAD
cargo metadata --locked --format-version 1 | jq -er '.packages[] | select(.name == "cockpit-cli") | .version'
```

Expected: a clean archived commit, version `0.1.0`, and no unapproved source
or workflow correction after WI-34 acceptance.

- [ ] **Step 2: Record external authorization**

Before creating a tag or dispatching a hosted run, record the human approval
for tag creation, Release publication, tap creation/PR, and public installation
testing in WI-35. Do not infer this approval from local tests.

### Task 2: Run source-quality and hosted candidate gates

**Files:**
- Read: `.github/workflows/release.yml`
- Record: `docs/work-items/WI-35.md`

- [ ] **Step 1: Create the immutable tag on the approved commit**

Run only after Task 1 approval:

```bash
approved_commit="$(git rev-parse HEAD)"
git tag -a v0.1.0 "$approved_commit" -m "ai-cockpit v0.1.0"
git push origin v0.1.0
```

Expected: the tag is new, points at the approved commit, and cannot be moved.

- [ ] **Step 2: Capture the hosted run and gate conclusions**

Record the run ID, event payload, source-quality conclusion, five build
conclusions, manifest/checksum validation, all smoke conclusions, and final
candidate attestation ID. A failed or cancelled gate stops the plan.

### Task 3: Verify the canonical candidate identity

**Files:**
- Read: `crates/cockpit-release/src/manifest.rs`
- Read: `crates/cockpit-release/src/archive.rs`
- Read: `crates/cockpit-release/src/handoff.rs`

- [ ] **Step 1: Compare manifest and assets**

Check that version, tag, commit, Cargo.lock digest, target, runner, archive,
SBOM, byte count, and SHA-256 values agree. Ensure `SHA256SUMS` names exactly
the ten archive/SBOM files and no extra published asset is present.

- [ ] **Step 2: Verify archive members and runtime version**

On each declared runner, extract the matching archive, run `ai-cockpit --version`
and `ai-cockpit --help`, and record the output with the target and digest.

### Task 4: Publish the provider Release

**Files:**
- Read: `.github/workflows/release.yml`
- Record: `docs/work-items/WI-35.md`

- [ ] **Step 1: Confirm the provider Release is absent**

Run:

```bash
gh api --include repos/xinglun/ai-cockpit/releases/tags/v0.1.0
```

Expected: HTTP 404 before publication; any existing Release stops the plan.

- [ ] **Step 2: Capture the published Release identity**

Record the provider Release URL/ID and the complete allowlisted asset inventory.
Verify every uploaded digest against the canonical manifest before continuing.

### Task 5: Validate and attest the post-publication handoff

**Files:**
- Read: `crates/cockpit-release/src/handoff.rs`
- Record: `homebrew-handoff.json` from the named hosted run

- [ ] **Step 1: Retrieve the handoff from the publication run**

Verify its attestation, workflow ref, run ID, tag, commit, provider Release ID,
manifest digest, Formula digest, destination, action, issue time, and expiry.

- [ ] **Step 2: Reject any mismatch or expiry**

Do not open a tap PR if the handoff is missing, expired, re-used, or bound to a
different Release, Formula, manifest, commit, or destination.

### Task 6: Create the protected tap and open the Formula PR

**Files:**
- External: `xinglun/homebrew-tap`
- Input: verified `homebrew-handoff.json` and `dist/Formula/ai-cockpit.rb`

- [ ] **Step 1: Confirm tap protection**

Record repository ownership, protected `main`, required review/check rules, and
the narrowly scoped identity permitted to create the PR. Never push directly to
the tap default branch.

- [ ] **Step 2: Open an identity-bound PR**

Use the exact Formula projection from the handoff and record the PR URL, source
branch, diff, handoff digest, and provider Release ID.

### Task 7: Capture adopter installation receipts

**Files:**
- Read: `docs/release/distribution.md`
- Record: WI-35 hosted installation receipts

- [ ] **Step 1: Test Homebrew on macOS ARM64 and Intel**

After the Formula PR is merged, record `brew install`, `brew test`,
`ai-cockpit --version`, `brew uninstall`, and optional tap removal results.

- [ ] **Step 2: Test archive installation on Linux and Windows**

Record exact checksum comparison, extraction, `--version`, and `--help` on each
declared target. Do not describe an untested platform as supported.

### Task 8: Close WI-35 with an evidence-backed release report

**Files:**
- Modify: `docs/work-items/WI-35.md`
- Modify: `docs/work-items/WI-35.zh-CN.md`
- Modify: `docs/work-items/WI-35.ja.md`

- [ ] **Step 1: Link every receipt**

Record hosted run, commit/tag, Release URL/ID, asset inventory, attestation IDs,
tap settings, PR URL/merge commit, installation receipts, and skipped/failed
external steps.

- [ ] **Step 2: Apply the final human decision**

Close only after two independent reviews and explicit approval. If any required
receipt is absent, keep WI-35 open and describe the exact blocker; do not claim
GA or fabricate a rollback/upgrade result.
