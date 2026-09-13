//! Battery diagnostics via Windows' built-in `powercfg /batteryreport` or WMI.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BatteryInfo {
    pub design_capacity_mwh: Option<u64>,
    pub full_charge_capacity_mwh: Option<u64>,
    pub health_percent: Option<f64>,
    pub cycle_count: Option<u32>,
    pub manufacturer: Option<String>,
    pub chemistry: Option<String>,
}

impl BatteryInfo {
    pub fn subscore(&self) -> Option<f64> {
        self.health_percent.map(crate::scoring::battery_subscore)
    }
}

pub fn parse_battery_report_xml(xml: &str) -> BatteryInfo {
    let mut info = BatteryInfo {
        design_capacity_mwh: None,
        full_charge_capacity_mwh: None,
        health_percent: None,
        cycle_count: None,
        manufacturer: None,
        chemistry: None,
    };

    for tag in ["DesignCapacity", "FullChargeCapacity"] {
        if let Some(v) = first_u64(xml, &format!("<{tag}>"), &format!("</{tag}>")) {
            match tag {
                "DesignCapacity" => info.design_capacity_mwh = Some(v),
                _ => info.full_charge_capacity_mwh = Some(v),
            }
        }
    }

    if let Some(v) = first_u64(xml, "<CycleCount>", "</CycleCount>") {
        info.cycle_count = Some(u32::try_from(v).unwrap_or(0));
    }

    if let Some(start) = xml.find("<Battery>") {
        let block = &xml[start..];
        if let Some(m) = text_between(block, "<Manufacturer>", "</Manufacturer>") {
            let m = m.trim();
            if !m.is_empty() {
                info.manufacturer = Some(m.to_string());
            }
        }
        if let Some(c) = text_between(block, "<Chemistry>", "</Chemistry>") {
            let c = c.trim();
            if !c.is_empty() {
                info.chemistry = Some(c.to_string());
            }
        }
    }

    info.health_percent = match (info.full_charge_capacity_mwh, info.design_capacity_mwh) {
        (Some(fcc), Some(dc)) if dc > 0 => Some(((fcc as f64 / dc as f64) * 1000.0).round() / 10.0),
        _ => None,
    };

    info
}

fn first_u64(haystack: &str, open: &str, close: &str) -> Option<u64> {
    let raw = text_between(haystack, open, close)?;
    raw.trim().parse::<u64>().ok()
}

fn text_between<'a>(haystack: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = haystack.find(open)? + open.len();
    let end = haystack[start..].find(close)? + start;
    Some(&haystack[start..end])
}

#[cfg(windows)]
#[tauri::command]
pub async fn scan_battery() -> Result<BatteryInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        scan_battery_impl().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join failed: {e}"))?
}

#[cfg(not(windows))]
#[tauri::command]
pub async fn scan_battery() -> Result<BatteryInfo, String> {
    Err("Battery scan requires Windows (powercfg / WMI).".into())
}

#[cfg(windows)]
fn scan_battery_impl() -> Result<BatteryInfo, String> {
    use serde::Deserialize;
    use wmi::COMLibrary;

    let com = match COMLibrary::without_security() {
        Ok(c) => c,
        Err(_) => unsafe { COMLibrary::assume_initialized() },
    };

    let conn = wmi::WMIConnection::with_namespace_path("ROOT\\WMI", com)
        .map_err(|e| format!("WMI connection failed: {e}"))?;

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct BatteryStaticData {
        designed_capacity: Option<u32>,
        manufacturer: Option<String>,
        chemistry: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct BatteryFullChargedCapacity {
        full_charged_capacity: Option<u32>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct BatteryCycleCount {
        cycle_count: Option<u32>,
    }

    let statics: Vec<BatteryStaticData> = conn
        .raw_query("SELECT DesignedCapacity, Manufacturer, Chemistry FROM BatteryStaticData")
        .map_err(|e| format!("BatteryStaticData query failed: {e}"))?;

    let fulls: Vec<BatteryFullChargedCapacity> = conn
        .raw_query("SELECT FullChargedCapacity FROM BatteryFullChargedCapacity")
        .map_err(|e| format!("BatteryFullChargedCapacity query failed: {e}"))?;

    let cycles: Vec<BatteryCycleCount> = conn
        .raw_query("SELECT CycleCount FROM BatteryCycleCount")
        .map_err(|e| format!("BatteryCycleCount query failed: {e}"))?;

    let stat = statics
        .into_iter()
        .find(|s| s.designed_capacity.unwrap_or(0) > 0);
    let Some(stat) = stat else {
        return Err("No battery present (desktop or missing driver).".into());
    };

    let full = fulls
        .into_iter()
        .find(|f| f.full_charged_capacity.unwrap_or(0) > 0)
        .and_then(|f| f.full_charged_capacity);

    let design = stat.designed_capacity;
    let health_percent = match (full, design) {
        (Some(f), Some(d)) if d > 0 => Some((f as f64 / d as f64) * 100.0),
        _ => None,
    };

    Ok(BatteryInfo {
        design_capacity_mwh: design.map(|v| v as u64),
        full_charge_capacity_mwh: full.map(|v| v as u64),
        health_percent,
        cycle_count: cycles.into_iter().find_map(|c| c.cycle_count),
        manufacturer: stat.manufacturer.filter(|m| !m.trim().is_empty()),
        chemistry: stat.chemistry.filter(|c| !c.trim().is_empty()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_HEALTHY: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<BatteryReport>
  <Batteries>
    <Battery>
      <Id>SMP HP04XL 1234</Id>
      <Manufacturer>SMP</Manufacturer>
      <SerialNumber>12345</SerialNumber>
      <Chemistry>LION</Chemistry>
      <DesignCapacity>68472</DesignCapacity>
      <FullChargeCapacity>61240</FullChargeCapacity>
      <CycleCount>142</CycleCount>
    </Battery>
  </Batteries>
</BatteryReport>"#;

    #[test]
    fn healthy_probook_like_fixture_parses() {
        let b = parse_battery_report_xml(FIXTURE_HEALTHY);
        assert_eq!(b.design_capacity_mwh, Some(68472));
        assert_eq!(b.full_charge_capacity_mwh, Some(61240));
        assert_eq!(b.cycle_count, Some(142));
    }
}
