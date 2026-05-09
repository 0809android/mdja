//! # mdja
//!
//! 日本語に最適化されたMarkdownパーサー
//!
//! ## 特徴
//!
//! - CommonMark + GFM完全対応（comrak基盤）
//! - 日本語見出しアンカー生成
//! - 目次（TOC）自動生成
//! - 読了時間計算（日本語文字数対応）
//! - frontmatter解析（YAML）
//! - シンタックスハイライト対応
//!
//! ## 使い方
//!
//! ```rust
//! use mdja::Document;
//!
//! let markdown = r#"
//! ---
//! title: サンプル記事
//! author: Taro
//! ---
//!
//! # はじめに
//!
//! これは**サンプル**です。
//! "#;
//!
//! let doc = Document::parse(markdown);
//! println!("{}", doc.html);
//! println!("読了時間: {}分", doc.reading_time);
//! ```

use comrak::{
    markdown_to_html,
    nodes::{AstNode, NodeCode, NodeValue},
    parse_document, Arena, ComrakOptions,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use std::sync::OnceLock;

// Python bindings
#[cfg(feature = "python")]
pub mod python;

// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm;

/// Markdownドキュメントの解析結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// HTML出力
    pub html: String,
    /// frontmatterメタデータ
    pub metadata: HashMap<String, String>,
    /// 型を保持したfrontmatterメタデータ
    pub metadata_raw: HashMap<String, serde_yaml::Value>,
    /// 目次（Markdown形式）
    pub toc: String,
    /// 目次（HTML形式）
    pub toc_html: String,
    /// 階層化された目次
    pub toc_tree: Vec<TocItem>,
    /// 見出し一覧
    pub headings: Vec<Heading>,
    /// 読了時間（分）
    pub reading_time: usize,
}

/// 見出し情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    /// 見出しテキスト
    pub text: String,
    /// レベル (1-6)
    pub level: usize,
    /// アンカーID
    pub id: String,
}

/// 階層化された目次項目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocItem {
    /// 見出しテキスト
    pub text: String,
    /// レベル (1-6)
    pub level: usize,
    /// アンカーID
    pub id: String,
    /// 子見出し
    pub children: Vec<TocItem>,
}

/// アンカーIDの生成方式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnchorStyle {
    /// 日本語かなを簡易ローマ字化する
    Romaji,
    /// ASCII英数字と区切りのみを残す
    Ascii,
}

/// Markdown解析オプション
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ParseOptions {
    /// 日本語の読了速度（文字/分）
    #[serde(alias = "readingSpeedJapanese")]
    pub reading_speed_japanese: usize,
    /// 英語の読了速度（単語/分）
    #[serde(alias = "readingSpeedEnglish")]
    pub reading_speed_english: usize,
    /// TOCに含める最小見出しレベル
    #[serde(alias = "tocMinLevel")]
    pub toc_min_level: usize,
    /// TOCに含める最大見出しレベル
    #[serde(alias = "tocMaxLevel")]
    pub toc_max_level: usize,
    /// アンカーIDの生成方式
    #[serde(alias = "anchorStyle")]
    pub anchor_style: AnchorStyle,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            reading_speed_japanese: 400,
            reading_speed_english: 200,
            toc_min_level: 1,
            toc_max_level: 6,
            anchor_style: AnchorStyle::Romaji,
        }
    }
}

impl Document {
    /// Markdownをパースして構造化データを返す
    ///
    /// # 例
    ///
    /// ```
    /// use mdja::Document;
    ///
    /// let doc = Document::parse("# Hello\n\nこんにちは");
    /// assert!(doc.html.contains("<h1"));
    /// assert_eq!(doc.headings.len(), 1);
    /// ```
    pub fn parse(markdown: &str) -> Self {
        Self::parse_with_options(markdown, &ParseOptions::default())
    }

    /// Markdownを指定オプションでパースして構造化データを返す
    pub fn parse_with_options(markdown: &str, options: &ParseOptions) -> Self {
        // frontmatterを抽出
        let (metadata_raw, metadata, content) = extract_frontmatter(markdown);

        // 見出しを抽出してアンカーIDを生成
        let headings = extract_headings(&content, options.anchor_style);

        let comrak_options = markdown_options();

        // HTML変換
        let html = rewrite_heading_ids(&markdown_to_html(&content, &comrak_options), &headings);

        // 目次生成
        let toc_headings = filter_toc_headings(&headings, options);
        let toc = generate_toc(&toc_headings);
        let toc_html = generate_toc_html(&toc_headings);
        let toc_tree = generate_toc_tree(&toc_headings);

        // 読了時間計算
        let reading_time = calculate_reading_time(&content, options);

        Document {
            html,
            metadata,
            metadata_raw,
            toc,
            toc_html,
            toc_tree,
            headings,
            reading_time,
        }
    }

    /// シンプルなHTML変換（メタデータ不要な場合）
    ///
    /// # 例
    ///
    /// ```
    /// use mdja::Document;
    ///
    /// let html = Document::to_html("**太字**");
    /// assert!(html.contains("<strong>"));
    /// ```
    pub fn to_html(markdown: &str) -> String {
        Self::parse(markdown).html
    }

