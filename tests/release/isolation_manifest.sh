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

stat_metadata() {
  if stat -c $'%a\t%s\t%Y' "$1" 2>/dev/null; then
    :
  else
    stat -f $'%Lp\t%z\t%m' "$1"
  fi
}

path_identity() {
  if stat -c $'%d\t%i' "$1" 2>/dev/null; then
    :
  else
    stat -f $'%d\t%i' "$1"
  fi
}

resolve_symlink_target() {
  local path=$1 target='' directory='' basename='' hops=0
  while [[ -L "$path" ]]; do
    ((hops += 1))
    [[ "$hops" -le 40 ]] || return 1
    target="$(readlink "$path")" || return 1
    if [[ "$target" == /* ]]; then path=$target; else path="$(dirname "$path")/$target"; fi
  done
  [[ -e "$path" ]] || return 1
  directory="$(cd "$(dirname "$path")" 2>/dev/null && pwd -P)" || return 1
  basename="$(basename "$path")"
  printf '%s/%s\n' "${directory%/}" "$basename"
}

manifest_record() {
  local path=$1 relative=$2 kind metadata mode size mtime digest='' target='' resolved=''
  if [[ -L "$path" ]]; then
    kind=symlink
    metadata="$(stat_metadata "$path")"
    target="$(readlink "$path")"
    if resolved="$(resolve_symlink_target "$path")"; then :; else resolved=''; fi
  elif [[ -d "$path" ]]; then
    kind=directory
    metadata="$(stat_metadata "$path")"
  elif [[ -f "$path" ]]; then
    kind=file
    metadata="$(stat_metadata "$path")"
    digest="sha256:$(sha256_file "$path")"
  elif [[ -e "$path" ]]; then
    kind=other
    metadata="$(stat_metadata "$path")"
  else
    kind=missing
    metadata=$'0\t0\t0'
  fi
  IFS=$'\t' read -r mode size mtime <<< "$metadata"
  jq -cn --arg path "$relative" --arg type "$kind" --arg mode "$mode" \
    --arg size "$size" --arg mtime "$mtime" --arg digest "$digest" \
    --arg target "$target" --arg resolvedTarget "$resolved" \
    '{path:$path,type:$type,mode:$mode,size:$size,mtime:$mtime,
      digest:(if $digest == "" then null else $digest end),
      target:(if $target == "" then null else $target end),
      resolvedTarget:(if $resolvedTarget == "" then null else $resolvedTarget end)}'
}

# Emit one deterministic typed JSON record for every entry below root.
# Symlinks are never followed and retain literal and resolved target metadata.
manifest_tree() {
  local root=$1 manifest=$2 path relative
  : > "$manifest"
  [[ -d "$root" ]] || return 0
  while IFS= read -r -d '' path; do
    relative="${path#"$root"/}"
    manifest_record "$path" "$relative"
  done < <(find "$root" -mindepth 1 -print0 | LC_ALL=C sort -z) > "$manifest"
}

# Record all tracked paths plus every path below .ai, including untracked
# entries. Exclude the declared output directory only when it is inside root.
manifest_source_checkout() {
  local root=$1 output=$2 manifest=$3 root_real output_real='' output_relative='' relative path record
  root_real="$(cd "$root" && pwd -P)" || return 1
  if [[ -d "$output" ]]; then
    output_real="$(cd "$output" && pwd -P)" || return 1
  elif [[ -d "$(dirname "$output")" ]]; then
    output_real="$(cd "$(dirname "$output")" && pwd -P)/$(basename "$output")" || return 1
  fi
  case "$output_real" in
    "$root_real") output_relative='.' ;;
    "$root_real"/*) output_relative="${output_real#"$root_real"/}" ;;
  esac
  : > "$manifest"
  while IFS= read -r -d '' relative; do
    if [[ -n "$output_relative" ]]; then
      case "$relative" in "$output_relative"|"$output_relative"/*) continue ;; esac
    fi
    record="$(manifest_record "$root_real/$relative" "$relative")"
    if [[ -n "$output_relative" && "$output_relative" == "$relative"/* ]]; then
      record="$(printf '%s\n' "$record" | jq -c '.size = null | .mtime = null')"
    fi
    printf '%s\n' "$record"
  done < <(
    {
      git -C "$root_real" ls-files -z --cached --others --exclude-standard
      if [[ -d "$root_real/.ai" ]]; then
        printf '.ai\0'
        find "$root_real/.ai" -mindepth 1 -print0 | while IFS= read -r -d '' path; do
          printf '%s\0' "${path#"$root_real"/}"
        done
      fi
    } | LC_ALL=C sort -zu
  ) > "$manifest"
}

validate_manifest_symlink_containment() {
  local root=$1 manifest=$2 root_real path target resolved
  root_real="$(cd "$root" && pwd -P)" || return 1
  while IFS=$'\t' read -r path target resolved; do
    [[ -n "$resolved" ]] || {
      printf 'unresolved symlink target below %s: %s -> %s\n' "$root_real" "$path" "$target" >&2
      return 1
    }
    case "$resolved" in
      "$root_real"|"$root_real"/*) ;;
      *)
        printf 'symlink target escapes allowed root %s: %s -> %s\n' "$root_real" "$path" "$resolved" >&2
        return 1
        ;;
    esac
  done < <(jq -r 'select(.type == "symlink") | [.path, .target, (.resolvedTarget // "")] | @tsv' "$manifest")
}
