//! Conversation orchestration: wake -> capture -> converse -> play.

use std::sync::Arc;

use anyhow::Result;

use crate::audio::{self, Sample};
use crate::config::Config;
use crate::net::BackendClient;
use crate::vad::VadListener;

pub struct Session {
    config: Arc<Config>,
    client: BackendClient,
}

impl Session {
    pub fn new(config: Config) -> Result<Self> {
        let client = BackendClient::new(
            &config.server.base_url,
            &config.server.converse_path,
            config.server.timeout_secs,
        )?;
        Ok(Self {
            config: Arc::new(config),
            client,
        })
    }

    pub async fn run(self) -> Result<()> {
        loop {
            self.wait_wake().await?;

            let pcm = self.capture_utterance().await?;
            if pcm.is_empty() {
                tracing::warn!("captured empty utterance, skipping");
                continue;
            }

            let reply = self.client.converse(&pcm).await?;
            tracing::info!(reply = %reply.text, "assistant");

            self.play(&reply.audio_pcm()?).await?;

            // Cooldown after playback so the speaker output isn't picked up
            // as a new wake (avoids an echo loop).
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;

            if !reply.continue_conversation {
                break;
            }
        }
        Ok(())
    }

    async fn wait_wake(&self) -> Result<()> {
        let cfg = self.config.clone();
        tokio::task::spawn_blocking(move || {
            let mut vad = VadListener::open(
                &cfg.audio.capture_device,
                cfg.audio.sample_rate,
                cfg.audio.channels,
                cfg.vad.energy_threshold,
            )?;
            vad.wait_wake()
        })
        .await??;
        tracing::info!("wake detected");
        Ok(())
    }

    async fn capture_utterance(&self) -> Result<Vec<u8>> {
        let cfg = self.config.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
            let mut rec = audio::open_recorder(
                &cfg.audio.capture_device,
                cfg.audio.sample_rate,
                cfg.audio.channels,
            )?;

            // 100ms buffer at the configured rate/channels.
            let frame = cfg.audio.sample_rate as usize / 10 * cfg.audio.channels as usize;
            let mut buf = vec![0i16; frame];
            let max_samples =
                cfg.audio.sample_rate as usize * cfg.vad.max_utterance_secs as usize * cfg.audio.channels as usize;

            let mut samples: Vec<Sample> = Vec::with_capacity(max_samples);
            while samples.len() < max_samples {
                let n = rec.read_samples(&mut buf)?;
                if n == 0 {
                    break;
                }
                samples.extend_from_slice(&buf[..n]);
                // TODO: stop early on end-of-speech via the hardware VAD
                // instead of always filling to max_utterance_secs.
            }

            // i16 samples -> little-endian bytes for the wire.
            let mut pcm = Vec::with_capacity(samples.len() * 2);
            for s in samples {
                pcm.extend_from_slice(&s.to_le_bytes());
            }
            Ok(pcm)
        })
        .await?
    }

    async fn play(&self, pcm: &[u8]) -> Result<()> {
        if pcm.is_empty() {
            return Ok(());
        }
        let cfg = self.config.clone();
        let pcm = pcm.to_vec();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut player = audio::open_player(
                &cfg.audio.playback_device,
                cfg.audio.sample_rate,
                cfg.audio.channels,
            )?;

            let samples: Vec<Sample> = pcm
                .chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0], c[1]]))
                .collect();
            player.write_samples(&samples)?;
            Ok(())
        })
        .await?
    }
}
