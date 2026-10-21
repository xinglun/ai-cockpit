# WI-1091 隔离候选源码 WIP 备份

本分支只保存现有源码和未完成状态，未验收，不能据此合并、部署或宣称验证完成。
本次操作仅备份；开发、构建和测试均已暂停。没有恢复原 WI、消费 seq6 或有争议的批准，没有替代真人审批，也没有部署 B。

## 来源与提交边界

- 目标：`xinglun/ai-cockpit`，新分支 `backup/wi1091-wip-20261021`。
- 本备份的唯一父提交：远端 main `b8ec2690976ab4c554aa9f5b2aa7fcca715156ca`。
- 原云端源码基线：`964e267ebfb29adb58ee5d0ed4dbefcc6a614a27`。该本地提交不在目标远端，且其历史含原始证据；本备份不上传这段本地提交历史，而仅从已核验的远端祖先创建源码快照提交。
- 最新隔离冻结版本：input73，62 个 Rust 文件，输入 manifest 原始 SHA-256 `54d0b5283f809659dc0607f8f0461c46e55d623906468933ed58daf615fead6d`。
- 另包含原源码基线相对远端祖先的 6 个必要源码/测试 fixture 文件，保留候选所依赖的代码上下文。共 68 个源码/fixture 文件，均按原字节复制；本说明是唯一新增的非源码文件。
- 没有改写原 WI 工作区；没有更新其 Contract、Summary、审批、收据、历史或证据。

## 当前真实状态

最新 input73 **不能编译**。其定向命令于 2026-10-21 09:00:51–09:01:07（Asia/Tokyo）结束，退出码 101：

1. 新增 `business_regressions/historical_input_race.rs` 重复定义 `ID`（E0428）。
2. 同文件缺少 `plan_resource_finalization` import（E0425）。

因此该新增竞态回归尚未运行，不能称为已经复现失败或已经修复。收到仅备份指令后，保留上述未完成源码，未继续修正或启动验证。

独立静态复审最新发现的未闭合 A 问题：`record_historical_finalization_recovery` 捕获并绑定输入 A 的原始摘要，执行 body 比较一次后仍将外部路径交给业务函数；业务函数等生命周期锁后再次读取该路径。持同一 WI 锁的协作写者可能在此期间将合法输入换成 B，形成“批准 A、读取/执行 B”的竞争。候选中已添加尚未编译成功的真实持锁写者回归草稿与仅 cfg(test) 的观察 hook，**生产输入不可变修复尚未实现**。它不能笼统归入 B。

候选 A 中已有的修正包括私有测试授权端口与生产不可用工厂分离、历史完成事实与当前权限分离、精确 epoch/receipt/current binding、Running/Unknown 防重执行、实际 effect 摘要、source observation 的 R0/R1 分离、固定来源材料 kernel 桥接，以及材料 receipt 更新后原生 preflight 刷新新 P、旧批准失效、typed recorder 和历史投影一致。以上为实现状态；完整最终验证仍未完成。

生产授权工厂仍为 `UnavailableAuthority`；mock 仅用于显式 cfg(test) 端口。B 的真实持久保护/外部竞争保证未部署，28 项真实公有或跨进程正向仍独立阻塞。任何私有单生命周期测试结果均不能替代 B。

## 已有阶段结果：严格按输入区分

以下只摘录原地保留记录中的已知终态，不是对本备份树重新运行的结果，不能跨输入累加为全部通过。

