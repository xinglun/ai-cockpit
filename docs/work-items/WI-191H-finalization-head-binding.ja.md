---
title: WI-191H — Governance finalization head binding
status: implemented
---

# WI-191H — Governance finalization head binding

WI-191 の不変な pre-merge receipt は archive commit `70c17e4` を正しく束縛しましたが、その receipt の commit により PR #152 は `8f5a025` に進みました。WI-191H は任意の head drift を identity とみなさず、この自己参照的な governance append を明示します。`governanceAppendRevision` を束縛できるのは最初の unmerged-to-merged transition だけです。3 つの resource head は同時に進み、Git は旧 head が ancestor であること、および range 内の全変更が同じ Work Item の通常 finalization receipt JSON の新規追加であることを証明します。cleanup transition は新しい head を維持します。foreign path、不正な receipt 名、symlink、非追加変更、非 ancestor revision、非 merge transition、後続の drift は fail closed です。
