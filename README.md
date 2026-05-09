# mdja

日本語に最適化されたMarkdownパーサー

[![Crates.io](https://img.shields.io/crates/v/mdja.svg)](https://crates.io/crates/mdja)
[![Documentation](https://docs.rs/mdja/badge.svg)](https://docs.rs/mdja)
[![License](https://img.shields.io/crates/l/mdja.svg)](https://github.com/0809android/mdja#license)

## 特徴

- ✨ **CommonMark + GFM完全対応** - [comrak](https://github.com/kivikakk/comrak)基盤で高速・正確
- ✨ **日本語見出しアンカー生成** - `# はじめに` → `id="hajimeni"`
- ✨ **目次（TOC）自動生成** - Markdown形式で目次を出力
- ✨ **HTML/階層TOC対応** - Markdown TOC、HTML TOC、ツリー構造を出力
- ✨ **読了時間計算** - 日本語文字数を考慮した精密な計算
- ✨ **frontmatter解析** - YAMLメタデータの自動抽出
- ✨ **型付きメタデータ** - 文字列化したメタデータとYAML型を保ったメタデータを両方取得
- ✨ **GFM機能** - テーブル、タスクリスト、取り消し線、自動リンクなど
- ✨ **マルチ言語対応** - Rust、Python、JavaScript（WASM）、CLIツール
- ✨ **シンプルなAPI** - 1行でパース可能
- ✨ **充実したテスト** - ユニットテストとドキュメントテストで主要機能を検証

## インストール

### Rustから使う

`Cargo.toml`に追加：

```toml
[dependencies]
mdja = "0.1.1"
```

### Pythonから使う

```bash
maturin develop --features python
```

### JavaScriptから使う

```bash
wasm-pack build --features wasm --target bundler --out-dir js/pkg
```

### CLIツールとして使う

```bash
cargo install mdja
```

## 基本的な使い方

### Rust

```rust
use mdja::Document;

fn main() {
    let markdown = r#"
---
title: サンプル記事
author: Taro
---

# はじめに

これは**サンプル**記事です。

## 特徴

- 簡単に使える
- 高速
- 日本語対応
"#;

    let doc = Document::parse(markdown);

    // HTML出力
    println!("{}", doc.html);

    // メタデータ
    println!("タイトル: {}", doc.metadata.get("title").unwrap());

    // 目次
    println!("## 目次\n{}", doc.toc);

    // 読了時間
    println!("読了時間: {}分", doc.reading_time);

    // 見出し一覧
    for heading in doc.headings {
        println!("{} (id: {})", heading.text, heading.id);
    }
}
```

### シンプルな変換

```rust
use mdja::Document;

let html = Document::to_html("# Hello\n\n**World**");
println!("{}", html);
```

### CLIツール

```bash
# ファイルからHTML生成
mdja input.md output.html

# 標準出力に表示
mdja input.md

# 標準入力から読み込み
cat input.md | mdja

# JSONで構造化結果を出力
mdja --json input.md

# TOCだけを出力
mdja --toc --toc-max-level 3 input.md

# frontmatterだけをJSONで出力
mdja --metadata input.md
```

## API リファレンス

### `Document::parse(markdown: &str) -> Document`

Markdownをパースして構造化データを返します。

```rust
let doc = Document::parse("# Hello");
```

**返り値: `Document`**

```rust
pub struct Document {
    pub html: String,                              // HTML出力
    pub metadata: HashMap<String, String>,         // 文字列化したfrontmatterメタデータ
    pub metadata_raw: HashMap<String, Value>,      // YAML型を保ったfrontmatterメタデータ
    pub toc: String,                               // 目次（Markdown形式）
    pub toc_html: String,                          // 目次（HTML形式）
    pub toc_tree: Vec<TocItem>,                    // 階層化された目次
    pub headings: Vec<Heading>,                    // 見出し一覧
    pub reading_time: usize,                       // 読了時間（分）
}
```

### `Document::to_html(markdown: &str) -> String`

シンプルなHTML変換（メタデータ不要な場合）。

```rust
let html = Document::to_html("**太字**");
```

### `Heading`

```rust
pub struct Heading {
    pub text: String,   // 見出しテキスト
    pub level: usize,   // レベル (1-6)
    pub id: String,     // アンカーID
}
```

## 機能詳細

### 解析オプション

`ParseOptions` で読了速度、TOCの対象レベル、アンカー生成方式を指定できます。

```rust
use mdja::{AnchorStyle, Document, ParseOptions};

let options = ParseOptions {
    toc_min_level: 2,
    toc_max_level: 3,
    reading_speed_japanese: 500,
    reading_speed_english: 250,
    anchor_style: AnchorStyle::Romaji,
};

let doc = Document::parse_with_options("# はじめに", &options);
```

### frontmatter解析

YAML形式のメタデータを自動抽出します。

```markdown
---
title: 記事タイトル
author: Taro
date: 2025-01-17
tags: rust, markdown
---

# 本文
```

```rust
let doc = Document::parse(markdown);
println!("{}", doc.metadata.get("title").unwrap());  // "記事タイトル"
println!("{}", doc.metadata.get("author").unwrap()); // "Taro"
```

### 日本語見出しアンカー

日本語の見出しからアンカーIDを自動生成します。

```markdown
# はじめに
## インストール方法
```

↓

```html
<h1><a href="#hajimeni" aria-hidden="true" class="anchor" id="hajimeni"></a>はじめに</h1>
<h2><a href="#insutoruhouhou" aria-hidden="true" class="anchor" id="insutoruhouhou"></a>インストール方法</h2>
```

```rust
let doc = Document::parse("# はじめに");
assert_eq!(doc.headings[0].id, "hajimeni");
```

拗音や促音にも対応しています。

```rust
let doc = Document::parse("# キャッシュ\n## ティーカップ");
assert_eq!(doc.headings[0].id, "kyasshu");
assert_eq!(doc.headings[1].id, "tikappu");
```

### 目次（TOC）生成

見出しから自動的にMarkdown形式、HTML形式、ツリー形式の目次を生成します。

```rust
let doc = Document::parse("# H1\n## H2\n### H3");
println!("{}", doc.toc);
// - [H1](#h1)
//   - [H2](#h2)
//     - [H3](#h3)

println!("{}", doc.toc_html);
```

### 読了時間計算

日本語と英語の文字数を考慮して読了時間を計算します。

- 日本語: 400文字/分
- 英語: 200単語/分

```rust
let text = "あ".repeat(800);
let doc = Document::parse(&text);
assert_eq!(doc.reading_time, 2);  // 2分
```

### GFM（GitHub Flavored Markdown）

#### テーブル

```markdown
| ヘッダー1 | ヘッダー2 |
|----------|----------|
| セル1     | セル2     |
```

#### タスクリスト

```markdown
- [x] 完了したタスク
- [ ] 未完了のタスク
```

#### 取り消し線

```markdown
~~古い情報~~ 新しい情報
```

#### 自動リンク

```markdown
https://example.com
```

#### 脚注

```markdown
本文[^1]

[^1]: 脚注の内容
```

## ユースケース

- **静的サイトジェネレーター** - ブログ記事のHTML変換
- **CMSシステム** - ユーザー入力のMarkdown処理
- **ドキュメント生成** - 技術文書のHTML化
- **APIサーバー** - Markdown→HTML変換エンドポイント
- **CLIツール** - バッチ処理でのMarkdown変換
- **Webアプリケーション** - WASM経由でブラウザ内変換

## サンプル実行

```bash
# サンプルプログラムを実行
cargo run --example basic

# テストを実行
cargo test

# CLIツールをビルド
cargo build --release

# CLIツールを使用
./target/release/mdja examples/sample.md

# ドキュメントを生成
cargo doc --open
```

## パフォーマンス

[comrak](https://github.com/kivikakk/comrak)をベースにしているため、高速かつ正確なパースが可能です。

- CommonMark準拠
- GFM完全対応
- 大規模ドキュメントにも対応

## 技術スタック

- **パーサー**: [comrak](https://github.com/kivikakk/comrak) (CommonMark + GFM)
- **YAML**: [serde_yaml](https://github.com/dtolnay/serde-yaml)
- **正規表現**: [regex](https://github.com/rust-lang/regex)
- **Python bindings**: [PyO3](https://github.com/PyO3/pyo3) (オプション)
- **WASM bindings**: [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) (オプション)

## ロードマップ

- [x] 基本的なMarkdownパース
- [x] frontmatter解析
- [x] 日本語見出しアンカー生成
- [x] 目次生成
- [x] 読了時間計算
- [x] GFM対応
- [x] CLIツール
- [ ] Pythonバインディング
- [ ] WASMバインディング
- [ ] シンタックスハイライト統合
- [ ] カスタムレンダラー
- [ ] プラグインシステム

## コントリビューション

プルリクエストを歓迎します！

1. このリポジトリをフォーク
2. フィーチャーブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

## ライセンス

このプロジェクトは、以下のいずれかのライセンスでデュアルライセンスされています：

- MITライセンス ([LICENSE-MIT](LICENSE-MIT) または http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) または http://www.apache.org/licenses/LICENSE-2.0)

お好みのライセンスをお選びください。

## 謝辞

このライブラリは[comrak](https://github.com/kivikakk/comrak)をベースにしており、日本語ブログ・ドキュメント向けに最適化された追加機能を提供しています。

---

## English Summary

**mdja** - Japanese-optimized Markdown parser

### Features

- CommonMark + GFM support (powered by comrak)
- Japanese heading anchor generation (`# はじめに` → `id="hajimeni"`)
- Automatic table of contents generation
- Reading time calculation (Japanese character aware)
- Frontmatter parsing (YAML)
- Multi-language support (Rust, Python, JavaScript, CLI)
- Simple API

### Installation

```toml
[dependencies]
mdja = "0.1.1"
```

### Usage

```rust
use mdja::Document;

let doc = Document::parse("# Hello\n\n**World**");
println!("{}", doc.html);
println!("Reading time: {} min", doc.reading_time);
```

### License

MIT OR Apache-2.0
