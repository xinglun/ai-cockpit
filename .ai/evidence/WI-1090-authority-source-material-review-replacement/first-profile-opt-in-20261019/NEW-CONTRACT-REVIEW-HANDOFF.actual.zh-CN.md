Outcome: 🟡 首次材料政策修约成功；等待实际新 Contract 真人确认

原始政策请求ff93…323f执行恰一次，exit0，native seq3。所有8项材料的blob/hunk、manifest、scanner身份、Finding0与唯一raw syntax Unknown均匹配已批准固定材料。Contract只改变指定profile与Runtime自动推导capability marker。

Native preflight要求新的confirm_review。已有c9确认仅保留历史，未重放；Ray的有条件材料风险接受不被记录成新Contract已经审阅。consent文件未创建，材料record未调用，未重跑测试；源码和三语文档保持b1字节。

| 绑定字段 | 实际值 |
|---|---|
| repositoryId | `sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b` |
| workItemId | `WI-1090-authority-source-material-review-replacement` |
| contractDigest | `sha256:b6d2f736ba2c9872294578dea6e6c7dfae528f0c46c33ce603599d31b73a5214` |
| preflightDecisionDigest | `sha256:530d11843008f5fe306fdc19fae652d30724caa9ab39d7293eabd7c0d92cadca` |
| repositorySnapshotDigest | `sha256:8706ea4bcac43244c73cb2431c2eb6970ee0c448a9f2bcdc6f130aa58fd9cc25` |
| runtimeVersion | `1.0.1` |
| runtimeDigest | `sha256:b5cbd8570df5d3cb2004ae517fe88df6136c66d92fc1913070d18f2c8f67f257` |
| protocolVersion | `1` |
| environmentObservationDigest | `sha256:8c6d22e61ed275b8eb9f5f61d45c13108d003436fddee33f30561955898d1a9e` |
| materialRequestDigest | `sha256:c5ddefb60c57f35b2f1da6cbc4a4268cbf1039aa9bfe15b961a764d9dd23dae7` |
| materialSourceSnapshotDigest | `sha256:ffdcd5af8387e969dec46902088117057f82d986ef948d5e3d69c09776c52329` |
| materialManifestDigest | `sha256:13d21e04a58c46874688c8998c67ea2582efe9be74f0070af3f37ad4ccc943ac` |
| materialInspectionReviewProfileDigest | `sha256:e2a40be4695c566d35fac65e5854678b8877cb9d23828d10cea3a82cdaf172e9` |
| policyRequestRawSha256 | `ff93edd6797c0c6e973e1e3355024d84e96d4ca8d616604b37eec859785f323f` |
| nativeAmendmentRequestDigest | `sha256:f73939ab70c9c2fc750360f306552d2c6055acc577ce5ef2c0c897e09fd2ab94` |
| nativeAmendmentJournalDigest | `sha256:aba91250f9f3d5afc32cf729e73ac5da931ed9d7e5209b34cc80e6d44ffca9c0` |
| admissionDigest | `sha256:73e09598798dcb016b850968cdf0586dd273274cb6fb38195b692180468041c2` |
| preflightAt | `2026-10-19T22:28:32Z` |

待Ray确认此新完整绑定后，再由agent:Codex按实际授权原生记录human:Ray的confirm_review并重新查询准入。材料风险条件继续有效，但不得预造新绑定审阅或继承旧决定。

当前verification因合法政策修约为stale/not_ready，旧七项真实通过收据原样保留，不称新Contract已验证。Native status blocking=false、humanDecisionRequired=true、admissionState=needs_human_decision；safeActions中出现材料record不替代真人门槛。

环境：固定Runtime b5cbd…，agent doctor VERIFIED，doctor red仅已知self-host runtimeCodeInRepository=true；兼容性COMPATIBLE。WI1090无coordination registration，不伪造generation或绑定，也未修改任何协调记录。

其他WI、Sentinel、发布、采用、真实终态均未操作。收益仍未声明；本次仅建立获批政策及可复核新绑定。原生Outcome已返回；宿主显示确认未知。
