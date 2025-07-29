# Dstack Crate

This crate provides rust clients for communicating with both the current dstack server and the legacy tappd service, which are available inside dstack.

## Installation

```toml
[dependencies]
dstack-rust = { git = "https://github.com/Dstack-TEE/dstack.git", package = "dstack-rust" }
```

## Basic Usage

### DstackClient (Current API)

```rust
use dstack_sdk::dstack_client::DstackClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = DstackClient::new(None); // Uses env var or default to Unix socket

    // Get system info
    let info = client.info().await?;
    println!("Instance ID: {}", info.instance_id);

    // Derive a key
    let key_resp = client.get_key(Some("my-app".to_string()), None).await?;
    println!("Key: {}", key_resp.key);
    println!("Signature Chain: {:?}", key_resp.signature_chain);

    // Generate TDX quote
    let quote_resp = client.get_quote(b"test-data".to_vec()).await?;
    println!("Quote: {}", quote_resp.quote);
    let rtmrs = quote_resp.replay_rtmrs()?;
    println!("Replayed RTMRs: {:?}", rtmrs);

    // Emit an event
    client.emit_event("BootComplete".to_string(), b"payload-data".to_vec()).await?;

    Ok(())
}
```

### TappdClient (Legacy API)

```rust
use dstack_sdk::tappd_client::{TappdClient, QuoteHashAlgorithm};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TappdClient::new(None); // Uses env var or default to Unix socket

    // Get system info
    let info = client.info().await?;
    println!("Instance ID: {}", info.instance_id);
    println!("App Name: {}", info.app_name);

    // Derive a key
    let key_resp = client.derive_key("my-app").await?;
    println!("Key: {}", key_resp.key);
    println!("Certificate Chain: {:?}", key_resp.certificate_chain);

    // Generate TDX quote with default SHA512
    let quote_resp = client.tdx_quote(b"test-data".to_vec()).await?;
    println!("Quote: {}", quote_resp.quote);
    let rtmrs = quote_resp.replay_rtmrs()?;
    println!("Replayed RTMRs: {:?}", rtmrs);

    // Generate TDX quote with specific hash algorithm
    let quote_resp = client.tdx_quote_with_hash_algorithm(
        b"test-data".to_vec(),
        QuoteHashAlgorithm::Sha256
    ).await?;

    // Generate raw quote (exactly 64 bytes)
    let raw_data = vec![0u8; 64];
    let raw_quote = client.raw_quote(raw_data).await?;

    Ok(())
}
```

## Features

### DstackClient Initialization

```rust
let client = DstackClient::new(Some("http://localhost:8000"));
```
- `endpoint`: Optional HTTP URL or Unix socket path (`/var/run/dstack.sock` by default)
- Will use the `DSTACK_SIMULATOR_ENDPOINT` environment variable if set

### TappdClient Initialization (Legacy API)

```rust
let client = TappdClient::new(Some("/var/run/tappd.sock"));
```
- `endpoint`: Optional HTTP URL or Unix socket path (`/var/run/tappd.sock` by default)
- Will use the `TAPPD_SIMULATOR_ENDPOINT` environment variable if set
- Supports the legacy tappd.sock API for backwards compatibility

## API Methods

### DstackClient Methods

#### `info(): InfoResponse`
Fetches metadata and measurements about the CVM instance.

#### `get_key(path: Option<String>, purpose: Option<String>) -> GetKeyResponse`
Derives a key for a specified path and optional purpose.
- `key`: Private key in hex format
- `signature_chain`: Vec of X.509 certificate chain entries

#### `get_quote(report_data: Vec<u8>) -> GetQuoteResponse`
Generates a TDX quote with a custom 64-byte payload.
- `quote`: Hex-encoded quote
- `event_log`: Serialized list of events
- `replay_rtmrs()`: Reconstructs RTMR values from the event log

#### `emit_event(event: String, payload: Vec<u8>)`
Sends an event log with associated binary payload to the runtime.

#### `get_tls_key(...) -> GetTlsKeyResponse`
Requests a key and X.509 certificate chain for RA-TLS or server/client authentication.

### TappdClient Methods (Legacy API)

#### `info(): TappdInfoResponse`
Fetches metadata and measurements about the CVM instance.

#### `derive_key(path: &str) -> DeriveKeyResponse`
Derives a key for a specified path.
- `key`: Private key (PEM or hex format)
- `certificate_chain`: Vec of X.509 certificate chain entries

#### `derive_key_with_subject(path: &str, subject: &str) -> DeriveKeyResponse`
Derives a key with a custom certificate subject.

#### `derive_key_with_subject_and_alt_names(path: &str, subject: Option<&str>, alt_names: Option<Vec<String>>) -> DeriveKeyResponse`
Derives a key with full certificate customization.

#### `tdx_quote(report_data: Vec<u8>) -> TdxQuoteResponse`
Generates a TDX quote using SHA512 hash algorithm.

#### `tdx_quote_with_hash_algorithm(report_data: Vec<u8>, algorithm: QuoteHashAlgorithm) -> TdxQuoteResponse`
Generates a TDX quote with a specific hash algorithm (SHA256, SHA384, SHA512, etc.).

#### `raw_quote(report_data: Vec<u8>) -> TdxQuoteResponse`
Generates a TDX quote with exactly 64 bytes of raw report data.

### Structures
- `GetKeyResponse`: Holds derived key and signature chain

- `GetQuoteResponse`: Contains the TDX quote and event log, with RTMR replay support

- `InfoResponse`: CVM instance metadata, including image and runtime measurements

## API Reference

### Running the Simulator

For local development without TDX devices, you can use the simulator under `sdk/simulator`.

Run the simulator with:

```bash
git clone https://github.com/Dstack-TEE/dstack.git
cd dstack/sdk/simulator
./build.sh
./dstack-simulator
```
Set the endpoint in your environment:

```
export DSTACK_SIMULATOR_ENDPOINT=/path/to/dstack-simulator/dstack.sock
```

## Examples

See the `examples/` directory for comprehensive usage examples:

- `examples/dstack_client_usage.rs` - Complete example using the current DstackClient API
- `examples/tappd_client_usage.rs` - Complete example using the legacy TappdClient API

Run examples with:
```bash
cargo run --example dstack_client_usage
cargo run --example tappd_client_usage
```

## License

Apache License
