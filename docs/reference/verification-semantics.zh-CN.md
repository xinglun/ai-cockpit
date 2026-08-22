# Verification 语义

AI Cockpit 对 Verification 记录两个相互独立的问题：

| 维度 | 含义 |
| --- | --- |
| `VerificationTier`（`T0`–`T3`） | Verification 流程需要达到的强度或权威程度。 |
| `EvidenceAssurance` | 证据来源：`SelfDeclared`、`RepositoryVerified`、`ProviderVerified` 或 `EnterpriseVerified`。 |

`T3` 不等于 `ProviderVerified` 或 `EnterpriseVerified`。它只表示该要求
需要权威 Verification；最终 Assurance 取决于实际绑定的 Evidence。

`VerificationRequirement` 记录所需 Tier、所需 Assurance、原因，以及触发
要求的组织/项目/发布 Policy、Stage 和 Protected Gate 引用。Runtime 不会
从 Tier 反推 Policy，也不会静默提升 Evidence Assurance。未满足要求时，
治理缺口必须保持可见，不能靠展示层变绿。

生成的 implementation approach 属于仓库本地证据。Work Item 归档时，
它必须与 contract、summary、outcome、events、reports 和并行 intelligence
sidecar 一起移动，不能在 active 目录留下孤儿文件。仍持有的仓库本地并行
slot 会阻止归档，必须先显式释放。

Wire schema 严格校验（verification semantics `schemaVersion: 1`）：必须显式提供
`schemaVersion`；未知字段、未知 Tier 或未知 Assurance 都 fail closed。旧的 `AssuranceLevel` 消费者仍
保持 Wire 兼容；新代码应使用 `EvidenceAssurance` 名称。

实现证据：`crates/cockpit-protocol/src/lib.rs`、
`crates/cockpit-verification/src/lib.rs` 和
`crates/cockpit-repository/src/lib.rs`。
