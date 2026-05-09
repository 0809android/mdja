//! WASM bindings for mdja

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{Document as RustDocument, ParseOptions};

/// WASM wrapper for Document
#[wasm_bindgen]
#[derive(Clone)]
pub struct Document {
    inner: RustDocument,
}

#[wasm_bindgen]
impl Document {
    /// Parse Markdown and return a Document object
    ///
    /// # Example (JavaScript)
    ///
    /// ```js
    /// import { Document } from 'mdja';
    ///
    /// const doc = Document.parse("# Hello\n\nWorld");
    /// console.log(doc.html);
    /// ```
    #[wasm_bindgen]
    pub fn parse(markdown: &str) -> Document {
        Document {
            inner: RustDocument::parse(markdown),
        }
    }

    /// Parse Markdown with custom options supplied as a JavaScript object
    #[wasm_bindgen(js_name = parseWithOptions)]
    pub fn parse_with_options(markdown: &str, options: JsValue) -> Result<Document, JsValue> {
        let options = serde_wasm_bindgen::from_value::<ParseOptions>(options)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Document {
            inner: RustDocument::parse_with_options(markdown, &options),
        })
    }

    /// Convert Markdown to HTML (simple conversion)
    ///
    /// # Example (JavaScript)
    ///
    /// ```js
    /// import { Document } from 'mdja';
    ///
    /// const html = Document.toHtml("**bold**");
    /// console.log(html);
    /// ```
    #[wasm_bindgen(js_name = toHtml)]
    pub fn to_html(markdown: &str) -> String {
        RustDocument::to_html(markdown)
    }

    /// Get HTML output
    #[wasm_bindgen(getter)]
    pub fn html(&self) -> String {
        self.inner.html.clone()
    }

    /// Get frontmatter metadata as JSON string
    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.metadata).unwrap_or(JsValue::NULL)
    }

    /// Get raw frontmatter metadata with YAML types preserved
    #[wasm_bindgen(getter, js_name = metadataRaw)]
    pub fn metadata_raw(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.metadata_raw).unwrap_or(JsValue::NULL)
    }

    /// Get table of contents (Markdown format)
    #[wasm_bindgen(getter)]
    pub fn toc(&self) -> String {
        self.inner.toc.clone()
    }

    /// Get table of contents (HTML format)
    #[wasm_bindgen(getter, js_name = tocHtml)]
    pub fn toc_html(&self) -> String {
        self.inner.toc_html.clone()
    }

    /// Get hierarchical table of contents
    #[wasm_bindgen(getter, js_name = tocTree)]
    pub fn toc_tree(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.toc_tree).unwrap_or(JsValue::NULL)
    }

    /// Get reading time in minutes
    #[wasm_bindgen(getter, js_name = readingTime)]
    pub fn reading_time(&self) -> usize {
        self.inner.reading_time
    }

    /// Get list of headings as JSON
    #[wasm_bindgen(getter)]
    pub fn headings(&self) -> JsValue {
        let headings: Vec<WasmHeading> = self
            .inner
            .headings
            .iter()
            .map(|h| WasmHeading {
                text: h.text.clone(),
                level: h.level,
                id: h.id.clone(),
            })
            .collect();
        serde_wasm_bindgen::to_value(&headings).unwrap_or(JsValue::NULL)
    }
}

/// Heading structure for WASM (serializable)
#[derive(Serialize, Deserialize)]
pub struct WasmHeading {
    pub text: String,
    pub level: usize,
    pub id: String,
}
