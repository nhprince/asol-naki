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

    if let Ok(conn) = wmi::WMIConnection::with_namespace_path("ROOT\\WMI", com) {
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

        let statics: Result<Vec<BatteryStaticData>, _> = conn.raw_query("SELECT DesignedCapacity, Manufacturer, Chemistry FROM BatteryStaticData");
        let fulls: Result<Vec<BatteryFullChargedCapacity>, _> = conn.raw_query("SELECT FullChargedCapacity FROM BatteryFullChargedCapacity");
        let cycles: Result<Vec<BatteryCycleCount>, _> = conn.raw_query("SELECT CycleCount FROM BatteryCycleCount");

        if let (Ok(s_list), Ok(f_list)) = (statics, fulls) {
            let stat = s_list.into_iter().find(|s| s.designed_capacity.unwrap_or(0) > 0);
            if let Some(stat) = stat {
                let full = f_list
                    .into_iter()
                    .find(|f| f.full_charged_capacity.unwrap_or(0) > 0)
                    .and_then(|f| f.full_charged_capacity);

                let design = stat.designed_capacity;
                let health_percent = match (full, design) {
                    (Some(f), Some(d)) if d > 0 => Some((f as f64 / d as f64) * 100.0),
                    _ => None,
                };

                let cycle_val = cycles.ok().and_then(|c| c.into_iter().find_map(|item| item.cycle_count));

                return Ok(BatteryInfo {
                    design_capacity_mwh: design.map(|v| v as u64),
                    full_charge_capacity_mwh: full.map(|v| v as u64),
                    health_percent,
                    cycle_count: cycle_val,
                    manufacturer: stat.manufacturer.filter(|m| !m.trim().is_empty()),
                    chemistry: stat.chemistry.filter(|c| !c.trim().is_empty()),
                });
            }
        }
    }

    if let Ok(conn_cim) = wmi::WMIConnection::new(com) {
        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct Win32Battery {
            design_capacity: Option<u32>,
            full_charge_capacity: Option<u32>,
            estimated_charge_remaining: Option<u16>,
            name: Option<String>,
        }

        if let Ok(batteries) = conn_cim.raw_query::<Win32Battery>("SELECT DesignCapacity, FullChargeCapacity, EstimatedChargeRemaining, Name FROM Win32_Battery") {
            if let Some(b) = batteries.into_iter().next() {
                let design = b.design_capacity.map(|v| v as u64);
                let full = b.full_charge_capacity.map(|v| v as u64);
                let health_percent = match (full, design) {
                    (Some(f), Some(d)) if d > 0 => Some((f as f64 / d as f64) * 100.0),
                    _ => b.estimated_charge_remaining.map(|v| v as f64),
                };

                return Ok(BatteryInfo {
                    design_capacity_mwh: design,
                    full_charge_capacity_mwh: full,
                    health_percent,
                    cycle_count: None,
                    manufacturer: b.name.filter(|m| !m.trim().is_empty()),
                    chemistry: None,
                });
            }
        }
    }

    Err("No battery present (desktop or missing driver).".into())
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
        assert_eq!(b.health_percent, None);
    }
}
