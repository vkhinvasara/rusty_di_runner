use std::str::FromStr;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::Credentials;
use crate::init_tracing;
use crate::models::analyze_result::AnalyzeResult;
use tokio::runtime::Runtime;

/// A client for analyzing documents using Azure Document Intelligence API.
///
/// This client provides batch processing capabilities for document analysis
/// using the Azure Document Intelligence service (formerly Form Recognizer).
///
/// Args:
///     list[Credentials]: List of resource credentials
///
/// Example:
///     >>> client = RustyAnalysisClient(
///     ...     "https://myservice.cognitiveservices.azure.com",
///     ...     "your-api-key"
///     ... )
#[pyclass]
pub struct RustyAnalysisClient {
    runtime: Runtime,
    pub(crate) credentials: Vec<Credentials>,
}
#[derive(Clone, Debug, PartialEq, Default)]
pub enum OutputContentFormat{
    #[default]
    Text,
    Markdown,
}

impl FromStr for OutputContentFormat {
    type Err = PyErr; // We use PyErr so we can return it directly to Python if needed

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "text" => Ok(OutputContentFormat::Text),
            "markdown" => Ok(OutputContentFormat::Markdown),
            _ => Err(PyValueError::new_err(format!(
                "Invalid output format: '{}'. Expected 'text' or 'markdown'.",
                s
            ))),
        }
    }
}

impl std::fmt::Display for OutputContentFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputContentFormat::Markdown => write!(f, "markdown"),
            OutputContentFormat::Text => write!(f, "text"),
        }
    }
}

#[pymethods]
impl RustyAnalysisClient {
    /// Create a new RustyAnalysisClient instance.
    ///
    /// Args:
    ///     credentials (list[Credentials]): List of Credentials objects containing
    ///     endpoint URLs and API keys for Azure Document Intelligence services
    ///
    /// Returns:
    ///     RustyAnalysisClient: A new client instance configured with the provided credentials
    ///
    /// Example:
    ///     >>> from rusty_di_runner import RustyAnalysisClient, Credentials
    ///     >>> creds = [
    ///     ...     Credentials(
    ///     ...         endpoint="https://your-resource.cognitiveservices.azure.com",
    ///     ...         api_key="your-api-key"
    ///     ...     )
    ///     ... ]
    ///     >>> client = RustyAnalysisClient(credentials=creds)
    #[new]
    #[pyo3(signature = (credentials, enable_logs))]
    pub fn new(credentials: Vec<Credentials>, enable_logs: bool) -> PyResult<Self> {
        // Initialize Tracing
        if enable_logs{
            init_tracing();
        }

        Ok(Self {
            credentials,
            runtime: Runtime::new().unwrap(),
        })
    }
    /// Process multiple documents from URLs concurrently.
    ///
    /// Analyzes a batch of documents accessible via URLs using the specified
    /// Document Intelligence model. All documents are processed in parallel
    /// for maximum throughput.
    ///
    /// Args:
    ///     model_id (str): The Document Intelligence model ID
    ///         (e.g., 'prebuilt-layout', 'prebuilt-invoice')
    ///     document_urls (list[str]): List of publicly accessible document URLs
    ///     features (list[str] | None): Optional list of analysis features to enable
    ///         (e.g., ['ocrHighResolution', 'formulas', 'styleFont']). Defaults to None.
    ///     output_format (str | None): Optional output content format. Valid values are:
    ///         - 'text' (default): Plain text representation with line breaks
    ///         - 'markdown': Markdown formatted output preserving document structure
    ///         Defaults to 'text' if not specified.
    ///
    /// Returns:
    ///     list: List of results where each item is either:
    ///         - dict: Successfully analyzed document result with full analyzeResult
    ///         - Exception: Error object if processing failed for that document
    ///
    /// Example:
    ///     >>> urls = [
    ///     ...     "https://example.com/doc1.pdf",
    ///     ...     "https://example.com/doc2.pdf"
    ///     ... ]
    ///     >>> results = client.process_batch_documents_from_urls("prebuilt-layout", urls)
    ///     >>> # With optional features
    ///     >>> results = client.process_batch_documents_from_urls(
    ///     ...     "prebuilt-layout",
    ///     ...     urls,
    ///     ...     features=['ocrHighResolution', 'formulas']
    ///     ... )
    ///     >>> for i, result in enumerate(results):
    ///     ...     if isinstance(result, Exception):
    ///     ...         print(f"Document {i} failed: {result}")
    ///     ...     else:
    ///     ...         print(f"Document {i} content: {result.get('content', '')[:100]}")
    #[pyo3(signature = (model_id, document_urls, features=None, output_format= None, max_rps=15), text_signature = "(self, model_id, document_urls, features=None, max_rps=15)")]
    pub fn process_batch_documents_from_urls(
        &self,
        py: Python,
        model_id: String,
        document_urls: Vec<String>,
        features: Option<Vec<String>>,
        output_format: Option<String>,
        max_rps: Option<usize>,
    ) -> PyResult<Vec<Py<PyAny>>> {


        let semaphore_size: usize = max_rps.unwrap_or(15) * self.credentials.len();
        let format_enum = match output_format {
            Some(s) => OutputContentFormat::from_str(&s)?, // Use our impl
            None => OutputContentFormat::default(),
        };
        let output_format = format_enum.to_string();
        let rust_results = py.detach(move || {
            self.runtime.block_on(async {
                self.process_documents_async_from_urls(
                    &model_id,
                    document_urls,
                    features,
                    &output_format,
                    semaphore_size,
                )
                .await
            })
        });
        let mut py_results = Vec::new();
        let py_exception = py.import("builtins")?.getattr("Exception")?;

        for res in rust_results {
            match res {
                Ok(json_value) => match serde_json::from_value::<AnalyzeResult>(json_value) {
                    Ok(analyze_result_struct) => {
                        py_results.push(Py::new(py, analyze_result_struct)?.into_any());
                    }
                    Err(e) => {
                        let msg = format!("Deserialization Error: {}", e);
                        py_results.push(py_exception.call1((msg,))?.unbind());
                    }
                },
                Err(err_string) => {
                    py_results.push(py_exception.call1((err_string,))?.unbind());
                }
            }
        }

        Ok(py_results)
    }

