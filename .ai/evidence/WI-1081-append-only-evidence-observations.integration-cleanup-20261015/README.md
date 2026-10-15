# WI-1081 normal integration and exact ordinary cleanup

The final head `18313fb764098f498200863ef4b2d63aee25d9c7` completed
CI38685734612 with all four jobs success and independent Raydot review with no
P1/P2. Normal GitHub Ready/merge returned actual merge
`edd1f9933eb3cd8628e4d427df156de1cc76ba5e`. Its parents and tree match the
actual tested synthetic merge `50faaa71e590e2ded159433437490b9ddb0f90ee`,
with actual first parent `b6b88e7eef11abb464d9097c8795c808b17c88b9`.
Stale PR-event comparison metadata is not substituted for that actual parent.

Only the correct `codex/wi1081-append-only-evidence-observations` branch and
`/workspace/ai-cockpit-wi1081-append-only-evidence-observations` clean merged
worktree were removed through normal Git commands. Two empty unheld ignored
lifecycle lock files (including an older WI-1080-named lock in this WI-1081
worktree) were backed up byte-for-byte; no other worktree or resource was cleaned.
All five other worktrees, their heads/status/dirty bytes, and the mistaken remote
`codex/wi1081-post-merge-lifecycle-layout` ref were compared and preserved.
WI-1078 resources remain outside authorization. No force, bypass or new test run.

Native ordinary-cleanup sequence 1 records branch/worktree removed, state
verified and no failure codes. Its immutable binding retains original head
`00e89d21d6a00d09819f8353ae0a1391b954f171`; the approved close-evidence
descendant `18313fb7` was checked to contain only added .ai files and to be merged.
No close binding, Contract, original archive or verification receipt was rewritten.

Original archive identity remains
`sha256:963433dbb29e9192a242629a83ef6e8f75febbe631ece5b40fbfdcedce3de082`,
Contract canonical digest remains
`sha256:fa050a50662e772dbe28afecd44e92b22e41749197b39ea78dc02806dc921d20`.
Ray's direct approval source is in the prior human-close evidence. Its message
authored time and ID remain explicitly unknown; actual Runtime record time is
separate and no timestamp is invented. Old failed CI and original evidence remain.

After integration, verification and closure remain verified/green, while Runtime
reports material_review_receipt_stale, repository_material_inspection_unavailable
and user_visible_benefit_not_declared as unknowns. These are preserved separately
from verified cleanup and do not become an invented fresh material review.
Intent alignment, usage coverage, host display confirmation and original bounded
repository stdout remain limited as recorded.

Official job logs here are the exact UTF-8 bytes of the decoded text returned by
the GitHub connector (including CRLF where present), not the original ZIP bytes.
Raw whitespace is preserved. The byte manifest excludes itself.

The temporary maintenance control is retained until this checkpoint is normally
pushed, read back and clean. Its later normal removal is a subsequent observation,
recorded in the PR handoff and local audit; this checkpoint does not claim to
contain evidence generated after its own publication. Main-push CI is a separate
run and must be reported at its actual current state.
