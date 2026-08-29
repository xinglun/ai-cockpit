---
author: AI Cockpit maintainers
title: "C# stack adaptation"
description: "C# adopter 向けの Rust-native、repository-bound な対応表。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-391-reference-csharp-adaptation
capabilityClaims:
  - csharp_adaptation_guidance
---

# C# stack adaptation

[English](csharp-adaptation.md) · [简体中文](csharp-adaptation.zh-CN.md)

このページは pinned reference の `examples/csharp/README.md` を節ごとに比較し、.NET repository
へ継承できる governance の意味だけを Rust-native に対応させます。第二の Contract schema、.NET
toolchain の保証、または第二 technology adopter acceptance receipt ではありません。

## 節ごとの比較

| Reference section | 継承する意味 | Rust-native boundary |
| --- | --- | --- |
| Source metadata | title と keywords は C# adaptation example を示します。 | target page は Rust documentation の canonical metadata を使います。source front matter は説明情報であり、capability や authority の record ではありません。 |
| Installation | source は immutable template tag/raw base を要求し、installer で C# を選択して adoption file を生成します。 | repository の外に immutable shared Runtime を一つ install し、archive/binary の SHA-256 を確認してから `attach --repo <path>` を明示します。Runtime は source installer を実行せず、Makefile を生成せず、provider を暗黙に選びません。 |
| Quality gates and guards | format、test、warning build、production/test path boundary を明示します。 | `dotnet format --verify-no-changes`、`dotnet test`、`dotnet build -warnaserror` は adopter/provider の責任です。path boundary は Contract scope/outOfScope または repository policy で表し、source YAML guard file は要求しません。 |
| Contract example | Work Item が identity、mode、scope、guidance、verification、acceptance を宣言します。 | 現在の Rust Contract を `work-item new` で生成し、人が所有する intent、scope、acceptance、authority、evidence 要求を入力します。source field 名は JSON-wire compatibility を意味しません。 |
| `guidelinesCompliance` example | Summary が human guideline の達成方法と evidence を説明します。 | Contract の `guidelines` を保持し、numbered acceptance evidence、`intentAlignment`、または delegated evidence で証明を bind します。型なしの compliance claim を追加せず、evidence なしで true にしません。 |

## source installer をコピーしない install と attach

Install は Runtime boundary であり、project template の生成ではありません。immutable public Release を選び、
archive と binary の SHA-256 を確認し、repository の外に binary を一度だけ install します。その後、各 repository を明示的に bind します。

```bash
repo=/path/to/csharp-repository
ai-cockpit --version
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

profile を confirm する前に検出された project fact を review します。Agent adapter install は別の明示的な repository
操作であり、global Agent/MCP configuration を変更しません。[Install AI Cockpit](../getting-started/installation.ja.md)、
[Adopter configuration](../getting-started/adopter-configuration.ja.md)、[Installed Runtime lifecycle](installed-lifecycle.ja.md) を参照してください。

Reference の `AI_COCKPIT_TEMPLATE_REF`、`AI_COCKPIT_TEMPLATE_RAW_BASE`、`curl` で取得する `install.sh`、
`--stack csharp --update-makefile --create-adoption` はコピーしません。これは source installer の flow です。
Rust route は Runtime delivery、repository scaffold、project policy、Agent discovery を別々に review できる boundary として保ちます。

## adopter が所有する C# verification evidence

次の command は project fact の例であり、Runtime が提供する command ではありません。

```bash
dotnet format --verify-no-changes
dotnet test
dotnet build -warnas-error
```

Human-owned Contract の準備後、installed Runtime で fresh result を bind します。

```bash
ai-cockpit verify --repo "$repo" --work-item WI-csharp-change \
  --command dotnet --args test
```

追加の check は別々の verification entry として宣言できます。repository の proportional `light`、`standard`、`strict`
profile が必須 check を決めます。強い Verification Tier は Evidence Assurance ではありません。Hosted CI、provider attestation、
enterprise control は delegated evidence のままです。

source の coverage 提案（production `src/**`、test `tests/**` または `**/*Tests/**`）は policy の考え方であり、必須 directory layout ではありません。
repository-relative Contract boundary と現在の scope-overlap validator を使い、不明または unsafe な pattern は fail-closed とします。

## 現在の Contract mapping

まず not-ready skeleton を作成し、fact と decision を分離します。

```bash
ai-cockpit work-item new --repo "$repo" --id csharp-change --mode code
```

Human owner は intent、goal、scope、out-of-scope、acceptance、authority、required evidence を定義してから `preflight` を実行します。

- `protocolVersion` と optional `contractVersion` は Rust protocol を識別します。source の `contractVersion: 2` と直接の wire contract 互換ではありません。
- `workItemId`、`mode`、`scope`、`guidelines` は governance の意味を保ち、repository-relative safety validation を受けます。
- `verification` は descriptive typed checks です。fresh execution の代わりにも permission にもなりません。strict/release `checkpointPolicy` が Agent Risk floor を提供し、source `ai*` 名を第二 registry としてコピーしません。
- `acceptanceCriteria`、numbered acceptance evidence、`intentAlignment` で観測可能な完了を保ち、guideline result を推測しません。
- `authority` は repository-local declaration です。human identity、provider approval、enterprise assurance は external evidence です。

## 継承 boundary と非主張

Attached C# repository は自身の `.ai/` と Agent adapter を通じて、shared Runtime から同じ reader-facing workflow、stop state、Outcome rule、evidence boundary を継承します。
repository identity、Contract、snapshot、Work Item、evidence は明示的な `--repo` で分離されます。

このページは `install.sh`、`Makefile.ai.stack`、source Python check、source guard YAML、source JSON example をコピーしません。また、この Rust repository が C# adopter acceptance を実行済みとも主張しません。
将来の multi-technology acceptance は別の evidence-bound batch とします。

[Reference parity](reference-parity.ja.md) では source command や JSON-wire compatibility ではなく semantic/documentation parity として記録します。
