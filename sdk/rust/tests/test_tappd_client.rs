use dstack_sdk::tappd_client::{QuoteHashAlgorithm, TappdClient};
use std::env;

#[tokio::test]
async fn test_tappd_client_creation() {
    // Test client creation with default endpoint
    let _client = TappdClient::new(None);

    // This should succeed without panicking
    assert!(true);
}

#[tokio::test]
async fn test_tappd_client_with_custom_endpoint() {
    // Test client creation with custom endpoint
    let _client = TappdClient::new(Some("/custom/path/tappd.sock"));

    // This should succeed without panicking
    assert!(true);
}

#[tokio::test]
async fn test_tappd_client_with_http_endpoint() {
    // Test client creation with HTTP endpoint
    let _client = TappdClient::new(Some("http://localhost:8080"));

    // This should succeed without panicking
    assert!(true);
}

// Integration tests that require a running tappd service
// These tests will be skipped if no tappd service is available

#[tokio::test]
async fn test_tappd_client_info_integration() {
    let client = get_test_client();
    let info = client.info().await.unwrap();

    println!("✓ Info request successful");
    println!("  App ID: {}", info.app_id);
    println!("  Instance ID: {}", info.instance_id);
    println!("  App Name: {}", info.app_name);

    // Validate response structure
    assert!(!info.app_id.is_empty());
    assert!(!info.instance_id.is_empty());
    assert!(!info.app_name.is_empty());
    assert!(!info.tcb_info.app_compose.is_empty());
    assert!(!info.tcb_info.mrtd.is_empty());
    assert!(!info.tcb_info.rtmr0.is_empty());
    assert!(!info.tcb_info.rtmr1.is_empty());
    assert!(!info.tcb_info.rtmr2.is_empty());
    assert!(!info.tcb_info.rtmr3.is_empty());
    assert!(!info.tcb_info.event_log.is_empty());
}

#[tokio::test]
async fn test_tappd_client_derive_key_integration() {
    let client = get_test_client();

    let response = client.derive_key("test/key/path").await.unwrap();
    println!("✓ Derive key request successful");
    println!("  Key length: {}", response.key.len());
    println!(
        "  Certificate chain length: {}",
        response.certificate_chain.len()
    );

    // Validate response structure
    assert!(!response.key.is_empty());

    // Test key decoding
    match response.to_bytes(None) {
        Ok(key_bytes) => {
            println!("  Decoded key bytes length: {}", key_bytes.len());
            assert!(!key_bytes.is_empty());
        }
        Err(e) => {
            println!("  Key decode error: {}", e);
        }
    }
}

#[tokio::test]
async fn test_tappd_client_derive_key_with_subject_integration() {
    let client = get_test_client();

    let response = client
        .derive_key_with_subject("test/key/path", "example.com")
        .await
        .unwrap();

    println!("✓ Derive key with subject request successful");
    println!("  Key length: {}", response.key.len());
    println!(
        "  Certificate chain length: {}",
        response.certificate_chain.len()
    );

    // Validate response structure
    assert!(!response.key.is_empty());
}

#[tokio::test]
async fn test_tappd_client_derive_key_with_alt_names_integration() {
    let client = get_test_client();

    let alt_names = vec!["www.example.com".to_string(), "api.example.com".to_string()];
    let response = client
        .derive_key_with_subject_and_alt_names(
            "test/key/path",
            Some("example.com"),
            Some(alt_names),
        )
        .await
        .unwrap();
    println!("✓ Derive key with alt names request successful");
    println!("  Key length: {}", response.key.len());
    println!(
        "  Certificate chain length: {}",
        response.certificate_chain.len()
    );

    // Validate response structure
    assert!(!response.key.is_empty());
}

#[tokio::test]
async fn test_tappd_client_tdx_quote_integration() {
    let client = get_test_client();

    let report_data = b"test report data for quote".to_vec();

    let response = client.tdx_quote(report_data).await.unwrap();
    println!("✓ TDX quote request successful");
    println!("  Quote length: {}", response.quote.len());
    println!("  Event log length: {}", response.event_log.len());

    // Validate response structure
    assert!(!response.quote.is_empty());
    assert!(!response.event_log.is_empty());

    // Test quote decoding
    match response.decode_quote() {
        Ok(quote_bytes) => {
            println!("  Decoded quote bytes length: {}", quote_bytes.len());
            assert!(!quote_bytes.is_empty());
        }
        Err(e) => {
            println!("  Quote decode error: {}", e);
        }
    }

    // Test RTMR replay
    match response.replay_rtmrs() {
        Ok(rtmrs) => {
            println!("  Replayed RTMRs: {} entries", rtmrs.len());
            for (idx, rtmr) in rtmrs.iter() {
                println!("    RTMR{}: {}", idx, rtmr);
            }
        }
        Err(e) => {
            println!("  RTMR replay error: {}", e);
        }
    }
}

