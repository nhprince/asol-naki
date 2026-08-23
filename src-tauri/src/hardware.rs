//! Hardware identification.
//!
//! Phase 0: basic CPU/RAM/OS via sysinfo (cross-platform).
//! Phase 1: full pull adds GPU + motherboard/BIOS. On Windows these come
//! from WMI (authoritative); on other platforms we return None so the UI
//! can show "unavailable on this OS" honestly instead of fake data.

use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct GpuInfo {
    pub name: String,
    /// VRAM bytes if the driver reports it (WMI AdapterRAM is u32-capped at
    /// 4 GB; treated as best-effort).
    pub vram_bytes: Option<u64>,
    pub driver_version: Option<String>,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct FullHardwareInfo {
    // --- CPU ---
    pub cpu_name: String,
    pub cpu_threads: usize,
    /// Physical cores via WMI on Windows; None elsewhere (sysinfo dropped it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_cores_physical: Option<u32>,

    // --- Memory / OS ---
    pub total_memory_mb: u64,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,

    // --- Identity (Windows/WMI only) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motherboard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bios_vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bios_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpus: Option<Vec<GpuInfo>>,
}

/// Phase-0 basic summary (kept for compatibility with the POC UI flow).
pub fn collect_basic_info(sys: &mut System) -> BasicHardwareInfo {
    let full = collect_full_info(sys);
    BasicHardwareInfo {
        cpu_name: full.cpu_name,
        cpu_threads: full.cpu_threads,
        total_memory_mb: full.total_memory_mb,
        os_name: full.os_name,
        os_version: full.os_version,
        kernel_version: full.kernel_version,
        hostname: full.hostname,
    }
}

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

/// Full hardware identity pull. Pure over `System` + a platform backend.
pub fn collect_full_info(sys: &mut System) -> FullHardwareInfo {
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpu_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_default();
    let kernel_version = System::kernel_version().unwrap_or_default();
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());

    // `mut` is only exercised by the Windows WMI backend; allowed here so
    // the non-Windows CI job (where identity stays honestly None) is clean.
    #[allow(unused_mut)]
    let mut info = FullHardwareInfo {
        cpu_name,
        cpu_threads: sys.cpus().len(),
        cpu_cores_physical: None,
        total_memory_mb: sys.total_memory() / (1024 * 1024),
        os_name,
        os_version,
        kernel_version,
        hostname,
        motherboard: None,
        bios_vendor: None,
        bios_version: None,
        gpus: None,
    };

    #[cfg(windows)]
    apply_windows_wmi(&mut info);

    info
}

// ---------------------------------------------------------------------------
// Windows WMI backend
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn apply_windows_wmi(info: &mut FullHardwareInfo) {
    // COM init failures must never break the whole scan; fields stay None —
    // honest absence beats fabricated data.
    let _ = try_apply_windows_wmi(info);
}

#[cfg(windows)]
fn try_apply_windows_wmi(info: &mut FullHardwareInfo) -> Result<(), wmi::WMIError> {
    use serde::Deserialize;
    use wmi::COMLib;

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Win32Processor {
        number_of_cores: Option<u32>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Win32BaseBoard {
        manufacturer: Option<String>,
        product: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Win32Bios {
        manufacturer: Option<String>,
        smbiosbiosversion: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Win32VideoController {
        name: Option<String>,
        adapter_ram: Option<u32>,
        driver_version: Option<String>,
    }

    let com = COMLib::create_instance()?;
    let conn = wmi::WMIConnection::new(com)?;

    info.cpu_cores_physical = conn
        .raw_query::<Win32Processor>("SELECT NumberOfCores FROM Win32_Processor")
        .ok()
        .and_then(|rows| rows.into_iter().next())
        .and_then(|p| p.number_of_cores);

    if let Ok(Some(bb)) = conn
        .raw_query::<Win32BaseBoard>("SELECT Manufacturer, Product FROM Win32_BaseBoard")
        .map(|mut v| v.next())
        .transpose()
    {
        let m = bb.manufacturer.unwrap_or_default().trim().to_string();
        let p = bb.product.unwrap_or_default().trim().to_string();
        let combined = format!("{m} {p}").trim().to_string();
        if !combined.is_empty() {
            info.motherboard = Some(combined);
        }
    }

    if let Ok(Some(bios)) = conn
        .raw_query::<Win32Bios>("SELECT Manufacturer, SMBIOSBIOSVersion FROM Win32_BIOS")
        .map(|mut v| v.next())
        .transpose()
    {
        info.bios_vendor = bios
            .manufacturer
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        info.bios_version = bios
            .smbiosbiosversion
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
    }

    if let Ok(gpus) = conn.raw_query::<Win32VideoController>(
        "SELECT Name, AdapterRAM, DriverVersion FROM Win32_VideoController",
    ) {
        let list: Vec<GpuInfo> = gpus
            .into_iter()
            .filter_map(|g| {
                let name = g.name?.trim().to_string();
                if name.is_empty() || name.to_lowercase().contains("basic display") {
                    return None;
                }
                Some(GpuInfo {
                    name,
                    vram_bytes: g.adapter_ram.map(|r| r as u64),
                    driver_version: g.driver_version,
                })
            })
            .collect();
        if !list.is_empty() {
            info.gpus = Some(list);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn scan_hardware_basic() -> Result<BasicHardwareInfo, String> {
    let mut sys = System::new();
    Ok(collect_basic_info(&mut sys))
}

#[tauri::command]
pub fn scan_hardware_full() -> Result<FullHardwareInfo, String> {
    let mut sys = System::new();
    Ok(collect_full_info(&mut sys))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_info_is_sane() {
        let mut sys = System::new();
        let info = collect_basic_info(&mut sys);
        assert!(!info.cpu_name.is_empty());
        assert!(info.cpu_threads >= 1);
        assert!(!info.os_name.is_empty());
    }

    #[test]
    fn full_info_sane_and_graceful_off_windows() {
        let mut sys = System::new();
        let info = collect_full_info(&mut sys);
        assert!(!info.cpu_name.is_empty());
        assert!(info.cpu_threads >= 1);
        assert!(info.total_memory_mb > 0);
        // Off-Windows these must stay None (never fabricated). On the CI's
        // Linux runners that invariant holds; on Windows they may populate.
        #[cfg(not(windows))]
        {
            assert!(info.motherboard.is_none());
            assert!(info.gpus.is_none());
            assert!(info.cpu_cores_physical.is_none());
        }
    }
}
