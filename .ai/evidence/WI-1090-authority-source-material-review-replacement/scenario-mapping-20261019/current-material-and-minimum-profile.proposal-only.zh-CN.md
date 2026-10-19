# WI1090 当前材料与最小 profile 提案（未批准、未执行）

当前观察 HEAD `c1affb57188b152cfe15f00b4328302ca775d5d6`；治理 Runtime `b5cbd8570df5d3cb2004ae517fe88df6136c66d92fc1913070d18f2c8f67f257`。Contract `sha256:c9a3bda349078f1fc15891e9e789d100952cbadaea12eb464f2024c90617ce9c`；当前 request `sha256:2bfb58e93dd464897aeb4c3ef07c138e11372dba182ea70e578da267af14f233`；sourceSnapshot `sha256:edc336e22e8205a1738e7aa3670e186b4419c520c20c5e0dc81e9a63e9aa5505`；materialManifest `sha256:333c731ee12c7cde9baab8b87025b96827d8a4b3c7322a1b6be40f9336f7e46e`。

原生 plan 有8项：7 clean，1 structurally reviewable syntax Unknown，Finding0。reviewEnabled=false，reviewDiagnostic=material_review_not_enabled，profileDigest=null；当前不具备材料 record 准入。

唯一有限审阅项：`crates/cockpit-repository/src/material_review.rs`，完整 blob 166063 bytes，SHA `sha256:0ebba06d433ff1aa1a25fbc70d59d8735575958142438c89a813c6381d571a48`；changedHunkDigest `sha256:09435234cf7961bdc1248e876d9215cc57a5c61601c889f6946bcd43076c2e60`。从 immutable base b8ec269 到当前提交共21个零上下文 hunk、1700 added/15 removed lines；已按 Native 的 [addedLines, addedLineOrigins(afterLine,hunkIndex), removedLines] 序列独立重算并精确匹配 digest。该有限批准需要同时覆盖完整blob身份与精确hunk集合，不是泛化的未来文件批准。

完整blob/u0 patch/Native摘要输入保留在本外部包；源码本来就在已推送c1affb57的对应Git blob中。另一个Rust测试文件与六项文档在当前扫描下为clean；不把clean称为真人审阅已完成，也不把技术代码审阅当作当前risk consent。

## 一操作最小政策提案

`first-profile-opt-in.amend.request.proposal-only.json` raw SHA `ff93edd6797c0c6e973e1e3355024d84e96d4ca8d616604b37eec859785f323f`。expectedContractDigest为当前c9，仅对当前null的 `/governanceProfile` 作初次set。拟配置 materialInspectionReview schema1、仅 repository_material_inspection_unavailable/readable_committed_rust_syntax_unknown、self_declared、拟reviewer agent:Raydot，以及新的WI1090专用authoritySource路径。因为当前根是null，初次对象set与旧WI1089已存在对象的非法set不同；不复用旧44881提案。

`acceptResidualRisk=true` 是待审政策配置，绝非已有风险同意。该请求没有原生运行，没有写入Contract或伪造consent文件；当前新authoritySource文件不存在。scope、七项verification、base/risk/authority、sources及历史均不改。requiredRuntimeCapabilities应由Native派生，不能手改。

## 后续只有实际授权后才可依序进行

1. Raydot审阅这一精确政策提案；本次技术映射不自动授权opt-in或risk acceptance。
2. 若批准，原生amend并读取真正的新Contract/preflight/snapshot/environment/admission；这些未来digest当前未知，不静态猜算成已批准绑定。
3. 如Native要求，取得实际新Contract的独立confirm_review。旧c9确认不可重放；确认也不代替材料风险接受。必要的三语文档/parity投影仍在既有六路径scope内，按实际新绑定处理。
4. 在实际最终政策及必要文档变更之后重新读取真正的最终material plan。当前profile=null的request不是未来record绑定；比较所有成员/blob/hunk/scanner/policy。
5. 针对该实际完整绑定取得有权主体的新WI1090材料残余风险决定和实际reviewer结论，如实记录角色/来源/assurance。原生evidenceRefs仅结构校验，不认证真人。技术审阅、M6合成决定、WI1089或WI1087旧批准均不继承。
6. 只有Native实际准入后才材料record。再读状态与plan确定哪些验证确已失效及需要执行；本轮不预跑七项，不宣称当前c9收据必然对未来新Contract新鲜。

这一政策变更可能使现有验证/文档投影失效；由实际Native绑定判断，不能为省检查伪造复用，也不因还没执行的提案现在重跑检查。没有真实WI终态、消费者修约、Sentinel、合并、发版或Runtime采用。
