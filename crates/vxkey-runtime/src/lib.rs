//! VXKey policy boundary around the unmodified upstream orthography/engine.
//! Keyboard callbacks use immutable config, never disk, clipboard, UI or IPC.
pub mod config;
pub mod engine;
pub mod history;
pub mod input_policy;
pub mod period;
pub mod recovery;
pub mod service;
pub mod shortcut;
pub mod storage;
