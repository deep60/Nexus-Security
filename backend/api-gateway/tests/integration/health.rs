use reqwest::StatusCode;

fn api_base_url() -> Option<String> {
    std::env::var("VERDYX_API_BASE_URL")
        .ok()
        .map(|url| url.trim_end_matches('/').to_string())
}

#[tokio::test]
#[ignore = "requires a running api-gateway; set VERDYX_API_BASE_URL"]
async fn health_endpoint_responds() {
    let Some(base_url) = api_base_url() else {
        eprintln!("skipping live API test; VERDYX_API_BASE_URL is not set");
        return;
    };
    let response = reqwest::get(format!("{base_url}/health"))
        .await
        .expect("health request should complete");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires a running api-gateway; set VERDYX_API_BASE_URL"]
async fn api_v1_health_endpoint_responds() {
    let Some(base_url) = api_base_url() else {
        eprintln!("skipping live API test; VERDYX_API_BASE_URL is not set");
        return;
    };
    let response = reqwest::get(format!("{base_url}/api/v1/health"))
        .await
        .expect("v1 health request should complete");

    assert!(response.status().is_success());
}