    /// 指定オプションでMarkdownをHTMLに変換する
    pub fn to_html_with_options(markdown: &str, options: &ParseOptions) -> String {
        Self::parse_with_options(markdown, options).html
    }
}

fn markdown_options() -> ComrakOptions {
    let mut options = ComrakOptions::default();
    options.extension.strikethrough = true;
    options.extension.tagfilter = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;

    // 見出しIDを書き換えるため、まずcomrakに標準の見出しアンカーを生成させる
    options.extension.header_ids = Some(String::new());

    options
}

/// frontmatterを抽出
fn extract_frontmatter(
    markdown: &str,
) -> (
    HashMap<String, serde_yaml::Value>,
    HashMap<String, String>,
    String,
) {
    if let Some(caps) = frontmatter_re().captures(markdown) {
        let yaml_str = caps.get(1).map_or("", |m| m.as_str());
        let content = caps.get(2).map_or("", |m| m.as_str());

        // YAMLとして読めない場合はfrontmatter扱いにせず、入力を本文として残す。
        let metadata_raw: HashMap<String, serde_yaml::Value> = if yaml_str.trim().is_empty() {
            HashMap::new()
        } else {
            match serde_yaml::from_str(yaml_str) {
                Ok(metadata) => metadata,
                Err(_) => return (HashMap::new(), HashMap::new(), markdown.to_string()),
            }
        };

        // 文字列に変換
        let metadata: HashMap<String, String> = metadata_raw
            .iter()
            .map(|(k, v)| (k.clone(), metadata_value_to_string(v)))
            .collect();

        (metadata_raw, metadata, content.to_string())
    } else {
        (HashMap::new(), HashMap::new(), markdown.to_string())
    }
}

fn metadata_value_to_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => String::new(),
        _ => serde_yaml::to_string(value)
            .unwrap_or_else(|_| format!("{value:?}"))
            .trim()
            .to_string(),
    }
}

/// 見出しを抽出してアンカーIDを生成
fn extract_headings(markdown: &str, anchor_style: AnchorStyle) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut used_ids: HashMap<String, usize> = HashMap::new();

    let arena = Arena::new();
    let options = markdown_options();
    let root = parse_document(&arena, markdown, &options);

    for node in root.descendants() {
        let level = match node.data.borrow().value {
            NodeValue::Heading(ref heading) => heading.level as usize,
            _ => continue,
        };
        let text = collect_inline_text(node);
        let id = unique_anchor_id(generate_anchor_id(&text, anchor_style), &mut used_ids);

        headings.push(Heading { text, level, id });
    }

    headings
}

fn collect_inline_text<'a>(node: &'a AstNode<'a>) -> String {
    fn collect<'a>(node: &'a AstNode<'a>, output: &mut String) {
        match node.data.borrow().value {
            NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
                output.push_str(literal);
            }
            NodeValue::LineBreak | NodeValue::SoftBreak => output.push(' '),
            _ => {
                for child in node.children() {
                    collect(child, output);
                }
            }
        }
    }

    let mut text = String::new();
    collect(node, &mut text);
    text
}

/// 日本語見出しからアンカーIDを生成
fn generate_anchor_id(text: &str, anchor_style: AnchorStyle) -> String {
    // HTMLタグを削除
    let text = html_tag_re().replace_all(text, "");

    let mut id = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if is_anchor_separator(c) {
            id.push('-');
        } else if c.is_ascii_alphanumeric() {
            id.push(c.to_ascii_lowercase());
        } else if anchor_style == AnchorStyle::Romaji {
            append_romanized_japanese(c, &mut chars, &mut id);
        }
    }

    // 連続するハイフンを1つにまとめ
    let id = repeated_hyphen_re().replace_all(&id, "-");

    // 前後のハイフンを削除
    id.trim_matches('-').to_string()
}

fn append_romanized_japanese(c: char, chars: &mut Peekable<Chars<'_>>, output: &mut String) {
    if is_sokuon(c) {
        if let Some(next) = chars.peek().and_then(|next| romanize_japanese(*next)) {
            if let Some(first) = next.chars().next() {
                output.push(first);
            }
        }
        return;
    }

    if let Some(next) = chars.peek() {
        if let Some(romanized) = romanize_japanese_pair(c, *next) {
            output.push_str(romanized);
            chars.next();
            return;
        }
    }

    if is_small_kana(c) {
        return;
    }

    if let Some(romanized) = romanize_japanese(c) {
        output.push_str(romanized);
    }
}

fn romanize_japanese(c: char) -> Option<&'static str> {
    japanese_romaji_map().get(&c).copied()
}

fn romanize_japanese_pair(c: char, next: char) -> Option<&'static str> {
    japanese_romaji_pair_map().get(&(c, next)).copied()
}

fn is_anchor_separator(c: char) -> bool {
    matches!(c, ' ' | '　')
}

fn is_sokuon(c: char) -> bool {
    matches!(c, 'っ' | 'ッ')
}

