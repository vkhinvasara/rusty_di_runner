use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use futures::future::join_all;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::{
    clients::document_intelligence::{analyze_document_from_file_path, analyze_document_from_urls},
    models::analysis_client::RustyAnalysisClient,
};

// TODO Add enum routing to batches
// pub enum ClientType {
//     DocumentIntelligenceClient(BatchType),
//     DocumentAnalysisClient(BatchType)
// }
// pub enum BatchType{
//     BlobUrls,
//     FilePaths
// }
// impl BatchType{
//     fn process_batch(&self){
//         match self{
//             BatchType::BlobUrls
//         }
//     }
// }
// impl ClientType{
//     fn process(&self){
//         match self{
//             ClientType::DocumentAnalysisClient(batch)=>{
//                 batch.process_batch()
//             },
//             _ => anyhow
//         }
//     }
// }
impl RustyAnalysisClient {
    pub async fn process_documents_async_from_urls(
        &self,
        model_id: &str,
        document_urls: Vec<String>,
        features: Option<Vec<String>>,
        output_format: &str,
        semaphore_size: usize,
    ) -> Vec<Result<Value, String>> {
        let client = Client::new();
        let cred_list = Arc::new(self.credentials.clone());
        let list_len = cred_list.len();
        let semaphore = Arc::new(Semaphore::new(semaphore_size));
        let current_index = Arc::new(AtomicUsize::new(0));
        let tasks = document_urls.into_iter().map(|url| {
            let client = client.clone();
            let cred_list_clone = cred_list.clone();
            let index_counter = current_index.clone();
            let model_id_str = model_id.to_string();
            let features = features.clone();
            let semaphore = semaphore.clone();
            let output_format = output_format.to_owned();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let old_index = index_counter.fetch_add(1, Ordering::Relaxed);
                let actual_index = old_index % list_len;
                let creds = cred_list_clone[actual_index].clone();

                analyze_document_from_urls(&client, &model_id_str, creds, &url, &output_format, &features).await
            })
        });

        let results = join_all(tasks).await;

        results
            .into_iter()
            .map(|join_result| match join_result {
                Err(join_err) => Err(format!("Task panicked: {}", join_err)),
                Ok(api_result) => match api_result {
                    Ok(analysis) => Ok(analysis),
                    Err(api_err) => Err(format!("API Error: {}", api_err)),
                },
            })
            .collect()
    }

    pub async fn process_documents_async_from_file_paths(
        &self,
        model_id: &str,
        file_paths: Vec<String>,
        features: Option<Vec<String>>,
        output_format: &str,
        semaphore_size: usize,
    ) -> Vec<Result<Value, String>> {
        let client = Client::new();
        let semaphore = Arc::new(Semaphore::new(semaphore_size));
        let cred_list = Arc::new(self.credentials.clone());
        let current_index = Arc::new(AtomicUsize::new(0));
        let list_len = cred_list.len();
        let tasks = file_paths.into_iter().map(|url| {
            let client = client.clone();
            let cred_list_clone = cred_list.clone();
            let index_counter = current_index.clone();
            let model_id_str = model_id.to_string();
            let features = features.clone();
            let semaphore = semaphore.clone();
            let output_format = output_format.to_owned();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let old_index = index_counter.fetch_add(1, Ordering::Relaxed);
                let actual_index = old_index % list_len;
                let creds = cred_list_clone[actual_index].clone();
                analyze_document_from_file_path(&client, &model_id_str, creds, &url, &output_format, &features)
                    .await
            })
        });

        let results = join_all(tasks).await;

        results
            .into_iter()
            .map(|join_result| match join_result {
                Err(join_err) => Err(format!("Task panicked: {}", join_err)),
                Ok(api_result) => match api_result {
                    Ok(analysis) => Ok(analysis),
                    Err(api_err) => Err(format!("API Error: {}", api_err)),
                },
            })
            .collect()
    }
}
