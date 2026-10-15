# 旧无 origin v1：保留历史的后续提案（仅方案）

## 当前事实与原始Contract范围

来源为 GitHub #1004 的 Ray 本地验收回报 comment6191800813。固定源码b1487517的Mac arm64候选回报：新生成origin/v2的恢复、幂等、删除/篡改拒绝及真实Retained→Deleted链通过，repository/CLI集成各6/6。云端读取了该评论，未独立读取验收机的原始report/manifest/log文件或重跑Mac验收。

最初的Contract提交994a065a就明确保留旧v1记录且不静默迁移，并要求如实报告歧义；此次是对既定边界的现场确认，不是扩展WI-1084。

同一回报明确：旧v1没有origin，删除其唯一receipt后仍closed/blocking=false；保留完整旧v1直接重试得到closeout_recovery_receipt_conflict。WI-1084的有界v2保护成立，不等于既有v1历史已获得同等保护。#1004必须保持OPEN，不能因WI-1084归档、关闭或PR1040合并而自动关闭。

## 最小后续路线

1. **先只读盘点与冻结。** 盘点有可信原件的旧v1、其源/目标repo身份、Git common-dir、原close/Contract/archive/verification及完整清单的raw字节摘要、已提交Git blob与外部可信备份。记录receipt是否仍在、是否存在独立预期清单；缺失/不明保持原状。不要删除现存冲突文件，不移动工作区来掩盖冲突。
2. **再审阅显式追加协议。** 如需覆盖旧历史，另行审阅一份版本化、opt-in的legacy provenance observation/recertification设计。它必须绑定可信旧v1原件digest和完整归档身份，记录当前真实采集时间、操作者及来源；追加独立的预期清单/观察锚，而不是伪造一份声称旧时就存在的origin。旧v1、close、Contract、archive及payload原始字节全部保留。
3. **以独立可信预期约束缺失。** 新观察只有在一个独立保留且可验证的catalog/已提交历史中确立“该恢复必须有这份receipt”，后续才可据此拒绝receipt缺失。若receipt及全部独立锚都已消失，仅凭一个外观普通的合法close不能一般性区分“曾恢复后证据丢失”与“普通close”。此时要求可信源或明确人工调查，标记legacy_unproven/needs_review，不伪造可检测性或新历史。
4. **明确兼容与能力边界。** 当前closeout-recover拒绝覆盖已发布v1的冲突行为保持。未来升级必须是单独、显式、版本化且支持能力检查的操作；只有新协议得到支持、绑定及授权成立后才追加记录。不把旧Runtime的可读兼容性当成新保护保证，不静默迁移、不原地覆盖或弱化普通close校验。

## 后续方案的验收矩阵

- 原v1及全部历史文件raw字节不变；显式新增观察记录不回填历史时间或身份。
- 具有可信独立锚的旧v1在receipt缺失、摘要不符、外来repo/WI/Contract、清单截断、非regular文件及冲突时拒绝认可完成；普通close不被误判为恢复缺失。
- 无可恢复独立来源时保持来源不明及人工调查边界；不把所有锚同时丢失声称为自动可检测。
- v2现有删除/篡改、真实Retained→Deleted、相同请求幂等与中断恢复继续保持；CLI/MCP使用同一校验器；历史verification继续not_ready/historical_evidence_not_revalidated，不伪称重跑。
- 旧v1原地重试冲突保留为真实回归事实。未来显式升级的成功、幂等、冲突及中断分别验收，不能靠换工作区绕过。
- 协议、范围、Runtime兼容及验收经Raydot技术审阅并获必要授权后再实施；本提案不创建WI、不写代码、不迁移或修改Sentinel。

## 来源摘要已补正

更正评论6191899624提供完整SHA-256：`84afc073492b30366068e15f74a860606969150ae15076b1a351115916804bac`，已核对64个hex字符。来源说明该值来自原retained-file-digests.json的receipt/source/destination三列，并在原Sentinel文件字节上重新计算；该清单在原manifest中的SHA-256为`237dc82103030c0f65127d5c9da178daca9a6c6b7eb02cea590bf06dc1cc463e`。原59位评论及本地快照继续保留，更正作为追加来源，不再要求补正。云端记录来源声明，不冒称读取了Mac本地文件。

## 必须进入关闭确认的残余声明

“我认可WI-1084只交付已声明的新origin/v2完整性范围；我已知旧无origin v1删除唯一receipt仍可能显示closed/blocking=false，完整旧v1原地重试仍拒绝冲突。此次WI关闭与PR集成不宣称旧v1问题解决，不授权迁移或伪造origin，#1004继续OPEN。Mac验收属于固定候选的外部现场回报，未升级为云端原始日志独立验证；Deleted来源摘要已按追加更正完整记录，不再要求补正。”