fn is_small_kana(c: char) -> bool {
    matches!(
        c,
        'ぁ' | 'ァ'
            | 'ぃ'
            | 'ィ'
            | 'ぅ'
            | 'ゥ'
            | 'ぇ'
            | 'ェ'
            | 'ぉ'
            | 'ォ'
            | 'ゃ'
            | 'ャ'
            | 'ゅ'
            | 'ュ'
            | 'ょ'
            | 'ョ'
            | 'ゎ'
            | 'ヮ'
    )
}

const KANA_ROMAJI: &[(char, &str)] = &[
    // 長音記号はスキップ（前の母音を延ばすだけなので）
    ('ー', ""),
    ('〜', ""),
    ('～', ""),
    ('あ', "a"),
    ('ア', "a"),
    ('い', "i"),
    ('イ', "i"),
    ('う', "u"),
    ('ウ', "u"),
    ('え', "e"),
    ('エ', "e"),
    ('お', "o"),
    ('オ', "o"),
    ('か', "ka"),
    ('カ', "ka"),
    ('き', "ki"),
    ('キ', "ki"),
    ('く', "ku"),
    ('ク', "ku"),
    ('け', "ke"),
    ('ケ', "ke"),
    ('こ', "ko"),
    ('コ', "ko"),
    ('さ', "sa"),
    ('サ', "sa"),
    ('し', "shi"),
    ('シ', "shi"),
    ('す', "su"),
    ('ス', "su"),
    ('せ', "se"),
    ('セ', "se"),
    ('そ', "so"),
    ('ソ', "so"),
    ('た', "ta"),
    ('タ', "ta"),
    ('ち', "chi"),
    ('チ', "chi"),
    ('つ', "tsu"),
    ('ツ', "tsu"),
    ('て', "te"),
    ('テ', "te"),
    ('と', "to"),
    ('ト', "to"),
    ('な', "na"),
    ('ナ', "na"),
    ('に', "ni"),
    ('ニ', "ni"),
    ('ぬ', "nu"),
    ('ヌ', "nu"),
    ('ね', "ne"),
    ('ネ', "ne"),
    ('の', "no"),
    ('ノ', "no"),
    ('は', "ha"),
    ('ハ', "ha"),
    ('ひ', "hi"),
    ('ヒ', "hi"),
    ('ふ', "fu"),
    ('フ', "fu"),
    ('へ', "he"),
    ('ヘ', "he"),
    ('ほ', "ho"),
    ('ホ', "ho"),
    ('ま', "ma"),
    ('マ', "ma"),
    ('み', "mi"),
    ('ミ', "mi"),
    ('む', "mu"),
    ('ム', "mu"),
    ('め', "me"),
    ('メ', "me"),
    ('も', "mo"),
    ('モ', "mo"),
    ('や', "ya"),
    ('ヤ', "ya"),
    ('ゆ', "yu"),
    ('ユ', "yu"),
    ('よ', "yo"),
    ('ヨ', "yo"),
    ('ら', "ra"),
    ('ラ', "ra"),
    ('り', "ri"),
    ('リ', "ri"),
    ('る', "ru"),
    ('ル', "ru"),
    ('れ', "re"),
    ('レ', "re"),
    ('ろ', "ro"),
    ('ロ', "ro"),
    ('わ', "wa"),
    ('ワ', "wa"),
    ('を', "wo"),
    ('ヲ', "wo"),
    ('ん', "n"),
    ('ン', "n"),
    ('が', "ga"),
    ('ガ', "ga"),
    ('ぎ', "gi"),
    ('ギ', "gi"),
    ('ぐ', "gu"),
    ('グ', "gu"),
    ('げ', "ge"),
    ('ゲ', "ge"),
    ('ご', "go"),
    ('ゴ', "go"),
    ('ざ', "za"),
    ('ザ', "za"),
    ('じ', "ji"),
    ('ジ', "ji"),
    ('ず', "zu"),
    ('ズ', "zu"),
    ('ぜ', "ze"),
    ('ゼ', "ze"),
    ('ぞ', "zo"),
    ('ゾ', "zo"),
    ('だ', "da"),
    ('ダ', "da"),
    ('ぢ', "di"),
    ('ヂ', "di"),
    ('づ', "du"),
    ('ヅ', "du"),
    ('で', "de"),
    ('デ', "de"),
    ('ど', "do"),
    ('ド', "do"),
    ('ば', "ba"),
    ('バ', "ba"),
    ('び', "bi"),
    ('ビ', "bi"),
    ('ぶ', "bu"),
    ('ブ', "bu"),
    ('べ', "be"),
    ('ベ', "be"),
    ('ぼ', "bo"),
    ('ボ', "bo"),
    ('ぱ', "pa"),
    ('パ', "pa"),
    ('ぴ', "pi"),
    ('ピ', "pi"),
    ('ぷ', "pu"),
    ('プ', "pu"),
    ('ぺ', "pe"),
    ('ペ', "pe"),
    ('ぽ', "po"),
    ('ポ', "po"),
    ('ヴ', "vu"),
];

const KANJI_ROMAJI: &[(char, &str)] = &[('方', "hou"), ('法', "hou")];

