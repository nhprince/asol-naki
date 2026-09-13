//! Hardware identification.

use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_bytes: Option<u64>,
    pub driver_version: Option<String>,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct FullHardwareInfo {
    pub cpu_name: String,
    pub cpu_threads: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_cores_physical: Option<u32>,

    pub total_memory_mb: u64,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub motherboard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bios_vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bios_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpus: Option<Vec<GpuInfo>>,
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

#[cfg(windows)]
fn apply_windows_wmi(info: &mut FullHardwareInfo) {
    let _ = try_apply_windows_wmi(info);
}

#[cfg(windows)]
fn try_apply_windows_wmi(info: &mut FullHardwareInfo) -> Result<(), wmi::WMIError> {
    use serde::Deserialize;
    use wmi::COMLibrary;

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

    let com = match COMLibrary::without_security() {
        Ok(c) => c,
        Err(_) => unsafe { COMLibrary::assume_initialized() },
    };

    let conn = wmi::WMIConnection::new(com)?;

    info.cpu_cores_physical = conn
        .raw_query::<Win32Processor>("SELECT NumberOfCores FROM Win32_Processor")
        .ok()
        .and_then(|rows| rows.into_iter().next())
        .and_then(|p| p.number_of_cores);

    if let Ok(Some(bb)) = conn
        .raw_query::<Win32BaseBoard>("SELECT Manufacturer, Product FROM Win32_BaseBoard")
        .map(|rows| rows.into_iter().next())
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
        .map(|rows| rows.into_iter().next())
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

#[tauri::command]
pub async fn scan_hardware_basic() -> Result<BasicHardwareInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut sys = System::new();
        Ok(collect_basic_info(&mut sys))
    })
    .await
    .map_err(|e| format!("Task join failed: {e}"))?
}

#[tauri::command]
pub async fn scan_hardware_full() -> Result<FullHardwareInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut sys = System::new();
        Ok(collect_full_info(&mut sys))
    })
    .await
    .map_err(|e| format!("Task join failed: {e}"))?
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
    }
}
