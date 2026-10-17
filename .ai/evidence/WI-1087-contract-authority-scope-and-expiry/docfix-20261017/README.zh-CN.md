# WI-1087 P0 文档投影 CI 修复

## 真实范围与准入

由 agent:Codex 根据 Raydot 在主线程01a11856-d6ba-72e0-9aab-25a34cd2f018的普通技术scope审阅执行；这不是human confirm_review。前置Contract7f827f8bf5b6eaacd599045f243f74ee3a89998bde781fc5aae65af83f877eee逐字绑定核对一致。Native sequence=3，changeId=wi1087-p0-planning-projection-scope-20261017，requestDigest=sha256:3b1cdf5462d35edb89ae830a4e2ee76dd230dd0d6ad7819d897f14f7c5c3f98e，journalDigest=sha256:65b35a075e3dd86f070f7e84656b53f0461c619ef450e8b3d6c2d28ea116bef0，newContractDigest=sha256:58529cee9b0eeb656132434a47c7f57374f1cf8e87cf8885710dd394192e7f1d。

唯一Contract变更是scope追加docs/reference/reference-parity.md、.zh-CN.md、.ja.md；其余字段、base、authority、验收、排除项均不改。原生prepared/committed记录保留。登记generation5匹配新Contract及实际head，Runtime环境基线67a5f6afcf18be42280c1dee66bfe0e26fb8e07911fc9980733b8ee32a0042d6；check返回pending=false，仅相对此环境基线，不推断旧环境连续性。

## 两个真实失败与六文件diff

此前CI38986839707源8fb70634b3a1d84637873ee5c51522f069138e50、tested3cd928fd00a0eaf6b5f911d6bad66761e06e29ee：docs_acceptance因三语WI简报缺lastVerifiedBy失败；docs_governance_integrity因三份parity缺预归档行失败。原诊断成员及ZIP来源sha保留，不改失败历史。

本次六文档各只新增一行：

- 三语WI简报新增lastVerifiedBy: agent:Codex，真实执行者仅完成本次文档检查；status仍planned、authority仍design-and-registration-only。不是产品验收通过或真人决定。
- 三语parity各新增一条条件预归档行，明确仅规划、planning/not_ready、authority=missing、checkpointCount=0、产品验收not_run、未来终态记录不声称存在；原有全部行和两侧内容保持。

当前gate强制条件模板为英文In progress → Implemented after verified close及对应中日文；这里的verified close是将来条件，不能删改gate或将当前状态写作verified/Implemented。解释列明确没有当前实现/验收通过主张。新增lastVerifiedBy是执行者元数据，不是产品验证凭据。完整六文件diff原字节见six-files.diff.json中的Base64；rawSHA与原始diff一致。该封装仅避免补丁上下文空格触发新增证据文件的尾随空格检查，不裁剪原字节、不改变gate。

## 本地实际检查

执行者agent:Codex；按单检查最大900秒边界执行，实际无超时：

- bash tests/docs/documentation_acceptance.sh：exit0，41.081秒；包含文档链接/元数据、三语状态、35参考记录、26当前命令示例等现有检查。
- python3 tests/ci/governance_integrity_gate.py --repo . --report <bound report>：exit0，916 WI，findings=[]。
- python3 tests/docs/work_item_status_consistency.py --repo .：exit0。
- git diff --check：exit0。

未放宽任何gate、规则或断言，未新增产品代码/产品验收测试，未运行Native正式verify。上述结果只证明文档检查，不替代P1–P4实施、真实授权或31项基础/18变体验收。

## 保留项与下一步

WI1085收据1020B/rawSHA d85bcc5d38ed159e1c92a409d0a0b8feeb36bb16c41826d0e11531e5c565a9b5；WI1086收据1038B/rawSHA 7304e582f78f5a49ec751f01f5497a3a310a66c5621fa3353813869c961a6a8d；主工作区原件、备份和当前工作树三侧一致。本轮不产生新清理、不改旧close/archive/receipt、不删除备份或stash。

Native当前not_ready、authority=missing；policyReviewRequired=true和humanDecisionRequired仍如实保留。未执行start/preflight/confirm_review，不生成假的preflightDecisionDigest，不以代理技术审阅替代后续真人门禁。

下一步正常提交推送当前分支，更新现有Draft PR1045并读取新head的CI实际结果。PR不转Ready、不合并、不删除，expiry产品功能仍未实施。CI日志及后续API观察可在交接中读回，不追加无穷证据提交。
