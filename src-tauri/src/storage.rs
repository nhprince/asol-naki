//! Storage diagnostics via bundled `smartctl` (smartmontools, GPL).
//!
//! `smartctl --json` output varies by drive type (NVMe vs ATA/SATA) and
//! firmware (risk R8) — the parser is tolerant, every real-world oddity we
//! meet becomes a fixture + regression test. The Windows-only shell-out is
//! isolated from parsing so all logic is tested cross-platform.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StorageInfo {
    pub model_name: Option<String>,
    pub serial: Option<String>,
    /// "nvme" or "ata" — drives report different SMART shapes.
    pub protocol: Option<String>,
    /// Total user capacity in bytes (ground truth for fake-capacity checks).
    pub total_capacity_bytes: Option<u64>,
    pub sector_size: Option<u64>,
    /// Overall SMART health: "passed" / "failed" / raw percent.
    pub smart_status: Option<String>,
    /// NVMe: percentage_used (0 = new, >100 = past rated endurance).
    pub nvme_percentage_used: Option<f64>,
    /// ATA: Reallocated_Sector_Ct raw value.
    pub realloc_sector_count: Option<u64>,
    /// ATA: Current_Pending_Sector raw value.
    pub pending_sector_count: Option<u64>,
    /// NVMe media errors, ATA equivalent best-effort.
    pub media_errors: Option<u64>,
    pub power_on_hours: Option<u64>,
    /// Temperature in Celsius if reported.
    pub temperature_c: Option<f64>,
}

impl StorageInfo {
    /// 0–10 sub-score. Healthy pass = 10; deductions for wear signals.
    /// Calibration expected to change after ground-truth testing.
    pub fn subscore(&self) -> f64 {
        let mut score: f64 = match self.smart_status.as_deref() {
            Some(s) if s.eq_ignore_ascii_case("failed") => 0.0,
            _ => 10.0,
        };

        if let Some(pu) = self.nvme_percentage_used {
            // 10 at 0% used → 0 at 100%+ used.
            score = score.min((100.0 - pu.clamp(0.0, 100.0)) / 10.0);
        }
        if let Some(r) = self.realloc_sector_count {
            if r > 0 {
                // Any reallocated sectors is damage; scale down hard.
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

/// Parse `smartctl --json --all` output for one drive.
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

    // Capacity appears under user_capacity.bytes (both protocols).
    info.total_capacity_bytes = v
        .pointer("/user_capacity/bytes")
        .and_then(|x| x.as_u64())
        .or_else(|| v.pointer("/nvme_total_capacity").and_then(|x| x.as_u64()));

    info.sector_size = v
        .pointer("/logical_block_size")
        .and_then(|x| x.as_u64())
        .or_else(|| v.pointer("/sector_sizes/logical").and_then(|x| x.as_u64()));

    // SMART overall-health: ATA has smart_status.passed; NVMe too (v7+).
    if v.pointer("/smart_status/passed") == Some(&serde_json::Value::Bool(true)) {
        info.smart_status = Some("passed".into());
    } else if v.pointer("/smart_status/passed").is_some() {
        info.smart_status = Some("failed".into());
    } else if let Ok(pct) = v
        .pointer("/smart_health_status")
        .and_then(|x| x.as_i64())
        .ok_or(())
    {
        // Legacy field: value 197-style percent or 0/1; keep raw string.
        info.smart_status = Some(pct.to_string());
    }

    // NVMe namespace: smart_health_information.percent_used etc.
    if let Some(nvme) = v.get("nvme_smart_health_information_log") {
        info.nvme_percentage_used = num_f64(nvme, "percent_used");
        info.media_errors = nvme.get("media_errors").and_then(|x| x.as_u64());
        info.power_on_hours = nvme.get("power_on_hours").and_then(|x| x.as_u64());
        info.temperature_c = num_f64(nvme, "temperature");
    }

    // ATA attributes table: id.name keyed entries.
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
                "Temperature_Celsius" => {
                    if info.temperature_c.is_none() {
                        info.temperature_c = raw.map(|r| r as f64);
                    }
                }
                _ => {}
            }
        }
    }

    info
}

fn str_field<'a>(v: &'a serde_json::Value, path: &[&str]) -> Option<String> {
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

/// ATA attribute raw values arrive as {"raw": {"value": N, "string": "..."}}.
fn raw_value(attr: &serde_json::Value) -> Option<u64> {
    attr.pointer("/raw/value")
        .and_then(|x| x.as_u64())
        .or_else(|| {
            // Fall back to parsing the string form ("123" or "123 (Min/Max ...)").
            attr.pointer("/raw/string")
                .and_then(|x| x.as_str())
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse().ok())
        })
}

#[cfg(windows)]
#[tauri::command]
pub fn scan_storage() -> Result<Vec<StorageInfo>, String> {
    let json = run_smartctl_all().map_err(|e| e.to_string())?;
    // smartctl --scan-json lists devices; --all on no arg scans everything
    // but may interleave multiple JSON docs. We split defensively.
    Ok(split_json_documents(&json)
        .iter()
        .filter(|doc| doc.contains("\"device\""))
        .map(|d| parse_smartctl_json(d))
        .collect())
}

