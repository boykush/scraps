---
name: ade-rule-dsl
description: ADE の rule DSL で書かれたルールを読むときに使う。adr の MCP の list_rules が返すルールや .rule ファイルが、何を求めていて何を対象から外しているかを、DSL の reference で確かめる。
---

# ADE の rule DSL

[boykush/adr](https://github.com/boykush/adr) のルールは、決定のうち各リポジトリのセッションが守ることを [ADE](https://github.com/phi42/ad-enforcement-tool) の DSL で書いたもの。adr の MCP の `list_rules` が、書かれたままを返す。

ルールが何を求めているかは [references/dsl-reference.md](references/dsl-reference.md) で確かめる。記憶や似た DSL から推し量らない。reference は、adr がルールを検査するのと同じ版の ADE から写してある。

## 読む

- ルールは、それを読んだセッションが作業しているリポジトリについての言明。パスはそのリポジトリの root から見る
- `#` のコメントは、パターンや `exclude` をそう書いた理由。なぜそう決めたかは決定が持つので、`get_decision` で読む
- ルールを実行する仕組みは無く、ADE の CLI や plugin も入れない。作業がルールを満たしているかは、作業するセッション自身と PR のレビューが確かめる

ルールを書くのは boykush/adr だけで、書き方はそちらが持つ。別のリポジトリでルールを足したくなったら、書かずに adr へ持ち込む。
