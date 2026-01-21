// Integration tests for the Rust backend
// These tests verify that the API endpoints work correctly

use desktop_lib::db;
use desktop_lib::error::Result;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to wait for server to be ready
async fn wait_for_server(url: &str) -> bool {
    for _ in 0..10 {
        if reqwest::get(url).await.is_ok() {
            return true;
        }
        sleep(Duration::from_millis(100)).await;
    }
    false
}

#[tokio::test]
#[ignore] // Ignore by default - requires server to be running
async fn test_health_check() -> Result<()> {
    let base_url = std::env::var("TEST_API_URL")
        .unwrap_or_else(|_| "http://localhost:9119".to_string());
    
    if !wait_for_server(&format!("{}/v1/ping", base_url)).await {
        eprintln!("Server not available at {}, skipping test", base_url);
        return Ok(());
    }
    
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/v1/ping", base_url))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await?;
    assert_eq!(body["message"], "pong pong");
    
    Ok(())
}

#[tokio::test]
#[ignore] // Ignore by default - requires server to be running
async fn test_tag_crud() -> Result<()> {
    let base_url = std::env::var("TEST_API_URL")
        .unwrap_or_else(|_| "http://localhost:9119".to_string());
    
    if !wait_for_server(&format!("{}/v1/ping", base_url)).await {
        eprintln!("Server not available at {}, skipping test", base_url);
        return Ok(());
    }
    
    let client = reqwest::Client::new();
    
    // Create a tag
    let create_response = client
        .post(&format!("{}/v1/tags", base_url))
        .json(&json!({ "name": "test-tag-integration" }))
        .send()
        .await?;
    
    assert!(create_response.status().is_success());
    let tag: serde_json::Value = create_response.json().await?;
    assert_eq!(tag["name"], "test-tag-integration");
    let tag_id = tag["id"].as_str().unwrap();
    
    // Get all tags
    let list_response = client
        .get(&format!("{}/v1/tags", base_url))
        .send()
        .await?;
    
    assert!(list_response.status().is_success());
    let tags: Vec<serde_json::Value> = list_response.json().await?;
    assert!(tags.iter().any(|t| t["id"].as_str() == Some(tag_id)));
    
    Ok(())
}
