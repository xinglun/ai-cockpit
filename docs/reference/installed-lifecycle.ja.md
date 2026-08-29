---
author: AI Cockpit maintainers
title: Installed Runtime lifecycle
description: Installation、repository attach、upgrade、rollback、uninstall の境界。
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/installed-lifecycle.md
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - shared_runtime_lifecycle
---

# Installed Runtime lifecycle

[English](installed-lifecycle.md) · [简体中文](installed-lifecycle.zh-CN.md) · [日本語](installed-lifecycle.ja.md)

Installation は machine に一つの shared `ai-cockpit` Runtime を置きます。Repository を attach したり project を選んだり、全 lifecycle が完了した証明を作ったりはしません。Attach は明示的です。

```text
ai-cockpit attach --repo /path/to/repository
ai-cockpit inspect --repo /path/to/repository
ai-cockpit doctor --repo /path/to/repository
```

Repository は `.ai/cockpit.toml`、Contract、evidence、Knowledge、adapter record を所有します。Runtime に persistent な current repository や global active Work Item はありません。

## Release と repository boundary

Install と upgrade は名前付きの immutable public Release archive と SHA-256/manifest を使います。Release distribution、Homebrew、SBOM、provenance、rollback、post-release adopter acceptance は[Release and distribution](../release/distribution.ja.md)に記載され、repository-local Contract の外部です。Moving branch や workspace binary は release evidence ではありません。

Runtime-only upgrade は通常 repository bytes を変更しません。Schema migration は別の明示的で review 済みの operation とし、plan、backup/rollback evidence、human decision が必要です。Runtime upgrade は historical evidence を書き換えません。

Uninstall も proposal と execution の境界です。Repository owner が明示的に disposal を認めない限り evidence を保持します。Local binary を削除しても installer、provider、sandbox、enterprise retention の完了を意味しません。

### 安全な uninstall

Repository owner が installed Runtime または repository attachment の削除を決めた場合だけ使います。最初に read-only で存在する AI Cockpit file を記録し、record を保存するか purge するか確認します。次に write しない removal plan を作り、対象 path、Unknown、recovery route を review します。実行前には別の confirmation が必要です。承認済みで範囲を限定した removal だけを実行し、無関係な project work は削除しません。終了後は removal receipt を検証して evidence を保持します。ownership、scope、recovery が Unknown なら停止して repository owner に相談します。local binary の削除だけで完全な disposal とはみなしません。

## Reference source との対応

Reference の Python installer stage、Make target、generated status、migration record は conformance material であり、コピー対象ではありません。Rust は shared Runtime、明示的 repository context、typed receipt、public artifact acceptance harness を使います。Provider/enterprise operation は外部の検証可能な evidence reference として残ります。
