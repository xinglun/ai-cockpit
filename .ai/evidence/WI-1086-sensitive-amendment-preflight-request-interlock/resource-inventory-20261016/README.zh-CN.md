# 只读磁盘盘点与精确清理候选

未删除、移动、裁剪或清空任何文件。当前 /workspace 可用约 685465600 bytes；inode 可用 1930634，空间瓶颈是字节。没有观察到活动 Cargo/rustc 构建；PPID1 僵尸仍为 21,316。

## 可供精确授权的最小候选

`workspace-static-candidates.manifest.json`：仅 WI-1085 的 ignored target 下 279 个普通、非可执行 Cargo .rlib/.rmeta/.o/.d 文件，预计释放 316006400 bytes（按 inode 去重并扣除保留硬链接）。逐文件路径、SHA-256、device/inode/mode/size/mtime/linkCount 已记录。清单 SHA-256 `86e83a12e1f355454ea7268e8df36b7c7ab3e4f097bb2941e3ebbc6b836dd9a6`。该目标当前没有活动构建；保留其 Runtime/测试可执行文件、动态库、全部源码和治理证据。此候选不构成删除授权，也不修改 WI-1085 的任何资源。未来恢复旧 target 会重新编译所列中间产物。

最大的 21,670,039,552-byte 旧缓存树主要是原批清理明确排除的可执行/动态内容；继续排除。当前 WI-1086 target、候选 CLI、共享 Runtime、工具链/Cargo 下载库、Git common-dir、所有 worktree 的源码与 .ai、旧 control/review 的未提交文件、WI-1078 资源和独有历史证据均排除。不得删除整个 .home、debug、deps 或 target。

/tmp 是另一文件系统；旧 WI-1081 静态中间产物约 949,751,808 allocated bytes 仅盘点，不列入本候选且不能提高 /workspace 容量。

进程链接扫描无已观察到的旧缓存引用，但有 55 个不可读 proc 条目，未声称全局无引用；未来执行必须重查真实进程、全部清单文件及父目录非 symlink、身份稳定、字节哈希与排除项。未证实安全者跳过并报告，不自动扩围。

约 316 MB 的最小候选不足以证明完整重建所需余量；在新的容量决定前继续暂停大构建。旧完整失败回执与原始 stdout/stderr 全部保留。