#[cfg(not(windows))]
#[tauri::command]
pub fn scan_storage() -> Result<Vec<StorageInfo>, String> {
    Err("Storage scan requires Windows + bundled smartctl.".into())
}

/// Split concatenated smartctl JSON documents (one per drive).
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
                        // Only keep documents that look like drive reports.
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

/// Run bundled smartctl across all drives. Windows only; binary location is
/// resolved relative to the exe (resources/smartctl/smartctl.exe), with a
/// PATH fallback for development machines that have it installed.
#[cfg(windows)]
fn run_smartctl_all() -> std::io::Result<String> {
    use std::process::Command;

    let exe_dir = std::env::current_exe()?;
    let bundled = exe_dir
        .parent()
        .map(|p| p.join("resources/smartctl/smartctl.exe"))
        .unwrap_or_else(|| std::path::PathBuf::from("smartctl.exe"));

    let cmd = if bundled.exists() {
        bundled
    } else {
        std::path::PathBuf::from("smartctl")
    };

    let out = Command::new(cmd)
        .arg("--scan")
        .arg("--json")
        .arg("--all")
        .output()?;

    if !out.status.success() && out.stdout.is_empty() {
        return Err(std::io::Error::other(format!(
            "smartctl failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
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

    const FIXTURE_ATA_FAILING: &str = r#"{
      "device": {"name": "/dev/sda", "type": "sat", "protocol": "ATA"},
      "model_name": "WDC WD10JPVX-22JC3T0",
      "serial_number": "WX21A23N5432",
      "user_capacity": {"blocks": 1953525168, "bytes": 1000204886016},
      "sector_sizes": {"logical": 512, "physical": 4096},
      "smart_status": {"passed": false},
      "ata_smart_attributes": {"table": [
        {"id": 5, "name": "Reallocated_Sector_Ct", "raw": {"value": 96, "string": "96"}},
        {"id": 197, "name": "Current_Pending_Sector", "raw": {"value": 8, "string": "8"}},
        {"id": 9, "name": "Power_On_Hours", "raw": {"value": 30145, "string": "30145"}},
        {"id": 194, "name": "Temperature_Celsius", "raw": {"value": 33, "string": "33"}}
      ]}
    }"#;

    #[test]
    fn nvme_fixture_parses_completely() {
        let s = parse_smartctl_json(FIXTURE_NVME);
        assert_eq!(s.model_name.as_deref(), Some("Samsung SSD 980 PRO 1TB"));
        assert_eq!(s.protocol.as_deref(), Some("nvme"));
        assert_eq!(s.total_capacity_bytes, Some(1000204886016));
        assert_eq!(s.sector_size, Some(512));
        assert_eq!(s.smart_status.as_deref(), Some("passed"));
        assert_eq!(s.nvme_percentage_used, Some(3.0));
        assert_eq!(s.media_errors, Some(0));
        assert_eq!(s.power_on_hours, Some(2871));
        assert_eq!(s.temperature_c, Some(41.0));
        assert!(s.subscore() >= 9.0, "fresh NVMe should score high");
    }

    #[test]
    fn failing_ata_drive_scores_zero_and_parses_attrs() {
        let s = parse_smartctl_json(FIXTURE_ATA_FAILING);
        assert_eq!(s.smart_status.as_deref(), Some("failed"));
        assert_eq!(s.realloc_sector_count, Some(96));
        assert_eq!(s.pending_sector_count, Some(8));
        assert_eq!(s.power_on_hours, Some(30145));
        assert_eq!(s.subscore(), 0.0); // failed status dominates
    }

    #[test]
    fn realloc_sectors_reduce_score_but_not_below_floor() {
        let xml = FIXTURE_NVME.replace("\"percentage_used\": 3", "\"percent_used\": 3");
        let _ = xml;
        let mut s = parse_smartctl_json(FIXTURE_ATA_FAILING);
        s.smart_status = Some("passed".into()); // keep drive alive, isolate wear math
        s.pending_sector_count = Some(0);
        s.realloc_sector_count = Some(50);
        let score = s.subscore();
        assert!(
            score < 6.0 && score > 0.0,
            "realloc=50 should hurt: {score}"
        );
    }

    #[test]
    fn garbage_json_returns_empty_info_without_panic() {
        let s = parse_smartctl_json("definitely not json {{{");
        assert_eq!(s.model_name, None);
        assert_eq!(s.total_capacity_bytes, None);
        assert_eq!(s.subscore(), 10.0); // no negative signals → default healthy
    }

    #[test]
    fn split_handles_concatenated_documents() {
        let two = format!("{FIXTURE_NVME}\n{FIXTURE_ATA_FAILING}");
        let docs = split_json_documents(&two);
        assert_eq!(docs.len(), 2);
    }
}
