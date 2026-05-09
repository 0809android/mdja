use mdja::Document;

fn main() {
    println!("=== mdja デモ ===\n");

    // 基本的な使い方
    println!("【基本的なMarkdown変換】");
    let markdown = "# はじめに\n\nこれは**太字**で、これは*斜体*です。";
    let doc = Document::parse(markdown);
    println!("入力:\n{}\n", markdown);
    println!("HTML出力:\n{}\n", doc.html);

    // frontmatter付き
    println!("【frontmatter解析】");
    let markdown_with_meta = r#"---
title: Rustで始めるMarkdown
author: Taro
date: 2025-01-17
tags: rust, markdown
---

# Markdownパーサー

**mdja**は日本語に最適化されたMarkdownパーサーです。
"#;

    let doc = Document::parse(markdown_with_meta);
    println!("メタデータ:");
    for (key, value) in &doc.metadata {
        println!("  {}: {}", key, value);
    }
    println!();

    // 目次生成
    println!("【目次生成】");
    let markdown_with_toc = r#"
# はじめに

## インストール方法

### Rustから使う

### Pythonから使う

## 使い方

### 基本的な使い方

### 応用例
"#;

    let doc = Document::parse(markdown_with_toc);
    println!("目次:\n{}", doc.toc);

    // 読了時間
    println!("【読了時間計算】");
    let long_text = "あ".repeat(800);
    let doc = Document::parse(&long_text);
    println!("テキスト: {}文字", long_text.len());
    println!("読了時間: {}分\n", doc.reading_time);

    // GFM機能
    println!("【GFM（GitHub Flavored Markdown）機能】");

    // テーブル
    println!("テーブル:");
    let table_md = r#"
| 機能 | 対応状況 |
|------|---------|
| テーブル | ✓ |
| タスクリスト | ✓ |
| 取り消し線 | ✓ |
"#;
    let doc = Document::parse(table_md);
    println!("{}\n", doc.html);

    // タスクリスト
    println!("タスクリスト:");
    let tasklist_md = r#"
- [x] 基本機能実装
- [x] テスト作成
- [ ] ドキュメント整備
- [ ] crates.io公開
"#;
    let doc = Document::parse(tasklist_md);
    println!("{}\n", doc.html);

    // 取り消し線
    println!("取り消し線:");
    let strikethrough_md = "~~古い情報~~ 新しい情報";
    let doc = Document::parse(strikethrough_md);
    println!("{}\n", doc.html);

    // 日本語見出しアンカー
    println!("【日本語見出しアンカー】");
    let jp_headings = r#"
# はじめに
## インストール方法
## 使い方
### 基本的な使い方
"#;
    let doc = Document::parse(jp_headings);
    println!("見出し一覧:");
    for heading in &doc.headings {
        println!(
            "  {} (id: {}) - レベル{}",
            heading.text, heading.id, heading.level
        );
    }
    println!();

    // コードブロック
    println!("【シンタックスハイライト対応】");
    let code_md = r#"
```rust
fn main() {
    println!("Hello, mdja!");
}
```
"#;
    let doc = Document::parse(code_md);
    println!("{}", doc.html);
}
