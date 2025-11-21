mod clients;
mod models;
mod utils;
use crate::models::*;
use crate::utils::logger::init_tracing;

use pyo3::prelude::*;

/// Rust-powered Azure Document Intelligence client with concurrent processing.
///
/// This module provides a high-performance client for Azure Document Intelligence API,
/// enabling concurrent batch processing of documents from URLs or local files.
///
/// Classes:
///     RustyAnalysisClient: Main client for document analysis operations
///
/// Example:
///     >>> from rusty_di_runner import RustyAnalysisClient
///     >>> client = RustyAnalysisClient(
///     ...     endpoint="your-resource.cognitiveservices.azure.com",
///     ...     api_key="your-api-key"
///     ... )
///     >>> results = client.process_batch_documents_from_urls(
///     ...     "prebuilt-layout",
///     ...     ["https://example.com/doc.pdf"]
///     ... )
#[pymodule]
fn rusty_di_runner(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Money pieces
    m.add_class::<RustyAnalysisClient>()?;
    m.add_class::<Credentials>()?;

    // Model classes
    m.add_class::<AnalyzeResult>()?;
    m.add_class::<DocumentPage>()?;
    m.add_class::<DocumentLine>()?;
    m.add_class::<DocumentWord>()?;
    m.add_class::<DocumentParagraph>()?;
    m.add_class::<DocumentTable>()?;
    m.add_class::<DocumentSpan>()?;

    Ok(())
}
