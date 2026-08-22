#!/usr/bin/env bash

# Shared release-isolation manifest helpers. This file is sourced by the
# adopter acceptance harnesses and can also be sourced by its regression test.
# It never creates or removes a temporary root by itself.

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    sha256sum "$1" | awk '{print $1}'
  fi
}

sha256_text() {
  if command -v shasum >/dev/null 2>&1; then
    printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
  else
    printf '%s' "$1" | sha256sum | awk '{print $1}'
  fi
}

stat_metadata() {
  if stat -f '%Lp\t%z\t%m' "$1" 2>/dev/null; then
    :
  else
    stat -c '%a\t%s\t%Y' "$1"
  fi
}

# Emit one deterministic tab-separated record for every entry below root:
# relative-path, type, mode, size, mtime, digest. Symlinks are never followed.
manifest_tree() {
  local root=$1 manifest=$2 path relative kind metadata mode size mtime digest target
  : > "$manifest"
  [[ -d "$root" ]] || return 0
  while IFS= read -r -d '' path; do
    relative="${path#"$root"/}"
    metadata="$(stat_metadata "$path")"
    IFS=$'\t' read -r mode size mtime <<< "$metadata"
    if [[ -L "$path" ]]; then
      kind=symlink
      target="$(readlink "$path")"
      digest="sha256:$(sha256_text "$target")"
    elif [[ -d "$path" ]]; then
      kind=directory
      digest=null
    elif [[ -f "$path" ]]; then
      kind=file
      digest="sha256:$(sha256_file "$path")"
    else
      kind=other
      digest=null
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$relative" "$kind" "$mode" "$size" "$mtime" "$digest"
  done < <(find "$root" -mindepth 1 -print0 | LC_ALL=C sort -z) > "$manifest"
}
