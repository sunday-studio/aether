use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use url::Url;

/// Extracted metadata from a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub favicon_url: Option<String>,
    pub site_name: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub content_type: Option<String>,
    pub metadata_json: Option<serde_json::Value>,
}

/// Metadata extractor for URLs
pub struct MetadataExtractor;

impl MetadataExtractor {
    /// Extract metadata from a URL
    /// Tries local HTML parsing first, then falls back to external APIs
    pub async fn extract(url: &str) -> Result<ExtractedMetadata> {
        // Validate URL
        let parsed_url = Url::parse(url)
            .map_err(|e| AppError::BadRequest(format!("Invalid URL: {}", e)))?;

        // Try local extraction first
        match Self::extract_from_html(url).await {
            Ok(metadata) if Self::is_metadata_complete(&metadata) => {
                return Ok(metadata);
            }
            Ok(metadata) => {
                // Partial metadata, try to enhance with external APIs
                if let Ok(enhanced) = crate::utils::metadata::providers::fetch_from_external(url).await {
                    return Ok(Self::merge_metadata(metadata, enhanced));
                }
                Ok(metadata)
            }
            Err(_) => {
                // Local extraction failed, try external APIs
                match crate::utils::metadata::providers::fetch_from_external(url).await {
                    Ok(metadata) => Ok(metadata),
                    Err(e) => {
                        tracing::warn!("Failed to extract metadata for {}: {}", url, e);
                        // Return minimal metadata with just the URL
                        Ok(ExtractedMetadata {
                            title: Some(parsed_url.host_str().unwrap_or("").to_string()),
                            description: None,
                            image_url: None,
                            favicon_url: Self::extract_favicon_url(&parsed_url),
                            site_name: parsed_url.host_str().map(|s| s.to_string()),
                            author: None,
                            published_at: None,
                            content_type: None,
                            metadata_json: None,
                        })
                    }
                }
            }
        }
    }

    /// Extract metadata from HTML content
    async fn extract_from_html(url: &str) -> Result<ExtractedMetadata> {
        // Fetch HTML content
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; Aether/1.0; +https://github.com/sunday-studio/aether)")
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch URL: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::BadRequest(format!(
                "Failed to fetch URL: HTTP {}",
                response.status()
            )));
        }

        let html_content = response
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read response: {}", e)))?;

        let document = Html::parse_document(&html_content);
        let parsed_url = Url::parse(url)
            .map_err(|e| AppError::BadRequest(format!("Invalid URL: {}", e)))?;

        // Extract Open Graph tags (primary)
        let og_title = Self::select_meta_content(&document, "property", "og:title");
        let og_description = Self::select_meta_content(&document, "property", "og:description");
        let og_image = Self::select_meta_content(&document, "property", "og:image");
        let og_site_name = Self::select_meta_content(&document, "property", "og:site_name");
        let og_type = Self::select_meta_content(&document, "property", "og:type");
        let og_author = Self::select_meta_content(&document, "property", "article:author");

        // Extract Twitter Card tags (fallback)
        let twitter_title = Self::select_meta_content(&document, "name", "twitter:title");
        let twitter_description = Self::select_meta_content(&document, "name", "twitter:description");
        let twitter_image = Self::select_meta_content(&document, "name", "twitter:image");

        // Extract standard meta tags (final fallback)
        let standard_title = Self::select_title(&document);
        let standard_description = Self::select_meta_content(&document, "name", "description");
        let standard_author = Self::select_meta_content(&document, "name", "author");

        // Extract published date
        let published_at = Self::select_meta_content(&document, "property", "article:published_time")
            .or_else(|| Self::select_meta_content(&document, "name", "date"))
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // Extract favicon
        let favicon_url = Self::extract_favicon(&document, &parsed_url);

        // Resolve relative image URLs
        let image_url = og_image
            .or(twitter_image)
            .map(|img| Self::resolve_url(&parsed_url, &img));

        // Build metadata JSON
        let mut metadata_json = serde_json::Map::new();
        if let Some(ref og_type) = og_type {
            metadata_json.insert("og:type".to_string(), serde_json::Value::String(og_type.clone()));
        }

        Ok(ExtractedMetadata {
            title: og_title
                .or(twitter_title)
                .or(standard_title),
            description: og_description
                .or(twitter_description)
                .or(standard_description),
            image_url,
            favicon_url,
            site_name: og_site_name,
            author: og_author.or(standard_author),
            published_at,
            content_type: og_type,
            metadata_json: if metadata_json.is_empty() {
                None
            } else {
                Some(serde_json::Value::Object(metadata_json))
            },
        })
    }

    /// Select meta tag content by property or name
    fn select_meta_content(document: &Html, attr: &str, value: &str) -> Option<String> {
        let selector = Selector::parse(&format!("meta[{}='{}']", attr, value)).ok()?;
        document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string())
    }

    /// Select page title
    fn select_title(document: &Html) -> Option<String> {
        let selector = Selector::parse("title").ok()?;
        document
            .select(&selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Extract favicon URL
    fn extract_favicon(document: &Html, base_url: &Url) -> Option<String> {
        // Try <link rel="icon"> or <link rel="shortcut icon">
        let icon_selector = Selector::parse("link[rel='icon'], link[rel='shortcut icon'], link[rel='apple-touch-icon']").ok()?;
        if let Some(icon) = document.select(&icon_selector).next() {
            if let Some(href) = icon.value().attr("href") {
                return Some(Self::resolve_url(base_url, href));
            }
        }

        // Fallback to default favicon.ico
        Some(Self::resolve_url(base_url, "/favicon.ico"))
    }

    /// Extract favicon URL from base URL (fallback)
    fn extract_favicon_url(base_url: &Url) -> Option<String> {
        Some(Self::resolve_url(base_url, "/favicon.ico"))
    }

    /// Resolve relative URL to absolute URL
    fn resolve_url(base: &Url, relative: &str) -> String {
        base.join(relative)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| relative.to_string())
    }

    /// Check if metadata is complete enough
    fn is_metadata_complete(metadata: &ExtractedMetadata) -> bool {
        metadata.title.is_some() && metadata.description.is_some()
    }

    /// Merge two metadata objects, preferring the first one
    fn merge_metadata(primary: ExtractedMetadata, secondary: ExtractedMetadata) -> ExtractedMetadata {
        ExtractedMetadata {
            title: primary.title.or(secondary.title),
            description: primary.description.or(secondary.description),
            image_url: primary.image_url.or(secondary.image_url),
            favicon_url: primary.favicon_url.or(secondary.favicon_url),
            site_name: primary.site_name.or(secondary.site_name),
            author: primary.author.or(secondary.author),
            published_at: primary.published_at.or(secondary.published_at),
            content_type: primary.content_type.or(secondary.content_type),
            metadata_json: primary.metadata_json.or(secondary.metadata_json),
        }
    }
}
