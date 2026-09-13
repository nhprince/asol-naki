//! EDID display parsing via Windows registry.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct DisplayInfo {
    pub manufacturer: Option<String>,
    pub product_code: Option<u16>,
    pub serial_number: Option<u32>,
    pub manufacture_week: Option<u8>,
    pub manufacture_year: Option<u16>,
    pub horizontal_px: Option<u16>,
    pub vertical_px: Option<u16>,
    pub preferred_refresh_hz: Option<f64>,
    pub diagonal_cm: Option<f64>,
    pub edid_valid: bool,
}

pub fn parse_edid(raw: &[u8]) -> DisplayInfo {
    let mut info = DisplayInfo {
        manufacturer: None,
        product_code: None,
        serial_number: None,
        manufacture_week: None,
        manufacture_year: None,
        horizontal_px: None,
        vertical_px: None,
        preferred_refresh_hz: None,
        diagonal_cm: None,
        edid_valid: false,
    };

    if raw.len() < 128 {
        return info;
    }

    if raw[0..8] != [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00] {
        return info;
    }

    let checksum: u8 = raw[0..128].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    if checksum != 0 {
        return info;
    }

    info.edid_valid = true;

    let pnp = u16::from_be_bytes([raw[8], raw[9]]);
    let c1 = (((pnp >> 10) & 0x1F) as u8 + b'A' - 1) as char;
    let c2 = (((pnp >> 5) & 0x1F) as u8 + b'A' - 1) as char;
    let c3 = ((pnp & 0x1F) as u8 + b'A' - 1) as char;
    info.manufacturer = Some(format!("{c1}{c2}{c3}"));

    info.product_code = Some(u16::from_le_bytes([raw[10], raw[11]]));
    info.serial_number = Some(u32::from_le_bytes([raw[12], raw[13], raw[14], raw[15]]));

    if raw[16] != 0xFF {
        info.manufacture_week = Some(raw[16]);
    }
    info.manufacture_year = Some(1990 + raw[17] as u16);

    let h_cm = raw[21] as f64;
    let v_cm = raw[22] as f64;
    if h_cm > 0.0 && v_cm > 0.0 {
        info.diagonal_cm = Some((h_cm * h_cm + v_cm * v_cm).sqrt());
    }

    let timing = &raw[54..72];
    let pixel_clock_khz = u16::from_le_bytes([timing[0], timing[1]]) as u32 * 10;
    if pixel_clock_khz > 0 {
        let h_active = (((timing[4] as u16 & 0xF0) << 4) | timing[2] as u16) as f64;
        let h_blank = (((timing[4] as u16 & 0x0F) << 8) | timing[3] as u16) as f64;
        let v_active = (((timing[7] as u16 & 0xF0) << 4) | timing[5] as u16) as f64;
        let v_blank = (((timing[7] as u16 & 0x0F) << 8) | timing[6] as u16) as f64;

        info.horizontal_px = Some(h_active as u16);
        info.vertical_px = Some(v_active as u16);

        let h_total = h_active + h_blank;
        let v_total = v_active + v_blank;
        if h_total > 0.0 && v_total > 0.0 {
            let refresh = (pixel_clock_khz as f64 * 1000.0) / (h_total * v_total);
            info.preferred_refresh_hz = Some((refresh * 10.0).round() / 10.0);
        }
    }

    info
}

#[cfg(windows)]
#[tauri::command]
pub async fn scan_display() -> Result<Vec<DisplayInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let edids = read_windows_registry_edids().map_err(|e| e.to_string())?;
        Ok(edids.iter().map(|raw| parse_edid(raw)).collect())
    })
    .await
    .map_err(|e| format!("Task join failed: {e}"))?
}

#[cfg(not(windows))]
#[tauri::command]
pub async fn scan_display() -> Result<Vec<DisplayInfo>, String> {
    Err("Display EDID scan requires Windows registry.".into())
}

#[cfg(windows)]
fn read_windows_registry_edids() -> std::io::Result<Vec<Vec<u8>>> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let display_key = hkcu.open_subkey(r"SYSTEM\CurrentControlSet\Enum\DISPLAY")?;

    let mut edids = Vec::new();

    for vendor in display_key.enum_keys().filter_map(|k| k.ok()) {
        let Ok(vendor_key) = display_key.open_subkey(&vendor) else { continue; };
        for dev in vendor_key.enum_keys().filter_map(|k| k.ok()) {
            let Ok(dev_key) = vendor_key.open_subkey(&dev) else { continue; };
            let Ok(param_key) = dev_key.open_subkey("Device Parameters") else { continue; };

            if let Ok(val) = param_key.get_raw_value("EDID") {
                if val.vtype == REG_BINARY {
                    edids.push(val.bytes);
                }
            }
        }
    }

    Ok(edids)
}
