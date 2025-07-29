use dstack_sdk::dstack_client::{DstackClient, TlsKeyConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a DstackClient with default endpoint (/var/run/dstack.sock)
    let client = DstackClient::new(None);

    // Or create with a custom endpoint
    // let client = DstackClient::new(Some("/custom/path/dstack.sock"));

    // Or create with HTTP endpoint for simulator
    // let client = DstackClient::new(Some("http://localhost:8000"));

    println!("DstackClient created successfully!");

    // Example usage (these will fail without a running dstack service):

    // 1. Get system info
    match client.info().await {
        Ok(info) => {
            println!("System info retrieved successfully!");
            println!("  App ID: {}", info.app_id);
            println!("  Instance ID: {}", info.instance_id);
            println!("  App Name: {}", info.app_name);
            println!("  Device ID: {}", info.device_id);
            println!("  Compose Hash: {}", info.compose_hash);
            println!("  TCB Info - MRTD: {}", info.tcb_info.mrtd);
            println!("  TCB Info - RTMR0: {}", info.tcb_info.rtmr0);
        }
        Err(e) => {
            println!("Failed to get system info: {}", e);
        }
    }

    // 2. Derive a key
    match client
        .get_key(Some("my-app".to_string()), Some("encryption".to_string()))
        .await
    {
        Ok(response) => {
            println!("Key derived successfully!");
            println!("  Key length: {}", response.key.len());
            println!(
                "  Signature chain length: {}",
                response.signature_chain.len()
            );

            // Decode the key
            match response.decode_key() {
                Ok(key_bytes) => {
                    println!("  Decoded key bytes length: {}", key_bytes.len());
                }
                Err(e) => {
                    println!("  Key decode error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to derive key: {}", e);
        }
    }

    // 3. Generate TDX quote
    let report_data = b"Hello, dstack world!".to_vec();
    match client.get_quote(report_data).await {
        Ok(response) => {
            println!("TDX quote generated successfully!");
            println!("  Quote length: {}", response.quote.len());
            println!("  Event log length: {}", response.event_log.len());

            // Decode the quote
            match response.decode_quote() {
                Ok(quote_bytes) => {
                    println!("  Decoded quote bytes length: {}", quote_bytes.len());
                }
                Err(e) => {
                    println!("  Quote decode error: {}", e);
                }
            }

            // Replay RTMRs from event log
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
        Err(e) => {
            println!("Failed to get TDX quote: {}", e);
        }
    }

    // 4. Emit an event
    let event_payload = b"Application started successfully".to_vec();
    match client
        .emit_event("AppStart".to_string(), event_payload)
        .await
    {
        Ok(()) => {
            println!("Event emitted successfully!");
        }
        Err(e) => {
            println!("Failed to emit event: {}", e);
        }
    }

    // 5. Get TLS key for server authentication
    let tls_config = TlsKeyConfig::builder()
        .subject("my-app.example.com")
        .alt_names(vec![
            "www.my-app.com".to_string(),
            "api.my-app.com".to_string(),
        ])
        .usage_server_auth(true)
        .usage_client_auth(false)
        .usage_ra_tls(true)
        .build();

    match client.get_tls_key(tls_config).await {
        Ok(response) => {
            println!("TLS key generated successfully!");
            println!("  Key length: {}", response.key.len());
            println!(
                "  Certificate chain length: {}",
                response.certificate_chain.len()
            );
        }
        Err(e) => {
            println!("Failed to get TLS key: {}", e);
        }
    }

    // 6. Get a simple key without purpose
    match client.get_key(Some("simple-key".to_string()), None).await {
        Ok(response) => {
            println!("Simple key derived successfully!");
            println!("  Key: {}", response.key);
        }
        Err(e) => {
            println!("Failed to derive simple key: {}", e);
        }
    }

    // 7. Generate quote with minimal report data
    let minimal_data = vec![0x01, 0x02, 0x03, 0x04];
    match client.get_quote(minimal_data).await {
        Ok(response) => {
            println!("Minimal quote generated successfully!");

            // Parse and display event log
            match response.decode_event_log() {
                Ok(events) => {
                    println!("  Event log contains {} events", events.len());
                    for (i, event) in events.iter().enumerate().take(3) {
                        // Show first 3 events
                        println!(
                            "    Event {}: IMR={}, Type={}, Event='{}'",
                            i, event.imr, event.event_type, event.event
                        );
                    }
                }
                Err(e) => {
                    println!("  Failed to parse event log: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to get minimal quote: {}", e);
        }
    }

    Ok(())
}
