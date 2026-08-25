---
author: AI Cockpit maintainers
title: "WI-262 Release version-consistency cleanup"
workItemId: WI-262-release-version-consistency-cleanup
description: "post-release version consistency の cleanup を決定的かつ fail-closed にする。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_cleanup
  - release_truth_preservation
  - isolated_release_regression
---

# WI-262：Release version-consistency の cleanup

## 目的

post-release の `tests/release/version_consistency.sh` は
`release-manifest.json` を隔離ディレクトリへダウンロードします。以前の
EXIT trap は metadata の一時ファイルだけを削除して `rmdir` を試みていたため、
成功したチェックでも manifest が残り、cleanup 失敗が黙って無視されていました。

この Work Item では cleanup を明示的な後置条件にします。成功経路と manifest
検証失敗経路の両方で隔離 download directory を削除します。cleanup に失敗した
場合は `release truth unchanged` を付けた fail-closed 結果を報告し、公開 Release
を rewrite または unpublish しません。

## Scope

- `tests/release/version_consistency.sh`
- `tests/release/version_consistency_test.sh`
- この Work Item の三言語文書

回帰テストは隔離 temporary root、fake `gh` provider、注入した cleanup failure を
使います。成功経路と manifest failure 経路で一時ファイルが残らず、注入した
failure が表示されて Release truth を変更しないことを確認します。

## 検証

```text
bash -n tests/release/version_consistency.sh
bash tests/release/version_consistency_test.sh
cargo test --locked --workspace
```

test wrapper は source fallback を build せず、GitHub に接続しません。fake provider
を workspace version に bind し、cleanup 結果を assertion します。

## Acceptance boundary

Cleanup は運用上の衛生であり、公開権限ではありません。cleanup failure は command
結果と evidence に表示しますが、公開済み Release を未公開へ戻したり、Release
metadata を変更したりしてはいけません。
