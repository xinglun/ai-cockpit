# Bootstrap Work Item

この repository には AI Cockpit がまだ install されておらず、V1 template を
install してはいけません。Rust runtime が自分自身を governance できるまで、
すべての変更を Markdown Work Item に記録し、人がレビューします。

各 Work Item は一つの branch、base revision、change scope、evidence bundle、
outcome を使います。文章だけで完了を宣言することはできません。

必須セクションは Intent、Goal、Scope、Out of Scope、Sources、Unknowns、
Acceptance Criteria、Required Evidence、Base Revision、Changed Files、
Verification、Human Decisions、Outcome です。

canonical な英語ファイルには `.zh-CN.md` と `.ja.md` の意味的同等版を用意します。
Runtime behavior を変更する場合、三言語の文書を同じ Work Item で更新しなければ
未完了です。

WI-16 完了までは通常の Rust command を使い、V1 の `make ai-*` command は使いません。

