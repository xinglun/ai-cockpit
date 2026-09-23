#!/usr/bin/env bash

# The release close reducer keeps the persisted state fail-closed whenever a
# receipt validation records a failure root.  The workflow and its regression
# test both use this small, side-effect-free boundary.
record_release_close_failure() {
  local reason="$1"
  state=failed
  reasons+=("$reason")
}

finalize_release_close_state() {
  if ((${#reasons[@]} > 0)); then
    state=failed
  fi
}
