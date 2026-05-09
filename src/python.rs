//! Python bindings for mdja
#![allow(non_local_definitions)]

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::{AnchorStyle, Document as RustDocument, ParseOptions as RustParseOptions};

/// Python wrapper for Document
#[pyclass(name = "Document")]
#[derive(Clone)]
pub struct PyDocument {
    inner: RustDocument,
}

#[pymethods]
impl PyDocument {
    /// Parse Markdown and return a Document object
    ///
    /// Args:
    ///     markdown (str): Markdown text to parse
    ///
    /// Returns:
    ///     Document: Parsed document with HTML, metadata, TOC, etc.
    ///
    /// Example:
    ///     >>> import mdja
    ///     >>> doc = mdja.Document.parse("# Hello\n\nWorld")
    ///     >>> print(doc.html)
    #[staticmethod]
    fn parse(markdown: &str) -> PyResult<Self> {
        Ok(PyDocument {
            inner: RustDocument::parse(markdown),
        })
    }

    /// Parse Markdown with custom options
    #[staticmethod]
    fn parse_with_options(markdown: &str, options: &PyParseOptions) -> PyResult<Self> {
        Ok(PyDocument {
            inner: RustDocument::parse_with_options(markdown, &options.inner),
        })
    }

    /// Convert Markdown to HTML (simple conversion)
    ///
    /// Args:
    ///     markdown (str): Markdown text to convert
    ///
    /// Returns:
    ///     str: HTML output
    ///
    /// Example:
    ///     >>> import mdja
    ///     >>> html = mdja.Document.to_html("**bold**")
    ///     >>> print(html)
    #[staticmethod]
    fn to_html(markdown: &str) -> PyResult<String> {
        Ok(RustDocument::to_html(markdown))
    }

    /// HTML output
    #[getter]
    fn html(&self) -> PyResult<String> {
        Ok(self.inner.html.clone())
    }

    /// Frontmatter metadata as a dictionary
    #[getter]
    fn metadata(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        for (key, value) in &self.inner.metadata {
            dict.set_item(key, value)?;
        }
        Ok(dict.into())
    }

    /// Raw frontmatter metadata as JSON
    #[getter]
    fn metadata_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.metadata_raw)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Table of contents (Markdown format)
    #[getter]
    fn toc(&self) -> PyResult<String> {
        Ok(self.inner.toc.clone())
    }

    /// Table of contents (HTML format)
    #[getter]
    fn toc_html(&self) -> PyResult<String> {
        Ok(self.inner.toc_html.clone())
    }

    /// Hierarchical table of contents as JSON
    #[getter]
    fn toc_tree_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.toc_tree)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Reading time in minutes
    #[getter]
    fn reading_time(&self) -> PyResult<usize> {
        Ok(self.inner.reading_time)
    }

    /// List of headings
    #[getter]
    fn headings(&self) -> PyResult<Vec<PyHeading>> {
        Ok(self
            .inner
            .headings
            .iter()
            .map(|h| PyHeading {
                text: h.text.clone(),
                level: h.level,
                id: h.id.clone(),
            })
            .collect())
    }

    fn __repr__(&self) -> String {
        format!(
            "Document(headings={}, reading_time={}min)",
            self.inner.headings.len(),
            self.inner.reading_time
        )
    }

    fn __str__(&self) -> String {
        self.inner.html.clone()
    }
}

/// Python wrapper for ParseOptions
#[pyclass(name = "ParseOptions")]
#[derive(Clone)]
pub struct PyParseOptions {
    inner: RustParseOptions,
}

#[pymethods]
impl PyParseOptions {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustParseOptions::default(),
        }
    }

    #[getter]
    fn reading_speed_japanese(&self) -> usize {
        self.inner.reading_speed_japanese
    }

    #[setter]
    fn set_reading_speed_japanese(&mut self, value: usize) {
        self.inner.reading_speed_japanese = value;
    }

    #[getter]
    fn reading_speed_english(&self) -> usize {
        self.inner.reading_speed_english
    }

    #[setter]
    fn set_reading_speed_english(&mut self, value: usize) {
        self.inner.reading_speed_english = value;
    }

    #[getter]
    fn toc_min_level(&self) -> usize {
        self.inner.toc_min_level
    }

    #[setter]
    fn set_toc_min_level(&mut self, value: usize) {
        self.inner.toc_min_level = value;
    }

    #[getter]
    fn toc_max_level(&self) -> usize {
        self.inner.toc_max_level
    }

    #[setter]
    fn set_toc_max_level(&mut self, value: usize) {
        self.inner.toc_max_level = value;
    }

    #[getter]
    fn anchor_style(&self) -> &'static str {
        match self.inner.anchor_style {
            AnchorStyle::Romaji => "romaji",
            AnchorStyle::Ascii => "ascii",
        }
    }

    #[setter]
    fn set_anchor_style(&mut self, value: &str) -> PyResult<()> {
        self.inner.anchor_style = match value {
            "romaji" => AnchorStyle::Romaji,
            "ascii" => AnchorStyle::Ascii,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "anchor_style must be 'romaji' or 'ascii'",
                ))
            }
        };
        Ok(())
    }
}

/// Python wrapper for Heading
#[pyclass(name = "Heading")]
#[derive(Clone)]
pub struct PyHeading {
    #[pyo3(get)]
    pub text: String,
    #[pyo3(get)]
    pub level: usize,
    #[pyo3(get)]
    pub id: String,
}

#[pymethods]
impl PyHeading {
    fn __repr__(&self) -> String {
        format!(
            "Heading(text='{}', level={}, id='{}')",
            self.text, self.level, self.id
        )
    }

    fn __str__(&self) -> String {
        self.text.clone()
    }
}

/// mdja Python module
#[pymodule]
fn mdja(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDocument>()?;
    m.add_class::<PyHeading>()?;
    m.add_class::<PyParseOptions>()?;

    // Module docstring
    m.add(
        "__doc__",
        "日本語に最適化されたMarkdownパーサー\n\n\
        Features:\n\
        - CommonMark + GFM support\n\
        - Japanese heading anchor generation\n\
        - Table of contents generation\n\
        - Reading time calculation\n\
        - Frontmatter parsing\n\n\
        Example:\n\
            >>> import mdja\n\
            >>> doc = mdja.Document.parse('# Hello\\n\\nWorld')\n\
            >>> print(doc.html)\n\
            >>> print(f'Reading time: {doc.reading_time} min')\n\
    ",
    )?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
