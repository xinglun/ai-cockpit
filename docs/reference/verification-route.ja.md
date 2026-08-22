# Verification route

AI Cockpit は、明示的なステージ、ポリシー計画、依存/影響グラフ、実行、証拠レシート、CI 境界を通して検証事実を扱います。

## ステージ

ステージは `task`、`pre_ci`、`pr`、`merge`、`release` に固定されます。`pre_ci` はローカルフィードバックであり、
`pr`、`merge`、`release` は独立した provider または保護ゲートです。不明なステージは fail-closed になります。
ステージは assurance レベルではありません。

## 直交する二つの軸

`VerificationTier`（`T0`–`T3`）は検証強度、`EvidenceAssurance`
（`SelfDeclared`、`RepositoryVerified`、`ProviderVerified`、`EnterpriseVerified`）は証拠の来歴を表します。
T3 は provider/enterprise assurance を意味せず、実際の provider または外部証拠が必要です。

Planner は tier を提案できますが、要求は Organization Policy、Project Policy、Release Policy、または Protected Gate に追跡可能でなければなりません。

## ルートとレシート

ルートは `DependencyConfidence`（`complete`、`partial`、`unknown`）と影響/保護ノードの事実を保持します。
コストと再利用は助言情報であり、要求を弱めません。`VerificationPlanReceipt` はステージ、初期/最終 tier、
独立した assurance、理由、エスカレーション、実行事実を記録し、tier の降格は fail-closed です。

Work Item route ではさらに `workItemId`、`repositoryId`、repository snapshot digest、`baseRevision`、Policy 参照、required tier/assurance、affected paths、dependency confidence を bind します。Lifecycle 検証は宣言された Policy requirement を再解決し、binding の欠落・stale・改ざんを拒否します。`pr`、`merge`、`release` route は実行境界で有効な base revision を必須とし、`task` は base revision に依存しません。

Effective Policy が `T3` または `ProviderVerified` を要求する場合、local Runtime はその要求を満たしたと主張できず、完了 Evidence を書く前に停止します。Hosted/provider Evidence は実際の provider から取得する必要があります。typed verification requirement のない repository は、従来の no-policy route と legacy receipt 互換性を維持します。

物理実行は共有できますが、各 Work Item は固有の束縛済み証拠レシートを持ちます。他の Work Item のレシートを認可証拠として流用してはなりません。

## CI 境界

`pre_ci` は hosted CI の証拠ではありません。CI shadow 期間は Runtime 検証と既存 Cargo チェックを併用し、CI 結果で赤いガバナンス判断を上書きしません。
重複チェックの削除は、後続の明示的な収束フェーズで行います。

コスト観測は助言テレメトリに限られ、不明または partial の証拠を緑のガバナンスに変えません。
