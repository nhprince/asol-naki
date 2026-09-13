//! Storage diagnostics via bundled `smartctl` (smartmontools, GPL) or WMI fallback.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct StorageInfo {
    pub model_name: Option<String>,
    pub serial: Option<String>,
    pub protocol: Option<String>,
    pub total_capacity_bytes: Option<u64>,
    pub sector_size: Option<u64>,
    pub smart_status: Option<String>,
    pub nvme_percentage_used: Option<f64>,
    pub realloc_sector_count: Option<u64>,
    pub pending_sector_count: Option<u64>,
    pub media_errors: Option<u64>,
    pub power_on_hours: Option<u64>,
    pub temperature_c: Option<f64>,
}

impl StorageInfo {
    pub fn subscore(&self) -> f64 {
        let mut score: f64 = match self.smart_status.as_deref() {
            Some(s) if s.eq_ignore_ascii_case("failed") => 0.0,
            _ => 10.0,
        };

        if let Some(pu) = self.nvme_percentage_used {
            score = score.min((100.0 - pu.clamp(0.0, 100.0)) / 10.0);
        }
        if let Some(r) = self.realloc_sector_count {
            if r > 0 {
                score = score.min(6.0 - (r.min(200) as f64 / 50.0));
            }
        }
        if let Some(p) = self.pending_sector_count {
            if p > 0 {
                score = score.min(5.0);
            }
        }
        if let Some(m) = self.media_errors {
            if m > 0 {
                score = score.min(4.0);
            }
        }

        (score.clamp(0.0, 10.0) * 10.0).round() / 10.0
    }
}

pub fn parse_smartctl_json(json: &str) -> StorageInfo {
    let mut info = StorageInfo {
        model_name: None,
        serial: None,
        protocol: None,
        total_capacity_bytes: None,
        sector_size: None,
        smart_status: None,
        nvme_percentage_used: None,
        realloc_sector_count: None,
        pending_sector_count: None,
        media_errors: None,
        power_on_hours: None,
        temperature_c: None,
    };

    let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else {
        return info;
    };

    info.model_name = str_field(&v, &["model_name"]).or_else(|| str_field(&v, &["model_family"]));
    info.serial = str_field(&v, &["serial_number"]);
    info.protocol = str_field(&v, &["device_protocol"]).map(|s| s.to_lowercase());

    info.total_capacity_bytes = v
        .pointer("/user_capacity/bytes")
        .and_then(|x| x.as_u64())
        .or_else(|| v.pointer("/nvme_total_capacity").and_then(|x| x.as_u64()));

    info.sector_size = v
        .pointer("/logical_block_size")
        .and_then(|x| x.as_u64())
        .or_else(|| v.pointer("/sector_sizes/logical").and_then(|x| x.as_u64()));

    if v.pointer("/smart_status/passed") == Some(&serde_json::Value::Bool(true)) {
        info.smart_status = Some("passed".into());
    } else if v.pointer("/smart_status/passed").is_some() {
        info.smart_status = Some("failed".into());
    } else if let Ok(pct) = v
        .pointer("/smart_health_status")
        .and_then(|x| x.as_i64())
        .ok_or(())
    {
        info.smart_status = Some(pct.to_string());
    }

    if let Some(nvme) = v.get("nvme_smart_health_information_log") {
        info.nvme_percentage_used = num_f64(nvme, "percentage_used");
        info.media_errors = nvme.get("media_errors").and_then(|x| x.as_u64());
        info.power_on_hours = nvme.get("power_on_hours").and_then(|x| x.as_u64());
        info.temperature_c = num_f64(nvme, "temperature");
    }

    if let Some(attrs) = v
        .pointer("/ata_smart_attributes/table")
        .and_then(|t| t.as_array())
    {
        for attr in attrs {
            let name = attr.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let raw = raw_value(attr);
            match name {
                "Reallocated_Sector_Ct" | "Reallocated_Event_Count" => {
                    if info.realloc_sector_count.is_none() {
                        info.realloc_sector_count = raw;
                    }
                }
                "Current_Pending_Sector" => info.pending_sector_count = raw,
                "Power_On_Hours" => {
                    if info.power_on_hours.is_none() {
                        info.power_on_hours = raw;
                    }
                }
                "Temperature_Celsius" if info.temperature_c.is_none() => {
                    info.temperature_c = raw.map(|r| r as f64);
                }
                _ => {}
            }
        }
    }

    info
}

