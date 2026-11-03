//! ALSA backend (Linux only). Requires `libasound` at build time.
//!
//! Note: this module is only compiled for `cfg(target_os = "linux")`, so it is
//! verified when cross-compiling for the device, not on macOS.

use anyhow::{Context, Result};
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};

use super::{Player, Recorder, Sample};

pub struct AlsaRecorder {
    pcm: PCM,
    channels: usize,
}

pub struct AlsaPlayer {
    pcm: PCM,
    channels: usize,
}

fn open(device: &str, dir: Direction, sample_rate: u32, channels: u16) -> Result<PCM> {
    let pcm =
        PCM::new(device, dir, false).with_context(|| format!("open ALSA PCM `{device}`"))?;
    {
        let hwp = HwParams::any(&pcm).context("query hw params")?;
        hwp.set_access(Access::RWInterleaved).context("set access")?;
        hwp.set_format(Format::s16()).context("set format")?;
        hwp.set_rate(sample_rate, ValueOr::Nearest).context("set rate")?;
        hwp.set_channels(channels as u32).context("set channels")?;
        pcm.hw_params(&hwp).context("apply hw params")?;
    }
    pcm.prepare().context("prepare pcm")?;
    Ok(pcm)
}

impl AlsaRecorder {
    pub fn open(device: &str, sample_rate: u32, channels: u16) -> Result<Self> {
        Ok(Self {
            pcm: open(device, Direction::Capture, sample_rate, channels)?,
            channels: channels as usize,
        })
    }
}

impl Recorder for AlsaRecorder {
    fn read_samples(&mut self, buf: &mut [Sample]) -> Result<usize> {
        let io = self.pcm.io_i16().context("open i16 io")?;
        let frames = buf.len() / self.channels;
        let n = io
            .readi(&mut buf[..frames * self.channels])
            .context("read frames")?;
        Ok(n * self.channels)
    }
}

impl AlsaPlayer {
    pub fn open(device: &str, sample_rate: u32, channels: u16) -> Result<Self> {
        Ok(Self {
            pcm: open(device, Direction::Playback, sample_rate, channels)?,
            channels: channels as usize,
        })
    }
}

impl Player for AlsaPlayer {
    fn write_samples(&mut self, buf: &[Sample]) -> Result<()> {
        let io = self.pcm.io_i16().context("open i16 io")?;
        let mut written = 0;
        while written < buf.len() {
            let n = io.writei(&buf[written..]).context("write frames")?;
            if n == 0 {
                break;
            }
            written += n * self.channels;
        }
        Ok(())
    }
}