    /// Process multiple documents from local file paths concurrently.
    ///
    /// Analyzes a batch of local documents using the specified Document Intelligence
    /// model. Files are read and uploaded in parallel for maximum efficiency.
    ///
    /// Args:
    ///     model_id (str): The Document Intelligence model ID
    ///         (e.g., 'prebuilt-layout', 'prebuilt-invoice')
    ///     file_paths (list[str]): List of local file paths to process
    ///     features (list[str] | None): Optional list of analysis features to enable
    ///         (e.g., ['ocrHighResolution', 'formulas', 'styleFont']). Defaults to None.
    ///     output_format (str | None): Optional output content format. Valid values are:
    ///         - 'text' (default): Plain text representation with line breaks
    ///         - 'markdown': Markdown formatted output preserving document structure
    ///         Defaults to 'text' if not specified.
    ///
    /// Returns:
    ///     list: List of results where each item is either:
    ///         - dict: Successfully analyzed document result with full analyzeResult
    ///         - Exception: Error object if processing failed for that document
    ///
    /// Supported file formats:
    ///     PDF (.pdf), JPEG (.jpg, .jpeg), PNG (.png), TIFF (.tiff, .tif), BMP (.bmp)
    ///
    /// Example:
    ///     >>> file_paths = [
    ///     ...     "/documents/invoice1.pdf",
    ///     ...     "/documents/receipt2.jpg"
    ///     ... ]
    ///     >>> results = client.process_batch_documents_from_file_paths(
    ///     ...     "prebuilt-invoice",
    ///     ...     file_paths
    ///     ... )
    ///     >>> # With optional features
    ///     >>> results = client.process_batch_documents_from_file_paths(
    ///     ...     "prebuilt-invoice",
    ///     ...     file_paths,
    ///     ...     features=['ocrHighResolution']
    ///     ... )
    ///     >>> for i, result in enumerate(results):
    ///     ...     if isinstance(result, Exception):
    ///     ...         print(f"File {i} failed: {result}")
    ///     ...     else:
    ///     ...         pages = result.get('pages', [])
    ///     ...         print(f"File {i} has {len(pages)} pages")
    #[pyo3(signature=(model_id, file_paths, features=None, output_format=None, max_rps=15), text_signature = "(self, model_id, file_paths, features=None,  output_format='text', max_rps=15)")]
    fn process_batch_documents_from_file_paths(
        &self,
        py: Python,
        model_id: String,
        file_paths: Vec<String>,
        features: Option<Vec<String>>,
        output_format: Option<String>,
        max_rps: Option<usize>,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let semaphore_size = max_rps.unwrap_or(15) * self.credentials.len();
        let format_enum = match output_format {
            Some(s) => OutputContentFormat::from_str(&s)?, // Use our impl
            None => OutputContentFormat::default(),
        };
        let output_format = format_enum.to_string();
        let rust_results = py.detach(move || {
            self.runtime.block_on(async {
                self.process_documents_async_from_file_paths(
                    &model_id,
                    file_paths,
                    features,
                    &output_format,
                    semaphore_size,
                )
                .await
            })
        });
        let mut py_results = Vec::new();
        let py_exception = py.import("builtins")?.getattr("Exception")?;

        for res in rust_results {
            match res {
                Ok(json_value) => match serde_json::from_value::<AnalyzeResult>(json_value) {
                    Ok(analyze_result_struct) => {
                        py_results.push(Py::new(py, analyze_result_struct)?.into_any());
                    }
                    Err(e) => {
                        let msg = format!("Deserialization Error: {}", e);
                        py_results.push(py_exception.call1((msg,))?.unbind());
                    }
                },
                Err(err_string) => {
                    py_results.push(py_exception.call1((err_string,))?.unbind());
                }
            }
        }

        Ok(py_results)
    }
}
