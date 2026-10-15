# Exact batch cache proposal — approval pending

This is a read-only proposal, not deletion authority. Only the separately recorded old incremental cleanup was approved and completed. No new large build is running.

Manifest SHA256: `8e12a74c56ea839341532475e0fca0eeb77000c0f65552d482e9ddc9d336e539`. It lists2112 regular non-executable files. Expected reclaim after deduplicating inodes and excluding retained hard links:2330873856 bytes(2.17GiB); projected workspace availability2966331392 bytes(2.76GiB) from the timestamped inventory. Actual freed bytes must be measured after any later explicit authorization.

| Candidate root (files from manifest only) | Files | Estimated reclaim bytes |
| --- | ---: | ---: |
| `/workspace/.home/.cache/ai-cockpit-verify-target/ec9dc514276685dc3ff47011e8497b3987b3c90466848455758df44466684773/debug/build` | 146 | 471040 |
| `/workspace/.home/.cache/ai-cockpit-verify-target/ec9dc514276685dc3ff47011e8497b3987b3c90466848455758df44466684773/debug/deps` | 871 | 1383702528 |
| `/workspace/.tmp-wi1083-target-00461714-20261015/debug/build` | 260 | 843776 |
| `/workspace/.tmp-wi1083-target-00461714-20261015/debug/deps` | 835 | 945856512 |

All selected files are regenerable old static/compiler/generated intermediates. There are no active cargo/rustc/Runtime build processes in the recorded observation. Each selected path is non-symlink and mode/inode/device/size/mtime-bound. The containing directories are not proposed for wholesale deletion. Old debug/deps executable/dynamic content remains protected. Runtime/test/build-script executables and dynamic/debug libraries, all source/.ai/evidence/uncommitted content, the current WI1084 target and all WI1078 recovery resources are excluded. Ten exact Runtime aliases and their SHA256 values are recorded in proposal.json.

/tmp candidates were observed but are excluded from this batch and not counted toward workspace capacity; it is a different filesystem. initial-inspection-including-build-scripts.json is the broader read-only inventory, NOT the proposed deletion list. candidate-files.manifest.json is the sole exact proposed list.

Before an approved deletion, re-check no active build uses the old caches; verify the exact manifest hash, all paths/modes/non-symlink parents and stable file identities; record df/df-i and protected Runtime hashes before and after. Do not expand the list. The next long verification waits for the coordinator's exact batch storage authorization.