const JAPANESE_ROMAJI_PAIRS: &[((char, char), &str)] = &[
    (('き', 'ゃ'), "kya"),
    (('キ', 'ャ'), "kya"),
    (('き', 'ゅ'), "kyu"),
    (('キ', 'ュ'), "kyu"),
    (('き', 'ょ'), "kyo"),
    (('キ', 'ョ'), "kyo"),
    (('ぎ', 'ゃ'), "gya"),
    (('ギ', 'ャ'), "gya"),
    (('ぎ', 'ゅ'), "gyu"),
    (('ギ', 'ュ'), "gyu"),
    (('ぎ', 'ょ'), "gyo"),
    (('ギ', 'ョ'), "gyo"),
    (('し', 'ゃ'), "sha"),
    (('シ', 'ャ'), "sha"),
    (('し', 'ゅ'), "shu"),
    (('シ', 'ュ'), "shu"),
    (('し', 'ょ'), "sho"),
    (('シ', 'ョ'), "sho"),
    (('じ', 'ゃ'), "ja"),
    (('ジ', 'ャ'), "ja"),
    (('じ', 'ゅ'), "ju"),
    (('ジ', 'ュ'), "ju"),
    (('じ', 'ょ'), "jo"),
    (('ジ', 'ョ'), "jo"),
    (('ち', 'ゃ'), "cha"),
    (('チ', 'ャ'), "cha"),
    (('ち', 'ゅ'), "chu"),
    (('チ', 'ュ'), "chu"),
    (('ち', 'ょ'), "cho"),
    (('チ', 'ョ'), "cho"),
    (('に', 'ゃ'), "nya"),
    (('ニ', 'ャ'), "nya"),
    (('に', 'ゅ'), "nyu"),
    (('ニ', 'ュ'), "nyu"),
    (('に', 'ょ'), "nyo"),
    (('ニ', 'ョ'), "nyo"),
    (('ひ', 'ゃ'), "hya"),
    (('ヒ', 'ャ'), "hya"),
    (('ひ', 'ゅ'), "hyu"),
    (('ヒ', 'ュ'), "hyu"),
    (('ひ', 'ょ'), "hyo"),
    (('ヒ', 'ョ'), "hyo"),
    (('び', 'ゃ'), "bya"),
    (('ビ', 'ャ'), "bya"),
    (('び', 'ゅ'), "byu"),
    (('ビ', 'ュ'), "byu"),
    (('び', 'ょ'), "byo"),
    (('ビ', 'ョ'), "byo"),
    (('ぴ', 'ゃ'), "pya"),
    (('ピ', 'ャ'), "pya"),
    (('ぴ', 'ゅ'), "pyu"),
    (('ピ', 'ュ'), "pyu"),
    (('ぴ', 'ょ'), "pyo"),
    (('ピ', 'ョ'), "pyo"),
    (('み', 'ゃ'), "mya"),
    (('ミ', 'ャ'), "mya"),
    (('み', 'ゅ'), "myu"),
    (('ミ', 'ュ'), "myu"),
    (('み', 'ょ'), "myo"),
    (('ミ', 'ョ'), "myo"),
    (('り', 'ゃ'), "rya"),
    (('リ', 'ャ'), "rya"),
    (('り', 'ゅ'), "ryu"),
    (('リ', 'ュ'), "ryu"),
    (('り', 'ょ'), "ryo"),
    (('リ', 'ョ'), "ryo"),
    (('ヴ', 'ァ'), "va"),
    (('ヴ', 'ィ'), "vi"),
    (('ヴ', 'ゥ'), "vu"),
    (('ヴ', 'ェ'), "ve"),
    (('ヴ', 'ォ'), "vo"),
    (('て', 'ぃ'), "ti"),
    (('テ', 'ィ'), "ti"),
    (('で', 'ぃ'), "di"),
    (('デ', 'ィ'), "di"),
    (('と', 'ぅ'), "tu"),
    (('ト', 'ゥ'), "tu"),
    (('ど', 'ぅ'), "du"),
    (('ド', 'ゥ'), "du"),
];

fn japanese_romaji_map() -> &'static HashMap<char, &'static str> {
    static MAP: OnceLock<HashMap<char, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        KANA_ROMAJI
            .iter()
            .chain(KANJI_ROMAJI.iter())
            .copied()
            .collect()
    })
}

fn japanese_romaji_pair_map() -> &'static HashMap<(char, char), &'static str> {
    static MAP: OnceLock<HashMap<(char, char), &'static str>> = OnceLock::new();
    MAP.get_or_init(|| JAPANESE_ROMAJI_PAIRS.iter().copied().collect())
}

fn unique_anchor_id(base_id: String, used_ids: &mut HashMap<String, usize>) -> String {
    let base_id = if base_id.is_empty() {
        "heading".to_string()
    } else {
        base_id
    };
    let count = used_ids.entry(base_id.clone()).or_insert(0);
    *count += 1;

    if *count == 1 {
        base_id
    } else {
        format!("{base_id}-{count}")
    }
}

