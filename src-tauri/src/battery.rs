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
    let report = run_powercfg_battery_report().map_err(|e| e.to_string())?;
    Ok(parse_battery_report_xml(&report))
}

#[cfg(not(windows))]
#[tauri::command]
pub fn scan_battery() -> Result<BatteryInfo, String> {
    Err("Battery scan requires Windows (powercfg).".into())
}

/// Generate + read the battery report in a temp dir. Windows-only shell-out;
/// parsing lives in `parse_battery_report_xml` so tests never touch this.
#[cfg(windows)]
fn run_powercfg_battery_report() -> std::io::Result<String> {
    use std::process::Command;

    let tmp = std::env::temp_dir().join("asol-naki-battery.xml");
    // Flag order matters on some Windows builds: /XML must precede the
    // /output path, otherwise powercfg silently emits HTML instead.
    let status = Command::new("powercfg")
        .args(["/batteryreport", "/XML", "/output"])
        .arg(&tmp)
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other("powercfg /batteryreport failed"));
    }

    let body = std::fs::read_to_string(&tmp)?;
    // Defensive: if powercfg ignored /XML and emitted HTML anyway, tell the
    // caller clearly instead of returning a parse-empty result.
    if body.trim_start().starts_with("<!DOCTYPE html")
        || body.contains("<html")
    {
        return Err(std::io::Error::other(
            "powercfg produced HTML instead of XML report",
        ));
    }
    Ok(body)
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
