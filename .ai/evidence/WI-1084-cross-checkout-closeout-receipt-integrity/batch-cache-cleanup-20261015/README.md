# Approved exact cache cleanup

Ray directly approved the previously committed 2,112-file manifest, SHA-256 `8e12a74c56ea839341532475e0fca0eeb77000c0f65552d482e9ddc9d336e539`. The original proposal and its broader read-only inventory remain unchanged.

Execution rechecked every candidate against its regular, non-executable file identity and timestamp, checked directory parents without symlinks, and checked there were no active Cargo/rustc/Runtime builds. Only exact manifest files were unlinked with a parent directory descriptor. No containing directories were deleted.

All 2,112 listed files are absent. All 700 other entries in the selected build/deps trees retain their identities and metadata. All 10 explicitly protected Runtime paths retain their exact SHA-256 and identities. All other worktrees, human files, Runtime binaries, WI-1078 resources, current WI-1084 target and historical evidence were outside the deletion list.

Workspace available bytes rose from 633,966,592 to 2,964,455,424: observed increase 2,330,488,832 bytes (about 2.17 GiB). File/log bookkeeping makes this slightly below the inventory allocation estimate; no estimated bytes are presented as actual release. Free inodes rose by 2,107 net. `/tmp` available bytes were unchanged. Raw df/df-i and per-file deletion log are retained.

Inherited PID-1 zombies remain: this operation did not change PID 1, system permissions/security/configuration, or kill processes. This cache cleanup is distinct from Work Item resource finalization and grants no merge or close authority.
