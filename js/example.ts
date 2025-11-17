// mdja WASM bindings example (TypeScript)
import { Document } from './index';

console.log('=== mdja WASM TypeScript デモ ===\n');

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
const metadata = doc2.metadata as Record<string, string>;
for (const [key, value] of Object.entries(metadata)) {
    console.log(`  ${key}: ${value}`);
}
console.log();

// TypeScript type annotations
interface Heading {
    text: string;
    level: number;
    id: string;
}

// 見出し一覧
console.log('【見出し一覧】');
const jpHeadings = `
# はじめに
## インストール方法
## 使い方
`;
const doc3 = Document.parse(jpHeadings);
console.log('見出し:');
const headings = doc3.headings as Heading[];
for (const heading of headings) {
    const indent = '  '.repeat(heading.level - 1);
    console.log(`${indent}- ${heading.text} (id: ${heading.id})`);
}
console.log();

// シンプルな変換
console.log('【シンプルなHTML変換】');
const html: string = Document.toHtml('**太字** と *斜体*');
console.log(`HTML: ${html}`);
