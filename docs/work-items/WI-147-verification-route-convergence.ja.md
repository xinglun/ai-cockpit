# WI-147 — Verification route convergence

## 目的

Contract と Policy の要求を型付き VerificationStage、計画、依存/影響事実、実行レシート、CI 境界へ接続し、再利用やコスト最適化がガバナンス要求を弱めないようにします。

## 設計基準

検証セマンティクスを先に統一します。`VerificationTier` と `EvidenceAssurance` は直交し、Planner はポリシーに追跡可能な要求だけを提案します。
依存信頼度は partial を許容し、物理実行の共有は認可レシートの共有ではありません。CI shadow 比較は維持します。

## 受入境界

不明なステージと tier の降格は fail-closed です。レシートはルート事実と Runtime/Repository identity を記録します。
コスト観測は助言情報、空の計画の並列度はゼロ、無効な identity は unknown です。既存 Cargo チェックの削除や provider/enterprise assurance の捏造は行いません。

[Verification route](../reference/verification-route.ja.md) と英語・中国語版を参照してください。
