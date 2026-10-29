---
author: AI Cockpit maintainers
title: "WI-1114-release-runtime-convergence — Runtime 发布收敛"
description: "收敛 Mac 行为、CI 成本、公开帮助、Sentinel 验收及验证到收尾的可信证据衔接。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1114-release-runtime-convergence
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1114-release-runtime-convergence
---

# WI-1114 — Runtime 发布收敛

本 WI 严格限定五个子任务。范围、权限和准入以原生 Contract 与 Runtime 当前投影为准，
本页只记录进度，不授予权限。WI-1113 已交付到 `main`，但原生收尾仍待完成；其原始证据
和恢复资源保留，不因本 WI 启动就自动退役。

## 范围

1. **Mac：** 实现并真实验证 macOS arm64 支持的监督边界，覆盖普通退出、超时、父端断开、
   协议失败及脱离进程组而持有管道的后代。明确限制和不确定性，保持 Linux、Windows 保证。
2. **CI 成本：** 分解编译、执行、证据复用与清理耗时。只消除已证明重复的工作，保持完整
   target 覆盖。仅在共享实现与等价负例有依据时将适用脚本迁移 Rust；语言变化本身不是提速。
3. **帮助：** 从 Clap 导出公开命令目录，验证实际短/长帮助和 discovery description，保持
   英文、日文与简体中文参考一致，不公开 private supervisor。
4. **Sentinel：** 正式发布经明确批准后，在消费方独立副本中固定同一不可变包、二进制和
   inspect 身份，分别验收 #1004、#1041 及相关 amendment-provenance 回归，保留原输入与失败。
5. **证据与收尾：** 在后续元数据或聚合失败前持久化真实执行事实。CI 证据只有经过明确的
   provider、源码、Runtime 和逐命令等价核验才可接纳，真实缺口仍执行；进度输出不是证据或权限。

## 验证与交付顺序

先运行小型真实正负例，再独立审阅、执行绑定最终源码的检查及必需的平台 CI。局部模块
通过不等于完整验证或其他平台通过。性能以可比较的 CI13 quality **4,701 秒**为基线，目标
至少减少 30% 且不超过 55 分钟，并披露缓存条件及保留的覆盖；当前尚未证明该收益。

质量 CI **总预算 8 轮，包含 main**，发布 workflow **总预算 2 轮**；当前已用 **0/8**、
**0/2**。本地单 worker、每条命令最多 900 秒、受保护的有限进程清理，磁盘至少保留 1 GiB。
失败和不确定结果保留，不盲重试，不删除恢复证据。

先完成实现和源码 CI，再取得精确版本、资产批准并正式发布；随后由 Sentinel 验收同一
不可变正式包。候选、plan-only 或 AI Cockpit 自测不能替代消费方验收。必需消费验收未完成时
WI 仍进行中。`v1.1.0` 只是提案，不是已批准的 tag 或 release。

## 当前状态与边界

Sequence 2 的真实 owner 确认已由原生 Runtime 登记，初始 checkpoint 已完成。局部执行事实
持久化与帮助模块已有定向证据；整轮中断恢复、可信 CI 接纳、Mac 验收、提速实测和正式包
消费验收仍待完成。本 WI 尚未启动质量 CI 或发布 workflow。

当前控制不授权正式发布、共享/生产 Runtime 替换、不可恢复删除、提前关闭 issue 或伪造
finish/archive/close。精确 Runtime 切换和原生人工决定仍各自绑定。归档前文档预登记只是准备，
不证明已经 archive 或 close。
