pub mod artifact;
pub mod config;
pub mod dlp;
pub mod pricing;
pub mod session;
pub mod status;
pub mod think;
pub mod tracking;

/// Crate-wide lock serializing every test that mutates process-global
/// `HOME`/`USERPROFILE`. Tests run in parallel threads within one process, and
/// those env vars drive global config path resolution — without a *single*
/// shared lock, `config` and `pricing` tests race and intermittently resolve
/// each other's temp dirs (flaky Windows-only `assert!(path.exists())`).
#[cfg(test)]
pub(crate) static ENV_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
