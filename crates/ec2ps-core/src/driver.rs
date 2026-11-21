//! Compatibility wrapper re-exporting hypervisor module symbols.
//!
//! All hypervisor drivers and prerequisite checking logic have been refactored
//! into `ec2ps_core::hypervisor`.

pub use crate::hypervisor::{
    check_hypervisor_prerequisites, create_default_driver, get_hypervisor_install_guide,
    HypervDriver, QemuDriver, VirtualMachineDriver,
};
