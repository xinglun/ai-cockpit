# WI-1086 fixture 调用顺序只读普查

Outcome: 🟡 普查完成，最小新增范围仍为单文件；未解除宿主恢复暂停。

## 结论

源码证实本次仅需新增 `crates/cockpit-repository/tests/recovery_decision.rs`。其 `pending_retry_survives_a_bounded_contract_amendment` 在1479行 amendment 后，1488行直接调用原人审fixture helper；189–218行 helper 本身不执行preflight。现有正式失败 e571cea6… 已在该调用的217行报告“requires a recorded green, yellow, or red preflight result”。

保留 Raydot 已通过的最小方向：该调用前记录当前Contract的原生preflight，随后使用原helper及末尾原准入断言。原先未应用patch原样随本目录保留。没有发现支持再增加其他关联源码文件的证据。

本轮仅生成typed amendment草案，未调用amend、preflight、controls、测试或验证；未修改任何产品/测试源码、Contract、Summary或receipt。没有记录confirm_review，也不生成新的Contract/preflight digest。

## 观察绑定与覆盖

- 普查基于提交 `7e256abcd4bfd7748db28053876d52f20b104127`；产品源码仍为已审阅的 `2a5910e967585c3589af382d29143f1b50b0a2c1`。
- 当前Contract：`sha256:e7245a06dd9fc2a45346800a300183c2e3207a40a71bf579f2c5ec138538387f`。
- 当前snapshot：`sha256:0840ed40a921b36f42e61d97d8db8eb664882c5239bdd6f9336744cea3918e54`。
- 扫描当前crates、tests、scripts中的401个文本源/fixture文件：211 Rust、59 shell、35 Python、96 JSON；各文件字节哈希见source-corpus.json。
- 搜索真实amend/revalidate入口、CLI/MCP命令、confirmation字段、receipt构造器、helper定义和调用；补查不含confirm字面量的decisionEvidence/preflightDecisionDigest位置及公共测试模块。
- 15个文件含confirmation marker：12个tests路径，另3个是生产请求选项/validator实现，后者没有被当成测试确认调用。
- 字符串/注释屏蔽后的函数索引仅作定位，随后人工核对调用顺序、同一Contract路径与预期成功/拒绝。它不是完整控制流证明，不是新执行结果。
- 11处正向 amendment 后确认的数据/调用点中，10处有当前原生preflight，1处为上述缺口。9处record_human_preflight_review helper调用全部逐一核对；另2处是直接构造并成功记录确认数据。

## 正向调用点核对

表内数字为本次固定源码行号；“确认”对直接JSON构造标注为数据行，其成功controls调用在随后，helper调用则为实际调用行。

| 文件/测试 | amendment | 当前原生preflight | 确认数据/调用 | 判定 |
|---|---:|---:|---:|---|
| CLI scope_amendment.rs / same_intent_amendment_recovers | 194 | 219 | 243（数据） | 已满足 |
| Repository contract_amendment.rs / sensitive_amendment_requires_human_review_before_verification | 559 | 562 | 586 | 已满足 |
| 同文件 / post_verification_amendment_receipt_lists_invalidated_required_checks | 742 | 744 | 745 | 已满足 |
| lifecycle_entry.rs / scenario_coverage_plan_can_fill_only_a_missing_plan | 891 | 907 | 908 | 已满足 |
| 同文件 / first_typed_required_verification… 首次声明 | 1549 | 1573 | 1579 | 已满足 |
| 同测试 / 验证后的sources amendment | 1629 | 1641 | 1643 | 已满足 |
| 同文件 / verification_preconditions_accept_complete_repository_bound_custom_evidence | 2211 | 2272 | 2278 | 已满足 |
| 同文件 / empty_amendment_invalidation… | 2928 | 2935 | 2937 | 已满足 |
| material_review.rs / stale_receipt_after_valid_contract_amendment_is_classified_stale | 1221 | 1262 | 1270（数据） | 已满足 |
| status_projection.rs / status_does_not_admit_finish_when_required_scenario_controls_are_incomplete | 3088 | 3107 | 3108 | 已满足 |
| recovery_decision.rs / pending_retry_survives_a_bounded_contract_amendment | 1479 | 缺失 | 1488 | 唯一新增必要范围 |

