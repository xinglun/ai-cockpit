# 跨 Work Item 协调

此能力只由候选 Runtime 提供。Runtime `0.2.105` 继续负责生命周期，不读取
协调记录。候选 Runtime 负责能力发现、仓库本地协调读写、影响准入和精确组合
验证；这不是双向兼容承诺。

## 支持边界

协调根目录由 Git 的 common directory 解析，因此同一仓库的 linked worktree
可以协作。独立 clone 和跨机器协调不支持。记录位于
`.ai-cockpit/coordination/v1/`，并绑定仓库、WI、Contract、worktree/head、候选
Runtime 能力和执行代次。

登记与查询都会从 canonical Git topology 和 active Contract 重新观察这些绑定；
调用方声明的 repository id、branch、head、Contract digest 或 evidence 路径本身
不构成准入事实。后续登记观察到 head、Contract 或 declaration 改变时会追加去重的
Impact 事件。Outcome 发布只提供信息，不会使消费者失效；恢复记录会保存当前提供方
代次、head 和 Contract digest，因此提供方已经进入新代次后仍可解决旧事件，不需要
删除历史。

普通单 WI 生命周期仍是快速路径：除非 WI 明确参与协作，否则不扫描、不写入
协调存储。

只读查询：

```text
ai-cockpit work-item coordination inspect --repo <path>
```

登记、影响报告、暂停请求/确认、恢复消费和组合验证都是显式写入入口：

```text
ai-cockpit work-item coordination register --repo <path> --input registration.json
ai-cockpit work-item coordination report-impact --repo <path> --input event.json
ai-cockpit work-item coordination publish-outcome --repo <path> --id <wi> --generation <n> --outcome-id <outcome>
ai-cockpit work-item coordination request-pause --repo <path> --input request.json
ai-cockpit work-item coordination acknowledge --repo <path> --request-id <id> --state acknowledged
ai-cockpit work-item coordination resume --repo <path> --id <wi> --generation <n>
ai-cockpit work-item coordination recover --repo <path> --event-id <id> --consumer-work-item-id <wi> --consumer-generation <n>
ai-cockpit work-item composition --repo <path> --id <wi> --generation <n> --input composition.json
```

查询不会修复、消费、确认或创建目录。恢复消费按事件、提供方代次和消费者
代次幂等；旧代次会被拒绝。影响事件按身份去重，只阻塞受影响消费者，无关 WI
继续执行。暂停请求、确认、安全暂停、不可用/过期和恢复是不同状态。

发布成果是针对当前登记代次和已声明成果的显式写入。Runtime 会核验当前消费者
要求的验证 receipt，再追加绑定精确证据字节的 `OutcomePublished` 事件。只读查询
不会发布、修复或消费事件；发布后若证据字节改变，该绑定失效，不能满足验证依赖。

组合验证会先刷新依赖准入，并在启动任何验证进程前拒绝安全暂停的目标。Runtime
随后核验目标 topology、每个已登记参与者的 head/Contract、完整的必需检查覆盖和
前置条件，再在临时 linked worktree 中按声明顺序组合。它复用共享的有界验证执行器，
具有有限超时和有界输出；启动进程前先落盘 in-progress attempt，逐节点落盘，并记录
超时、父进程中断可恢复状态和清理结果。调用方提供的 identity digest 不授权复用：
Runtime 会观察目标 tree、锁文件和配置文件、实际解析的 executable、有效进程环境及
声明的输入文件。只有实际观察到的 executable、命令、有效环境、声明输入文件字节和
上游 receipt 与成功前序节点一致时才复用；缺少或无法观察的节点输入会禁用复用。
完全重复可以启动零进程；声明输入变化会重跑对应节点及其依赖节点。

CLI 与 MCP 调用同一仓库服务。Outcome 分开展示实现、临时组合、当前组合适用性、
真实目标合并和清理；目标或参与者事实变化后，历史通过仍保留但会标为过期。没有证明
的收益或缺少可比基线会保持为未知，不会被写成性能结论。
