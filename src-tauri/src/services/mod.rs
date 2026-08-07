//! Services layer. Owns all I/O (Discord IPC, catalog HTTP, session engine,
//! spoofer); depends on `domain` and `infra`, never on the frontend.

pub mod catalog;
pub mod discord;
pub mod memory;
pub mod session;
pub mod spoofer;
