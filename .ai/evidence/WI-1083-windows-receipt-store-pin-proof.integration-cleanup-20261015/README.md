# WI-1083 integration and exact cleanup audit

PR #1039 merged normally at `1e3d0ebf8403fd301a783173422b4c1a32d9b236`,
with exact reviewed/verified head `93a2f1ff761c2ac35fea96e846302b25bc3c2a14`.
The actual merge parents and tree were checked against the final all-green CI merge.

The earlier two automatic approval refusals and Draft-restoration snapshots are
history, preserved verbatim. No alternate merge path was used. Ray then supplied
a direct current user message explicitly approving Ready, this exact head merge,
and corresponding normal clean branch/worktree cleanup. Its observation time
is recorded separately; no message timestamp or human decision time is invented.
The same normal GitHub connector then accepted Ready and merge.

`native-cleanup-receipt.stdout` and the Runtime-owned decision receipt record
sequence 1, binding `sha256:d76cd5a8fa337ead01fd1cfb60ef9de411ddf6ec12024ac99811ac5a3852403e`,
with branch/worktree removed, state verified and no failure codes. The binding
retains the original archive head `1807274505e08af745a6dcfffd4d403735d62e57`;
the merged close-metadata descendant was `93a2f1ff`. No binding was rewritten.

The exact ignored empty unheld lock and CPython cache bytes were retained here
before normal worktree removal. The conservative initial ignored-file check and
a corrected read-only Git absent-ref exit-code expectation are described, without
repeating any successful cleanup action. All six other worktrees, their dirty
bytes, and the separate mistaken remote WI-1081 ref were compared and retained.

The original Contract, archive, failed/successful verification history and close
decision are unchanged. No historical verification was rerun. Main-push CI is
a separate run and must be reported from its actual current state.

Unknowns remain: owner-declared user-visible benefit, intent alignment, host display
confirmation, historical ENOSPC bytes-versus-inodes attribution, and the original
repository stdout truncation. Usage coverage remains unknown where no receipts exist.
