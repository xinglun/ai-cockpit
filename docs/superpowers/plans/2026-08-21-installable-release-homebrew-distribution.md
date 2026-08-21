# Installable Release and Homebrew Distribution Implementation Plan

> **For agentic workers:** Execute this plan task-by-task with the bootstrap Work Item `docs/work-items/WI-34.md`. Steps use checkbox syntax for tracking. Do not install AI Cockpit, create `.ai/`, commit, push, publish, create tags, dispatch hosted workflows, or mutate `xinglun/homebrew-tap`.

**Goal:** Implement a locally reproducible, fail-closed release and installation contract for the `ai-cockpit` binary, with GitHub Release assets as the source of truth and Homebrew Formula generation ready for a separately authorized upstream tap.

**Architecture:** Add a focused `cockpit-release` Rust package that owns strict manifest, archive, Formula, checksum, and cross-repository handoff types. The release workflow builds five target archives, aggregates and verifies one canonical manifest, runs staged installation smoke jobs, and publishes only from an immutable tag after all gates pass. Documentation describes the public commands without claiming that the external tap or Release already exists.

**Tech Stack:** Rust 1.94; serde/serde_json; sha2/hex; tar/flate2/zip; chrono; clap; GitHub Actions; Homebrew fixture Formula; POSIX shell and PowerShell smoke commands.

**Spec:** `docs/superpowers/specs/2026-08-21-installable-release-homebrew-distribution-design.md`

## Global Constraints

- `cockpit.toml` remains TOML; no V1 installation and no `.ai/` directory may be created.
- The copyright notice is exactly `Copyright (c) 2026 Ray`; add the complete MIT text at repository root as `LICENSE`.
- The five targets are `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`.
- Validation images are macOS 15 Intel, macOS 15 ARM64, Ubuntu 24.04 ARM64/x86_64, Windows Server 2025 x86_64, plus macOS 15 ARM64/Intel Homebrew smoke images.
- Archives contain exactly one executable (`ai-cockpit` or `ai-cockpit.exe`), `LICENSE`, and the short archive README; no symlinks, path traversal, target-suffixed executable, or extra files are accepted.
- Manifest and handoff JSON use typed structs, deterministic field order, one trailing LF, lowercase hexadecimal SHA-256 values, sorted arrays, and `deny_unknown_fields`.
- Production Formula URLs are HTTPS GitHub Release URLs only; fixture Formula URLs may use an ephemeral loopback HTTP server and are never accepted by the production generator.
- `workflow_dispatch` may build and verify but never publish; external tap creation, credentials, tags, Releases, PRs, merges, and hosted dispatch remain WI-35 scope.
- Use bootstrap commands from `docs/work-items/README.md`: Cargo tests, rustfmt, Clippy with warnings denied, and recorded evidence. Do not substitute `make ai-*` commands.

> The detailed checkboxes below are the original TDD sequence and acceptance
> checklist. The authoritative current state is `Execution Status` at the end;
> open workflow and hosted gates must remain visibly open even when local code
> for an earlier task exists.

---

### Task 1: Add the approved license and release-package boundary

**Files:**
- Create: `LICENSE`
- Create: `crates/cockpit-release/Cargo.toml`
- Create: `crates/cockpit-release/src/lib.rs`
- Create: `crates/cockpit-release/src/error.rs`
- Modify: `Cargo.toml`
- Test: `crates/cockpit-release/tests/package_boundary.rs`

**Interfaces:**
- Produces package `cockpit-release` with a library API and a `cockpit-release` binary entry point for later workflow tasks.
- `ReleaseError` is the single public error enum for malformed input, digest mismatch, archive violation, URL policy violation, and handoff expiry.
- The crate uses workspace version `0.1.0`, edition `2024`, Rust `1.94`, and `license.workspace = true`.

- [ ] Write a failing boundary test that loads the root `LICENSE`, asserts the first line is `MIT License`, asserts the copyright line is exactly `Copyright (c) 2026 Ray`, and rejects an archive when the license is absent.
- [ ] Run `cargo test -p cockpit-release --test package_boundary`; observe failure because the package, license, and archive validator do not exist.
- [ ] Add the complete standard MIT license text with the approved notice, add `crates/cockpit-release` to the workspace, and define only the dependencies needed by the later manifest/archive/formula/handoff modules.
- [ ] Add the smallest public error type and an empty CLI that returns a structured unsupported-command error until later tasks register subcommands.
- [ ] Run the boundary test again and require PASS; inspect `cargo metadata --locked` to confirm all workspace package versions remain `0.1.0`.

### Task 2: Define and test the canonical release manifest

**Files:**
- Create: `crates/cockpit-release/src/manifest.rs`
- Create: `crates/cockpit-release/tests/manifest.rs`
- Modify: `crates/cockpit-release/src/lib.rs`

