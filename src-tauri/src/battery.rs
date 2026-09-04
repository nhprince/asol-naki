//! Battery diagnostics via Windows' built-in `powercfg /batteryreport`.
//!
//! `powercfg` emits an XML report; we extract design capacity, full-charge
//! capacity, and cycle count, then compute health. Parsing is split from
//! process invocation so ALL logic is testable cross-platform on fixture XML.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BatteryInfo {
    pub design_capacity_mwh: Option<u64>,
    pub full_charge_capacity_mwh: Option<u64>,
    /// full_charge ÷ design × 100; None when either capacity is unknown.
    pub health_percent: Option<f64>,
    pub cycle_count: Option<u32>,
    /// First battery's manufacturer string if present.
    pub manufacturer: Option<String>,
    /// Chemistry string (e.g. "LION") if present.
    pub chemistry: Option<String>,
}

impl BatteryInfo {
    /// Sub-score 0–10 via the shared scoring curve (None → None: no data,
    /// never fabricate a score).
    pub fn subscore(&self) -> Option<f64> {
        self.health_percent.map(crate::scoring::battery_subscore)
    }
}

/// Extract battery facts from a `powercfg /batteryreport /XML` document.
///
/// Tolerant by design (risk R8): any missing field stays `None`, malformed
/// values are skipped — real-world reports vary across OEMs and Windows builds.
pub fn parse_battery_report_xml(xml: &str) -> BatteryInfo {
    let mut info = BatteryInfo {
        design_capacity_mwh: None,
        full_charge_capacity_mwh: None,
        health_percent: None,
        cycle_count: None,
        manufacturer: None,
        chemistry: None,
    };

    // DesignCapacity / FullChargeCapacity appear inside <Battery> blocks as
    // <DesignCapacity>684720</DesignCapacity> style tags (values in mWh).
    for tag in ["DesignCapacity", "FullChargeCapacity"] {
        if let Some(v) = first_u64(xml, &format!("<{tag}>"), &format!("</{tag}>")) {
            match tag {
                "DesignCapacity" => info.design_capacity_mwh = Some(v),
                _ => info.full_charge_capacity_mwh = Some(v),
            }
        }
    }

    // Cycle count: <CycleCount>233</CycleCount> (newer reports).
    if let Some(v) = first_u64(xml, "<CycleCount>", "</CycleCount>") {
        info.cycle_count = Some(u32::try_from(v).unwrap_or(0));
    }

    // Manufacturer / chemistry inside the first <BatteryInfo> block.
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
pub fn scan_battery() -> Result<BatteryInfo, String> {
    let report = scan_battery_impl().map_err(|e| e.to_string())?;
    Ok(report)
}

#[cfg(not(windows))]
#[tauri::command]
pub fn scan_battery() -> Result<BatteryInfo, String> {
    Err("Battery scan requires Windows (powercfg).".into())
}

/// Read battery facts directly from the WMI ROOT\WMI namespace (Windows).
///
/// Why not `powercfg /batteryreport`? Ground-truth on the ProBook showed
/// powercfg's XML flag is unreliable across Windows builds (emits HTML or a
/// blank file), and its report layout changes between versions. WMI gives
/// us the three numbers we need straight from the battery driver:
///   - BatteryStaticData.DesignedCapacity        (mWh, design spec)
///   - BatteryFullChargedCapacity.FullChargedCapacity (mWh, current full)
///   - BatteryCycleCount.CycleCount               (cycles)
/// Manufacturer/chemistry come from BatteryStaticData too.
#[cfg(windows)]
fn scan_battery_impl() -> Result<BatteryInfo, String> {
    use serde::Deserialize;
    use wmi::COMLibrary;

    let com = COMLibrary::without_security().map_err(|e| format!("COM init failed: {e}"))?;
    let conn = wmi::WMIConnection::with_namespace_path("ROOT\\WMI", com)
        .map_err(|e| format!("WMI connection failed: {e}"))?;

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct BatteryStaticData {
        designed_capacity: Option<u32>,
        manufacturer: Option<String>,
        chemistry: Option<String>,
        serial_number: Option<String>,
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

    // These classes return per-battery instances; laptops have one but we
    // take the first non-empty value defensively.
    let statics: Vec<BatteryStaticData> = conn
        .raw_query(
            "SELECT DesignedCapacity, Manufacturer, Chemistry, SerialNumber FROM BatteryStaticData",
        )
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
        // Desktops / docks without a battery are normal, not an error.
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

    const FIXTURE_NO_CYCLE: &str = r#"<BatteryReport><Batteries><Battery>
      <DesignCapacity>50000</DesignCapacity>
      <FullChargeCapacity>25000</FullChargeCapacity>
    </Battery></Batteries></BatteryReport>"#;

    #[test]
    fn healthy_probook_like_fixture_parses() {
        let b = parse_battery_report_xml(FIXTURE_HEALTHY);
        assert_eq!(b.design_capacity_mwh, Some(68472));
        assert_eq!(b.full_charge_capacity_mwh, Some(61240));
        assert_eq!(b.cycle_count, Some(142));
        assert_eq!(b.manufacturer.as_deref(), Some("SMP"));
        assert_eq!(b.chemistry.as_deref(), Some("LION"));
        let health = b.health_percent.unwrap();
        assert!((health - 89.4).abs() < 0.1, "health={health}");
        assert_eq!(
            b.health_percent.map(crate::scoring::battery_subscore),
            b.subscore()
        );
    }

    #[test]
    fn missing_fields_stay_none_never_panic() {
        let b = parse_battery_report_xml(FIXTURE_NO_CYCLE);
        assert_eq!(b.cycle_count, None);
        assert_eq!(b.manufacturer, None);
        assert_eq!(b.design_capacity_mwh, Some(50000));
        assert_eq!(b.health_percent, Some(50.0));
    }

    #[test]
    fn empty_report_is_all_none() {
        let b = parse_battery_report_xml("<BatteryReport></BatteryReport>");
        assert_eq!(b.design_capacity_mwh, None);
        assert_eq!(b.health_percent, None);
        assert_eq!(b.subscore(), None);
    }

    #[test]
    fn garbage_input_does_not_crash() {
        let b = parse_battery_report_xml("this is not xml at all <<>>");
        assert_eq!(b.health_percent, None);
    }

    #[test]
    fn zero_design_capacity_guarded() {
        let xml = r#"<Battery><DesignCapacity>0</DesignCapacity><FullChargeCapacity>5</FullChargeCapacity></Battery>"#;
        let b = parse_battery_report_xml(xml);
        assert_eq!(b.health_percent, None); // avoid div-by-zero nonsense
    }
}
