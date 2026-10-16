# 待 Raydot 的最终材料绑定审阅

这是审阅请求，不是批准或决定收据。两行修正源码已提交并推送，source HEAD为 `f1fc7b88766d32b03d9bb705133afe8a1cfdc44a`。

| 绑定 | 当前值 |
|---|---|
| repositoryId | `sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b` |
| Work Item | `WI-1085-legacy-v1-closeout-source-registration` |
| Contract | `sha256:7fea3f7d77255e61459ff1c51ac46ce9e9b73ff4a627939ad3b844d96ed093a1` |
| requestDigest | `sha256:661c2ccff137c0ab8ba11172bf38443de3e97f9c1daad94f762658ff50d47790` |
| sourceSnapshotDigest | `sha256:034d7b8bd19efcac2a577bb43915337700f5fcad0ff793f4b6adc4af23eb7b6c` |
| materialManifestDigest | `sha256:c5c06bb3647d37f31da300b11389cd3ff562d8c4ad3a94eee142302568de1c0b` |
| profileDigest | `sha256:8eed971dcbb631b144496cbd91fe2d92059a7aea023552d7dffa2187c6832045` |
| analysisImplementationDigest | `sha256:9dcee9c81b0888942520a821f502071bd078a16da9473fdd63f8568a87e54f80` |
| effectivePolicyDigest | `sha256:4022dbdd989b6b05ee6116c7fc73b9465a373c1ed42b01999f77ea65d8d8921a` |
| 原生 request 文件 raw SHA256 | `acce52e26cfeb8e8e1ccf7fb54aa99a69352b71a44c55517437a4e5ad0fe800f` |

旧原始请求94bdbf…和scope调整后的中间请求81878b…均不可套用。当前相较81878b…仅增加两份clean测试条目（material_review.rs与usage.rs），27条材料中原25条的全部事实保持一致；sourceSnapshot/manifest/requestDigest因此更新，Contract/profile/analysis/policy均不变。

仅两项可审阅Unknown，均为 readable_committed_rust_syntax_unknown，内容与hunk/blob digest和旧请求完全相同：

- `crates/cockpit-repository/src/legacy_closeout_source.rs`
  - changedHunkDigest: `sha256:efe7571aebfc0cad1a1fa7a3b2288241f133640c46c659b4ab2675791541bdef`
  - afterBlobDigest: `sha256:a1bf4777908ab94c3645eb23685b93a2d29d44902680a0ca124b415530752ee7`
- `crates/cockpit-repository/tests/legacy_closeout_source.rs`
  - changedHunkDigest: `sha256:d8679cdcd44abfa53350e8a4ad6856a70943db3e20358d90b00a1edcd7381af4`
  - afterBlobDigest: `sha256:8ff977bb0bae80322a0a95e4a97e2298f544611384eb1a41201bd801d64789a5`

0 Finding；blockedByFinding=false；reviewEnabled=true。请 Raydot 对**此 requestDigest**作出真实材料审阅决定；受当前profile约束的reviewerActor为agent:Raydot、assurance=self_declared。尚未记录此决定。

真人confirm_review已真实原生记录，当前native humanDecisionRequired=false；没有新的真人门禁。此材料审阅不替代正式验证或验收。收到实际决定并原生记录后，先重新核对空间/预算/准入，再按单worker、同一隔离target、890+10秒、现有Tini与observer runner执行正式验证。当前未运行修正后的测试，也未宣称绿色。
