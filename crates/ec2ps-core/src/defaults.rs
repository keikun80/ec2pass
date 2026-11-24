//! Centralized defaults, limits, and timing constants shared across ec2pass crates.
//!
//! Values here were previously scattered as inline literals. Keep behavior-affecting
//! numbers in this module so they can be audited and tuned in one place.

use std::time::Duration;

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

/// Host loopback address used for SSH port-forwards and QMP sockets.
pub const LOOPBACK: &str = "127.0.0.1";

/// Guest IPv4 address assigned by QEMU user-mode (SLIRP) networking.
pub const SLIRP_GUEST_IP: &str = "10.0.2.15";

/// Fallback SSH login user when an image does not declare one (Amazon Linux convention).
pub const DEFAULT_SSH_USER: &str = "ec2-user";

/// First host port handed out for guest SSH forwarding (overridable via `local.ssh-port-base`).
pub const DEFAULT_SSH_PORT_BASE: u16 = 10022;

/// First host port handed out for QMP control sockets (overridable via `local.qmp-port-base`).
pub const DEFAULT_QMP_PORT_BASE: u16 = 14444;

/// Maximum number of candidate ports probed when allocating a free port.
pub const PORT_ALLOC_MAX_PROBES: u16 = 1000;

/// Minimum guest memory. QEMU args clamp to this, so settings must reject anything smaller.
pub const MIN_MEMORY_BYTES: u64 = 256 * MIB;

/// Minimum VHDX/disk size created by the Windows (Hyper-V) backend.
pub const MIN_DISK_BYTES_WINDOWS: u64 = 2 * GIB;

/// Maximum attempts to generate a unique random instance name.
pub const PETNAME_MAX_ATTEMPTS: usize = 100;

/// Timing constants (timeouts, poll intervals, retry counts).
pub mod timeouts {
    use super::Duration;

    /// HTTP HEAD/GET timeout for image catalog probes.
    pub const HTTP_PROBE: Duration = Duration::from_secs(5);
    /// HTTP timeout for fetching the AL2023 release-notes RSS feed.
    pub const HTTP_RSS: Duration = Duration::from_secs(8);

    /// TCP connect timeout when probing a guest SSH port.
    pub const SSH_TCP_PROBE: Duration = Duration::from_millis(1500);
    /// Interval between readiness polls while waiting for a guest to boot.
    pub const SSH_READY_POLL: Duration = Duration::from_secs(2);
    /// Delay between SSH retries on transient connection errors.
    pub const SSH_RETRY_DELAY: Duration = Duration::from_secs(1);
    /// Maximum SSH attempts on transient connection errors.
    pub const SSH_MAX_ATTEMPTS: u32 = 6;

    /// Delay between QMP connect attempts / shutdown polls.
    pub const QMP_POLL: Duration = Duration::from_millis(150);
    /// Maximum QMP connect attempts.
    pub const QMP_CONNECT_ATTEMPTS: u32 = 10;
    /// Grace period for a guest to power down before forcing `quit`.
    pub const QMP_SHUTDOWN_WAIT: Duration = Duration::from_secs(5);
    /// Extra settle time after QEMU exits so OS file locks are released.
    pub const QEMU_LOCK_RELEASE: Duration = Duration::from_millis(250);
    /// Settle time after a forced QMP `quit`.
    pub const QEMU_FORCE_QUIT_SETTLE: Duration = Duration::from_millis(300);

    /// Delay between `qemu-img snapshot` retries (disk may still be locked).
    pub const DISK_SNAPSHOT_RETRY_DELAY: Duration = Duration::from_millis(500);
    /// Maximum `qemu-img snapshot` attempts.
    pub const DISK_SNAPSHOT_MAX_ATTEMPTS: u32 = 6;

    /// Timeout for the lightweight guest metrics probe (`/proc/loadavg`, meminfo).
    pub const GUEST_METRICS_EXEC: Duration = Duration::from_millis(1500);

    /// CLI polling interval for streamed progress / daemon state.
    pub const CLI_POLL: Duration = Duration::from_millis(100);
}
