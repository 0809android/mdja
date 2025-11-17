#!/usr/bin/env python3
"""
mdja Python bindings example
"""

import mdja

def main():
    print("=== mdja Python デモ ===\n")

    # 基本的な使い方
    print("【基本的なMarkdown変換】")
    markdown = "# はじめに\n\nこれは**太字**で、これは*斜体*です。"
    doc = mdja.Document.parse(markdown)
    print(f"入力:\n{markdown}\n")
    print(f"HTML出力:\n{doc.html}\n")

    # frontmatter付き
    print("【frontmatter解析】")
    markdown_with_meta = """---
title: Rustで始めるMarkdown
author: Taro
date: 2025-01-17
tags: rust, markdown
---

# Markdownパーサー

**mdja**は日本語に最適化されたMarkdownパーサーです。
"""

    doc = mdja.Document.parse(markdown_with_meta)
    print("メタデータ:")
    for key, value in doc.metadata.items():
        print(f"  {key}: {value}")
    print()

    # 目次生成
    print("【目次生成】")
    markdown_with_toc = """
# はじめに

## インストール方法

### Rustから使う

### Pythonから使う

## 使い方

### 基本的な使い方

### 応用例
"""

    doc = mdja.Document.parse(markdown_with_toc)
    print(f"目次:\n{doc.toc}")

    # 読了時間
    print("【読了時間計算】")
    long_text = "あ" * 800
    doc = mdja.Document.parse(long_text)
    print(f"テキスト: {len(long_text)}文字")
    print(f"読了時間: {doc.reading_time}分\n")

    # 見出し一覧
    print("【見出し一覧】")
    jp_headings = """
# はじめに
## インストール方法
## 使い方
### 基本的な使い方
"""
    doc = mdja.Document.parse(jp_headings)
    print("見出し:")
    for heading in doc.headings:
        indent = "  " * (heading.level - 1)
        print(f"{indent}- {heading.text} (id: {heading.id})")
    print()

    # シンプルな変換
    print("【シンプルなHTML変換】")
    html = mdja.Document.to_html("**太字** と *斜体*")
    print(f"HTML: {html}")

    # Document オブジェクトの表示
    print("\n【Documentオブジェクト】")
    doc = mdja.Document.parse("# テスト\n\n本文")
    print(f"repr: {repr(doc)}")
    print(f"読了時間: {doc.reading_time}分")


if __name__ == "__main__":
    main()
