//! callmeju — LLM voice-terminal thin client.
//!
//! Runs on the RK3308 smart speaker (the "thin" side): it listens for the
//! hardware wake event, captures audio, hands it to a backend that runs
//! ASR -> LLM -> TTS, and plays the returned speech. All heavy lifting lives
//! on a LAN host or cloud server.

pub mod audio;
pub mod config;
pub mod net;
pub mod session;
pub mod vad;
