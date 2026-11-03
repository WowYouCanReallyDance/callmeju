//! Wake detection via audio energy (voice-activity detection).
//!
//! The RK3308 has a hardware VAD sound card (card0) used for low-power
//! wake-word detection, but that path is occupied by the vendor's `aiVoiceD1`
//! and relies on a proprietary `keyword.irf` model. Instead we run a simple
//! software VAD on the main PCM device: keep reading PCM and treat a sustained
//! RMS energy spike as a wake.
//!
//! On non-Linux platforms a stdin mock is used for development.

use anyhow::Result;

#[cfg(target_os = "linux")]
mod imp {
    use anyhow::{Context, Result};

    use crate::audio::{self, Sample};

    pub struct VadListener {
        rec: Box<dyn audio::Recorder>,
        buf: Vec<Sample>,
        threshold: f32,
        trigger_frames: u32,
    }

    impl VadListener {
        pub fn open(device: &str, sample_rate: u32, channels: u16, threshold: f32) -> Result<Self> {
            let rec = audio::open_recorder(device, sample_rate, channels)
                .context("open VAD capture device")?;
            // 100 ms buffer at the configured rate/channels.
            let frame = sample_rate as usize / 10 * channels as usize;
            Ok(Self {
                rec,
                buf: vec![0; frame],
                threshold,
                trigger_frames: 3,
            })
        }

        /// Block until sustained loud audio (a wake) is detected.
        pub fn wait_wake(&mut self) -> Result<()> {
            let mut streak = 0u32;
            loop {
                let n = self.rec.read_samples(&mut self.buf)?;
                if n == 0 {
                    continue;
                }
                let rms = rms(&self.buf[..n]);
                tracing::debug!(rms, streak, "vad energy");
                if rms > self.threshold {
                    streak += 1;
                    if streak >= self.trigger_frames {
                        return Ok(());
                    }
                } else {
                    streak = 0;
                }
            }
        }
    }

    fn rms(samples: &[Sample]) -> f32 {
        let sum: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
        (sum / samples.len() as f64).sqrt() as f32
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use anyhow::Result;

    pub struct VadListener;

    impl VadListener {
        pub fn open(
            _device: &str,
            _sample_rate: u32,
            _channels: u16,
            _threshold: f32,
        ) -> Result<Self> {
            Ok(Self)
        }

        pub fn wait_wake(&mut self) -> Result<()> {
            tracing::info!("[mock vad] press Enter to simulate a wake word");
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            Ok(())
        }
    }
}

pub struct VadListener(imp::VadListener);

impl VadListener {
    pub fn open(device: &str, sample_rate: u32, channels: u16, threshold: f32) -> Result<Self> {
        Ok(Self(imp::VadListener::open(
            device,
            sample_rate,
            channels,
            threshold,
        )?))
    }

    pub fn wait_wake(&mut self) -> Result<()> {
        self.0.wait_wake()
    }
}
