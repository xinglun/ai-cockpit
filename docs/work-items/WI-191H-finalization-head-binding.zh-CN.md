---
title: WI-191H — 治理 finalization head 绑定
status: implemented
---

# WI-191H — 治理 finalization head 绑定

WI-191 的不可变 pre-merge receipt 正确绑定 archive commit `70c17e4`，而提交该 receipt 又将 PR #152 推进到 `8f5a025`。WI-191H 显式表达这种自指式治理追加，而不把任意 head 漂移视为身份变化。只有第一次 unmerged-to-merged transition 可以绑定 `governanceAppendRevision`；三个资源 head 必须同步推进，Git 必须证明旧 head 是祖先，并且区间内每个变化都必须是同一 Work Item 新增的普通 finalization receipt JSON。cleanup transition 保持新 head 不变。foreign path、错误 receipt 名、symlink、非追加变化、非祖先 revision、非 merge transition 与后续漂移全部 fail closed。
