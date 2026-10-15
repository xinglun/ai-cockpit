# WI-1084 收尾阻塞与最小提案（未执行）

## 已接受的验证与真实边界

Raydot 独立接受 head `7f723bfb7a6dab4c07952f3fca1a7fe701e91faf` 的 Native13/13 和 CI38718628905 四任务成功，无新增P1/P2。Windows library114/114，此前失败的 second-hop 用例本轮为ok；quality40通过、2不适用。Windows library实际905.94秒、本地Native645.2456/900秒分别保留。`ci-final-bundle.json` 保存完整 decoded API 日志与步骤元数据。

投影准入允许finish，blockers=[]，humanDecisionRequired=false，verification=verified/fresh。正常绑定finish实际尝试一次，exit1：canonical material Unknowns: repository_material_inspection_unavailable。Runtime写入failedGate=finish.lifecycle、outcomeState=blocked，生命周期仍checkpointed。未调用archive，未生成归档。Outcome查询仍显示已声明验证通过；实际finish拒绝必须单独报告。

唯一规范材料Unknown是 `crates/cockpit-repository/tests/closeout_recovery.rs`，原因为readable_committed_rust_syntax_unknown；reviewable=true，无Finding。当前reviewEnabled=false，reviewDiagnostic=material_review_not_enabled。303个既有证据与源码路径字节未改。Contract、正式回执、两个RED及旧失败全部保留。精确request/manifest/source/hunk/blob/Contract/Runtime绑定见 `blocked-closeout-bindings.json` 和 `material-plan.stdout`。

## 最小受支持提案

`proposed-amendment.request.json` 仅提议在Contract的governanceProfile中声明materialInspectionReview：限定上述唯一Unknown和cause、self_declared、reviewerActor=agent:Raydot、明确接受剩余风险。当前governanceProfile为null；原有任何其他字段必须保持。Runtime typed amendment还会自动安装material-inspection-review capability guard，并保留已有capabilities。提案没有应用；authoritySource文字仍须Raydot审阅并取得必要授权，不能被当成已有人类批准。

范围：仅显式启用现有材料审阅机制；不改源码、scanner、断言、门禁、目标、原始证据或其他WI。兼容性：现有严格schema和Runtime已支持该profile，代理决策的assurance只能self_declared，不能升格为人类/发布/提供方批准。

这是敏感的人类持有Contract变化。获得必要授权后，才可通过work-item amend --request记录真实变更和revalidation，再按新准入取得所需的全新identity-bound policy review，生成新的material-review plan，由被声明的评审者提供严格decision和逐项digest-bound evidence。原始scanner Unknown继续保留；仅允许通过受支持receipt消解许可的effective Unknown。其他cause、Finding、身份或字节不匹配继续阻断。

验收：新Contract/profile/capability guard合法；source、Contract、request、manifest、hunk、blob、reviewer和证据digest全部对应；303个历史路径不改；不批量清Unknown；被新Contract失效的验证按实际Runtime要求恢复，本地完整验证继续900秒上限；新记录head的必需CI如实核验；finish/archive必须实际成功并有Runtime记录，才可声称已归档。旧13/13回执只属于其原Contract，不能冒充新Contract验收。

不通过修改其他工作区、scanner阈值或等价代码改写来制造Clean。本交接没有授权额外源码改动。

## 真人关闭与资源批准尚不具备归档绑定

当前Contract没有resourceContext，不补造provider finalization计划或receipt。潜在对象仅PR1040、分支codex/wi1084-closeout-receipt-integrity、工作树/workspace/ai-cockpit-wi1084-closeout-receipt-integrity和其现有Git common-dir。尚未选择control checkout；旧dirty controls及所有WI1078资源保留。后续ordinary/provider路线须Raydot按真实准入确认。

没有archive digest，当前不能构造可执行的close批准。human close、Draft转Ready、合并、分支/工作树删除和额外清理均停在执行前。收尾门禁实际通过后，再准备一次最终head、Contract、archive、receipt和精确cleanup对象确认。

## 空间监控

遵照Ray新增要求，每轮构建/完整验证前后检查工作区、临时目录和实际构建缓存所在文件系统的可用字节和inode；长任务适度采样，避免高频遍历。依本轮采样估计临时构建峰值并留余量；不足先报告和在既有授权内处理，不等ENOSPC。当前只进行小型材料记录，余量足够；后续长任务须重新采样估计。监控不增加删除授权。当前实测见storage-monitor-current.json。
