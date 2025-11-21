use pyo3::prelude::*;
use secrecy::SecretString;

#[pyclass]
#[derive(Clone)]
#[pyo3(from_py_object)]
/// Represents authentication credentials for API access.
///
/// This struct holds the necessary credentials to authenticate and connect
/// to an API endpoint.
///
/// # Fields
///
/// * `api_key` - A secret string containing the API key for authentication.
///   This field is kept private to prevent accidental exposure.
/// * `endpoint` - The API endpoint URL. This field is exposed to Python via
pub struct Credentials {
    pub api_key: SecretString,
    #[pyo3(get)]
    pub endpoint: String,
}
#[pymethods]
impl Credentials {
    #[new]
    #[pyo3(signature=(endpoint, api_key))]
    pub fn new(endpoint: String, api_key: String) -> Self {
        Self {
            api_key: SecretString::from(api_key),
            endpoint,
        }
    }
}
