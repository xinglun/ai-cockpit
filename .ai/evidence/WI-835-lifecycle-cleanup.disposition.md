# WI-835 lifecycle disposition

Captured against `origin/main` at `28e721b353ae4d29894f99b7062cf465e4192dcd` on 2026-09-14. The inventory contains the 25 current non-main remote branches. PR facts were read from the GitHub pull-request listing, and worktree facts were read from the local `git worktree list` plus each worktree's porcelain status. No branch or worktree was deleted while producing this report.

Disposition rule: `delete-candidate` requires an exact reviewed merge fact, a Runtime-verified archive/finalization/close chain, a clean exact worktree, and no branch-only content. Every other row is preserved with its blocking reason.

| branch | remote tip | PR fact | worktree fact | disposition |
|---|---|---|---|---|
| `codex/wi829-release-trigger-repair` | `224e7a0e60585dbd91f192a5b155fbe7bcaeb2e1` | #809 merged | clean; `/private/tmp/ai-cockpit-wi829-release-trigger` | preserve: Runtime close not proven |
| `codex/wi828-release-source-path` | `33b47f05f4f7e5ea3383ac3411dcb8518a5ab108` | #816 merged at `ddf4973b17f1ae11cf47c124617763ed007b0e05` | dirty (1); `/private/tmp/ai-cockpit-wi828-source-path` | preserve: remote tip differs and worktree is dirty |
| `codex/wi-802-release-v0-2-91-recovery` | `3df2ada0b577c87ec9f5a5a4fcf5dd7e7a6e23d2` | #778 merged at `e70052c607040ab4caa420b35d10571b14945f3a` | dirty (2), local HEAD `748ff4fee425556f1466054fac9b33a3b5f9e2dc` | preserve: divergent local evidence |
| `codex/wi-804-release-route-ordering` | `3f23a04d21af285bd45e310b9b49ce9f5c550c71` | #781 merged | clean; branch is not locally bound (the similarly named worktree is bound to WI-806) | preserve: ambiguous local resource binding |
| `codex/wi-803-release-v0-2-91-aggregate-recovery` | `473133f9a9e990838e9c3ce2ee78fa42551956cc` | #780 merged | clean; `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-803-release-v0-2-91-aggregate-recovery` | preserve: Runtime close not proven |
| `codex/wi-821-no-resource-document-promotion-v2` | `5341ad8225406a6c8d5ece6fbe10a975caf71029` | #800 merged | clean; `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-821-no-resource-document-promotion-v2` | preserve: distinct historical snapshot/evidence |
| `codex/wi-801-release-v0-2-91` | `5df82b95b8d6b15d80a631e4c4037432e9c8aa96` | #777 merged | dirty (13); `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-801-release-v0-2-91` | preserve: dirty generated evidence |
| `codex/wi-810-release-recovery-mode` | `6c1ba6de9a5f21d7f7290d26024717cc60ab5eda` | #788 closed without a merged_at fact | clean worktree but local HEAD differs (`7e122560a5cfe894fc4279aac9eb0ee5b6f9a17a`) | preserve: closed-unmerged and divergent local evidence |
| `codex/wi-809-release-v0-2-91-final-acceptance` | `795b3a66f745e9d23d2fa12dfd97503f67559b42` | #787 merged | clean; `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-809-release-v0-2-91-final-acceptance` | preserve: Runtime close not proven |
| `codex/wi-809-release-contract-scope` | `873cc3ee180f6671b0dc128e3db84a51343b24bc` | #786 closed without merge | clean; `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-809-release-contract-scope` | preserve: unmerged PR |
| `codex/wi-818-evidence-contract-repair` | `8f9968343a031b75340a8d80d779c68dd1898a03` | #797 merged | dirty (1), local HEAD `582b86d8791a5ab1fed369f4c825af9aa905f7c5` | preserve: dirty evidence and local divergence |
| `codex/wi-816-outcome-action-binding` | `920cbc3685353859d049f4576d6b7b56bd9f5562` | #794 merged | clean; `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-816-outcome-action-binding` | preserve: Runtime close not proven |
| `codex/wi-808-release-handoff-recovery` | `9e07fb27bcd0054d994becc61b697f5446b35655` | #785 merged | clean; `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-808-release-handoff-recovery` | preserve: Runtime close not proven |
| `codex/wi-804-release-tag-input` | `a2f49b2eb7c7b721670f507cc3773183a36ad2cc` | #782 merged | no local worktree found | preserve: exact worktree cleanup not proven |
| `codex/wi828-release-source-identity` | `a402a35676ef9befdbdfa802ae7461b181a7dec8` | #815 merged | clean; `/private/tmp/ai-cockpit-wi828-source-identity` | preserve: Runtime close not proven |
| `codex/wi832-release-adopter-reuse` | `ce050baafd2cc9258c5a95fb986a691c5961f692` | #812 merged | clean; `/private/tmp/ai-cockpit-wi832-release-adopter-reuse` | preserve: Runtime close not proven |
| `codex/wi-811-release-recovery-finalize-order` | `cf2f5539d88ee8031a2314c44d492ae7bbc65811` | #789 merged | clean; `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-811-release-recovery-finalize-order` | preserve: Runtime close not proven |
| `codex/wi-806-release-preflight-base` | `d961db0734d44b6a0356614151411f49039943d1` | #783 merged | clean; `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-804-release-route-ordering` | preserve: branch/worktree binding is non-canonical |
| `codex/wi830-release-dispatch-syntax` | `d98da82f5cc22b789006e7000382a5a0c19dbfde` | #810 merged at `54efb51622528771047e453a937aa92c0f3612e0` | clean; `/private/tmp/ai-cockpit-wi830-release-dispatch-syntax` | preserve: remote tip has extra content |
| `codex/wi-815-post-release-acceptance` | `e377ce5649b5acf167c2f353846bed7333612d3a` | #796 merged | dirty (3); `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-815-post-release-acceptance` | preserve: dirty generated evidence |
| `codex/wi831-release-identity-final` | `ebd90e7482ceea70130904b85139aed6f67a2f2c` | #811 merged | dirty (5); `/private/tmp/ai-cockpit-wi831-release-identity-final` | preserve: dirty generated evidence |
| `codex/wi-814-archived-pr-gate` | `f06471c7191d4d06cfc4ee17b0a5f4b12b1a7f30` | #792 merged | clean; `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-814-archived-pr-gate` | preserve: Runtime close not proven |
| `codex/wi-807-release-preflight-clean-failure` | `e851dd0bc7a84de32d6eacfd9300a51738195519` | #784 merged | clean; `/Users/sei-rinn/dev/workspace_rust/ai-cockpit/.worktrees/wi-807-release-preflight-clean-failure` | preserve: Runtime close not proven |
| `codex/wi828-source-scanner` | `d7cd259c57aac23f65d0f23b6de8985243e188cd` | #817 merged | clean; `/private/tmp/ai-cockpit-wi828-source-scanner` | preserve: Runtime close not proven |
| `codex/wi828-release-v0-2-92` | `fafcb0012d4b6075665f8a29a96f26054bbe315c` | #808 merged | clean; `/private/tmp/ai-cockpit-wi828-release` | preserve: release Work Item lifecycle not proven closed |

Summary: 25/25 remote branches have a disposition; 0 are currently safe delete-candidates under the stated rule. Three earlier branches were deleted only after exact close/cleanliness checks. This report does not authorize deletion by itself; Runtime finalization and close evidence remain required for each Work Item.
