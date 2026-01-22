use crate::error::{AppError, Result};
use crate::utils::metadata::extractor::ExtractedMetadata;
use chrono::{DateTime, Utc};
use serde::Deserialize;

/// External API provider for metadata extraction
pub enum MetadataProvider {
    LinkPreview,
    Microlink,
}

/// LinkPreview API response
#[derive(Debug, Deserialize)]
struct LinkPreviewResponse {
    title: Option<String>,
    description: Option<String>,
    image: Option<String>,
    url: String,
    site_name: Option<String>,
}

/// Microlink API response
#[derive(Debug, Deserialize)]
struct MicrolinkResponse {
    data: MicrolinkData,
}

#[derive(Debug, Deserialize)]
struct MicrolinkData {
    title: Option<String>,
    description: Option<String>,
    image: Option<MicrolinkImage>,
    publisher: Option<String>,
    author: Option<String>,
    date: Option<String>,
    lang: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MicrolinkImage {
    url: Option<String>,
}

/// Fetch metadata from external APIs
pub async fn fetch_from_external(url: &str) -> Result<ExtractedMetadata> {
    // Try providers in order
    match fetch_from_microlink(url).await {
        Ok(metadata) => return Ok(metadata),
        Err(e) => tracing::debug!("Microlink failed for {}: {}", url, e),
    }

    match fetch_from_linkpreview(url).await {
        Ok(metadata) => return Ok(metadata),
        Err(e) => tracing::debug!("LinkPreview failed for {}: {}", url, e),
    }

    Err(AppError::Internal("All external metadata providers failed".to_string()))
}

/// Fetch metadata from Microlink API
async fn fetch_from_microlink(url: &str) -> Result<ExtractedMetadata> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let api_url = format!("https://api.microlink.io?url={}", urlencoding::encode(url));
    
    let response = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Microlink API request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "Microlink API returned error: HTTP {}",
            response.status()
        )));
    }

    let microlink_response: MicrolinkResponse = response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse Microlink response: {}", e)))?;

    let data = microlink_response.data;
    let published_at = data.date
        .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
        .map(|dt| dt.with_timezone(&Utc));

    Ok(ExtractedMetadata {
        title: data.title,
        description: data.description,
        image_url: data.image.and_then(|img| img.url),
        favicon_url: None,
        site_name: data.publisher,
        author: data.author,
        published_at,
        content_type: None,
        metadata_json: None,
    })
}

/// Fetch metadata from LinkPreview API
/// Note: LinkPreview requires an API key, so this is a placeholder
async fn fetch_from_linkpreview(_url: &str) -> Result<ExtractedMetadata> {
    // LinkPreview requires an API key
    // For now, return an error to indicate it's not configured
    // In the future, this could check for an API key in settings
    Err(AppError::Internal("LinkPreview API key not configured".to_string()))
}
