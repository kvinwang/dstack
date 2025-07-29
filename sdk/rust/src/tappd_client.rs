use anyhow::{bail, Context as _, Result};
use hex::{encode as hex_encode, FromHexError};
use http_client_unix_domain_socket::{ClientUnix, Method};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Digest;
use std::collections::HashMap;
use std::env;

use crate::dstack_client::{BaseClient, EventLog};

const INIT_MR: &str = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

/// Hash algorithms supported by the TDX quote generation
#[derive(Debug, Clone)]
pub enum QuoteHashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
    Sha3_384,
    Sha3_512,
    Keccak256,
    Keccak384,
    Keccak512,
    Raw,
}

impl QuoteHashAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha384 => "sha384",
            Self::Sha512 => "sha512",
            Self::Sha3_256 => "sha3-256",
            Self::Sha3_384 => "sha3-384",
            Self::Sha3_512 => "sha3-512",
            Self::Keccak256 => "keccak256",
            Self::Keccak384 => "keccak384",
            Self::Keccak512 => "keccak512",
            Self::Raw => "raw",
        }
    }
}

fn replay_rtmr(history: Vec<String>) -> Result<String, FromHexError> {
    if history.is_empty() {
        return Ok(INIT_MR.to_string());
    }
    let mut mr = hex::decode(INIT_MR)?;
    for content in history {
        let mut content_bytes = hex::decode(content)?;
        if content_bytes.len() < 48 {
            content_bytes.resize(48, 0);
        }
        mr.extend_from_slice(&content_bytes);
        mr = sha2::Sha384::digest(&mr).to_vec();
    }
    Ok(hex_encode(mr))
}

fn get_tappd_endpoint(endpoint: Option<&str>) -> String {
    if let Some(e) = endpoint {
        return e.to_string();
    }
    if let Ok(sim_endpoint) = env::var("TAPPD_SIMULATOR_ENDPOINT") {
        return sim_endpoint;
    }
    "/var/run/tappd.sock".to_string()
}

#[derive(Debug)]
pub enum TappdClientKind {
    Http,
    Unix,
}

/// Response from a key derivation request
#[derive(Serialize, Deserialize)]
pub struct DeriveKeyResponse {
    /// The derived key (PEM format for certificates, hex for raw keys)
    pub key: String,
    /// The certificate chain
    pub certificate_chain: Vec<String>,
}

impl DeriveKeyResponse {
    /// Decodes the key from PEM format and extracts the raw ECDSA P-256 private key bytes
    pub fn decode_key(&self) -> Result<Vec<u8>, anyhow::Error> {
        use x509_parser::der_parser::der::parse_der;
        use x509_parser::pem::parse_x509_pem;

        let key_content = self.key.trim();

        let (_, pem) = parse_x509_pem(key_content.as_bytes()).context("Failed to parse PEM")?;
        // Parse PKCS#8 PrivateKeyInfo structure
        // PKCS#8 format: SEQUENCE { version, algorithm, privateKey }
        let (_, der_seq) = parse_der(&pem.contents).context("Failed to parse DER")?;
        let sequence = der_seq.as_sequence().context("Expected SEQUENCE")?;
        if sequence.len() < 3 {
            bail!("Invalid PKCS#8 structure: expected at least 3 elements");
        }

        // The privateKey is the 3rd element (index 2) and should be an OCTET STRING
        let private_key_data = sequence[2]
            .content
            .as_slice()
            .context("Could not extract privateKey data")?;

        // For ECDSA keys, the private key is wrapped in another DER structure
        // Parse the inner ECDSA private key structure
        let (_, inner_der) = parse_der(private_key_data).context("Failed to parse inner DER")?;

        let inner_sequence = inner_der.as_sequence().context("Expected inner SEQUENCE")?;

        if inner_sequence.len() < 2 {
            return Err(anyhow::anyhow!("Invalid ECDSA private key structure"));
        }

        // The actual private key value is the 2nd element (index 1) as OCTET STRING
        let key_bytes = inner_sequence[1]
            .content
            .as_slice()
            .context("Could not extract key bytes")?;

        if key_bytes.len() != 32 {
            bail!(
                "Expected 32-byte ECDSA P-256 private key, got {} bytes",
                key_bytes.len()
            );
        }

        Ok(key_bytes.to_vec())
    }
}

/// Response from a TDX quote request
#[derive(Serialize, Deserialize)]
pub struct TdxQuoteResponse {
    /// The TDX quote in hexadecimal format
    pub quote: String,
    /// The event log associated with the quote
    pub event_log: String,
    /// The hash algorithm used (if returned by server)
    #[serde(default)]
    pub hash_algorithm: Option<String>,
    /// The prefix used (if returned by server)
    #[serde(default)]
    pub prefix: Option<String>,
}

impl TdxQuoteResponse {
    pub fn decode_quote(&self) -> Result<Vec<u8>, FromHexError> {
        hex::decode(&self.quote)
    }

    pub fn decode_event_log(&self) -> Result<Vec<EventLog>, serde_json::Error> {
        serde_json::from_str(&self.event_log)
    }

    /// Replays RTMR history to calculate final RTMR values
    pub fn replay_rtmrs(&self) -> Result<HashMap<u8, String>> {
        let parsed_event_log: Vec<EventLog> = self.decode_event_log()?;
        let mut rtmrs = HashMap::new();
        for idx in 0..4 {
            let mut history = vec![];
            for event in &parsed_event_log {
                if event.imr == idx {
                    history.push(event.digest.clone());
                }
            }
            rtmrs.insert(idx as u8, replay_rtmr(history)?);
        }
        Ok(rtmrs)
    }
}

