# 最小可恢复交接与保留对象分类（2026-10-16）

Outcome: 🟡 本轮仅盘点、追加交接元数据并保存远端检查点。WI-1086 仍 checkpointed / yellow / verification not_ready；WI-1085 仍 checkpointed / yellow。本轮没有执行构建、测试、amend、恢复写入、安装、迁移、清理、归档、关闭或合并。

## 1. 源码与远端检查点

本轮开始时两个工作树均干净，普通 `git ls-remote --heads` 读回与本地 HEAD 一致。最终交接提交是这些检查点之后的材料提交，不能冒充产品源码提交或通过验证的提交。

| 对象 | 分支 | 本轮读取的准确 HEAD |
| --- | --- | --- |
| WI-1086 | `codex/wi1086-sensitive-amendment-preflight-request-interlock` | `95f459b5db3a51416041a682d1764fadd1a5c89a` |
| WI-1085 | `codex/wi1085-legacy-v1-closeout-source-registration` | `f49a001ece01209ed9e4cf743f879b7a5931143c` |
| WI-1078 历史 retained control | `codex/wi1078-retained-evidence-00461714`（仅本地引用） | `62427bce7e332d0e3a5741fa405ad62bac3718dc` |

WI-1086 产品源码仍为已审阅的 `2a5910e967585c3589af382d29143f1b50b0a2c1`。当前 Contract 为 `sha256:e7245a06dd9fc2a45346800a300183c2e3207a40a71bf579f2c5ec138538387f`，repositorySnapshot 为 `sha256:0840ed40a921b36f42e61d97d8db8eb664882c5239bdd6f9336744cea3918e54`。WI-1085 当前 Contract 为 `sha256:eae15cbbdbc36606635db999976611d3c036260296d2b5b24b9afa183ae03bd5`，snapshot 为 `sha256:19489503de52038fb36e93e091d7797d1b77825c4492fd6ca9e788f9c6b4d783`；本轮不推进它。

WI-1078 retained 分支的远端引用查询为空；不能再声称该远端分支存在。其上述提交仍由 GitHub API 真实读回，tree 为 `831ff8fa54841b5672549b26f191109bc0bb148b`，并实际读回 retained 的 source-binding 和 Deleted/close 绑定原文。历史 close 文件 SHA-256 为 `459e36b907043fd5b9a7c03efc0a3213e125945ee523821b2eedc1f45511ecda`、recordedAt 为 `2026-10-15T07:53:07.838291701Z`。这些是旧事件，不是本轮新 close，也不等于所有旧主机目录均已清理。

## 2. 主机原状与暂停边界

观察时 `/workspace` 可用 `1,156,005,888` 字节、空闲 inode `1,930,009`；`/tmp` 可用 `1,803,735,040` 字节、空闲 inode `2,247,044`，device 分别为 27 / 33。主机无活动 cargo/rustc。数字是采样事实，不是后续容量承诺。

`805031` / startTimeTicks `24465201` / PPID `805030` / python3 保持 **T**，其直接拥有的僵尸为 **1089**；PPID 1 僵尸为 **21316**，总僵尸 **22405**。保留旧 exec session `1926`。本轮不发送信号，不尝试注入、调试、提权、跨父进程回收或恢复旁路，不退出该暂停监督进程，不销毁旧环境。

已知监督错误是把不可用的 `/proc/<pid>/task/<tid>/children` 当作空集合，错误记录 ownedProcessCount=0；不能据此断言内核或容器的具体限制。后续监督必须真实等待自己拥有的子进程，进程接口缺失时不能默认为没有后代。旧监督的 execution-meta 未正常结束，不能宣称其正常完成。

## 3. 当前 283 文件备份：仍是恢复依赖

准确位置：`/tmp/wi1085-approved-build-preservation-20261016-nyk7nr4c`。

原始清单 `/workspace/wi1085-capacity-inventory-20261016/proposed-cleanup-283.manifest.json` 为 217198 字节，SHA-256 `5881b11f87fd77b5d97a0e528732687a44bc825248aeb8223dea6a5b026a860e`。备份内 `approved-source-manifest.json` 与之字节相同。283 个原文件共 **488903192** 逻辑字节，本轮逐项重读哈希、大小、mode 全部匹配；加清单及恢复说明共 **285** 文件。逐项结果见 `migration-current-backup-readback.json`。

