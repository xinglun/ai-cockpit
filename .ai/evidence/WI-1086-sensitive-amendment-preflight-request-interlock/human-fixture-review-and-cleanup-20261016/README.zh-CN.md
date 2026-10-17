# 当前真人确认与精确清理核验

Ray 的真实确认来自 Sentinel_5d7751dc43408191bcdd0ee444d28cdc，时间由 Raydot 提供为 2026-10-16 02:01 UTC（分钟精度）。核对 75e3efe2 的完整 Contract/preflight/snapshot/journal 后正常 controls 记录 confirm_review；recordedAt 使用实际记录时间。Runtime 正常追加新 digest-named preflight-review receipt，Summary.decisionEvidence 与真实 input 相同；旧 canonical receipt 字节完整保留，没有改用旧绑定。

279 个精确清单文件的 SHA-256、文件身份、非可执行属性和逐层 O_NOFOLLOW 父目录检查全部匹配。活动进程 FD 观察对宿主 Codex 等返回 EACCES/errno13，无法完成占用排除；依用户明确的“占用不明跳过”，279 个全部跳过，0 文件/0 目录删除，空间增量 0。未改权限/系统安全设置，未清理其他 target。逐文件核验和进程缺口见 cleanup-file-checks.json/cleanup-result.json。

确认后的初始准入允许验证；加入未提交测试源码后 material_review_projection_unavailable，验证阻塞。一个编排错误使失败断言之后仍执行 7.663 秒定向测试；全部原始输出保留为 unadmitted observation，不是正式验证。执行器已增加同进程失败即停止的 Runtime 准入检查，并实际观察阻塞时没有启动 Cargo。候选测试源码先提交，再刷新真实准入并重跑。
