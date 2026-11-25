pub mod catalog;
pub mod cloud_init;
pub mod defaults;
pub mod driver;
pub mod hypervisor;
pub mod petname;
pub mod ssh;
pub mod state;

use std::path::{Path, PathBuf};

/// Length in hex characters of a daemon auth token (32 random bytes).
pub const AUTH_TOKEN_HEX_LEN: usize = 64;

/// Stops this process's stdin/stdout/stderr handles from leaking into child processes.
///
/// On Windows, Rust spawns children with handle inheritance enabled, so a background child
/// (the auto-started daemon, QEMU) also inherits the parent's console or pipe handles even when
/// its own stdio is redirected. A caller like `ec2pass find | Out-Null` then never sees EOF and
/// hangs until the daemon exits. Clearing the inherit flag on our own std handles prevents
/// that; std handles explicitly given to a child are still duplicated for it. No-op elsewhere.
pub fn stop_std_handle_inheritance() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::{
            SetHandleInformation, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
        };
        use windows_sys::Win32::System::Console::{
            GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
        };
        for id in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            // SAFETY: GetStdHandle/SetHandleInformation only touch this process's handle table.
            unsafe {
                let h = GetStdHandle(id);
                if !h.is_null() && h != INVALID_HANDLE_VALUE {
                    SetHandleInformation(h, HANDLE_FLAG_INHERIT, 0);
                }
            }
        }
    }
}

/// Generates a cryptographically random hex auth token.
pub fn generate_auth_token() -> anyhow::Result<String> {
    use ring::rand::SecureRandom;
    let mut buf = [0u8; AUTH_TOKEN_HEX_LEN / 2];
    ring::rand::SystemRandom::new()
        .fill(&mut buf)
        .map_err(|_| anyhow::anyhow!("system RNG unavailable"))?;
    Ok(buf.iter().map(|b| format!("{:02x}", b)).collect())
}

/// Loads the daemon auth token from `path`, replacing it with a fresh random token when the
/// file is missing, empty, or holds a legacy (time+pid, guessable) token. New files are
/// created with owner-only permissions on Unix.
pub fn load_or_create_auth_token(path: &Path) -> anyhow::Result<String> {
    if let Ok(existing) = std::fs::read_to_string(path) {
        let t = existing.trim();
        if t.len() >= AUTH_TOKEN_HEX_LEN && t.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(t.to_string());
        }
    }
    let token = generate_auth_token()?;
    let _ = std::fs::remove_file(path);
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    use std::io::Write;
    opts.open(path)?.write_all(token.as_bytes())?;
    Ok(token)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token_random_and_persistent() {
        let dir = std::env::temp_dir().join(format!("ec2ps_token_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("auth_token");

        // Legacy time+pid token and empty files are replaced.
        std::fs::write(&path, "0018b2f0c1a2b3c400001234").unwrap();
        let t1 = load_or_create_auth_token(&path).unwrap();
        assert_eq!(t1.len(), AUTH_TOKEN_HEX_LEN);
        std::fs::write(&path, "").unwrap();
        let t2 = load_or_create_auth_token(&path).unwrap();
        assert_ne!(t1, t2);

        // A valid token is reused.
        assert_eq!(load_or_create_auth_token(&path).unwrap(), t2);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
