//! Client for the speech backend that runs ASR -> LLM -> TTS.
//!
//! Wire contract (v1): `POST {base_url}{converse_path}` with raw little-endian
//! 16-bit PCM in the body; the backend replies with JSON:
//!
//! ```json
//! {
//!   "text": "the assistant reply",
//!   "audio_base64": "<base64 of 16-bit PCM>",
//!   "continue_conversation": true
//! }
//! ```

use anyhow::{Context, Result};
use base64::Engine;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Reply {
    /// Assistant reply text (for logging / display).
    pub text: String,
    /// Synthesized reply audio, base64-encoded 16-bit PCM.
    #[serde(default)]
    pub audio_base64: String,
    /// Whether to keep the conversation going (multi-turn).
    #[serde(default = "default_true")]
    pub continue_conversation: bool,
}

fn default_true() -> bool {
    true
}

impl Reply {
    /// Decode the reply audio into raw PCM bytes.
    pub fn audio_pcm(&self) -> Result<Vec<u8>> {
        base64::engine::general_purpose::STANDARD
            .decode(&self.audio_base64)
            .context("decode audio base64")
    }
}

#[derive(Clone)]
pub struct BackendClient {
    http: reqwest::Client,
    converse_url: String,
}

impl BackendClient {
    pub fn new(base_url: &str, converse_path: &str, timeout_secs: u64) -> Result<Self> {
        let converse_url = format!("{}{}", base_url.trim_end_matches('/'), converse_path);
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .context("build HTTP client")?;
        Ok(Self { http, converse_url })
    }

    /// Send captured PCM audio and receive the text + TTS audio reply.
    pub async fn converse(&self, pcm: &[u8]) -> Result<Reply> {
        let resp = self
            .http
            .post(&self.converse_url)
            .header("Content-Type", "application/octet-stream")
            .body(pcm.to_vec())
            .send()
            .await
            .with_context(|| format!("POST {}", self.converse_url))?;

        let status = resp.status();
        let body = resp.bytes().await.context("read response body")?;
        if !status.is_success() {
            anyhow::bail!(
                "backend returned {status}: {}",
                String::from_utf8_lossy(&body)
            );
        }

        serde_json::from_slice(&body).context("parse backend reply")
    }
}