fn str_field(v: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut cur = v;
    for seg in path {
        cur = cur.get(seg)?;
    }
    cur.as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn num_f64(v: &serde_json::Value, key: &str) -> Option<f64> {
    v.get(key)?.as_f64()
}

fn raw_value(attr: &serde_json::Value) -> Option<u64> {
    attr.pointer("/raw/value")
        .and_then(|x| x.as_u64())
        .or_else(|| {
            attr.pointer("/raw/string")
                .and_then(|x| x.as_str())
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse().ok())
        })
}

#[cfg(windows)]
#[tauri::command]
pub async fn scan_storage() -> Result<Vec<StorageInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        // Try smartctl first
        if let Ok(json) = run_smartctl_all() {
            let parsed: Vec<StorageInfo> = split_json_documents(&json)
                .iter()
                .filter(|doc| doc.contains("\"device\""))
                .map(|d| parse_smartctl_json(d))
                .collect();

            if !parsed.is_empty() {
                return Ok(parsed);
            }
        }

        // Fall back to Windows WMI Win32_DiskDrive
        scan_storage_wmi_fallback()
    })
    .await
    .map_err(|e| format!("Task join failed: {e}"))?
}

#[cfg(not(windows))]
#[tauri::command]
pub async fn scan_storage() -> Result<Vec<StorageInfo>, String> {
    Err("Storage scan requires Windows + bundled smartctl or WMI.".into())
}

#[cfg(windows)]
fn scan_storage_wmi_fallback() -> Result<Vec<StorageInfo>, String> {
    use serde::Deserialize;
    use wmi::COMLibrary;

    let com = match COMLibrary::without_security() {
        Ok(c) => c,
        Err(_) => unsafe { COMLibrary::assume_initialized() },
    };

    let conn = wmi::WMIConnection::new(com)
        .map_err(|e| format!("WMI connection failed: {e}"))?;

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Win32DiskDrive {
        model: Option<String>,
        serial_number: Option<String>,
        size: Option<u64>,
        status: Option<String>,
        interface_type: Option<String>,
    }

    let drives: Vec<Win32DiskDrive> = conn
        .raw_query("SELECT Model, SerialNumber, Size, Status, InterfaceType FROM Win32_DiskDrive")
        .map_err(|e| format!("Win32_DiskDrive query failed: {e}"))?;

    let list = drives
        .into_iter()
        .map(|d| StorageInfo {
            model_name: d.model.filter(|s| !s.trim().is_empty()),
            serial: d.serial_number.filter(|s| !s.trim().is_empty()),
            protocol: d.interface_type.map(|s| s.to_lowercase()),
            total_capacity_bytes: d.size,
            sector_size: Some(512),
            smart_status: d.status.map(|s| s.to_lowercase()),
            nvme_percentage_used: None,
            realloc_sector_count: None,
            pending_sector_count: None,
            media_errors: None,
            power_on_hours: None,
            temperature_c: None,
        })
        .collect();

    Ok(list)
}

#[cfg_attr(not(windows), allow(dead_code))]
fn split_json_documents(text: &str) -> Vec<String> {
    let mut docs = Vec::new();
    let mut depth = 0i32;
    let mut start: Option<usize> = None;
    for (idx, ch) in text.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    start = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        let doc = &text[s..=idx];
                        let looks_like_drive =
                            doc.contains("\"model_name\"") || doc.contains("\"user_capacity\"");
                        if looks_like_drive {
                            docs.push(doc.to_string());
                        }
                    }
                    start = None;
                }
            }
            _ => {}
        }
    }
    if docs.is_empty() {
        docs.push(text.to_string());
    }
    docs
}