**Interfaces:**
- `ManifestDigest` validates exactly 64 lowercase hexadecimal characters and exposes `as_str()`.
- `ArtifactRecord` contains `target`, `os`, `architecture`, `runner_image`, `archive`, `sbom`, and `provenance_subject`.
- `ReleaseManifest` contains `schema_version`, `product`, `package`, `version`, `tag`, `commit`, `cargo_lock_sha256`, and sorted `artifacts`.
- `ReleaseManifest::parse_str(&str) -> Result<Self, ReleaseError>` rejects unknown fields, duplicate targets, malformed SHA-256 values, invalid commit/tag/version bindings, unsupported targets, and wrong artifact cardinality.
- `ReleaseManifest::canonical_bytes(&self) -> Result<Vec<u8>, ReleaseError>` emits deterministic UTF-8 JSON with one trailing LF.
- `ReleaseManifest::validate_staged(&self, dist: &Path) -> Result<ValidatedRelease, ReleaseError>` recomputes file size and SHA-256 for every archive and SBOM and validates `SHA256SUMS`.

- [ ] Write RED tests for the five-target happy path, unknown top-level and nested fields, duplicate targets, uppercase/short digests, wrong `v` tag, wrong package/version, missing SBOM, extra checksum line, unsorted artifacts, and archive digest mutation.
- [ ] Run `cargo test -p cockpit-release --test manifest`; confirm each test fails for the intended missing parser/validator behavior.
- [ ] Implement typed `serde` structs with `#[serde(deny_unknown_fields)]`, an explicit target allowlist, canonical ordering, and digest/commit/tag validation.
- [ ] Implement staged-file verification using streaming SHA-256 and bounded metadata reads; do not trust producer-provided byte counts or digests.
- [ ] Implement canonical `SHA256SUMS` generation sorted bytewise by filename and require exactly the ten archive/SBOM files named by the manifest.
- [ ] Run the focused manifest tests and a second mutation pass that changes only `provenanceSubject`, `cargoLockSha256`, archive bytes, and a filename; each mutation must fail closed.

### Task 3: Implement deterministic archive packaging and inspection

**Files:**
- Create: `crates/cockpit-release/src/archive.rs`
- Create: `crates/cockpit-release/tests/archive.rs`
- Modify: `crates/cockpit-release/src/lib.rs`
- Modify: `.gitignore` only if the release fixture directory is not already ignored

**Interfaces:**
- `ArchiveTarget::from_rust_target(&str) -> Result<Self, ReleaseError>` maps the five allowed triples to archive kind and executable name.
- `package_archive(input: &PackageInput, output: &Path) -> Result<ArchiveRecord, ReleaseError>` creates a deterministic `.tar.gz` or `.zip` with normalized metadata, stable member order, zeroed timestamps, and the exact three-member layout.
- `inspect_archive(path: &Path, target: ArchiveTarget) -> Result<ArchiveInspection, ReleaseError>` rejects traversal, symlinks, duplicate members, extra files, wrong executable name, missing license/README, and non-executable Unix mode.

- [ ] Write RED tests using a temporary executable, the approved `LICENSE`, and a fixed archive README; assert exact member names, mode, byte digest, and repeated-output equality.
- [ ] Add adversarial fixtures for `../escape`, absolute paths, symlink entries, duplicate names, missing license, extra files, and a Windows archive containing `ai-cockpit` instead of `ai-cockpit.exe`.
- [ ] Run `cargo test -p cockpit-release --test archive` and confirm the new tests fail before implementation.
- [ ] Implement tar/gzip and ZIP writers with normalized metadata and the inspection rules above; use streaming reads for digest calculation.
- [ ] Run the archive tests twice in fresh temporary directories and compare SHA-256 values and member listings byte-for-byte.
- [ ] Add CLI subcommands `package` and `inspect` that expose these APIs for workflow use and return non-zero on every validation error.

### Task 4: Generate production and fixture Formulae deterministically

**Files:**
- Create: `crates/cockpit-release/src/formula.rs`
- Create: `crates/cockpit-release/tests/formula.rs`
- Modify: `crates/cockpit-release/src/main.rs`
- Modify: `crates/cockpit-release/src/lib.rs`

**Interfaces:**
- `FormulaSource::Production { release_origin: String }` accepts only `https://github.com/xinglun/ai-cockpit/releases/download/` and derives URLs from manifest tag/archive names.
- `FormulaSource::Fixture { base_url: String }` accepts only an ephemeral `http://127.0.0.1:<port>/` origin and is marked test-only in generated Ruby.
- `render_formula(manifest: &ReleaseManifest, source: FormulaSource) -> Result<String, ReleaseError>` emits stable Ruby with `on_macos`, `on_arm`, `on_intel`, exact SHA-256 values, `bin.install "ai-cockpit"`, version/help tests, and an unsupported-platform guard.
- The CLI `formula` subcommand takes a manifest, source mode, and output path; production mode cannot receive fixture URLs.

