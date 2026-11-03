use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "callmeju", version, about = "LLM voice-terminal thin client")]
pub struct Cli {
    /// Path to the TOML configuration file
    #[arg(short, long, default_value = "callmeju.toml")]
    pub config: PathBuf,

    /// Override the backend server base URL
    #[arg(long)]
    pub server: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub server: ServerConfig,
    pub audio: AudioConfig,
    pub vad: VadConfig,
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    /// Base URL of the speech backend, e.g. http://192.168.1.10:8000
    pub base_url: String,
    /// Endpoint that accepts raw PCM and returns text + TTS audio
    #[serde(default = "default_converse_path")]
    pub converse_path: String,
    /// Request timeout in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioConfig {
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    #[serde(default = "default_channels")]
    pub channels: u16,
    /// ALSA capture device, e.g. "hw:1,0" (pcmC1D0c)
    pub capture_device: String,
    /// ALSA playback device, e.g. "hw:1,0" (pcmC1D0p)
    pub playback_device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VadConfig {
    /// evdev device exposing the hardware VAD, e.g. /dev/input/event1
    #[serde(default = "default_vad_device")]
    pub device: String,
    /// Maximum utterance length in seconds
    #[serde(default = "default_max_utterance_secs")]
    pub max_utterance_secs: u64,
    /// End-of-speech silence threshold in seconds (TODO: wire to VAD)
    #[serde(default = "default_silence_secs")]
    pub silence_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionConfig {
    #[serde(default = "default_true")]
    pub beep: bool,
}

fn default_converse_path() -> String {
    "/v1/converse".to_string()
}
fn default_timeout_secs() -> u64 {
    30
}
fn default_sample_rate() -> u32 {
    16000
}
fn default_channels() -> u16 {
    1
}
fn default_vad_device() -> String {
    "/dev/input/event1".to_string()
}
fn default_max_utterance_secs() -> u64 {
    15
}
fn default_silence_secs() -> u64 {
    2
}
fn default_true() -> bool {
    true
}

impl Config {
    pub fn load(cli: &Cli) -> Result<Self> {
        let mut cfg: Config = if cli.config.exists() {
            let text = std::fs::read_to_string(&cli.config)
                .with_context(|| format!("failed to read config {}", cli.config.display()))?;
            toml::from_str(&text)
                .with_context(|| format!("failed to parse config {}", cli.config.display()))?
        } else {
            Config::default()
        };

        if let Some(server) = &cli.server {
            cfg.server.base_url = server.clone();
        }

        Ok(cfg)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                base_url: "http://127.0.0.1:8000".to_string(),
                converse_path: default_converse_path(),
                timeout_secs: default_timeout_secs(),
            },
            audio: AudioConfig {
                sample_rate: default_sample_rate(),
                channels: default_channels(),
                capture_device: "hw:1,0".to_string(),
                playback_device: "hw:1,0".to_string(),
            },
            vad: VadConfig {
                device: default_vad_device(),
                max_utterance_secs: default_max_utterance_secs(),
                silence_secs: default_silence_secs(),
            },
            session: SessionConfig {
                beep: default_true(),
            },
        }
    }
}
