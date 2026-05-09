//! Python bindings for mdja

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::Document as RustDocument;

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

    /// Table of contents (Markdown format)
    #[getter]
    fn toc(&self) -> PyResult<String> {
        Ok(self.inner.toc.clone())
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
