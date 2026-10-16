# 已批准的精确 debug 清理与分支恢复

当前用户在本轮真实回复“批准仅删除该 debug 目录”；按此授权行动。后续父线程转述的 Ray 清理批准于行动后收到，仅作为补充来源，不倒填为先前授权。授权及受限占用信息均保留，不伪造原生 cleanup receipt。

唯一删除对象：`/workspace/ai-cockpit/target/wi1086-sensitive-amendment-preflight-request-interlock/debug/`。行动前目录逐项 device/inode/type/size/link 与已推送 32353165 清单一致，无符号链接、内部挂载、跨设备或未知目录变化。即时可读进程无占用匹配；PID 200/251 仍有已披露权限限制，未把它们算为空集合。无 cargo/rustc 活跃进程。

实际命令为 timeout 890s、TERM 后 10s KILL 的 `rm -rf --one-file-system -- <唯一精确路径>`，退出 0。目录实际消失。文件系统可用空间从 0 净增至 11,884,752,896 字节（约 11.07 GiB；此为观测的净可用变化，包含同时发生的元数据/日志写入）。默认沙箱随后成功启动。

保护检查：5 棵保护树的逐项元数据摘要删除前后完全一致，实际 Runtime、WI-1086 唯一清理收据、WI-1085 Contract 与失败尝试的逐字节 SHA256 均一致。保留 release Runtime、全部 .ai、/tmp 原始日志/报告和当前 WI-1085 target；没有额外删除。具体 hash 见 after-protected-check.json。

核对 f168de42 与其后 32353165 的远端树，与此前 /tmp 准备并已推送的对象树一致。原本地 fe205fa7 的25份未跟踪证据逐字节等于远端；用精确 pathspec 的具名 stash 保存后，正常 git merge --ff-only 至 3235316534fd0798fc6a83625df72a36662ab443。工作目录当时 clean，全部25份文件在当前树和 stash 第三父树中逐字节一致；stash b5350440b4ac707de48b679604a9ed086f15b498 保留未删除。未使用 reset、强推或丢弃未跟踪记录。

WI-1086 唯一清理收据仍仅在主检出保留为未跟踪文件，raw SHA256=7304e582f78f5a49ec751f01f5497a3a310a66c5621fa3353813869c961a6a8d；未为它另开 PR、改写或复制到缺少其关闭上下文的 WI-1085 生命周期。