| 输入 | 已知命令范围 | 实际终态 |
| --- | --- | --- |
| input60 | 独立 workspace/all-targets/all-features typecheck | 退出码 0；2026-10-21 07:29:50 Asia/Tokyo 完成 |
| input64 | 8 个指定目标负例，精确拒绝原因；真实 CLI/MCP 与私有 native fixture 分开 | 8/8，退出码 0 |
| input67 | source observation 全组和复审补测 | 23/23，退出码 0 |
| input71 | 4 项复审补测：材料先审后 typed verification/finish 同 commit；历史 recorder late/None/Unavailable；archive 时间矩阵；确定性 Running 防重执行 | 4/4，退出码 0 |
| input72 | 严格 workspace/all-targets/all-features Clippy，`-D warnings` | 退出码 0；2026-10-21 08:08:16 Asia/Tokyo 完成 |
| input73 | 新历史输入持锁竞争回归 | 编译退出 101，测试未运行 |

旧全库 timeout 及失败记录继续保留，不改称成功。最终同一冻结输入的完整可运行矩阵、全库分片、Clippy/typecheck、174 项逐项终态归并及独立复审证据包尚未完成。此前各输入的通过数不代表 input73 已通过。

## 明确排除与剩余未备份内容

- 原始 stdout/stderr、原始执行日志、事故证据、审批/人工响应、Runtime 收据及 `.ai`、`.git/.ai-cockpit` 中的本地新增或修改记录：均留在原环境，不加入本提交。
- 本地未上传的治理提交历史、历史 Git fixture 对象/材料导出、原工作项文档和参考账本改动：未转化为此分支上的新治理证据。
- Cargo target、编译产物、缓存、临时运行目录、凭据及个人配置：不加入本提交。
- 原隔离目录中的执行脚本、完整 manifest、扫描/审查记录及逐输入原始日志仍在云端。本说明中的源码校验表足以核对本次选定源码，但不能代替这些运行证据；本分支也不保证无需原历史 fixture 即可重现全部验证。
- 本次备份不宣称解决上述输入竞争，不宣称 B 部署或完整验收。后续只能在单独恢复开发授权后继续修正和验证。

## 源码逐文件原始 SHA-256

`candidate73` 为冻结候选原字节；`base-source` 为原本地源码基线相对已核验远端祖先的相关源码/fixture 原字节。

