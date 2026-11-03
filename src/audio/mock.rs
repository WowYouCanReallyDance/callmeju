//! Mock audio backends used for off-device development and tests.

use super::{Player, Recorder, Sample};

/// Recorder that yields silence.
pub struct MockRecorder;

impl Recorder for MockRecorder {
    fn read_samples(&mut self, buf: &mut [Sample]) -> anyhow::Result<usize> {
        for s in buf.iter_mut() {
            *s = 0;
        }
        Ok(buf.len())
    }
}

/// Player that discards samples.
pub struct MockPlayer;

impl Player for MockPlayer {
    fn write_samples(&mut self, buf: &[Sample]) -> anyhow::Result<()> {
        tracing::debug!(samples = buf.len(), "mock player discarded audio");
        Ok(())
    }
}