- [ ] Write RED tests for both macOS variants, stable repeated output, missing variant, duplicate target, non-HTTPS production URL, path traversal, fixture loopback acceptance, non-loopback fixture rejection, and unknown manifest fields.
- [ ] Run `cargo test -p cockpit-release --test formula` and confirm the expected failures.
- [ ] Implement the typed source policy and Ruby renderer with no network access and no runtime manifest discovery.
- [ ] Assert the generated production Formula contains only the fixed GitHub origin and the generated fixture Formula contains the test-only marker and loopback URL.
- [ ] Run the focused tests and compare generated Formula bytes across two invocations.

### Task 5: Implement the identity-bound Homebrew handoff

**Files:**
- Create: `crates/cockpit-release/src/handoff.rs`
- Create: `crates/cockpit-release/tests/handoff.rs`
- Modify: `crates/cockpit-release/src/main.rs`
- Modify: `crates/cockpit-release/src/lib.rs`

**Interfaces:**
- `HandoffDocument` contains `schema_version`, `request_id`, `issuer`, `destination`, `release`, `authorized_action`, `issued_at`, and `expires_at`.
- `HandoffDocument::new(...)` sets `request_id` to SHA-256 of the canonical document with `request_id` omitted.
- `HandoffDocument::validate(now: DateTime<Utc>) -> Result<(), ReleaseError>` enforces UTC RFC3339 seconds, expiry after issuance and no more than 24 hours later, five-minute clock skew, fixed source/destination/path/action, and digest bindings.
- `HandoffDocument::canonical_bytes()` and `write_json()` use the same deterministic serializer policy as the manifest.
- The CLI `handoff` subcommand emits the exact `homebrew-handoff.json` file and refuses an expired or mismatched binding.

- [ ] Write RED tests for request-id recomputation, unknown fields, changed source commit, changed manifest/formula digest, wrong destination path, unsupported action, 24-hour expiry boundary, expired handoff, future issuance beyond skew, and same-request retry identity.
- [ ] Run `cargo test -p cockpit-release --test handoff`; observe the intended missing-type failures.
- [ ] Implement the typed handoff and validation logic, including deterministic branch/request identity documentation in the serialized output metadata.
- [ ] Run focused tests and verify that changing any bound field changes `request_id` and invalidates the previous document.

### Task 6: Replace the release workflow with least-privilege gated jobs

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `.github/workflows/ci.yml`
- Create: `.github/workflows/release-fixture-smoke.yml` only if the staged smoke jobs cannot remain isolated in the release workflow
- Create: `tests/release/workflow_policy.sh`
- Create: `tests/release/fixtures/manifest-valid.json`
- Create: `tests/release/fixtures/manifest-invalid.json`

**Interfaces:**
- Build jobs upload only target-scoped artifacts from explicit runner labels and target triples.
- An aggregate job downloads all five artifacts, adds the approved `LICENSE` and archive README, invokes `cockpit-release package/inspect/validate`, emits one manifest and one `SHA256SUMS`, and uploads a single candidate bundle.
- A policy job checks tag/workspace/package/binary/commit/lockfile identity and rejects mutable tag reuse, missing targets, duplicate targets, mismatched checksums, unknown fields, and non-HTTPS production URLs.
- Homebrew ARM64/Intel fixture jobs download the candidate bundle, start an ephemeral loopback server, generate the fixture Formula, run `brew install`, `brew test`, `ai-cockpit --version`, `brew uninstall`, and assert the link is absent.
- Linux and Windows jobs extract their matching archive, verify SHA-256, inspect exact membership, and run `ai-cockpit --version`/`--help`.
- Publish is a separate job with `contents: write`, `if: startsWith(github.ref, 'refs/tags/')`, and `needs` on every build, aggregate, policy, and smoke job; manual dispatch has no publish path.

- [ ] Write workflow policy tests that fail on `ubuntu-latest`, `windows-latest`, workflow-wide write permissions, unpinned action tags, direct tap mutation, `curl | sh`, publish on `workflow_dispatch`, and missing smoke dependencies.
- [ ] Run the policy tests against the current workflow and record the expected RED failures.
- [ ] Pin every third-party action to a reviewed full commit SHA, use per-job permissions, replace moving runner aliases with the explicit matrix, and separate build/verify/publish responsibilities.
- [ ] Wire the aggregate job to invoke the release CLI and preserve exact artifact names, runner image, SBOM, provenance subject, and lockfile digest in the manifest.
- [ ] Add loopback fixture serving with a trap-based cleanup path and a test-only Formula source; ensure no fixture URL can reach the production renderer.
- [ ] Run shell syntax checks, workflow policy tests, and local dry-run commands against staged fixture data; record hosted smoke jobs as unexecuted until WI-35 authorization exists.

