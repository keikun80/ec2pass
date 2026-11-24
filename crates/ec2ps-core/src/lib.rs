pub mod catalog;
pub mod cloud_init;
pub mod defaults;
pub mod driver;
pub mod hypervisor;
pub mod petname;
pub mod ssh;
pub mod state;

use std::path::PathBuf;

/// Returns the data directory for `ec2passd`.
/// Can be overridden via `EC2PASS_HOME` environment variable.
pub fn default_data_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("EC2PASS_HOME").or_else(|_| std::env::var("EC2PS_HOME")) {
        return PathBuf::from(custom);
    }
    if let Some(proj) = dirs::data_local_dir() {
        let p = proj.join("ec2pass");
        if std::fs::create_dir_all(&p).is_ok() {
            return p;
        }
    }
    PathBuf::from(".ec2pass-data")
}
