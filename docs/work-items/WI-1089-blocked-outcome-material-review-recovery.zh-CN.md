---
author: AI Cockpit maintainers
title: "WI-1089 — 阻塞 Outcome 的材料审阅恢复"
description: "失败 Outcome 与敏感 Contract 修改后的有限恢复。"
audience:
  - maintainer
  - reviewer
authority: explicit-user-authorization
workItemId: WI-1089-blocked-outcome-material-review-recovery
status: in_progress
lastVerifiedBy: WI-1089-blocked-outcome-material-review-recovery
---

# WI-1089 — 阻塞 Outcome 的材料审阅恢复

敏感 Contract 修改后，持久化的失败 Outcome 仍为红色。当前 preflight 与身份绑定的真人
`confirm_review` 只确认当前 Contract，不接受材料扫描的残余风险。只有源代码为干净、
已提交的 Rust，扫描结果恰为一个允许审阅的语法 Unknown、没有 Finding，且当前请求与
Contract 的材料审阅政策精确匹配时，Runtime 才能提供
`record_material_review_decision`。材料决定独立作出，仍需对应审阅权限。

有效的材料收据可在正常前置条件下开启新的验证。真实通过的验证收据写入前，当前失败
Outcome 仍为红色；随后当前投影才会更新，追加式失败事件仍可读取。在此之前 finish 与
archive 仍被阻止。脏源码、不可用的
材料投影、Finding、矛盾证据、过期或外来审阅输入及错误的恢复决定均继续被拒绝。

保留的 canonical 收据不是 Missing。恢复入口通过有界、禁止跟随符号链接的读取，
对同一份原字节校验完整的类型化计划、执行和保留策略绑定；前驱 Contract 必须来自
已验证的原生修约链。历史 snapshot 与 producer 身份和当前源码、执行权限分别校验。
不支持或不完整的历史继续阻塞，并给出有界诊断；普通 Stale 不授予此操作。

等价夹具保留真实旧四项成功收据及真实 blocked Outcome。验收要求当前精确材料决定、
plan-only 零进程、单 worker 真实执行四项且零复用、fresh green preflight，然后实际
完成夹具 finish 和 archive。其他测试保留相同绑定下的合法复用，拒绝损坏或不完整
历史；真实检查失败须另行精确原生 retry。这不是 Sentinel 真实验收，也不授权实际
工作项 finish 或归档。

修约后的范围包括 repository 的修约、证据校验、状态投影三个实现文件，repository/CLI
测试以及六份三语 WI/parity 文档。先前六节点与组合源码结果继续保留为历史证据。
新候选须完成自己的检查和独立精确材料决定；旧的有限风险同意不覆盖新 Rust 字节。
PR1048 保持 Draft，WI1090 继续暂停。

coverage manifest 只覆盖分区的 workspace 节点，真实原生非分区收据也可能没有它。
因此历史恢复还必须取得原始原生 verification-attempt：文件名/内容摘要、前驱
Contract/范围、snapshot、producer、完整未改动收据及每节点命令/时限绑定均须一致。
所有声明的 argv 命令必须被覆盖，包括由 manifest 绑定的 workspace 展开。
缺少或篡改该证据时给出诊断并拒绝，正常非恢复兼容语义不变。这仍是现有原生保证
级别下的结构证据，不新增密码学生产者认证。
