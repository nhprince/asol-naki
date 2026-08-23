//! Basic hardware identification (Phase 0 proof-of-concept).
//!
//! Phase 1 will expand this into the full `hardware.rs` module
//! (GPU, motherboard/BIOS via WMI, per-core details) — see ROADMAP.md.

use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Serialize)]
pub struct BasicHardwareInfo {
    pub cpu_name: String,
    pub cpu_threads: usize,
    pub total_memory_mb: u64,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
}

/// Returns a flat summary of CPU / RAM / OS identity for the results screen.
///
/// Pure function over `System` so it stays unit-testable; the Tauri command
/// below is a thin shell over it.
pub fn collect_basic_info(sys: &mut System) -> BasicHardwareInfo {
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpu = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let (os_name, os_version, kernel) = {
        let name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let version = System::os_version().unwrap_or_default();
        let kernel = System::kernel_version().unwrap_or_default();
        (name, version, kernel)
    };

    BasicHardwareInfo {
        cpu_name: cpu,
        // NOTE(sysinfo 0.37): physical_core_count() no longer exists here.
        // Authoritative physical-core data comes in Phase 1 via Windows WMI
        // (Win32_Processor.NumberOfCores) inside the full hardware module.
        cpu_threads: sys.cpus().len(),
        total_memory_mb: sys.total_memory() / (1024 * 1024),
        os_name,
        os_version,
        kernel_version: kernel,
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
    }
}

#[tauri::command]
pub fn scan_hardware_basic() -> Result<BasicHardwareInfo, String> {
    let mut sys = System::new();
    Ok(collect_basic_info(&mut sys))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_info_is_sane() {
        let mut sys = System::new();
        let info = collect_basic_info(&mut sys);
        assert!(!info.cpu_name.is_empty());
        // sysinfo reports at least one logical CPU everywhere.
        assert!(info.cpu_threads >= 1);
        assert!(!info.os_name.is_empty());
    }
}
