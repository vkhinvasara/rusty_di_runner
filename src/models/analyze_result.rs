use crate::impl_to_dict;
use pyo3::prelude::*;
use pythonize::pythonize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[pyclass]
#[serde(rename_all(deserialize="camelCase"))]
pub struct AnalyzeResult {
    #[pyo3(get, set)]
    pub api_version: String,
    #[pyo3(get, set)]
    pub model_id: String,
    #[pyo3(get, set)]
    pub content: String,
    #[pyo3(get, set)]
    pub pages: Vec<DocumentPage>,
    #[pyo3(get, set)]
    pub paragraphs: Option<Vec<DocumentParagraph>>,
    #[pyo3(get, set)]
    pub tables: Option<Vec<DocumentTable>>,
    #[pyo3(get, set)]
    pub languages: Option<Vec<DocumentLanguage>>,
    // Add styles, documents, etc. if needed
}
impl_to_dict!(AnalyzeResult);
#[derive(Serialize, Deserialize, Clone, Debug)]
#[pyclass]
#[serde(rename_all(deserialize="camelCase"))]
pub struct DocumentPage {
    #[pyo3(get, set)]
    pub page_number: i32,
    #[pyo3(get, set)]
    pub angle: Option<f32>,
    #[pyo3(get, set)]
    pub width: Option<f32>,
    #[pyo3(get, set)]
    pub height: Option<f32>,
    #[pyo3(get, set)]
    pub unit: Option<String>,
    #[pyo3(get, set)]
    pub lines: Option<Vec<DocumentLine>>,
    #[pyo3(get, set)]
    pub words: Option<Vec<DocumentWord>>,
    #[pyo3(get, set)]
    pub spans: Vec<DocumentSpan>,
}
impl_to_dict!(DocumentPage);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[pyclass]
#[serde(rename_all(deserialize="camelCase"))]
pub struct DocumentLine {
    #[pyo3(get, set)]
    pub content: String,
    #[pyo3(get, set)] // Azure sends [x,y,x,y...], we store as Vec<f32> or similar
    pub polygon: Option<Vec<f32>>,
    #[pyo3(get, set)]
    pub spans: Vec<DocumentSpan>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[pyclass]
#[serde(rename_all(deserialize="camelCase"))]
pub struct DocumentWord {
    #[pyo3(get, set)]
    pub content: String,
    #[pyo3(get, set)]
    pub polygon: Option<Vec<f32>>,
    #[pyo3(get, set)]
    pub span: DocumentSpan,
    #[pyo3(get, set)]
    pub confidence: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[pyclass]
#[serde(rename_all(deserialize="camelCase"))]
pub struct DocumentParagraph {
    #[pyo3(get, set)]
    pub role: Option<String>,
    #[pyo3(get, set)]
    pub content: String,
    #[pyo3(get, set)]
    pub bounding_regions: Option<Vec<BoundingRegion>>,
    #[pyo3(get, set)]
    pub spans: Vec<DocumentSpan>,
}
impl_to_dict!(DocumentParagraph);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[pyclass]
#[serde(rename_all(deserialize="camelCase"))]
pub struct DocumentTable {
    #[pyo3(get, set)]
    pub row_count: i32,
    #[pyo3(get, set)]
    pub column_count: i32,
    #[pyo3(get, set)]
    pub cells: Vec<DocumentTableCell>,
    #[pyo3(get, set)]
    pub bounding_regions: Option<Vec<BoundingRegion>>,
    #[pyo3(get, set)]
    pub spans: Vec<DocumentSpan>,
}
impl_to_dict!(DocumentTable);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[pyclass]
#[serde(rename_all(deserialize="camelCase"))]
pub struct DocumentTableCell {
    #[pyo3(get, set)]
    pub row_index: i32,
    #[pyo3(get, set)]
    pub column_index: i32,
    #[pyo3(get, set)]
    pub content: String,
    #[pyo3(get, set)]
    pub bounding_regions: Option<Vec<BoundingRegion>>,
    #[pyo3(get, set)]
    pub spans: Vec<DocumentSpan>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[pyclass]
#[serde(rename_all(deserialize="camelCase"))]
pub struct DocumentSpan {
    #[pyo3(get, set)]
    pub offset: usize,
    #[pyo3(get, set)]
    pub length: usize,
}
impl_to_dict!(DocumentSpan);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[pyclass]
#[serde(rename_all(deserialize="camelCase"))]
pub struct BoundingRegion {
    #[pyo3(get, set)]
    pub page_number: i32,
    #[pyo3(get, set)]
    pub polygon: Vec<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[pyclass]
#[serde(rename_all(deserialize="camelCase"))]
pub struct DocumentLanguage {
    #[pyo3(get, set)]
    pub locale: String,
    #[pyo3(get, set)]
    pub spans: Vec<DocumentSpan>,
    #[pyo3(get, set)]
    pub confidence: f32,
}
