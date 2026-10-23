Outcome: 🟡 本轮最小补测完成；真实材料工作流仍待审阅

正式执行源码（含既有证据提交）：`2785cfb2f90a3d4763f1b8d0ee7bf20585635138`；最后测试源码提交：`5c5ce4a56cd43792014223292379c8813d48c836`。本轮仅修改既有 integration test 文件，生产代码未改。

- 新增一个参数矩阵，10个独立边界：foreign native identity、可单条原生读回的 disconnected predecessor、base 原生禁止及篡改反例、原生后续修约造成非末端来源锚点、直接 Contract 编辑、stale/foreign current review、P后真实 Rust blob/hunk变化、真实有效外来receipt历史pointer、Unix journal symlink。
- 每例先证明基线 eligible；变异后投影不给新决定入口或读查询 fail closed，record 拒绝；所有 `.ai` regular bytes、链接目标、路径集合及旧receipt不变，不生成新prepare/sidecar。base没有被称为成功原生修约。
- 首次定向测试exit101：攻击fixture prepared hash用了Value顺序；保留失败。改用原生typed字段序后1/1通过，非生产缺陷修复。
- 源码稳定后仅一次正式七项：7 executed / 0 reused / max1，419.18s，exit0。fmt、clippy及 lib14 / integration53 / status42 / CLI36 / MCP2全部通过。
- canonical raw SHA-256：`6d93218b5bcb8b32bb0dfd86bfa98e37fe700652f73f90cdcaeb17cd6759286a`；receiptDigest：`sha256:1808ad8ac5afa96dd18a7ab8e8c912116a9a9526c28eebe04c808374f810b573`。
- 22条原始条件重新对应新正式日志。M2/M5原生记录verified；M1–M6均verified。controls第一次因仅传5项而被原生拒绝，输出保留；第二次保留原有M6完整6项成功，无重复验证。

实际 status：checkpointed / verification verified+fresh / governance yellow；blocking=false、humanDecisionRequired=false。3个Unknown：intent_alignment_insufficient、repository_material_inspection_unavailable、user_visible_benefit_not_declared。原生全文Outcome返回“🟢 已声明的验证通过”，不是当前风险同意或工作项终态；它的治理理由仍包含意图对齐不足。Contract历史审阅保持原件，未重放。

当前Contract：`sha256:c9a3bda349078f1fc15891e9e789d100952cbadaea12eb464f2024c90617ce9c`。
最终材料request：`sha256:23e04c983732d47799e1ec444849fd89e0c92f3a5b13742ac636be30c8b6e3d8`；material sourceSnapshot：`sha256:ffdcd5af8387e969dec46902088117057f82d986ef948d5e3d69c09776c52329`；manifest：`sha256:13d21e04a58c46874688c8998c67ea2582efe9be74f0070af3f37ad4ccc943ac`。8项blob/hunk全部独立重算匹配，7 clean+1 reviewable syntax Unknown、Finding0。材料snapshot与原生verification repositorySnapshot属于不同域，不互换。

最小初次profile草案仍只set `/governanceProfile` 一项，raw SHA-256 `ff93edd6797c0c6e973e1e3355024d84e96d4ca8d616604b37eec859785f323f`，expectedContract=c9。profile目前null，未执行修约，未创建同意文件，未记录材料决定，未继承WI1089同意。实际未来Contract/preflight/material request仍须从原生新绑定读回。

M5.7实现验收是拒绝沿用旧批准/旧绑定，已由服务负例证明；真实WI1090风险同意是独立待办。请Raydot据最终源码及材料绑定审阅下一步政策/风险流程；当前草案不授权自动执行。

限制保留：仅合作lifecycle-lock下逻辑原子性，无power-loss/parent-fsync/不合作final-gap写者保证；self_declared不是真人认证。M6沿用实际ba1+33d79固定候选8e34…的两个合成fixture证据；reuse_not_configured仅证明真实执行，不证明已配置缓存失效或producer升级。未操作Sentinel或真实消费者修约/终态/合并/采用。完整3176路径原始清单未公开，原始证据仍保存。

影响：增强既有授权/历史边界的可验证性；用户可见收益仍是推断。原生Outcome已生成返回；宿主显示确认未知。下一步由Raydot审阅当前精确材料与最小政策草案，真实风险批准另行绑定实际新Contract/request。