### Task 7: Add installation, verification, rollback, and MCP documentation

**Files:**
- Modify: `docs/release/distribution.md`
- Modify: `docs/release/distribution.zh-CN.md`
- Modify: `docs/release/distribution.ja.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `README.ja.md`

**Interfaces:**
- All three language documents describe the same commands and boundaries: `brew install xinglun/tap/ai-cockpit`, version/checksum/attestation verification, upgrade, uninstall, optional tap removal, manual archives, locked Cargo Git installation, immutable-release rollback, MCP startup, and explicit repository attach.
- Documentation identifies macOS Homebrew as the primary path, archives as the Linux/Windows fallback, and Linuxbrew/crates.io as unsupported in WI-34.
- Documentation states that installation does not create `.ai`; `attach` is a separate explicit operation.
- Public Release, external tap, and real installation receipts are explicitly marked WI-35 evidence rather than current facts.

- [ ] Add the exact validated Cargo command `cargo install --git https://github.com/xinglun/ai-cockpit.git --tag v0.1.0 --locked --root "$HOME/.local" --bin ai-cockpit cockpit-cli` and the version/uninstall verification commands.
- [ ] Add Homebrew upgrade and rollback examples that name an immutable prior Release for manual rollback and explain that the unversioned Formula tracks current release.
- [ ] Add SHA-256 and GitHub attestation verification steps without claiming that a Release or tap currently exists.
- [ ] Update English, Chinese, and Japanese documents in one change and compare section headings, commands, and scope statements for semantic parity.

### Task 8: Complete local evidence and update WI-34 truthfully

**Files:**
- Modify: `docs/work-items/WI-34.md`
- Modify: `docs/work-items/WI-34.zh-CN.md`
- Modify: `docs/work-items/WI-34.ja.md`
- Create: `target/release-evidence/wi-34-local-summary.json` only as an ignored local evidence artifact; do not commit it

**Interfaces:**
- The Work Item Summary records changed files, exact commands, pass/fail output, unexecuted hosted/external steps, and the next WI-35 handoff boundary.
- Outcome may advance only to `configuration_complete`; it must not claim hosted verification, public Release, tap merge, or installation GA.

- [ ] Run focused `cockpit-release` tests after each task group and preserve RED/GREEN evidence in the Work Item Verification section.
- [ ] Run `cargo test --workspace --all-targets --all-features --quiet` twice consecutively, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --all -- --check`, `git diff --check`, and `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`.
- [ ] Run archive mutation tests, manifest mutation tests, Formula determinism tests, handoff expiry tests, workflow policy tests, and the isolated Cargo Git fixture installation test.
- [ ] Confirm `.ai/` is absent, no V1 files were copied, no external repository was changed, and all three WI-34 documents remain structurally and semantically aligned.
- [ ] Record hosted Homebrew/Linux/Windows smoke, tag creation, Release publication, tap PR, merge, and real install receipts as explicitly pending WI-35.
- [ ] Set WI-34 to `Implemented locally — hosted/external evidence deferred to WI-35` only after every local gate is green; otherwise leave it open with the exact blocker.

## Plan Self-Review

- Manifest schema, archive shape, Formula policy, handoff identity, workflow gates, documentation, and evidence update each have an explicit task.
- The root MIT license and its approved holder are handled before archive packaging.
- The plan never creates `.ai/`, installs V1, mutates external repositories, publishes a Release, or substitutes local evidence for hosted evidence.
- All production interfaces named in later tasks are introduced in earlier tasks; tests are specified before implementation in each code task.
- The hosted closure boundary is explicit: WI-34 can reach local `configuration_complete`, while WI-35 owns hosted and public-install receipts.

## Execution Status

- [x] Tasks 1–5: license, release package, manifest, archive, Formula, handoff, and CLI boundaries implemented with RED/GREEN tests.
- [x] Task 6 local scope: release/CI workflow, source-quality/version gates, immutable-tag/release checks, final-candidate attestation, post-publication handoff, least-privilege permissions, policy script, candidate aggregation, and staged smoke job definitions are implemented and locally checked.
- [x] Task 7: English, Simplified Chinese, and Japanese installation documentation implemented and linked from each README.
- [x] Task 8 local scope: two workspace runs, warnings-denied Clippy, rustfmt, diff,
  workflow policy/YAML/bash checks, Windows cross-target compile, locked V1 Oracle,
  Cargo Git fixture install, Formula syntax, docs links, and three-language parity are
  green.
- [ ] Task 8 hosted/external scope: Windows hosted runtime receipts, Homebrew/Linux/
  Windows hosted smoke receipts, WI-35 external tap/Release/public-install receipts
  remain pending and require separate authorization.
