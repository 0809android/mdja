// mdja WASM bindings example (Node.js)
const { Document } = require('./index.js');

console.log('=== mdja WASM デモ ===\n');

// 基本的な使い方
console.log('【基本的なMarkdown変換】');
const markdown = '# はじめに\n\nこれは**太字**で、これは*斜体*です。';
const doc = Document.parse(markdown);
console.log(`入力:\n${markdown}\n`);
console.log(`HTML出力:\n${doc.html}\n`);

// frontmatter付き
console.log('【frontmatter解析】');
const markdownWithMeta = `---
title: Rustで始めるMarkdown
author: Taro
date: 2025-01-17
---

# Markdownパーサー

**mdja**は日本語に最適化されたMarkdownパーサーです。
`;

const doc2 = Document.parse(markdownWithMeta);
console.log('メタデータ:');
const metadata = doc2.metadata;
for (const [key, value] of Object.entries(metadata)) {
    console.log(`  ${key}: ${value}`);
}
console.log();

// 目次生成
console.log('【目次生成】');
const markdownWithToc = `
# はじめに

## インストール方法

### Rustから使う

### Pythonから使う

## 使い方
`;

const doc3 = Document.parse(markdownWithToc);
console.log(`目次:\n${doc3.toc}`);

// 読了時間
console.log('【読了時間計算】');
const longText = 'あ'.repeat(800);
const doc4 = Document.parse(longText);
console.log(`テキスト: ${longText.length}文字`);
console.log(`読了時間: ${doc4.readingTime}分\n`);

// 見出し一覧
console.log('【見出し一覧】');
const jpHeadings = `
# はじめに
## インストール方法
## 使い方
`;
const doc5 = Document.parse(jpHeadings);
console.log('見出し:');
for (const heading of doc5.headings) {
    const indent = '  '.repeat(heading.level - 1);
    console.log(`${indent}- ${heading.text} (id: ${heading.id})`);
}
console.log();

// シンプルな変換
console.log('【シンプルなHTML変換】');
const html = Document.toHtml('**太字** と *斜体*');
console.log(`HTML: ${html}`);