精确完整文件名、函数名、源文件哈希与位置见source-review-matrix.json。对已满足的点不提出修改。

## 保留的负例和无关marker

| 位置 | 源码事实 | 处理 |
|---|---|---|
| CLI contract_amendment_processes.rs:668–807 | 原生amendment保留旧preflight字段，故意混绑当前Contract；CLI/MCP预期拒绝并断言Summary/Contract/journal/receipt不写 | 保留负例，不在拒绝前插入preflight |
| 同文件:353–656 | amendment429后已有preflight462；537构造的current_receipt只作为错误repository/Contract/snapshot副本的模板，随后expect_err，且断言不产生收据 | 不是缺失preflight的正向批准，不扩大范围 |
| 同文件:819–951 | 先有原生preflight，再在隔离fixture中变化missing/unknown/not_run、有效green/yellow/red及current Contract字段，按accepted参数检查结果 | 保留validator边界覆盖 |
| CLI scope_amendment.rs:278 | 已完成确认后的另一amend请求故意尝试改变intent，预期失败并断言Contract字节不变 | 无新的已接受amendment，不补确认 |
| Repository contract_amendment.rs:765及lifecycle_entry.rs:910 | 在已核对确认点之后还有amendment，用于失效/声明检查；后续没有正向新确认调用 | 不误报为本次模式 |
| MCP rpc.rs | work_item_amend字样只在工具清单；确认producer用于初始preflight/生命周期锁用例，1284–1356明确在1317先调用MCP preflight | 不属于post-amend缺口 |
| CLI collaboration_consistency.rs | 无amend入口；初始原生preflight在341，原synthetic decision helper在364调用 | 不扩大范围 |
| Repository preflight_review.rs | repository helper先原生preflight；刷新、旧/外来receipt拒绝与Contract变更用例分别核对，无缺失当前preflight的post-amend正向确认 | 不扩大范围 |
| Repository ci_quality_gate.rs | 1596先原生preflight，再构造1608确认数据，无amend入口 | 不扩大范围 |
| Protocol preflight_review.rs | 仅结构序列化/反序列化，无运行时确认写入 | 不扩大范围 |
| core/src/lib.rs、repository/src/lib.rs及governance_controls.rs | 生产选项生成/验证；非fixture确认调用 | 保留生产范围 |
| 额外decisionEvidence候选 | contract_preflight只断言digest；scenario_matrix_next_action删除旧decisionEvidence模拟待刷新状态；其余为生产schema、投影或历史恢复字段 | 不新增文件 |

## 单文件 typed amendment 草案

文件：`single-fixture.typed-amendment.DRAFT.request.json`

- 草案SHA-256：`e737071c8f02f6f4461a6a9267ff99ca9b3835fbfeb65f1e69f7aca09b38cd4e`。
- expectedContractDigest固定为当前e7245a06…，不推断后继digest。
- 仅向 /scope 加入上述一个测试路径。
- /concurrencyBoundary使用当前完整对象，仅向implementationPaths追加同一路径，其余值和数组顺序保持。
- 不扩大生产源码、其他测试、parity、验证命令、policy或清理范围。
- 当前草案未应用；尚未由Runtime生成新Contract或校验实际执行准入。未来应用前必须重核当前Contract、宿主恢复及Raydot解除暂停；不得用本草案作为启动测试的许可。

## 保全与下一步

805031的精确身份为startTimeTicks24465201，保持暂停T；1089个子僵尸事实及原exec1926保留，/tmp原字节备份不删除。本轮没有发送CONT/KILL，没有尝试注入、提权或系统权限修改；宿主恢复仍由Raydot独立核查。

当前Runtime仍checkpointed / verification not_ready，humanDecisionRequired=false；其run_verification准入不解除本次明确宿主暂停。原完整失败、收据完整性与同Contract有效批准复用保持，未声明最终通过。

交Raydot审阅结论：**关联范围保持recovery_decision.rs单文件**。宿主恢复前不应用草案、不改源码、不运行测试/验证。后续真实准入若要求额外输入，展示实际绑定；不伪造或重复记录同一政策批准。
