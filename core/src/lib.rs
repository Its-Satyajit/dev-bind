pub mod cert;
pub mod config;
pub mod detect;
pub mod dns;
pub mod hosts;
pub mod proxy;
pub mod runner;
pub mod setup;
pub mod trust;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