| 文件 | 来源 | SHA-256 |
| --- | --- | --- |
| `crates/cockpit-cli/src/main.rs` | candidate73 | `98c8b8ce1867d84983d3a908549eefeb3ef721ed604ed546045b75aaa69e88c0` |
| `crates/cockpit-cli/tests/closeout_recovery.rs` | candidate73 | `c7af75bb70a0858ba55b04ce842dfdad271820a08400c49356398e91510e5e13` |
| `crates/cockpit-cli/tests/contract_amendment_processes.rs` | candidate73 | `eb36f02edae55cc30795548ce2ec2a622f6e0b9ec0383f9bedf32b8b9a3d2026` |
| `crates/cockpit-cli/tests/legacy_closeout_source.rs` | candidate73 | `576bd58501ddd1fb458e6a06994245c3d882187f63e1d803f0c6567c43d4144a` |
| `crates/cockpit-cli/tests/material_review.rs` | base-source | `115f4f973c2ace4a6b8bc6fdf12881d0c6d6226a7c49f562bcfa6d648cdfab56` |
| `crates/cockpit-cli/tests/outcome_human_decision.rs` | candidate73 | `eae1be933ba43f6d25d7be201dfd3c8591e74320a19a49aca0565b1c544a5173` |
| `crates/cockpit-cli/tests/review_authority.rs` | candidate73 | `20a46b25ec21493e2a39d36ecf3be073f66acfbe0455b6fe3eb30ef251eb7660` |
| `crates/cockpit-cli/tests/verify.rs` | candidate73 | `1b30cafcff840a1ac102f5e4ed89158f7a254847397bfd2d6fd7abb4b8026019` |
| `crates/cockpit-git/src/lib.rs` | base-source | `a9c9438bc664bead3c1be1a913d60368e3cb95bcefdefb3bdfff6b0f1df66dc8` |
| `crates/cockpit-git/src/material_snapshot_v1.rs` | base-source | `2b5b28520f51463d5acc8eb762f0d18e2e36d740e269a0f35772c92bd4ca2c7c` |
| `crates/cockpit-mcp/src/lib.rs` | candidate73 | `070149d35d6d6aecf73300d6a56cc39599413da46bcc4abf7092e82b694a67ba` |
| `crates/cockpit-mcp/tests/material_review.rs` | candidate73 | `a49945db9dc157686a5af5221bd5a96c830a4a581c44763cf6067ce9917953b5` |
| `crates/cockpit-mcp/tests/review_authority.rs` | candidate73 | `7f60a1a8a4e92fa5504643344c279659518a227ac067f7b020f5b76a6da4f650` |
| `crates/cockpit-mcp/tests/rpc.rs` | candidate73 | `f43d40b214f462be7db3579b7e2f379e8b400f95d43784efa6932dca03452235` |
| `crates/cockpit-mcp/tests/usage.rs` | candidate73 | `951b8693dff799a165f00a09becb16cc3ebe80a6a9501b5dacd97532f3e039d0` |
| `crates/cockpit-protocol/src/lib.rs` | candidate73 | `fc7b63928a388171d10e8b8ad55d22d84dec4384f4d4df6d6dc41018ddf10576` |
| `crates/cockpit-protocol/src/review_authority.rs` | candidate73 | `a91ffbf8483a9758293dced07641e64b6ac94c75f4b703b5dd6b540d4fac9667` |
| `crates/cockpit-protocol/tests/material_review_versions.rs` | base-source | `f1c55eb1062fc31f97b597aca4f8cbe105cdc950801444f52d3e57d5c7a5b740` |
| `crates/cockpit-protocol/tests/review_authority.rs` | candidate73 | `c5fe2ff5dc2d241cdaaee70177bb67e3dda325dc4fa177eeeb3b919e88a44182` |
| `crates/cockpit-repository/src/action_admission.rs` | candidate73 | `4a166606564b94ef1f7d4d970a0ac60ec3a8b220d09a9dfa6b60c79460622e57` |
| `crates/cockpit-repository/src/business_regressions/active_work_item_retirement.rs` | candidate73 | `771ea9304293e01365bb4093f78395bb975b7b9b8b1456fa6d273bfb6d1310b3` |
| `crates/cockpit-repository/src/business_regressions/approved_boundary_fixtures.rs` | candidate73 | `b517d356d8ef674546ec33319c0cea109fb5b0c7b755ee271d304159d2eaf145` |
| `crates/cockpit-repository/src/business_regressions/archive_integrity.rs` | candidate73 | `c2a35558964584f0f29c93d2ede430082283c87a751a111798a2cdc6ab0f850f` |
| `crates/cockpit-repository/src/business_regressions/audit_query.rs` | candidate73 | `b5f70246a6f6f1efa033336472ff1ff8ad258e618fee4c5e202db9d850d54aaa` |
| `crates/cockpit-repository/src/business_regressions/ci_quality_gate.rs` | candidate73 | `a0f3ec72c1f20cdd377ccc090e3ff132e61fbf38ece85e79c95b3ce7fa263850` |
| `crates/cockpit-repository/src/business_regressions/cli_activation_negative.rs` | candidate73 | `e01f5116ada351b503b0d28b5cb4fe849675072b191b31be01b6e03c17a08632` |
| `crates/cockpit-repository/src/business_regressions/cli_closeout_recovery.rs` | candidate73 | `07cfe522051fb00f71019ba2d98a0c743ec6b0bfa0154fb848f184c824984188` |
| `crates/cockpit-repository/src/business_regressions/cli_legacy_closeout_source.rs` | candidate73 | `7780789ca6d0999f6bbb9d941f834bd81ef1297e807c61a20f1022d243abb5ef` |
| `crates/cockpit-repository/src/business_regressions/cli_outcome_negative.rs` | candidate73 | `47c7dcf1378f336d44283d07d08774683053683d44ca9c1afc3b6fdb64f2294a` |
| `crates/cockpit-repository/src/business_regressions/closeout_recovery.rs` | candidate73 | `72f8b774c025f2d3f73d1e0eafedece79ab32cd03f748e1dcae3d2b7b779939f` |
| `crates/cockpit-repository/src/business_regressions/contract_amendment.rs` | candidate73 | `d9948ad8de86b96da5f5dc5372923406988e7e855120c8924fe4b9813089007f` |
| `crates/cockpit-repository/src/business_regressions/contract_preflight.rs` | candidate73 | `439279c2be1f47d486f1543344ab32d415b90fd04c17b155f441131beb65662f` |
| `crates/cockpit-repository/src/business_regressions/historical_input_race.rs` | candidate73 | `eab077e08ebe246e62a0e1b5a71ee4ff7f830bd252e57539a45aa8ae8939523d` |
| `crates/cockpit-repository/src/business_regressions/material_review.rs` | candidate73 | `706e50ead9e7571da0ff553ca5937cdec7f05c8f7f7f6881eed8cf9f3b23e6b9` |
| `crates/cockpit-repository/src/business_regressions/mod.rs` | candidate73 | `397de51011fbf8751e69f145c62595d0b49d79be0827cadb193dd38f2d1c2388` |
| `crates/cockpit-repository/src/business_regressions/transport_fixture.rs` | candidate73 | `434afb5d4629342e00657312daaacbed8df66bccdf258ccd298265895995f553` |
| `crates/cockpit-repository/src/contract_amendment.rs` | candidate73 | `142e835b5b7ca3ac46cd7668f2c2f61353e02c54d5080171e35aab211d9ff289` |
| `crates/cockpit-repository/src/evidence_store.rs` | candidate73 | `a3d98913d3433f7e1bdd004a3caa7f695dcbf72a9473d9f787017c411a740153` |
| `crates/cockpit-repository/src/execution_context.rs` | candidate73 | `6e34033e756e66b4ddee32881400751c7c9e9623ed04b242caa075e25197f93a` |
| `crates/cockpit-repository/src/governance_controls.rs` | candidate73 | `aa673727658180f3dd808ef55128e04f9ad6ab00095f14fe9887a5152e1f4286` |
| `crates/cockpit-repository/src/legacy_closeout_source.rs` | candidate73 | `27294838f5ce3ef6521b4333a91bf5338d836bc4cc157adeac10d46f486ab360` |
| `crates/cockpit-repository/src/lib.rs` | candidate73 | `d90f6af80ef9d437ba7023e34e5bf9cf04b280c940800fb364c25bed40b21348` |
| `crates/cockpit-repository/src/lifecycle.rs` | candidate73 | `7cf2a33ef12a77835ef383086c74fc55df144fe1b023958c9ca34e0db0e17b55` |
| `crates/cockpit-repository/src/material_analysis_v1.rs` | candidate73 | `5dba4cd82efba72ede96658100c2b179db6680b298b82edf7237afe39ee4993d` |
| `crates/cockpit-repository/src/material_recipe.rs` | candidate73 | `b826459e4d9e8a851c41c6c2e8481a0dd54995ae52647b4947412a01ae3304d1` |
| `crates/cockpit-repository/src/material_review.rs` | candidate73 | `a5b7cc5d2f48cf779402fd2d4231b786871e7a08db2053bce0f603c6a6539be2` |
| `crates/cockpit-repository/src/resource_lifecycle.rs` | candidate73 | `909e80f333071a91811c8d267c81ca9d210da20dca57ac5ad2429393b06f4ac2` |
| `crates/cockpit-repository/src/review_authority.rs` | candidate73 | `d1167fefe689a85ee248a282bd7c4232b703abdf725ea37cddcb006586e255db` |
| `crates/cockpit-repository/src/review_authority_fixture.rs` | candidate73 | `85ecf351945da111e3d83822e11213f7a714a0da9f1582e6e5c84b83dd50d3ae` |
| `crates/cockpit-repository/src/review_authority_lifecycle_regressions.rs` | candidate73 | `37ff2ecac05f7d0e775b70038186261060a12a0d5fb41cfb6a9623cdc6a649ae` |
| `crates/cockpit-repository/src/review_authority_regressions.rs` | candidate73 | `f6741da0a8a3c3b440407c5141b061c4618c55499baebb84b6712e42baab0f85` |
| `crates/cockpit-repository/src/review_authority_v6_regressions.rs` | candidate73 | `2169f7730b285f39322d61cc76bfd2807305df1b1fd500eb2e3497f3ec4a88f9` |
| `crates/cockpit-repository/src/review_ledger.rs` | candidate73 | `cdd5dd4baf622dea63e91ccfaf20d41d7a73c195769ce34d95143fe4b1890be5` |
| `crates/cockpit-repository/src/rust_material.rs` | base-source | `fedf2bd1f8c516b633d0ad3581db4dd01ec42aa001fcc8257827f36ac4a18588` |
| `crates/cockpit-repository/src/source_observation_regressions.rs` | candidate73 | `7191d2738ce201b60c8d680c986145301ff3c3e1e1c80e1e5eee4fd31be9d7ad` |
| `crates/cockpit-repository/src/status_projection.rs` | candidate73 | `86830d9269754aa1a105df12c56fff7e1697f39ab9ed97661ec8b75c01a7f6a6` |
| `crates/cockpit-repository/src/usage.rs` | candidate73 | `4453dc634d403eef3f96783eb082c5c9ddaf4d6eee1dfd89ef7841d560b973e4` |
| `crates/cockpit-repository/tests/active_work_item_retirement.rs` | candidate73 | `4e45e6eb2cd010ade59d450a9ddc397dae5b0d262dfbb6a85d45df20c2d6e47a` |
| `crates/cockpit-repository/tests/archive_integrity.rs` | candidate73 | `6d42d040f913319969a798e74a1877345cdfe2cb6bbe8618743d7dff9d8b9343` |
| `crates/cockpit-repository/tests/audit_query.rs` | candidate73 | `b2fa178885b8690fd7872b3408b97dfb2bf97cac55a620905b11e81824088e4f` |
| `crates/cockpit-repository/tests/ci_quality_gate.rs` | candidate73 | `3efef1e29da31625373bdd5f5c3c858e66290332e514804e00fb6bf9c9562c73` |
| `crates/cockpit-repository/tests/closeout_recovery.rs` | candidate73 | `354ff0caede4466646d05625cbf6a9a8019f28c3b2b58d0b485b801dd058e9a2` |
| `crates/cockpit-repository/tests/contract_amendment.rs` | candidate73 | `84adc66c7554c950c27fbefa9f8bcb26330756b5bf37b112226a56e9b157d9fd` |
| `crates/cockpit-repository/tests/contract_preflight.rs` | candidate73 | `d2158a697105032a17f73fc2c61a5eaaa040f00d4b2f0d94ee6b7052d4adea00` |
| `crates/cockpit-repository/tests/fixtures/material_review_versions.json` | base-source | `ae26f62dd6da3ca72da9bc5b15d336fe253c138f9a6b9fd3338c62a23e86cc81` |
| `crates/cockpit-repository/tests/lifecycle_order.rs` | candidate73 | `42e9b134f5c8bb15c23c30a726d734554d5b94d8ab7650eecfd0faf72df1be5f` |
| `crates/cockpit-repository/tests/material_review.rs` | candidate73 | `0a7db548e639b872f77cc5a5975a26dbec48155151713616a40a976a56367f30` |
| `crates/cockpit-repository/tests/review_authority.rs` | candidate73 | `fc650b336d331e300e7346fffe3a6672d63f093bafc5e41efcac7caabde97379` |
