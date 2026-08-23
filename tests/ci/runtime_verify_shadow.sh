#!/usr/bin/env bash
set -euo pipefail

repo="${1:-${GITHUB_WORKSPACE:-$(pwd)}}"
output="${2:-${AI_COCKPIT_SHADOW_OUTPUT:-target/ci-runtime-verify-shadow.json}}"
runtime_tag="${AI_COCKPIT_RUNTIME_TAG:-v0.2.28}"

# Phase 1 is an execution smoke only. It proves that one immutable public
# Runtime can execute a repository-bound verification command. It does not
# claim policy-route/planner coverage, affected-graph completeness, or
# cross-Work-Item physical execution/evidence-receipt coverage.
shadow_boundary="execution_smoke"

die() {
  printf 'runtime verify shadow: %s\n' "$1" >&2
  exit 1
}

os="$(uname -s)"
arch="$(uname -m)"
case "$os:$arch" in
  Linux:x86_64)
    target="x86_64-unknown-linux-gnu"
    archive_sha256="350e29a1e0acbf484311ea75ece76f1e6bd9619d0133887a22f0736bcedec8e8"
    binary_sha256="c2382c8b5f78a091b74f5c899d9a02c8404fea4c3a9f49bbf93e4fc07b226e15"
    ;;
  Darwin:arm64)
    target="aarch64-apple-darwin"
    archive_sha256="7a9666f01826be78e0e3ec4f9d45bb951062670af5cfa4a2026971340ece4ce7"
    binary_sha256="23e9a9824f35b9e0d3706bdcbad34777bca5b0a4d362b4101407412a905dcb5b"
    ;;
  Darwin:x86_64)
    target="x86_64-apple-darwin"
    archive_sha256="71ed63fe00acdc69b0dd5d2a10724b3ef0c12ae2503ff1fb029d4a4e5cac0700"
    binary_sha256="2b093e16d658a4a2ffa36bbd0ed48a745dacd4438e3732818d3dfdd28c2d488e"
    ;;
  *)
    die "unsupported shadow host: $os/$arch"
    ;;
esac
archive="ai-cockpit-${runtime_tag}-${target}.tar.gz"
runtime_digest="sha256:${binary_sha256}"
release_base="https://github.com/xinglun/ai-cockpit/releases/download/${runtime_tag}"

[[ "$runtime_tag" == "v0.2.28" ]] || die 'only the immutable v0.2.28 baseline is allowed in Phase 1'
[[ -d "$repo" ]] || die "repository does not exist: $repo"
command -v curl >/dev/null || die 'curl is required'
command -v jq >/dev/null || die 'jq is required'
command -v shasum >/dev/null || die 'shasum is required'
command -v tar >/dev/null || die 'tar is required'

tmp_parent="${TMPDIR:-/tmp}"
[[ -d "$tmp_parent" ]] || die "TMPDIR is not a directory: $tmp_parent"
run_root="$(mktemp -d "$tmp_parent/ai-cockpit-ci-shadow.XXXXXX")"
cleanup() {
  find "$run_root" -depth -mindepth 0 -delete 2>/dev/null || true
}
trap cleanup EXIT

archive_path="$run_root/$archive"
download_source="$release_base/$archive"
curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
  "$download_source" > "$archive_path" || die 'public Release archive download failed'
printf '%s  %s\n' "$archive_sha256" "$archive_path" | shasum -a 256 -c - >/dev/null \
  || die 'public Release archive digest mismatch'

extract_root="$run_root/extracted"
mkdir -p "$extract_root"
tar -xzf "$archive_path" -C "$extract_root"
binary="$extract_root/ai-cockpit"
[[ -f "$binary" && -x "$binary" ]] || die 'public Release archive has no executable ai-cockpit'
actual_binary_sha256="$(shasum -a 256 "$binary" | awk '{print $1}')"
[[ "$actual_binary_sha256" == "$binary_sha256" ]] || die 'public Release binary digest mismatch'
version="$($binary --version | awk '{print $2}')"
[[ "$version" == "0.2.28" ]] || die "public Release binary version mismatch: $version"

mkdir -p "$(dirname "$output")"
verify_output="$run_root/verify.json"
if ! "$binary" verify --repo "$repo" --workers 2 > "$verify_output"; then
  die 'installed public Runtime verify command failed'
fi
jq -e --arg version "$version" --arg digest "$runtime_digest" \
  '.passed == true and .runtimeVersion == $version and .runtimeDigest == $digest' \
  "$verify_output" >/dev/null || die 'Runtime verify output is missing a passing result or identity'

jq -n \
  --arg tag "$runtime_tag" \
  --arg version "$version" \
  --arg archiveDigest "sha256:$archive_sha256" \
  --arg binaryDigest "$runtime_digest" \
  --arg platform "$target" \
  --arg downloadSource "$download_source" \
  --arg boundary "$shadow_boundary" \
  --slurpfile verify "$verify_output" \
  '{schemaVersion:1,phase:1,boundary:$boundary,tag:$tag,version:$version,archiveDigest:$archiveDigest,binaryDigest:$binaryDigest,platform:$platform,downloadSource:$downloadSource,verify:$verify[0],canonicalProfileRequired:true,nonClaims:["runtime_global_route","affected_graph","physical_execution_receipt"]}' \
  > "$output"
jq -e --arg tag "$runtime_tag" --arg digest "$runtime_digest" \
  '.phase == 1 and .boundary == "execution_smoke" and .tag == $tag and .binaryDigest == $digest and .canonicalProfileRequired == true and .verify.passed == true and (.nonClaims | index("runtime_global_route")) != null and (.nonClaims | index("affected_graph")) != null and (.nonClaims | index("physical_execution_receipt")) != null' \
  "$output" >/dev/null || die 'Runtime shadow receipt is malformed'