fn rewrite_heading_ids(html: &str, headings: &[Heading]) -> String {
    let mut index = 0;

    heading_html_re().replace_all(html, |caps: &regex::Captures| {
        let level = caps.get(1).map_or("", |m| m.as_str());
        let body = caps.get(2).map_or("", |m| m.as_str());
        let id = headings
            .get(index)
            .map(|heading| heading.id.as_str())
            .unwrap_or_default();
        index += 1;

        format!(
            "<h{level}><a href=\"#{}\" aria-hidden=\"true\" class=\"anchor\" id=\"{}\"></a>{body}</h{level}>",
            escape_html_attr(id),
            escape_html_attr(id)
        )
    })
    .to_string()
}

fn filter_toc_headings(headings: &[Heading], options: &ParseOptions) -> Vec<Heading> {
    let min_level = options.toc_min_level.clamp(1, 6);
    let max_level = options.toc_max_level.clamp(min_level, 6);

    headings
        .iter()
        .filter(|heading| heading.level >= min_level && heading.level <= max_level)
        .cloned()
        .collect()
}

/// 目次を生成
fn generate_toc(headings: &[Heading]) -> String {
    let mut toc = String::new();
    render_toc_items(&generate_toc_tree(headings), 0, &mut toc);
    toc
}

fn render_toc_items(items: &[TocItem], depth: usize, output: &mut String) {
    for item in items {
        let indent = "  ".repeat(depth);
        output.push_str(&format!(
            "{}- [{}](#{})\n",
            indent,
            escape_markdown_link_text(&item.text),
            item.id
        ));
        render_toc_items(&item.children, depth + 1, output);
    }
}

fn generate_toc_html(headings: &[Heading]) -> String {
    if headings.is_empty() {
        return String::new();
    }

    let mut html = String::from("<ul>\n");
    for heading in headings {
        html.push_str(&format!(
            "  <li class=\"toc-level-{}\"><a href=\"#{}\">{}</a></li>\n",
            heading.level,
            escape_html_attr(&heading.id),
            escape_html_text(&heading.text)
        ));
    }
    html.push_str("</ul>\n");
    html
}

fn generate_toc_tree(headings: &[Heading]) -> Vec<TocItem> {
    fn new_toc_item(heading: &Heading) -> TocItem {
        TocItem {
            text: heading.text.clone(),
            level: heading.level,
            id: heading.id.clone(),
            children: Vec::new(),
        }
    }

    fn attach_to_parent(parent: &mut TocItem, heading: &Heading) {
        if parent
            .children
            .last()
            .is_none_or(|last| heading.level <= last.level)
        {
            parent.children.push(new_toc_item(heading));
            return;
        }

        if let Some(last) = parent.children.last_mut() {
            attach_to_parent(last, heading);
        }
    }

    let mut root = Vec::new();
    for heading in headings {
        if root
            .last()
            .is_none_or(|last: &TocItem| heading.level <= last.level)
        {
            root.push(new_toc_item(heading));
        } else if let Some(last) = root.last_mut() {
            attach_to_parent(last, heading);
        }
    }
    root
}

/// 読了時間を計算（日本語対応）
fn calculate_reading_time(markdown: &str, options: &ParseOptions) -> usize {
    let text = collect_readable_text(markdown);

    let mut char_count: usize = 0;
    let mut english_text = String::new();

    for c in text.chars() {
        // 日本語文字は1文字として、英数字は単語としてカウント
        if is_japanese_char(c) {
            char_count += 1;
            english_text.push(' ');
        } else {
            english_text.push(c);
        }
    }

    let word_count = english_text
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .count();

    // 日本語: 400文字/分、英語: 200単語/分
    let japanese_time = char_count.div_ceil(options.reading_speed_japanese.max(1));
    let english_time = word_count.div_ceil(options.reading_speed_english.max(1));

    (japanese_time + english_time).max(1)
}

fn collect_readable_text(markdown: &str) -> String {
    fn collect<'a>(node: &'a AstNode<'a>, output: &mut String) {
        match node.data.borrow().value {
            NodeValue::CodeBlock(..) | NodeValue::HtmlBlock(..) => {}
            NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
                output.push_str(literal);
                output.push(' ');
            }
            NodeValue::LineBreak | NodeValue::SoftBreak => output.push(' '),
            _ => {
                for child in node.children() {
                    collect(child, output);
                }
            }
        }
    }

    let arena = Arena::new();
    let options = markdown_options();
    let root = parse_document(&arena, markdown, &options);
    let mut text = String::new();
    collect(root, &mut text);
    text
}

fn escape_markdown_link_text(input: &str) -> String {
    let mut escaped = String::new();
    for c in input.chars() {
        if matches!(c, '[' | ']' | '\\') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

fn escape_html_attr(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '"' => "&quot;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

fn escape_html_text(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

/// 日本語文字かどうか判定
fn is_japanese_char(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{309F}' | // ひらがな
        '\u{30A0}'..='\u{30FF}' | // カタカナ
        '\u{4E00}'..='\u{9FFF}' | // 漢字
        '\u{FF66}'..='\u{FF9F}'   // 半角カタカナ
    )
}

fn frontmatter_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?s)^---[ \t]*\r?\n(?:(.*?)\r?\n)?---[ \t]*(?:\r?\n(.*)|$)").unwrap()
    })
}

