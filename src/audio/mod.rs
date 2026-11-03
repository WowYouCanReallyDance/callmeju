//! Audio capture and playback abstraction.
//!
//! On Linux (the target device) audio goes through ALSA, giving direct
//! control over the RK3308 PCM devices (`pcmC1D0c` / `pcmC1D0p`). On other
//! platforms a mock backend keeps the crate buildable for development.

#[cfg(target_os = "linux")]
mod alsa;
#[cfg(not(target_os = "linux"))]
mod mock;

/// Canonical sample type used across the pipeline: 16-bit little-endian PCM.
pub type Sample = i16;

/// A capture stream producing interleaved PCM samples.
pub trait Recorder: Send {
    /// Read up to `buf.len()` samples; returns the number of samples read.
    fn read_samples(&mut self, buf: &mut [Sample]) -> anyhow::Result<usize>;
}

/// A playback stream consuming interleaved PCM samples.
pub trait Player: Send {
    fn write_samples(&mut self, buf: &[Sample]) -> anyhow::Result<()>;
}

/// Open a capture stream for the given ALSA device name.
pub fn open_recorder(device: &str, sample_rate: u32, channels: u16) -> anyhow::Result<Box<dyn Recorder>> {
    #[cfg(target_os = "linux")]
    {
        alsa::AlsaRecorder::open(device, sample_rate, channels)
            .map(|r| Box::new(r) as Box<dyn Recorder>)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (device, sample_rate, channels);
        Ok(Box::new(mock::MockRecorder))
    }
}

/// Open a playback stream for the given ALSA device name.
pub fn open_player(device: &str, sample_rate: u32, channels: u16) -> anyhow::Result<Box<dyn Player>> {
    #[cfg(target_os = "linux")]
    {
        alsa::AlsaPlayer::open(device, sample_rate, channels)
            .map(|p| Box::new(p) as Box<dyn Player>)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (device, sample_rate, channels);
        Ok(Box::new(mock::MockPlayer))
    }
}
