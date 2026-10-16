# WI-1086 最小修复实现检查点

Ray 本人真实绑定确认已落盘并通过 Runtime allowed 准入，见前一 human-review-20261016 阶段。当前 Contract 保持 5747b1fe…be119，同 Contract 批准继续复用。

## 实现

- validator 在写入任何新收据/Summary 前，要求 preflightContractDigest 等于当前 canonical Contract，且状态属于 green/yellow/red；missing、unknown、not_run 拒绝。已有同 Contract 真实收据的读取/复用规则不改。
- A 复用完整原生 prepared/committed journal 校验，再检查连续 Contract 变迁、链尾、当前身份、当前 invalidation hash/检查列表，以及原生 before_edit/revalidation 的身份、前驱、历史 snapshot 和验证失效事实。只在三个允许非终态加入请求生成 run_preflight；源快照变化仍由现有锁内 observation 校验。敏感修订 post-checkpoint 缺少这些事实时，旧 stale/material fallback 不代替其来源核验。
- B 保留 sensitive 前置分支及 Missing/Invalid 的原 state、outcome_state、blockers；只有无原阻断才投影 Yellow/needs_human_decision。合法 review 不清除原 Red/blocked，待办和事实继续可见。

## 真实测试阶段

validator 新4个边界负例原代码均失败，合法3状态+批准复用基线通过；最小修改后整份CLI/MCP amendment回归12/12通过。B的Missing原outcome被覆盖、Invalid原state被覆盖两个RED均真实观察，再转GREEN；每条同时断言三个维度及原Unknown。

A的documentation首错RED真实观察。Red fixture 的前3次探索（缺authority、已提交的越scope文件、缺schema的坏envelope）未得到期望的当前Red；保留这些失败日志，不计作governance_red互锁复现。补齐evidenceSchemaVersion=2的矛盾envelope后，CLI/MCP确实都因run_preflight不可用而失败（red-request-interlock-schema2），随后最小A修复转绿。保留矛盾证据原字节，绝未称其为历史passed验证。

当前整份CLI/MCP文件18/18通过，含两类请求生成、重复请求绑定已变化源码snapshot、不自动确认/不验证、三个允许非终态、14类坏journal/身份/hash/前驱/终态拒绝且无写入。另有B单元2/2、重算全部journal hash但Contract变迁断裂的专门测试1/1通过；该测试先证明原native reader接受单条hash和journal链，再证明新增当前绑定检查拒绝Contract链断裂。完整workspace、正式Runtime回执、Windows与技术最终审阅尚未完成。

source-bindings.json与source.patch固定这次源码字节；各轮元数据诚实记录源HEAD、当时测试文件SHA、工具链、命令、实际现存CLI路径/哈希。cargo --lib 观察到的CLI文件不代表该命令重新生成它。单build job/worker、隔离target、每次900秒上限；没有采用候选替代共享Runtime。

后续先提交源码、通过安装版Runtime刷新当前真实preflight/验证准入，按正常流程执行声明验证。保留原WI-1085、所有真实人审/失败证据及旧control/review资源。无合并、关闭、清理或实际Sentinel操作。WI-1087只做独立planned设计，不与共享产品文件并行。
