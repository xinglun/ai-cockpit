# WI-1078 exact resource cleanup and conditional close

Ray explicitly approved the published Retained/archive/Contract/resource bindings in the current direct user message. The exact text and unavailable author timestamp/message ID are preserved in human-approval.json.

Pre-cleanup: Native status remains archived with expected CleanupPending; source main and recovery-ref identities, original clean worktree/branch, same Runtime and all14 immutable historical files were read. GitHub PR1034 read first returned503; original bytes are retained and the explicit same-endpoint retry returned200. No deletion happened during the failed read.

Ruling: same-Contract historical verification is not rerun. Only the exact original WI1078 resources may be removed. Preserve old control Summary, old review source, recovery/control evidence, WI1084 and unrelated mistaken resources. Record Deleted from actual postconditions before conditional structured close.

This checkpoint precedes any cleanup. Later steps add separate observations, transition and close evidence; this pre-cleanup record remains history.