#[cfg(windows)]
fn run_smartctl_all() -> std::io::Result<String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let exe_dir = std::env::current_exe().unwrap_or_default();
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(p) = exe_dir.parent() {
        candidates.push(p.join("resources/smartctl/smartctl.exe"));
        candidates.push(p.join("smartctl/smartctl.exe"));
        candidates.push(p.join("smartctl.exe"));
        if let Some(pp) = p.parent() {
            candidates.push(pp.join("resources/smartctl/smartctl.exe"));
            candidates.push(pp.join("smartctl/smartctl.exe"));
        }
    }

    // System choco / smartmontools installation paths
    candidates.push(std::path::PathBuf::from(r"C:\Program Files\smartmontools\bin\smartctl.exe"));
    candidates.push(std::path::PathBuf::from(r"C:\ProgramData\chocolatey\bin\smartctl.exe"));
    candidates.push(std::path::PathBuf::from("smartctl.exe"));

    let cmd_opt = candidates.iter().find(|c| c.exists()).cloned();
    let Some(cmd) = cmd_opt else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "smartctl component missing from app bundle.",
        ));
    };

    let mut devices: Vec<String> = Vec::new();
    let out = Command::new(&cmd)
        .arg("--scan")
        .arg("--json")
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if let Ok(out) = out {
        let scan_text = String::from_utf8_lossy(&out.stdout).to_string();
        for l in scan_text.lines() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(l) {
                if let Some(name) = v
                    .get("device")
                    .and_then(|d| d.get("name"))
                    .and_then(|n| n.as_str())
                {
                    devices.push(name.to_string());
                }
            }
        }
    }

    for i in 0..16 {
        devices.push(format!(r"\\.\PhysicalDrive{i}"));
    }
    devices.dedup();

    // Pass 1: unelevated probe
    let mut docs = String::new();
    for dev in &devices {
        let out = Command::new(&cmd)
            .arg("--json")
            .arg("--all")
            .arg(dev)
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        if let Ok(out) = out {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.contains("\"device\"") {
                docs.push_str(&text);
                docs.push('\n');
            }
        }
    }
    if !docs.is_empty() {
        return Ok(docs);
    }

    // Pass 2: elevated PowerShell execution
    let temp_dir = std::env::temp_dir();
    let out_file = temp_dir.join("asol-naki-smartctl.json");
    let _ = std::fs::remove_file(&out_file);

    let drive_chain = devices
        .iter()
        .map(|d| format!(r#"& '{cmd_path}' --json --all '{d}'"#, cmd_path = cmd.display()))
        .collect::<Vec<_>>()
        .join(" ; ");

    let ps_command = format!(
        r#"$ErrorActionPreference = 'SilentlyContinue'; {drive_chain} | Out-File -FilePath '{out_path}' -Encoding utf8"#,
        out_path = out_file.display()
    );

    match shell_execute_runas("powershell.exe", &format!("-NoProfile -WindowStyle Hidden -Command \"{ps_command}\"")) {
        ShellRun::Ok => {
            for _ in 0..60 {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if let Ok(m) = std::fs::metadata(&out_file) {
                    if m.len() > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(400));
                        break;
                    }
                }
            }
            let body = std::fs::read_to_string(&out_file)?;
            Ok(body)
        }
        ShellRun::Cancelled => Ok(String::new()),
        ShellRun::Failed(msg) => Err(std::io::Error::other(msg)),
    }
}

#[cfg(windows)]
#[derive(Debug)]
enum ShellRun {
    Ok,
    Cancelled,
    Failed(String),
}

#[cfg(windows)]
fn shell_execute_runas(file: &str, params: &str) -> ShellRun {
    use std::os::windows::ffi::OsStrExt;

    fn to_wide(s: &str) -> Vec<u16> {
        std::ffi::OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let verb_w = to_wide("runas");
    let file_w = to_wide(file);
    let params_w = to_wide(params);

    let hinst = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            0,
            verb_w.as_ptr(),
            file_w.as_ptr(),
            params_w.as_ptr(),
            std::ptr::null(),
            0, // SW_HIDE
        )
    };

    let code = hinst as usize;
    match code {
        n if n > 32 => ShellRun::Ok,
        5 => ShellRun::Cancelled,
        n => ShellRun::Failed(format!("ShellExecuteW failed ({n})")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_NVME: &str = r#"{
      "device": {"name": "/dev/nvme0", "type": "nvme", "protocol": "NVMe"},
      "device_protocol": "NVMe",
      "model_name": "Samsung SSD 980 PRO 1TB",
      "serial_number": "S5GXNX0R123456X",
      "user_capacity": {"blocks": 1953525168, "bytes": 1000204886016},
      "logical_block_size": 512,
      "smart_status": {"passed": true, "nvme": {"value": 0}},
      "nvme_smart_health_information_log": {
        "critical_warning": 0,
        "temperature": 41,
        "available_spare": 100,
        "percentage_used": 3,
        "media_errors": 0,
        "power_on_hours": 2871
      }
    }"#;

    #[test]
    fn nvme_fixture_parses_completely() {
        let s = parse_smartctl_json(FIXTURE_NVME);
        assert_eq!(s.model_name.as_deref(), Some("Samsung SSD 980 PRO 1TB"));
    }
}
