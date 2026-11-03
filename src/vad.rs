//! Hardware voice-activity detection (wake word) via the vendor's evdev device.
//!
//! The RK3308 hardware VAD is exposed as a standard evdev device
//! (e.g. `/dev/input/event1`); we simply wait for a key-press event.
//! On non-Linux platforms a stdin-based mock is used for development.

use anyhow::Result;

#[cfg(target_os = "linux")]
mod imp {
    use anyhow::{Context, Result};
    use evdev::{Device, EventType};

    pub struct VadListener {
        device: Device,
    }

    impl VadListener {
        pub fn open(path: &str) -> Result<Self> {
            let device = Device::open(path).with_context(|| format!("open VAD device `{path}`"))?;
            Ok(Self { device })
        }

        /// Block until a wake (key-press) event is detected.
        ///
        /// TODO: the vendor's exact key code is not yet known; match any key
        /// press for now and log the code so we can pin it down.
        pub fn wait_wake(&mut self) -> Result<()> {
            loop {
                for ev in self.device.fetch_events().context("fetch evdev events")? {
                    if ev.event_type() == EventType::KEY {
                        tracing::debug!(code = ev.code(), value = ev.value(), "vad key event");
                        if ev.value() != 0 {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use anyhow::Result;

    pub struct VadListener;

    impl VadListener {
        pub fn open(_path: &str) -> Result<Self> {
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
    pub fn open(path: &str) -> Result<Self> {
        Ok(Self(imp::VadListener::open(path)?))
    }

    pub fn wait_wake(&mut self) -> Result<()> {
        self.0.wait_wake()
    }
}