/// TCB (Trusted Computing Base) information
#[derive(Serialize, Deserialize)]
pub struct TappdTcbInfo {
    /// The measurement root of trust
    pub mrtd: String,
    /// The value of RTMR0 (Runtime Measurement Register 0)
    pub rtmr0: String,
    /// The value of RTMR1 (Runtime Measurement Register 1)
    pub rtmr1: String,
    /// The value of RTMR2 (Runtime Measurement Register 2)
    pub rtmr2: String,
    /// The value of RTMR3 (Runtime Measurement Register 3)
    pub rtmr3: String,
    /// The event log entries
    pub event_log: Vec<EventLog>,
    /// The application compose file
    pub app_compose: String,
}

/// Response from a Tappd info request
#[derive(Serialize, Deserialize)]
pub struct TappdInfoResponse {
    /// The application identifier
    pub app_id: String,
    /// The instance identifier
    pub instance_id: String,
    /// The application certificate
    pub app_cert: String,
    /// Trusted Computing Base information
    pub tcb_info: TappdTcbInfo,
    /// The name of the application
    pub app_name: String,
}

/// The main client for interacting with the legacy Tappd service
pub struct TappdClient {
    /// The base URL for HTTP requests
    base_url: String,
    /// The endpoint for Unix domain socket communication
    endpoint: String,
    /// The type of client (HTTP or Unix domain socket)
    client: TappdClientKind,
}

impl BaseClient for TappdClient {}

impl TappdClient {
    pub fn new(endpoint: Option<&str>) -> Self {
        let endpoint = get_tappd_endpoint(endpoint);
        let (base_url, client) = match endpoint {
            ref e if e.starts_with("http://") || e.starts_with("https://") => {
                (e.to_string(), TappdClientKind::Http)
            }
            _ => ("http://localhost".to_string(), TappdClientKind::Unix),
        };

        TappdClient {
            base_url,
            endpoint,
            client,
        }
    }

    async fn send_rpc_request<S: Serialize, D: DeserializeOwned>(
        &self,
        path: &str,
        payload: &S,
    ) -> anyhow::Result<D> {
        match &self.client {
            TappdClientKind::Http => {
                let client = Client::new();
                let url = format!(
                    "{}/{}",
                    self.base_url.trim_end_matches('/'),
                    path.trim_start_matches('/')
                );
                let res = client
                    .post(&url)
                    .json(payload)
                    .header("Content-Type", "application/json")
                    .send()
                    .await?
                    .error_for_status()?;
                Ok(res.json().await?)
            }
            TappdClientKind::Unix => {
                let mut unix_client = ClientUnix::try_new(&self.endpoint).await?;
                let res = unix_client
                    .send_request_json::<_, _, Value>(
                        path,
                        Method::POST,
                        &[("Content-Type", "application/json")],
                        Some(&payload),
                    )
                    .await?;
                Ok(res.1)
            }
        }
    }

    /// Derives a key from the Tappd service using the path as both path and subject
    pub async fn derive_key(&self, path: &str) -> Result<DeriveKeyResponse> {
        self.derive_key_with_subject_and_alt_names(path, Some(path), None)
            .await
    }

    /// Derives a key from the Tappd service with a specific subject
    pub async fn derive_key_with_subject(
        &self,
        path: &str,
        subject: &str,
    ) -> Result<DeriveKeyResponse> {
        self.derive_key_with_subject_and_alt_names(path, Some(subject), None)
            .await
    }

    /// Derives a key from the Tappd service with full configuration
    pub async fn derive_key_with_subject_and_alt_names(
        &self,
        path: &str,
        subject: Option<&str>,
        alt_names: Option<Vec<String>>,
    ) -> Result<DeriveKeyResponse> {
        let subject = subject.unwrap_or(path);

        let mut payload = json!({
            "path": path,
            "subject": subject,
        });

        if let Some(alt_names) = alt_names {
            if !alt_names.is_empty() {
                payload["alt_names"] = json!(alt_names);
            }
        }

        let response = self
            .send_rpc_request("/prpc/Tappd.DeriveKey", &payload)
            .await?;
        Ok(response)
    }

    /// Sends a raw quote request with 64 bytes of report data
    pub async fn get_quote(&self, report_data: Vec<u8>) -> Result<TdxQuoteResponse> {
        if report_data.len() != 64 {
            bail!("Report data must be exactly 64 bytes for raw quote");
        }

        let payload = json!({
            "report_data": hex_encode(report_data),
        });

        let response = self
            .send_rpc_request("/prpc/Tappd.RawQuote", &payload)
            .await?;
        Ok(response)
    }

    /// Retrieves information about the Tappd instance
    pub async fn info(&self) -> Result<TappdInfoResponse> {
        #[derive(Deserialize)]
        struct RawInfoResponse {
            app_id: String,
            instance_id: String,
            app_cert: String,
            tcb_info: String,
            app_name: String,
        }

        let raw_response: RawInfoResponse = self
            .send_rpc_request("/prpc/Tappd.Info", &json!({}))
            .await?;

        let tcb_info: TappdTcbInfo = serde_json::from_str(&raw_response.tcb_info)?;

        Ok(TappdInfoResponse {
            app_id: raw_response.app_id,
            instance_id: raw_response.instance_id,
            app_cert: raw_response.app_cert,
            tcb_info,
            app_name: raw_response.app_name,
        })
    }
}
