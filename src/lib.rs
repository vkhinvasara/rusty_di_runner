use std::{path::Path, time::Duration};

use pyo3::{IntoPyObjectExt, prelude::*, types::PyDict};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;
use tokio::runtime::Runtime;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// A client for analyzing documents using Azure Document Intelligence API.
/// 
/// This client provides batch processing capabilities for document analysis
/// using the Azure Document Intelligence service (formerly Form Recognizer).
/// 
/// Args:
///     endpoint (str): The Azure Document Intelligence endpoint URL 
///         (e.g., "https://myservice.cognitiveservices.azure.com")
///     api_key (str): The API subscription key for authentication
/// 
/// Example:
///     >>> client = RustyAnalysisClient(
///     ...     "https://myservice.cognitiveservices.azure.com",
///     ...     "your-api-key"
///     ... )
#[pyclass]
struct RustyAnalysisClient{
    endpoint: String,
    api_key: String,
    runtime: Runtime
}


#[derive(serde::Deserialize, serde::Serialize)]
struct StatusResponse {
    status: String,
    #[serde(rename = "analyzeResult")]
    result: Option<Value>, 
}

#[pymethods]
impl RustyAnalysisClient{
    /// Create a new RustyAnalysisClient instance.
    ///
    /// Args:
    ///     endpoint (str): Azure Document Intelligence endpoint URL 
    ///         (e.g., 'https://your-resource.cognitiveservices.azure.com')
    ///     api_key (str): Azure subscription key for authentication
    ///
    /// Returns:
    ///     RustyAnalysisClient: A new client instance
    ///
    /// Example:
    ///     >>> client = RustyAnalysisClient(
    ///     ...     endpoint="https://your-resource.cognitiveservices.azure.com",
    ///     ...     api_key="your-api-key"
    ///     ... )
    #[new]
    #[pyo3(signature = (endpoint, api_key))]
    fn new(endpoint: String, api_key: String)-> PyResult<Self>{
        Ok(Self { 
            endpoint,
             api_key,
              runtime: Runtime::new().unwrap()
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
    #[pyo3(signature = (model_id, document_urls, features=None), text_signature = "(self, model_id, document_urls, features=None)")]
    fn process_batch_documents_from_urls(&self,py:Python, model_id: String, document_urls: Vec<String>, features: Option<Vec<String>>) -> PyResult<Vec<Py<PyAny>>>{
        let rust_results = py.detach(move ||{
            self.runtime.block_on(async{
             self.process_documents_async_from_urls(&model_id, document_urls, features).await
            })
        });
        let mut py_results = Vec::new();
        let py_exception = py.import("builtins")?.getattr("Exception")?;

        for res in rust_results {
            match res {
                // Success: Convert Rust struct to PyObject (e.g., a PyDict)
                Ok(analysis_result) => {
                    let json_val = serde_json::to_value(analysis_result)
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                    py_results.push(json_to_py(py, json_val)?);
                }
                // Failure: Create a Python Exception object
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
    #[pyo3(signature = (model_id, file_paths, features=None), text_signature = "(self, model_id, file_paths, features=None)")]
    fn process_batch_documents_from_file_paths(&self,py:Python, model_id: String, file_paths: Vec<String>, features: Option<Vec<String>>) -> PyResult<Vec<Py<PyAny>>>{
        let rust_results = py.detach(move ||{
            self.runtime.block_on(async{
             self.process_documents_async_from_file_paths(&model_id, file_paths, features).await
            })
        });
        let mut py_results = Vec::new();
        let py_exception = py.import("builtins")?.getattr("Exception")?;

        for res in rust_results {
            match res {
                
                Ok(analysis_result) => {
                    let json_val = serde_json::to_value(analysis_result)
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                    py_results.push(json_to_py(py, json_val)?);
                }
                
                Err(err_string) => {
                    py_results.push(py_exception.call1((err_string,))?.unbind());
                }
            }
        }

        Ok(py_results)
    }
}


use reqwest::Client;
use futures::future::join_all;

impl RustyAnalysisClient {
    
    async fn process_documents_async_from_urls(
        &self,
        model_id: &str,
        document_urls: Vec<String>,
        features: Option<Vec<String>>
    ) -> Vec<Result<Value, String>> {
        
        let client = Client::new(); 
        let tasks = document_urls.into_iter().map(|url| {
            let client = client.clone();
            let endpoint = self.endpoint.clone();
            let api_key = self.api_key.clone();
            let model_id_str = model_id.to_string(); 
            let features = features.clone();
            
            tokio::spawn(async move {
                analyze_document_from_urls(&client, &model_id_str, &endpoint, &api_key, &url, &features).await
            })
        });

        let results = join_all(tasks).await;

        
        results.into_iter().map(|join_result| {
            match join_result {
                Err(join_err) => Err(format!("Task panicked: {}", join_err)),
                Ok(api_result) => match api_result {
                    Ok(analysis) => Ok(analysis),
                    Err(api_err) => Err(format!("API Error: {}", api_err)),
                }
            }
        }).collect()
    }
    
    async fn process_documents_async_from_file_paths(
        &self,
        model_id: &str,
        file_paths: Vec<String>,
        features: Option<Vec<String>>
    ) -> Vec<Result<Value, String>> {
        
        let client = Client::new(); 
        let tasks = file_paths.into_iter().map(|url| {
            let client = client.clone();
            let endpoint = self.endpoint.clone();
            let api_key = self.api_key.clone();
            let model_id_str = model_id.to_string(); 
            let features = features.clone();
            tokio::spawn(async move {
                analyze_document_from_file_path(&client, &model_id_str, &endpoint, &api_key, &url, &features).await
            })
        });

        let results = join_all(tasks).await;

        
        results.into_iter().map(|join_result| {
            match join_result {
                Err(join_err) => Err(format!("Task panicked: {}", join_err)),
                Ok(api_result) => match api_result {
                    Ok(analysis) => Ok(analysis),
                    Err(api_err) => Err(format!("API Error: {}", api_err)),
                }
            }
        }).collect()
    }
}


async fn analyze_document_from_urls(
    client: &Client,
    model_id: &str,
    endpoint: &str,
    api_key: &str,
    document_url: &str,
    features: &Option<Vec<String>>
) -> anyhow::Result<Value> {

    let endpoint = endpoint.trim_end_matches('/');

    let api_version = "2024-11-30";
    let mut analyze_url = format!(
        "{}/documentintelligence/documentModels/{}:analyze?api-version={}",
        endpoint, model_id, api_version
    );
    if let Some(feature_list) = features
        && !feature_list.is_empty() {
            let features_param = feature_list.join(",");
            analyze_url.push_str(&format!("&features={}", features_param));
        }
    let response = client
        .post(&analyze_url)
        .header("Ocp-Apim-Subscription-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "urlSource": document_url
        }))
        .send()
        .await?
        .error_for_status()?; 
    
    let operation_location = response
        .headers()
        .get("operation-location")
        .ok_or_else(|| anyhow::anyhow!("Response missing 'operation-location' header"))?
        .to_str()?;
    
    println!("Operation Location: {}", operation_location);
    
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await; 
        
        let status_response = client
            .get(operation_location)
            .header("Ocp-Apim-Subscription-Key", api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<StatusResponse>()
            .await?;
        
        println!("Polling status response:{}", status_response.status.as_str());

        match status_response.status.as_str() {
            "succeeded" => return 
                status_response.result.ok_or_else(|| anyhow::anyhow!("API succeeded but returned no result"))
            ,
            "failed" => return Err(anyhow::anyhow!("Document analysis failed")),
            "running" | "notStarted" => continue,
            other => return Err(anyhow::anyhow!("Unknown status: {}", other)),
        }
    }
}

async fn analyze_document_from_file_path(
    client: &Client,
    model_id: &str,
    endpoint: &str,
    api_key: &str,
    file_path: &str,
    features: &Option<Vec<String>>
) -> anyhow::Result<Value> {
    let mut file = File::open(file_path).await
        .map_err(|e| anyhow::anyhow!("Failed to open file {}: {}", file_path, e))?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents).await
        .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?;
    
    // Determine content type based on file extension
    let content_type = get_content_type(file_path);
    let endpoint = endpoint.trim_end_matches('/');
    let api_version = "2024-11-30";
    let mut analyze_url = format!(
        "{}/documentintelligence/documentModels/{}:analyze?api-version={}",
        endpoint, model_id, api_version
    );

    if let Some(feature_list) = features
        && !feature_list.is_empty() {
            let features_param = feature_list.join(",");
            analyze_url.push_str(&format!("&features={}", features_param));
        }

    let mut headers = HeaderMap::new();
    headers.insert("Ocp-Apim-Subscription-Key", HeaderValue::from_str(api_key)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
    
    // Send file as binary data
    let response = client
        .post(&analyze_url)
        .headers(headers)
        .body(file_contents)
        .send()
        .await?
        .error_for_status()?; 
    
    println!("API Response Status: {:?}", response.status());
    println!("API Response Headers: {:?}", response.headers());
    
    let operation_location = response
        .headers()
        .get("operation-location")
        .ok_or_else(|| anyhow::anyhow!("Response missing 'operation-location' header"))?
        .to_str()?;
    
    println!("Operation Location: {}", operation_location);
    
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await; 
        
        let status_response = client
            .get(operation_location)
            .header("Ocp-Apim-Subscription-Key", api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<StatusResponse>()
            .await?;
        
        println!("Polling status response:{}", status_response.status.as_str());

        match status_response.status.as_str() {
            "succeeded" => return 
                status_response.result.ok_or_else(|| anyhow::anyhow!("API succeeded but returned no result"))
            ,
            "failed" => return Err(anyhow::anyhow!("Document analysis failed")),
            "running" | "notStarted" => continue,
            other => return Err(anyhow::anyhow!("Unknown status: {}", other)),
        }
    }
}

fn json_to_py(py: Python, val: Value) -> PyResult<Py<PyAny>> {
    match val {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => b.into_py_any(py),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py_any(py)
            } else if let Some(f) = n.as_f64() {
                f.into_py_any(py)
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid number: {}", n)))
            }
        }
        Value::String(s) => s.into_py_any(py),
        Value::Array(v) => {
            let mut py_list = Vec::new();
            for item in v {
                py_list.push(json_to_py(py, item)?);
            }
            py_list.into_py_any(py)
        }
        Value::Object(m) => {
            let dict = PyDict::new(py);
            for (k, v) in m {
                dict.set_item(k, json_to_py(py, v)?)?;
            }
            dict.into_py_any(py)
        }
    }
}

fn get_content_type(file_path: &str) -> &'static str {
    let path = Path::new(file_path);
    match path.extension().and_then(|s| s.to_str()) {
        Some("pdf") => "application/pdf",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("tiff") | Some("tif") => "image/tiff",
        Some("bmp") => "image/bmp",
        _ => "application/octet-stream",
    }
}

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
    m.add_class::<RustyAnalysisClient>()?;
    Ok(())
}