fn html_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<[^>]+>").unwrap())
}

fn repeated_hyphen_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"-+").unwrap())
}

fn heading_html_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r##"(?s)<h([1-6])><a href="#[^"]*" aria-hidden="true" class="anchor" id="[^"]*"></a>(.*?)</h[1-6]>"##,
        )
        .unwrap()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let doc = Document::parse("# Hello\n\nWorld");
        assert!(doc.html.contains("<h1"));
        assert!(doc.html.contains("World"));
    }

    #[test]
    fn test_parse_with_frontmatter() {
        let markdown = r#"---
title: Test Article
author: Taro
---

# Content
"#;
        let doc = Document::parse(markdown);
        assert_eq!(doc.metadata.get("title").unwrap(), "Test Article");
        assert_eq!(doc.metadata.get("author").unwrap(), "Taro");
    }

    #[test]
    fn test_parse_with_frontmatter_ending_at_eof() {
        let doc = Document::parse("---\ntitle: Test Article\n---");

        assert_eq!(doc.metadata.get("title").unwrap(), "Test Article");
        assert_eq!(doc.html, "");
        assert!(doc.headings.is_empty());
    }

    #[test]
    fn test_parse_with_empty_frontmatter() {
        let doc = Document::parse("---\n---\n# Content");

        assert!(doc.metadata.is_empty());
        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].text, "Content");
        assert!(!doc.html.contains("<hr"));
    }

    #[test]
    fn test_parse_with_frontmatter_allows_crlf() {
        let doc = Document::parse("---\r\ntitle: Test Article\r\n---\r\n# Content");

        assert_eq!(doc.metadata.get("title").unwrap(), "Test Article");
        assert_eq!(doc.headings[0].text, "Content");
    }

    #[test]
    fn test_parse_with_invalid_frontmatter_keeps_original_content() {
        let doc = Document::parse("---\ntitle: [unterminated\n---\n# Content");

        assert!(doc.metadata.is_empty());
        assert!(doc.html.contains("title: [unterminated"));
        assert!(doc.headings.iter().any(|heading| heading.text == "Content"));
    }

    #[test]
    fn test_heading_extraction() {
        let doc = Document::parse("# H1\n## H2\n### H3");
        assert_eq!(doc.headings.len(), 3);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[1].level, 2);
        assert_eq!(doc.headings[2].level, 3);
    }

    #[test]
    fn test_heading_extraction_supports_setext_headings() {
        let doc = Document::parse("はじめに1\n=====\n\nはじめに2\n-----");

        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].text, "はじめに1");
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].id, "hajimeni1");
        assert_eq!(doc.headings[1].text, "はじめに2");
        assert_eq!(doc.headings[1].level, 2);
        assert!(doc.html.contains(r##"id="hajimeni1""##));
        assert!(doc.html.contains(r##"id="hajimeni2""##));
    }

    #[test]
    fn test_heading_extraction_allows_indented_atx_headings() {
        let doc = Document::parse("   # H1\n    # Code");

        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].text, "H1");
        assert_eq!(doc.headings[0].level, 1);
    }

    #[test]
    fn test_heading_extraction_handles_empty_atx_heading() {
        let doc = Document::parse("#\n\n##");

        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].text, "");
        assert_eq!(doc.headings[0].id, "heading");
        assert_eq!(doc.headings[1].text, "");
        assert_eq!(doc.headings[1].id, "heading-2");
        assert!(doc.html.contains(r##"id="heading""##));
        assert!(doc.html.contains(r##"id="heading-2""##));
    }

    #[test]
    fn test_heading_extraction_strips_only_valid_atx_closing_sequence() {
        let doc = Document::parse("# C# ###\n# C#");

        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].text, "C#");
        assert_eq!(doc.headings[0].id, "c");
        assert_eq!(doc.headings[1].text, "C#");
        assert_eq!(doc.headings[1].id, "c-2");
    }

    #[test]
    fn test_heading_extraction_ignores_fenced_code() {
        let markdown = r#"# Title

```markdown
# Not a heading
```

~~~markdown
## Also not a heading
~~~

## Section
"#;
        let doc = Document::parse(markdown);
        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].text, "Title");
        assert_eq!(doc.headings[1].text, "Section");
        assert!(!doc.toc.contains("Not a heading"));
    }

    #[test]
    fn test_heading_extraction_respects_fence_length() {
        let markdown = "````\n# inside\n```\n# still inside\n````\n# outside";
        let doc = Document::parse(markdown);

        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].text, "outside");
        assert!(doc.html.contains(r##"id="outside""##));
    }

    #[test]
    fn test_heading_extraction_respects_fence_character() {
        let markdown = "~~~\n# inside\n```\n# still inside\n~~~\n# outside";
        let doc = Document::parse(markdown);

        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].text, "outside");
    }

    #[test]
    fn test_heading_extraction_requires_valid_closing_fence() {
        let markdown = "``` rust\n# inside\n``` close\n# still inside\n```\n# outside";
        let doc = Document::parse(markdown);

        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].text, "outside");
    }

    #[test]
    fn test_heading_extraction_ignores_invalid_backtick_opening_fence() {
        let markdown = "``` `bad`\n# heading\n```";
        let doc = Document::parse(markdown);

        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].text, "heading");
    }

    #[test]
    fn test_heading_extraction_matches_headings_inside_containers() {
        let markdown = "> # quoted\n\n- # item heading\n\n# outside";
        let doc = Document::parse(markdown);

        assert_eq!(doc.headings.len(), 3);
        assert_eq!(doc.headings[0].text, "quoted");
        assert_eq!(doc.headings[0].id, "quoted");
        assert_eq!(doc.headings[1].text, "item heading");
        assert_eq!(doc.headings[1].id, "item-heading");
        assert_eq!(doc.headings[2].text, "outside");
        assert_eq!(doc.headings[2].id, "outside");
        assert!(doc.html.contains(r##"id="quoted""##));
        assert!(doc.html.contains(r##"id="item-heading""##));
        assert!(doc.html.contains(r##"id="outside""##));
    }

    #[test]
    fn test_heading_extraction_ignores_markdown_inside_html_block() {
        let markdown = "# A\n\n<div>\n# not heading\n</div>\n\n# B";
        let doc = Document::parse(markdown);

        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].text, "A");
        assert_eq!(doc.headings[0].id, "a");
        assert_eq!(doc.headings[1].text, "B");
        assert_eq!(doc.headings[1].id, "b");
        assert!(doc.html.contains(r##"id="a""##));
        assert!(doc.html.contains(r##"id="b""##));
        assert!(!doc.toc.contains("not heading"));
    }

    #[test]
    fn test_duplicate_heading_ids_are_unique() {
        let doc = Document::parse("# はじめに\n## はじめに\n### はじめに");
        assert_eq!(doc.headings[0].id, "hajimeni");
        assert_eq!(doc.headings[1].id, "hajimeni-2");
        assert_eq!(doc.headings[2].id, "hajimeni-3");
        assert!(doc.html.contains(r#"id="hajimeni-2""#));
        assert!(doc.toc.contains("  - [はじめに](#hajimeni-2)"));
    }

    #[test]
    fn test_empty_anchor_falls_back_to_heading() {
        let doc = Document::parse("# 日本語\n## 日本語");
        assert_eq!(doc.headings[0].id, "heading");
        assert_eq!(doc.headings[1].id, "heading-2");
        assert!(doc.html.contains(r#"id="heading""#));
        assert!(doc.html.contains(r#"id="heading-2""#));
    }

    #[test]
    fn test_toc_escapes_markdown_link_text() {
        let doc = Document::parse("# [概要]\\確認");
        assert!(doc.toc.contains(r"- [\[概要\]\\確認]"));
    }

    #[test]
    fn test_japanese_anchor_id() {
        let id = generate_anchor_id("はじめに", AnchorStyle::Romaji);
        assert_eq!(id, "hajimeni");

        let id = generate_anchor_id("インストール方法", AnchorStyle::Romaji);
        assert_eq!(id, "insutoruhouhou");

        // スペース区切りの場合はハイフンになる
        let id = generate_anchor_id("インストール 方法", AnchorStyle::Romaji);
        assert_eq!(id, "insutoru-houhou");
    }

    #[test]
    fn test_japanese_anchor_id_handles_digraphs_and_sokuon() {
        assert_eq!(
            generate_anchor_id("キャッシュ", AnchorStyle::Romaji),
            "kyasshu"
        );
        assert_eq!(
            generate_anchor_id("ヴァージョン", AnchorStyle::Romaji),
            "vajon"
        );
        assert_eq!(
            generate_anchor_id("ティーカップ", AnchorStyle::Romaji),
            "tikappu"
        );
    }

    #[test]
    fn test_ascii_anchor_style_skips_japanese() {
        assert_eq!(
            generate_anchor_id("API はじめに 2025", AnchorStyle::Ascii),
            "api-2025"
        );
    }

    #[test]
    fn test_romaji_tables_have_unique_keys() {
        let mut seen = std::collections::HashSet::new();

        for (c, _) in KANA_ROMAJI.iter().chain(KANJI_ROMAJI.iter()) {
            assert!(seen.insert(*c), "duplicate romaji table key: {c}");
        }
    }

    #[test]
    fn test_japanese_anchor_id_matches_html() {
        let doc = Document::parse("# はじめに\n\n## インストール方法");
        assert!(doc.html.contains(r##"href="#hajimeni""##));
        assert!(doc.html.contains(r#"id="hajimeni""#));
        assert!(doc.html.contains(r##"href="#insutoruhouhou""##));
        assert!(doc.html.contains(r#"id="insutoruhouhou""#));
        assert!(doc.toc.contains("- [はじめに](#hajimeni)"));
        assert!(doc.toc.contains("  - [インストール方法](#insutoruhouhou)"));
    }

    #[test]
    fn test_toc_generation() {
        let doc = Document::parse("# First\n## Second\n### Third");
        assert!(doc.toc.contains("- [First](#first)"));
        assert!(doc.toc.contains("  - [Second](#second)"));
        assert!(doc.toc_html.contains(r##"<a href="#first">First</a>"##));
        assert_eq!(doc.toc_tree[0].children[0].text, "Second");
    }

    #[test]
    fn test_toc_tree_keeps_same_level_headings_as_siblings() {
        let doc = Document::parse("# First\n## Second\n## Third\n### Fourth\n## Fifth");
        assert_eq!(doc.toc_tree[0].children.len(), 3);
        assert_eq!(doc.toc_tree[0].children[0].text, "Second");
        assert_eq!(doc.toc_tree[0].children[1].text, "Third");
        assert_eq!(doc.toc_tree[0].children[1].children[0].text, "Fourth");
        assert_eq!(doc.toc_tree[0].children[2].text, "Fifth");
    }

    #[test]
    fn test_toc_markdown_matches_tree_depth_when_heading_levels_move_up() {
        let doc = Document::parse("### H3\n##### H5\n## H2\n#### H4");

        assert_eq!(
            doc.toc,
            "- [H3](#h3)\n  - [H5](#h5)\n- [H2](#h2)\n  - [H4](#h4)\n"
        );
    }

    #[test]
    fn test_parse_options_deserialize_from_camel_case_json() {
        let options: ParseOptions = serde_json::from_str(
            r#"{
                "tocMinLevel": 2,
                "tocMaxLevel": 3,
                "readingSpeedJapanese": 500,
                "readingSpeedEnglish": 250,
                "anchorStyle": "ascii"
            }"#,
        )
        .unwrap();

        assert_eq!(options.toc_min_level, 2);
        assert_eq!(options.toc_max_level, 3);
        assert_eq!(options.reading_speed_japanese, 500);
        assert_eq!(options.reading_speed_english, 250);
        assert_eq!(options.anchor_style, AnchorStyle::Ascii);
    }

    #[test]
    fn test_parse_options_filter_toc_and_reading_speed() {
        let options = ParseOptions {
            toc_min_level: 2,
            toc_max_level: 2,
            reading_speed_japanese: 200,
            ..ParseOptions::default()
        };
        let doc = Document::parse_with_options("# H1\n## H2\n### H3\n\nあああ", &options);
        assert!(!doc.toc.contains("[H1]"));
        assert!(doc.toc.contains("- [H2](#h2)"));
        assert!(!doc.toc.contains("  - [H2](#h2)"));
        assert!(!doc.toc.contains("[H3]"));

        let doc = Document::parse_with_options(&"あ".repeat(201), &options);
        assert_eq!(doc.reading_time, 2);
    }

    #[test]
    fn test_reading_time() {
        let text = "あ".repeat(400);
        let doc = Document::parse(&text);
        assert_eq!(doc.reading_time, 1);

        let text = "あ".repeat(800);
        let doc = Document::parse(&text);
        assert_eq!(doc.reading_time, 2);

        let text = "あ".repeat(401);
        let doc = Document::parse(&text);
        assert_eq!(doc.reading_time, 2);
    }

    #[test]
    fn test_reading_time_ignores_fenced_code_blocks() {
        let markdown = format!("~~~\n{}\n~~~\n本文", "あ".repeat(800));
        let doc = Document::parse(&markdown);

        assert_eq!(doc.reading_time, 1);
    }

    #[test]
    fn test_reading_time_ignores_html_blocks() {
        let markdown = format!("<div>\n{}\n</div>\n本文", "あ".repeat(800));
        let doc = Document::parse(&markdown);

        assert_eq!(doc.reading_time, 1);
    }

    #[test]
    fn test_english_reading_time_counts_words() {
        let text = (0..200).map(|_| "word").collect::<Vec<_>>().join(" ");
        let doc = Document::parse(&text);
        assert_eq!(doc.reading_time, 1);

        let text = (0..400).map(|_| "word").collect::<Vec<_>>().join(" ");
        let doc = Document::parse(&text);
        assert_eq!(doc.reading_time, 2);

        let text = (0..201).map(|_| "word").collect::<Vec<_>>().join(" ");
        let doc = Document::parse(&text);
        assert_eq!(doc.reading_time, 2);
    }

    #[test]
    fn test_gfm_table() {
        let markdown = r#"
| Header1 | Header2 |
|---------|---------|
| Cell1   | Cell2   |
"#;
        let doc = Document::parse(markdown);
        assert!(doc.html.contains("<table"));
    }

    #[test]
    fn test_gfm_strikethrough() {
        let doc = Document::parse("~~strikethrough~~");
        assert!(doc.html.contains("<del>") || doc.html.contains("strikethrough"));
    }

    #[test]
    fn test_gfm_tasklist() {
        let markdown = r#"
- [x] Done
- [ ] Todo
"#;
        let doc = Document::parse(markdown);
        assert!(doc.html.contains("checkbox") || doc.html.contains("checked"));
    }
}
