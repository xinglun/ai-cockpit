# WI1090 实际 finish 边界：集中最小修约提案

状态：**仅提案，未执行修约、未修改额外源码、没有新材料残余风险接受。**

## 已完成的实际交付

Ray 对70ce/3798/7b1e完整绑定的2026-10-20T02:09:31Z确认已原生录入，recorder agent:Codex。实现提交4237f5ec394c40a62b39b9b4ed8d2ee0fd9140fe，验证证据提交ce57c15c6d4a01bebd0def235a8ca64ae093ec8c均已推送。四实现、三回归、三流程文件及原自身三语投影在已批准37范围。两项真实文档门禁、fmt、三个测试目标正式6/6；execute6/reuse0/max1，匹配测试7+1+1。原生S=cba3，仓库B=b8ec；all3 ownership由实际Git差异登记，继承Rust2不进入本项责任。首次真实失败、27历史记录、manifest2、exact8和其余19项不变。

证据提交后native verification仍fresh、humanDecisionRequired=false，没有重复验证或确认。实际finish仅执行一次，于02:36:31Z exit1；未执行archive、Draft PR或新CI。

## 实际停止原因与新发现

当前唯一扫描Unknown是未改动的继承文件 `crates/cockpit-repository/src/material_review.rs`：readable_committed_rust_syntax_unknown；当前新增7个Rust文件均clean，Finding为空。继承该文件不等于已有当前Contract的风险接受。当前profile未启用，旧WI/旧Contract回执不能解除当前Unknown。70ce确认明确不含材料残余风险接受。

原生状态在finish之前及失败之后都仍给green、verification=fresh、safeActions包含finish、humanDecisionRequired=false。真实finish的canonical material guard拒绝，并原生保存outcome.state=blocked/failedGate=finish.lifecycle；返回给人的Outcome却显示绿色已声明验证通过。此为需要根因修复的实际流程缺陷，不能将用户提示当作修复。

已定位两处最小缺口：

- `crates/cockpit-repository/src/status_projection.rs:1456,1801` 合并材料Unknown但只在projection unavailable/Finding时阻断，checkpointed分支仍直接推荐finish；应把现有canonical material终态条件与动作准入对齐，保留前置收集/验证合法动作，返回完整缺失条件，不放宽材料守卫。
- `crates/cockpit-repository/src/lib.rs:12612` 只凭旧verificationRecoveryReconciled等于当前receipt digest且receipt完整，就跳过persisted blocked Outcome；没有证明该verification晚于这次新的失败。应绑定最新失败/恢复因果，不能旧成功覆盖新失败；未知或同秒顺序不明须保留阻断。材料条件真正解决时允许原生finish恢复，不凭证据提交要求重验。

这两个源码文件不在现37范围，实际修改前需要Raydot批准精确扩展。原7个Rust路径及三个回归文件保留；新增源码仅这两文件，scope共39。沿用现三套回归的source_change_observation_前缀和原六命令，无新框架、命令、controls、协议schema、CI/gate-policy或全局配置修改。

## 有限材料审阅政策（尚未授权）

提案单独启用已有governanceProfile.materialInspectionReview：reviewerActor=agent:Codex、assurance=self_declared、仅permittedCause=readable_committed_rust_syntax_unknown。实际授权来源标识仅为拟议值，须有真实Ray决定才成立。原生amend协议会自行派生受保护material-inspection-review capability，不手改protected数组。

有限授权只覆盖同一继承文件、固定after blob0ebba06d433ff1aa1a25fbc70d59d8735575958142438c89a813c6381d571a48、固定changed hunk09435234cf7961bdc1248e876d9215cc57a5c61601c889f6946bcd43076c2e60。新增/外来/不可审阅Unknown、Finding、该文件或语义绑定变化均不在授权内，须停并返回实际新绑定。self_declared不证明扫描完整，也不作真实身份认证。

## 执行和回归要求

Raydot先审阅同目录十操作严格提案，原AC3、九条既有验收、37路径、六命令及终态禁令保留。若技术批准，按实际原生amend生成新Contract、journal、preflight与材料request，不能将当前70ce/dc338当作未来决定。只在真实新Ray确认明确包含拟议有限委托及残余风险接受后，原生记录当前决定并在实际准入允许时改额外两文件。

用现有回归增加：已验证但材料Unknown未解除时不推荐finish；实际较晚失败不能被较早reconciliation隐藏；同秒/未知顺序拒绝；未知未解除不伪装绿色完整交付；当前合法材料依赖解决后无新增手动刷新、重复确认或无变化重验，保留历史失败；CLI/MCP读取一致。保持现有真实三路径ownership、并发、漂移、rollback与恢复测试。

只在源码固定、资源足够、原生准入允许时构建一个额外隔离候选；保留b5和source4237/9e7f旧候选及所有历史。实际验证、材料Agent审阅、当前fresh receipt、finish/archive按native admission推进；现有失败不能删除或篡改。不自动重跑六检查或CI。本提案未给出未来digest，未来必须读回实际绑定。

Draft PR正文可预备，但按既定交付顺序，在自身finish/archive实际允许完成后才发布独立Draft PR并观察真实CI终态。无merge、close、删除、发版、共享Runtime采用或Sentinel操作权限。