这是此前真人授权的备份后清理的恢复副本；历史实际删除 283、跳过 0、保留排除项 653，不是本轮删除。该历史执行证据已在 `7e256abcd4bfd7748db28053876d52f20b104127` 的 `backed-cleanup-and-full-verification-20261016` 下。原授权是主机维护批准，不伪装成 Runtime `confirm_review`。

**尚未建立这些原始二进制字节的独立持久副本。`/tmp` 会随环境丢失，不能现在删除，也不能将清单当作二进制备份。** 恢复时须先确认 WI-1085 构建/测试空闲并取得相应恢复授权，复核清单和每个备份哈希、目标父路径无 symlink，只复制到原清单的缺失路径，保留 mode，拒绝覆盖新文件或已变化文件，再记录读回；inode/ctime 会变化。

现有 Library `libfile_75acd3e43a588191ab0710bb0c0183f3`、`WI-1085-reproducible-build-cleanup-plan-20261016.tar.gz` v0（158203 字节，SHA-256 `6f5b896cb226355553776abd2ae5f2f02ae0a0acb8d669584bda5e47bd66722c`）只有 26 份盘点/方案/来源材料，**不含上述二进制字节**。本轮没有上传或重新物化它。

## 4. 历史对象按用途分类

只沿已有记录核对精确目录、直接小文件和指定 Runtime 路径，没有全盘搜寻或压缩旧目录。完整逐文件哈希、已比较的 Git 对象及限制见 `migration-historical-small-files.json`、`migration-retained-binaries.json` 和 `retention-classification.json`。其中“已匹配本地 Git 对象”与“独立远端原始字节已读回”分别记录。

| 类别 | 精确对象 | 结论及后续条件 |
| --- | --- | --- |
| A：仍需恢复 | 当前 283 文件 `/tmp` 备份；当前安装 Runtime；WI-1085 / WI-1086 保留候选 | 均未建立独立持久二进制副本。审批安全转存并校验，或由原生命周期明确终止恢复依赖后，才能重评清理。 |
| A：仍需恢复 | `/workspace/ai-cockpit-control/.ai/work-items/active/WI-1078-v1-0-1-post-merge-sync-finalization.summary.json` | 5488 字节，SHA-256 `ed2d0889a81dcc0e40273b274a6184aba197312dddcfda6ae900a5cdc5a98916`。未提交；`/workspace/wi1078-preserved-evidence-20261013/preflight-summary.control-original.json` 字节相同，但同主机副本不是独立持久保存。 |
| A：仍需恢复 | `/workspace/ai-cockpit-main37cc-review/crates/cockpit-repository/src/resource_lifecycle.rs` | 388828 字节，SHA-256 `f0fe9eb4f99c4f0dddd385c92072002cc07c0c0385d62abe99b924529d7376d5`。旧 review 未提交代码，保留 HEAD `feafd0fe72dc44c41107a85840f93bffc060a5ca` 及差异上下文；不在新源码分支中覆盖它。 |
| B：用途结束且独立原始字节已核实 | `/workspace/wi1084-final-approval-b1487517/after-merge-execution/retained-ignored-target-evidence/` 内的 `governance-integrity.json`、`post-formal-governance-integrity.json`、`retention-manifest.json`，**仅这三文件** | 前两份各 173015 字节，哈希 `994cf404cf29049a3750a930fd5b180a97b9626fd5e92cee60b5b4aab8760f15`；清单 1713 字节，哈希 `35fa7f68e5a51c38a515b3ae5960fb742dd1c71b74f5376a4bc6800d1c619c7b`。与远端 `95f459b5…` 已跟踪原文逐字节读回一致。原目的为 WI-1084 原资源清理前保留 ignored 报告；可交原 WI 清理流程处置，本轮不删除。逻辑字节共 347743，不能当作大容量回收方案。 |
| C：用途或独立保存待定 | `/workspace/wi1078-preserved-evidence-20261013` 的其余历史材料 | 26 个直接文件均在；除上述 Summary 外，尚未证明整套原始采集的独立副本。17 字节版本文本在其他 Git 路径字节相同不证明原采集来源。不能因为原 WI 已 closed 就推定此目录可删。 |
| C：用途或独立保存待定 | `/tmp/wi1078-retained-00461714-target`；`/tmp/wi1078-main-bcd76ae7-20261015`；`/tmp/wi1081-build-00461714-20261014`；`/tmp/wi1084-runtime-checkpoints-20261015` | 这些历史目录及指定候选仍在；仅核对精确候选和直接小文件，不推定整个 target 可删。WI-1078 第二目录的 source-binding 与远端 `62427bce…` 字节相同；若干其他小文件匹配本地已提交路径，不能把它们扩展成整个目录已持久化。WI-1081 本次路径中二进制哈希 `8396ed27…` 与旧归档生产者 `5cca63a1…` 不同，不能混绑。 |
| C：用途或独立保存待定 | 旧 WI-1084 记录的两个缓存根下 10 个受保护 Runtime 路径 | `protected-runtimes-post-recovery.json` 列明的 10 文件当前全部匹配历史字节；同 inode hardlink 不是独立备份。原始生产者用途或副本终止条件尚待核定，不能批量删除父缓存根。 |

