use reqwest::{
    Client,
    header::{CONTENT_TYPE, HeaderValue},
};
use secrecy::ExposeSecret;
use serde_json::Value;
use std::{path::Path, time::Duration};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::models::{StatusResponse, credentials::Credentials};
use crate::utils::get_content_type;

pub async fn analyze_document_from_urls(
    client: &Client,
    model_id: &str,
    creds: Credentials,
    document_url: &str,
    output_format: &str,
    features: &Option<Vec<String>>,
) -> anyhow::Result<Value> {
    let endpoint = creds.endpoint.trim_end_matches('/');

    let api_version = "2024-11-30";
    let mut analyze_url = format!(
        "{}/documentintelligence/documentModels/{}:analyze?api-version={}&outputContentFormat={}",
        endpoint, model_id, api_version, output_format
    );
    if let Some(feature_list) = features
        && !feature_list.is_empty()
    {
        let features_param = feature_list.join(",");
        analyze_url.push_str(&format!("&features={}", features_param));
    }

    let mut api_key_val = HeaderValue::from_str(creds.api_key.expose_secret())?;
    api_key_val.set_sensitive(true);
    let auth_header_value = api_key_val;

    let response = client
        .post(&analyze_url)
        .header("Content-Type", "application/json")
        .header("Ocp-Apim-Subscription-Key", auth_header_value.clone())
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

    info!(
        document_url = document_url,
        "Operation Location: {}", operation_location
    );

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let status_response = client
            .get(operation_location)
            .header("Ocp-Apim-Subscription-Key", auth_header_value.clone())
            .send()
            .await?
            .error_for_status()?
            .json::<StatusResponse>()
            .await?;

        info!(
            status = status_response.status.as_str(),
            "Polling status response: {}",
            status_response.status.as_str()
        );

        match status_response.status.as_str() {
            "succeeded" => {
                return status_response
                    .result
                    .ok_or_else(|| anyhow::anyhow!("API succeeded but returned no result"));
            }
            "failed" => return Err(anyhow::anyhow!("Document analysis failed")),
            "running" | "notStarted" => continue,
            other => return Err(anyhow::anyhow!("Unknown status: {}", other)),
        }
    }
}

pub async fn analyze_document_from_file_path(
    client: &Client,
    model_id: &str,
    creds: Credentials,
    file_path: &str,
    output_format: &str,
    features: &Option<Vec<String>>,
) -> anyhow::Result<Value> {
    let mut file = File::open(file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to open file {}: {}", file_path, e))?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?;
    let file_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();
    // Determine content type based on file extension
    let content_type = get_content_type(file_path);
    let endpoint = creds.endpoint.trim_end_matches('/');
    let api_version = "2024-11-30";
    let mut analyze_url = format!(
        "{}/documentintelligence/documentModels/{}:analyze?api-version={}&outputContentFormat={}",
        endpoint, model_id, api_version, output_format
    );

    if let Some(feature_list) = features
        && !feature_list.is_empty()
    {
        let features_param = feature_list.join(",");
        analyze_url.push_str(&format!("&features={}", features_param));
    }

    let mut api_key_val = HeaderValue::from_str(creds.api_key.expose_secret())?;
    api_key_val.set_sensitive(true);
    let auth_header_value = api_key_val;

    // Send file as binary data
    let response = client
        .post(&analyze_url)
        .header("Ocp-Apim-Subscription-Key", auth_header_value.clone())
        .header(CONTENT_TYPE, HeaderValue::from_static(content_type))
        .body(file_contents)
        .send()
        .await?
        .error_for_status()?;

    info!(
        file_name = file_name,
        status_code = response.status().as_u16(),
        "Document analysis request submitted"
    );

    let operation_location = response
        .headers()
        .get("operation-location")
        .ok_or_else(|| anyhow::anyhow!("Response missing 'operation-location' header"))?
        .to_str()?;

    info!(
        file_name = file_name,
        operation_location = operation_location,
        "Document analysis operation initiated"
    );

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let status_response = client
            .get(operation_location)
            .header("Ocp-Apim-Subscription-Key", auth_header_value.clone())
            .send()
            .await?
            .error_for_status()?
            .json::<StatusResponse>()
            .await?;

        info!(
            file_name = file_name,
            status = status_response.status.as_str(),
            operation_location = operation_location,
            "Polling document analysis status"
        );

        match status_response.status.as_str() {
            "succeeded" => {
                return status_response
                    .result
                    .ok_or_else(|| anyhow::anyhow!("API succeeded but returned no result"));
            }
            "failed" => return Err(anyhow::anyhow!("Document analysis failed")),
            "running" | "notStarted" => continue,
            other => return Err(anyhow::anyhow!("Unknown status: {}", other)),
        }
    }
}
