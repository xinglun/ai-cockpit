# Verification semantics

AI Cockpit は Verification について、二つの独立した問いを記録します。

| Dimension | 意味 |
| --- | --- |
| `VerificationTier`（`T0`–`T3`） | Verification 手順に求める強度・権威性。 |
| `EvidenceAssurance` | 結果 Evidence の出所：`SelfDeclared`、`RepositoryVerified`、`ProviderVerified`、`EnterpriseVerified`。 |

`T3` は `ProviderVerified` や `EnterpriseVerified` を意味しません。
権威ある Verification が必要だという要求だけを表し、最終的な Assurance
は結果に実際に束縛された Evidence から決まります。

`VerificationRequirement` は必要な Tier、必要な Assurance、理由、および
要求を発生させた Policy、Stage、Protected Gate の参照を記録します。
Runtime は Tier から Policy を推測せず、Evidence Assurance を暗黙に昇格
させません。要求を満たさない場合は Governance gap を可視化し、表示だけ
で Green にすることはできません。

生成された implementation approach は repository-local Evidence です。
Work Item の archive 時には contract、summary、outcome、events、report、
parallel intelligence sidecar とともに移動し、active ディレクトリに孤立
ファイルを残しません。保持中の repository-local parallel slot は archive
を阻止するため、先に明示的に解放する必要があります。

Wire schema は strict です（verification semantics `schemaVersion: 1`）。
`schemaVersion` は必須で、未知の field、Tier、Assurance は fail closed になります。既存の
`AssuranceLevel` 利用者は Wire 互換を保ち、新しいコードでは
`EvidenceAssurance` を使用します。

実装 Evidence：`crates/cockpit-protocol/src/lib.rs`、
`crates/cockpit-verification/src/lib.rs`、
`crates/cockpit-repository/src/lib.rs`。