“用完清除”落实为：B 进入原有精确资源清理评审；C 明确核定恢复依赖与来源，已结束并有可靠副本的精确对象转 B，仍唯一且必需的转 A。不能把 C 默认为永久保留，也不能先删后判断。本轮清理数为 **0**。

## 5. Runtime 与未完成验证

当前安装 Runtime：`/tmp/wi1084-closeout-execution-retained-b1487517/ai-cockpit`，52094536 字节，SHA-256 `5307ba5324d397e0fb971aa8b926ba8176336a0a6e76dfae7bd3a6e5cd11fa38`，没有替换。

WI-1085 候选为 `target/wi1085-legacy-closeout-source/debug/ai-cockpit`，53700216 字节，SHA-256 `8f4ce88a9f4ead070f935061ff421652ae53aec3eda75feabe17ce9bb2daf095`；其 deps 路径仍保留。WI-1086 候选为 `target/wi1086-sensitive-amendment-preflight-request-interlock/debug/ai-cockpit`，52132416 字节，SHA-256 `3da828ca22763d620f2b3d4e7121f9400bb02854bbdd1b1bb1f6f3ff0868688b`。两者均不是当前共享安装 Runtime。项目工具链声明 1.98.1；新环境须记录实际 rustc/cargo 输出、源码提交、构建命令、产物路径/哈希，不要求重建哈希与旧环境相同。

保留真实完整验证：执行 HEAD `b77eb89940c7c8869e31bdb08eecb15b44cc0e70`、attempt `e571cea63e6020dfcb89cdd72c6d69338a4c8d0c4d00f291f1032276bc6bc797`（raw SHA-256 `7bcee0c4e23ddf8f44912ff3bf6b9433b40e9ffa0b24bb652cf5d0e5c835c226`），13 节点执行、0 复用、12 通过 / repository 1 失败，588672ms，非超时。缺少 preflight 的正向 fixture 为 `pending_retry_survives_a_bounded_contract_amendment`。该失败及监督事故已经保存 `7e256abc…`；没有 canonical 成功验证回执，不以单项复跑或 CI 替代。

`95f459b5…` 的全量静态 fixture survey 与单文件 typed amendment **DRAFT** 已保存；草案 SHA-256 `e737071c8f02f6f4461a6a9267ff99ca9b3835fbfeb65f1e69f7aca09b38cd4e`。尚未应用，不预造新 Contract、preflightDecision 或 confirm_review。当前 Runtime 虽显示 run_verification admission allowed / humanDecisionRequired=false，父协调的主机暂停及本轮只读限制仍有效；不启动验证。

## 6. GitHub 已保存与旧环境独有

GitHub 可恢复两个活动分支源码、已提交治理记录、所有已提交失败尝试/静态 survey/清理方案及清单；本轮交接提交追加的是元数据。WI-1078 retained 的固定历史提交和本次实际读回文件如上；该工作树整体原始字节不据此推定已独立保存。

旧环境仍须保留：283 原始备份字节、当前及历史指定 Runtime/候选、未提交 Summary 和 review 代码、未证明独立保存的历史采集目录、暂停进程与原 exec 会话。进程状态不能迁移成新环境事实。

旧 WI-1079 Library 恢复包 `libfile_31b9783b20cc8191a0f9950da5af9047` / `WI-1079-resource-block-20261013.tar.gz` v0（20230 字节，SHA-256 `3d4617f63992e000ea345e3ddcc1cab52a2d52938787b886a1feda4fa4725697`）有历史保存记录，对应恢复文件已有 GitHub `.ai/evidence/WI-1079-resource-block-20261013/`；本轮未再次物化并验证包，不将旧保存记录说成当前包读取成功。本轮也不上传任何原件到 Library 或其他外部存储。

