//! Windows input adapter. No Mac source, packaging, permissions or updater code.
//! The portable state machine is tested on the development host; `native` still
//! requires Windows interactive acceptance. API delivery is never a document ACK.
pub mod config;
pub mod delivery;
pub mod session;
pub mod stream;

#[cfg(windows)]
pub mod native;

pub use funput_core::{InputMethod, ToneStyle};
pub use funput_engine::{EngineConfig, KeySource};
