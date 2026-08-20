# 敵対的検証

セキュリティ境界は fail-closed と evidence-driven です。conformance corpus は文字列ではなく、
decision state、blockers、unknowns、safe actions、required checks、authority、outcome state の
意味を比較します。

runtime 境界テストでは、repository text を data として扱うこと、Work Item ID の path traversal
防止、MCP evidence path の repository 内制限、allowlist と対象 cwd の検証、fresh な passed receipt
なしに finish が完了を自己宣言できないことも確認します。

失敗または未知の provider result は常に non-green です。human authority は decision requirement
を解決できますが、verification receipt を捏造できません。