## 7. 获准迁移后的原生恢复路径（本轮不执行）

1. 先由 Raydot 明确 A 类哪些原始字节需安全转存、目的地及授权；验证可读和 SHA-256。C 类优先核定用途，不复制全部缓存。独有材料未获独立保存前，旧环境与 `/tmp` 不得销毁或重置。
2. 在新环境检查 PID 1、僵尸基线、子进程可观察性、磁盘/inode、工具链；正常 fetch 已指定分支，固定经读回的交接 HEAD / WI-1085 HEAD，在同一新 common-dir 建干净 linked worktree。保留旧冲突及原始上下文，不靠换目录宣称旧问题消失。
3. 读新 checkout 的 AGENTS、`.ai/README.md`、当前机器治理记录和相关指南；通过正常支持方式准备 Runtime。安全转存旧安装字节或另行获准重建均须记录真实来源/哈希。重新执行 `inspect`、`status`、`doctor`、`agent doctor --json`、capability/help 和目标 WI 查询；所有 repository-bound 命令显式 `--repo`。核对 repositoryId、Contract、snapshot、material 以及旧证据来源差异。
4. 当前只读 `work-item coordination inspect --repo <repo>` 返回 registration/events/recoveries/requests 全空，common-dir `.ai-cockpit/coordination/v1` 不存在。没有可抄用的旧 generation。`check-environment-drift` 的真实接口为 `--repo <repo> --input <json>`；request 为 `schemaVersion`、`workItemId`、`expectedGeneration`，generation 必须来自真实注册。以前使用 `--id` 的查询只是参数错误，不是一次成功 drift 检查。
5. 新环境先查询实际支持与注册事实；不存在注册时不能伪造 generation 运行 drift，也不能为绕过准入制造 registration/lease。若正常授权流程需要并允许建立注册，使用 Runtime 真实采集基线。存在真实基线时先只读 check；有 pending drift 且实际准入允许才显式 record，再刷新 generation/admission。coordination recover 只能消费真实 impact event 与 consumer generation；`work-item recover --input` 只能用于真实被阻塞的生命周期恢复。两者不可互相替代，也不是主机事故的通用解锁口。
6. 旧 receipts/批准只保留为旧来源历史。重查实际准入：同一 Contract 有效批准按既有规则复用，不重复要求 Ray 批准同一政策；确有 Runtime 新要求或绑定变化则展示准确差异。父协调解除暂停后才考虑应用单文件 typed draft，当前 expectedContractDigest 必须匹配，按新事实重查准入；不能预生成确认或沿用虚构 digest。
7. 之后先解决单一 fixture 的正常 amendment，再按实际准入准备完整验证。保持单 worker、900 秒上限、断言不变；先验证监督与子进程等待机制。旧完整运行曾使 `/tmp` 余量低到 541863936 字节，仅比 512MiB 多 4993024 字节，须重新估算冷构建/临时目录峰值和双盘余量；当前约 1.15GB 工作盘余量不能证明新环境冷构建足够。不得删除当前恢复备份来腾挪验证空间。

最小决策依赖：A 类的安全持久化授权及可读副本、C 类的用途核定、可正确等待后代且资源充足的新执行环境、正常 Runtime 准入，以及 Raydot 解除主机/验证暂停。尚未迁移，不新增人审，不推进其他 WI。

## 8. 本轮检查与交付边界

本轮仅执行文件/进程/磁盘/Git/GitHub 读取、原始字节哈希检查及 Runtime 查询。当前分支没有 open PR，CI 仅 main push 或 pull_request 触发；交接提交不创建 PR、不请求 CI。本轮没有执行项目测试或验证命令。追加材料经过凭据模式检查并生成字节清单；不包含备份二进制、旧未提交代码全文、旧治理原件正文、认证配置或完整 HOME。

Runtime Outcome 已真实返回并以本报告及最终答复交接；主机是否已接受/显示完整 Outcome 尚无独立确认。后续动作是交 Raydot 审阅分类及迁移依赖；WI-1086 完整验证仍失败/未就绪，不能最终验收或关闭。