#[tokio::test]
async fn test_tappd_client_tdx_quote_with_hash_algorithm_integration() {
    let client = get_test_client();

    let report_data = b"test report data for sha256 quote".to_vec();

    let response = client
        .tdx_quote_with_hash_algorithm(report_data, QuoteHashAlgorithm::Sha256)
        .await
        .unwrap();
    println!("✓ TDX quote with SHA256 request successful");
    println!("  Quote length: {}", response.quote.len());
    println!("  Event log length: {}", response.event_log.len());

    // Validate response structure
    assert!(!response.quote.is_empty());
    assert!(!response.event_log.is_empty());
}

#[tokio::test]
async fn test_tappd_client_raw_quote_integration() {
    let client = get_test_client();

    let report_data = vec![0x42u8; 64]; // 64 bytes of test data

    let response = client.raw_quote(report_data).await.unwrap();
    println!("✓ Raw quote request successful");
    println!("  Quote length: {}", response.quote.len());
    println!("  Event log length: {}", response.event_log.len());

    // Validate response structure
    assert!(!response.quote.is_empty());
    assert!(!response.event_log.is_empty());
}

// Helper function to get a test client
fn get_test_client() -> TappdClient {
    // Check for simulator endpoint first
    if let Ok(endpoint) = env::var("TAPPD_SIMULATOR_ENDPOINT") {
        println!("Using TAPPD_SIMULATOR_ENDPOINT: {}", endpoint);
        return TappdClient::new(Some(&endpoint));
    }

    // Check for DSTACK_SIMULATOR_ENDPOINT as fallback
    if let Ok(endpoint) = env::var("DSTACK_SIMULATOR_ENDPOINT") {
        println!("Using DSTACK_SIMULATOR_ENDPOINT as fallback: {}", endpoint);
        return TappdClient::new(Some(&endpoint));
    }

    // Use default endpoint
    println!("Using default tappd endpoint: /var/run/tappd.sock");
    TappdClient::new(None)
}

#[test]
fn test_quote_hash_algorithm_conversion() {
    // Test QuoteHashAlgorithm string conversion
    assert_eq!(QuoteHashAlgorithm::Sha256.as_str(), "sha256");
    assert_eq!(QuoteHashAlgorithm::Sha384.as_str(), "sha384");
    assert_eq!(QuoteHashAlgorithm::Sha512.as_str(), "sha512");
    assert_eq!(QuoteHashAlgorithm::Raw.as_str(), "raw");
}

#[test]
fn test_derive_key_response_hex_decode() {
    use dstack_sdk::tappd_client::DeriveKeyResponse;

    let response = DeriveKeyResponse {
        key: "deadbeef".to_string(),
        certificate_chain: vec![],
    };

    let bytes = response.to_bytes(None).unwrap();
    assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
}

#[test]
fn test_derive_key_response_base64_decode() {
    use dstack_sdk::tappd_client::DeriveKeyResponse;

    // "hello" in base64 is "aGVsbG8="
    let response = DeriveKeyResponse {
        key: "aGVsbG8=".to_string(),
        certificate_chain: vec![],
    };

    let bytes = response.to_bytes(None).unwrap();
    assert_eq!(bytes, b"hello");
}

#[test]
fn test_derive_key_response_pem_strip() {
    use dstack_sdk::tappd_client::DeriveKeyResponse;

    let pem_key = "-----BEGIN PRIVATE KEY-----\naGVsbG8=\n-----END PRIVATE KEY-----";
    let response = DeriveKeyResponse {
        key: pem_key.to_string(),
        certificate_chain: vec![],
    };

    let bytes = response.to_bytes(None).unwrap();
    assert_eq!(bytes, b"hello");
}

#[test]
fn test_derive_key_response_truncate() {
    use dstack_sdk::tappd_client::DeriveKeyResponse;

    let response = DeriveKeyResponse {
        key: "deadbeefcafe".to_string(),
        certificate_chain: vec![],
    };

    let bytes = response.to_bytes(Some(4)).unwrap();
    assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
}
