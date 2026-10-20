# 四成员材料授权与最小原生修约

- 实际批准来自Ray2026-10-20T03:28:22Z Sentinel_4894c6afcf14819186152a312ace0394，对a9f8bdda精确提案；relay原文保留。
- native amendment6 exit0，Cbe0f、environment fd31、journal5e78。只改goal/AC10/authoritySource，全部其他字段逐值比较一致。第一版Agent审计误读verificationPlan字段而KeyError；改为实际verification字段后完整审计通过，没有编辑native记录。
- 父线程独立技术审阅报告94e554ad两处修复可接受，无新增阻断；原话保留。开发证据为11项套件+独立1项，CLI/MCP各1；不称最终同轮14项。
- 新preflight实际exit0但要求新的真人Contract确认；没有复用旧C98确认。只读material plan读回4成员与原批准精确相同，Finding0；没有记录材料接受或Agent审阅完成。
- 当前停于真实native needs_human_decision；精确完整绑定一次交Raydot，风险批准无需重问。六正式检查、实际自身finish/archive、DraftPR和CI待真实准入；不重启或重复已有任务。
