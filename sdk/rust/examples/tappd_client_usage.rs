use dstack_sdk::tappd_client::{QuoteHashAlgorithm, TappdClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a TappdClient with default endpoint (/var/run/tappd.sock)
    let client = TappdClient::new(None);

    // Or create with a custom endpoint
    // let client = TappdClient::new(Some("/custom/path/tappd.sock"));

    // Or create with HTTP endpoint for simulator
    // let client = TappdClient::new(Some("http://localhost:8080"));

    println!("TappdClient created successfully!");

    // Example usage (these will fail without a running tappd service):

    // 1. Derive a key
    match client.derive_key("my/key/path").await {
        Ok(response) => {
            println!("Key derived successfully!");
            println!(
                "Certificate chain length: {}",
                response.certificate_chain.len()
            );
        }
        Err(e) => {
            println!("Failed to derive key: {}", e);
        }
    }

    // 2. Get a TDX quote
    let report_data = b"Hello, world!".to_vec();
    match client.tdx_quote(report_data).await {
        Ok(response) => {
            println!("TDX quote generated successfully!");
            println!("Quote length: {}", response.quote.len());
        }
        Err(e) => {
            println!("Failed to get TDX quote: {}", e);
        }
    }

    // 3. Get a TDX quote with specific hash algorithm
    let report_data = b"Hello, world!".to_vec();
    match client
        .tdx_quote_with_hash_algorithm(report_data, QuoteHashAlgorithm::Sha256)
        .await
    {
        Ok(response) => {
            println!("TDX quote with SHA256 generated successfully!");
            println!("Quote length: {}", response.quote.len());
        }
        Err(e) => {
            println!("Failed to get TDX quote with SHA256: {}", e);
        }
    }

    // 4. Get instance info
    match client.info().await {
        Ok(info) => {
            println!("Instance info retrieved successfully!");
            println!("App ID: {}", info.app_id);
            println!("Instance ID: {}", info.instance_id);
            println!("App Name: {}", info.app_name);
        }
        Err(e) => {
            println!("Failed to get instance info: {}", e);
        }
    }

    Ok(())
}
