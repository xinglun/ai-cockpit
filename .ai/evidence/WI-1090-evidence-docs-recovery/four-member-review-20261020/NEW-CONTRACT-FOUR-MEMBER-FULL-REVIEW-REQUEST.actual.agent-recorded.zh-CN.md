# 新 Contract 精确审阅请求（原生实际要求，未确认）

Ray 已真实批准 a9f8bdda 四文件有限材料审阅和扫描盲区残余风险。它不等于本次新 Contract 确认，也不等于完成 Agent 审查。

原生修约 sequence6 已成功，只改变 goal 中材料描述、AC10 精确四成员边界和 authoritySource 实际批准来源；39路径、前9项AC、六检查、4 scenarios、protected runtime capabilities 和其他字段完全不变。独立技术审阅已由父线程报告可接受；不替代正式六项检查。

当前 Runtime 明确返回 `humanDecisionRequired=true`、`admissionState=needs_human_decision`、`reviewState=needs_human_confirmation`。原文："A previous human review cannot authorize a changed or foreign Contract." 和 "Record a new identity-bound receipt for this exact Contract and snapshot."

| 绑定 | 实际值 |
| --- | --- |
| contractDigest | `sha256:be0f85d58e29bcc709fde3a3a81d1e221afff96bb8989783d21acd649e42935d` |
| preflightDecisionDigest | `sha256:5ea29c1b4286388c29b7d2c30ab98931a18b00d5b3c4739c14a2b6cc77c27e8d` |
| repositorySnapshotDigest | `sha256:880e5602b421577f2ae0819db92b3911ba03885a3f217ac7a08b24c1c9efa7b4` |
| runtimeDigest | `sha256:e967d02a49806d0481272625bdc9813dc913a7aca1773aa734d4a095367b0a98` |
| environmentObservationDigest | `sha256:fd31dedbf592c5dd3ed01e3f9808175454ac3ee592babd823a1ff341da674ce4` |
| journalDigest | `sha256:5e783f96d6daa8d307df21c5b0cc3b68cb594bfe405e2f9b8e7871a5a01e12c0` |
| previousJournalDigest | `sha256:48dbd0e1e9b29eddbdbdc7fbca73ce6b05c389037e0d618d78a80a06adc8836f` |

- Work Item: `WI-1090-evidence-docs-recovery`
- Repository: `sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b`
- Branch: `codex/wi1090-evidence-docs-recovery`；当前证据保存前HEAD `a9f8bdda4333f23697bd6b97f1fab5a0dd8f9d07`，源码 `94e554ad836b19b4e027c3afda2159c4ace0fd18`。
- preflightAt: `2026-10-20T03:30:40Z`；checkpointCount1；decisionId=`contract-preflight-review`；请求=`confirm_review`。
- Runtime version1.0.1，路径 `/workspace/scratch/wi1090-additional-candidate-20261020/94e554ad836b19b4e027c3afda2159c4ace0fd18/ai-cockpit`。environment来自本次原生修约6，不冒用旧环境。
- Repository B=`b8ec2690976ab4c554aa9f5b2aa7fcca715156ca`；responsibility S=`cba3ea1661749ebbf1929bc523c5b86ff10eb7e7`；activationPrepared=`sha256:c7548ebca2858defb83b2aa472f07562cb41fd57f87cda2a0a13a5e4e76a98ab`。
- 当前材料 request=`sha256:f98cf3c3e5146c83cf10afd3c1046c9753fa240dd094d592235059fb5e8f4b6a`，manifest=`sha256:76009df3df2d11067ad3e78261ba55899349dce0046866e0723da893f724525b`，sourceSnapshot=`sha256:25b60b47c1e677df3b9174dbb61689be6f37841c7febb1828cb8839e69433e09`，profile=`sha256:711819ed3ca489e577a165080e3c305ee366cef0f87eed8580bed46ba4e18cf8`，policy=`sha256:4022dbdd989b6b05ee6116c7fc73b9465a373c1ed42b01999f77ea65d8d8921a`，analysis implementation=`sha256:ccf952067cbe6eba3dfa116f7a3d87293d69f6c7ddb5747953157ff63493cc83`。完整四成员blob/hunk和其余字段在同目录JSON；与a9f8bdda已批准集合完全一致，Finding0。

本次只交回新Contract的原生完整绑定；不重复申请四文件风险批准。记录新的真实confirm_review后，才继续真实Agent材料审阅、typed决定、当前六检查和自身收尾。当前无新正式检查、实际finish/archive或PR/CI执行。旧失败和C70ce验证保留，不能冒充新C结果。native amendment的verificationStarted=true表示之前已有验证，并非本次已启动六检查。
