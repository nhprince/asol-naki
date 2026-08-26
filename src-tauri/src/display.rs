//! Display / EDID reader — reads raw EDID blobs from the Windows registry
//! (HKLM\SYSTEM\CurrentControlSet\Enum\DISPLAY\*\*\Device Parameters\EDID),
//! parses them per the VESA E-EDID standard, and reports the *real* panel
//! identity: manufacturer, model, week/year, resolution, refresh rate.
//!
//! This is the anti-fraud signal for "claimed 144Hz IPS, actually 60Hz TN".
//! Non-Windows builds return the honest "requires Windows" error, matching
//! battery.rs/storage.rs convention.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DisplayInfo {
    /// e.g. "HPH" (HP) / "BOE" / "LGD" — decoded 3-letter IEEE PnP ID
    pub manufacturer: Option<String>,
    /// Product code (bytes 10-11, little-endian u16)
    pub product_code: Option<u16>,
    /// Serial number (bytes 12-15, little-endian u32)
    pub serial_number: Option<u32>,
    pub manufacture_week: Option<u8>,
    pub manufacture_year: Option<u16>,
    /// Native horizontal resolution in pixels (from preferred timing DTD)
    pub horizontal_px: Option<u32>,
    /// Native vertical resolution in pixels
    pub vertical_px: Option<u32>,
    /// Preferred (native) refresh rate in Hz, derived from pixel clock
    pub preferred_refresh_hz: Option<f64>,
    /// Viewable diagonal via Pythagoras on max h/v image size (cm)
    pub diagonal_cm: Option<f64>,
    /// True if a valid checksum and 128-byte base block were parsed
    pub edid_valid: bool,
}

/// Decode the 3-letter IEEE PnP vendor ID (bytes 8-9, big-endian u16).
/// Each of three 5-bit fields maps 1→'A' … 26→'Z'.
pub fn decode_pnp_id(bytes: &[u8]) -> String {
    let b = |i: usize| -> u16 { *bytes.get(i).unwrap_or(&0) as u16 };
    let combined = (b(8) << 8) | b(9);
    let letter = |c: u16| -> char {
        // 0 is reserved; clamp so we never emit garbage outside A-Z.
        let idx = c.clamp(1, 26) as u8;
        (b'A' + idx - 1) as char
    };
    format!(
        "{}{}{}",
        letter((combined >> 10) & 0x1F),
        letter((combined >> 5) & 0x1F),
        letter(combined & 0x1F)
    )
}

/// Verify the EDID checksum: sum of all 128 base-block bytes mod 256 == 0.
pub fn checksum_valid(edid: &[u8]) -> bool {
    if edid.len() < 128 {
        return false;
    }
    edid.iter().take(128).map(|&b| b as u32).sum::<u32>() % 256 == 0
}

/// Parse a 128-byte EDID base block into DisplayInfo.
///
/// Base-block layout (VESA E-EDID v1.x):
/// ```text
/// 0-7   header 00 FF FF FF FF FF FF 00
/// 8-9   manufacturer PnP ID (BE u16)
/// 10-11 product code (LE u16)
/// 12-15 serial (LE u32)
/// 16    week, 17  year-1990
/// 18-19 EDID version.revision
/// 21-22 max visible h/v size (cm)
/// 54-71 preferred timing descriptor (DTD #1)
/// 127   checksum
/// ```
pub fn parse_edid(edid: &[u8]) -> DisplayInfo {
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

    let header_ok = edid.len() >= 128 && edid[0] == 0x00 && edid[1] == 0xFF && edid[7] == 0x00;
    if !header_ok || !checksum_valid(edid) {
        return info;
    }
    info.edid_valid = true;

    info.manufacturer = Some(decode_pnp_id(edid));
    info.product_code = Some(u16::from_le_bytes([edid[10], edid[11]]));
    info.serial_number = Some(u32::from_le_bytes([edid[12], edid[13], edid[14], edid[15]]));
    info.manufacture_week = Some(if edid[16] == 0xFF { 0 } else { edid[16] });
    info.manufacture_year = Some(1990 + edid[17] as u16);

    let (w_cm, h_cm) = (edid[21] as f64, edid[22] as f64);
    if w_cm > 0.0 && h_cm > 0.0 {
        info.diagonal_cm = Some((w_cm * w_cm + h_cm * h_cm).sqrt());
    }

    // Preferred timing descriptor (DTD #1):
    //   54-55 pixel clock in 10 kHz units (LE u16)
    //   56    h_active low 8 bits      57  h_blank low 8 bits
    //   58    v_active low 8 bits      59  v_blank low 8 bits
    //   60    hi nibble: h_active high 4 | lo nibble: h_blank high 4
    //   61    hi nibble: v_active high 4 | lo nibble: v_blank high 4
    let px_clock_hz = u16::from_le_bytes([edid[54], edid[55]]) as f64 * 10_000.0;
    let h_active = ((((edid[60] >> 4) as u16) << 8) | edid[56] as u16) as f64;
    let h_blank = ((((edid[60] & 0x0F) as u16) << 8) | edid[57] as u16) as f64;
    let v_active = ((((edid[61] >> 4) as u16) << 8) | edid[58] as u16) as f64;
    let v_blank = ((((edid[61] & 0x0F) as u16) << 8) | edid[59] as u16) as f64;

    let h_total = h_active + h_blank;
    let v_total = v_active + v_blank;
    if px_clock_hz > 0.0 && h_total > 0.0 && v_total > 0.0 {
        let refresh = px_clock_hz / (h_total * v_total);
        info.horizontal_px = Some(h_active as u32);
        info.vertical_px = Some(v_active as u32);
        info.preferred_refresh_hz = Some((refresh * 10.0).round() / 10.0);
    }

    info
}

#[cfg(windows)]
fn read_edids_from_registry() -> Result<Vec<Vec<u8>>, String> {
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let display_root = hklm
        .open_subkey_with_flags(r"SYSTEM\CurrentControlSet\Enum\DISPLAY", KEY_READ)
        .map_err(|e| format!("Display registry unavailable: {e}"))?;

    let mut edids = Vec::new();
    for vendor_key in display_root.enum_keys().flatten() {
        let Ok(vendor_root) = display_root.open_subkey_with_flags(&vendor_key, KEY_READ) else {
            continue;
        };
        for instance in vendor_root.enum_keys().flatten() {
            let Ok(params) = vendor_root
                .open_subkey_with_flags(format!("{instance}\\Device Parameters"), KEY_READ)
            else {
                continue;
            };
            if let Ok(edid) = params.get_value::<Vec<u8>, _>("EDID") {
                if !edid.is_empty() {
                    edids.push(edid);
                }
            }
        }
    }
    Ok(edids)
}

#[cfg(windows)]
fn scan_display_impl() -> Result<Vec<DisplayInfo>, String> {
    let raw = read_edids_from_registry()?;
    Ok(raw
        .iter()
        .filter(|e| e.len() >= 128)
        .map(|e| parse_edid(e))
        .collect())
}

#[cfg(not(windows))]
fn scan_display_impl() -> Result<Vec<DisplayInfo>, String> {
    Err("Display scan requires Windows (registry EDID).".into())
}

#[tauri::command]
pub fn scan_display() -> Result<Vec<DisplayInfo>, String> {
    scan_display_impl()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parameters for building a synthetic EDID base block in tests.
    struct EdidFixture {
        pnp: (u8, u8),
        week: u8,
        year_offset: u8,
        w_cm: u8,
        h_cm: u8,
        timing: Option<TimingFixture>,
    }

    struct TimingFixture {
        px_clock_10khz: u16,
        h_active: u16,
        h_blank: u16,
        v_active: u16,
        v_blank: u16,
    }

    fn make_edid(f: &EdidFixture) -> Vec<u8> {
        let mut e = vec![0u8; 128];
        // Header magic + fixed fields (VESA base-block layout)
        e[..8].copy_from_slice(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
        e[8] = f.pnp.0;
        e[9] = f.pnp.1;
        e[10] = 0x34;
        e[11] = 0x12;
        e[12] = 0x78;
        e[13] = 0x56;
        e[14] = 0x34;
        e[15] = 0x12;
        e[16] = f.week;
        e[17] = f.year_offset;
        e[18] = 1;
        e[19] = 4;
        e[20] = 0x80;
        e[21] = f.w_cm;
        e[22] = f.h_cm;
        e[23] = 0x78;
        if let Some(t) = &f.timing {
            e[54] = (t.px_clock_10khz & 0xFF) as u8;
            e[55] = (t.px_clock_10khz >> 8) as u8;
            e[56] = (t.h_active & 0xFF) as u8;
            e[57] = (t.h_blank & 0xFF) as u8;
            e[58] = (t.v_active & 0xFF) as u8;
            e[59] = (t.v_blank & 0xFF) as u8;
            e[60] = ((t.h_active >> 8) as u8) << 4 | ((t.h_blank >> 8) as u8);
            e[61] = ((t.v_active >> 8) as u8) << 4 | ((t.v_blank >> 8) as u8);
        }
        let sum: u32 = e[..127].iter().map(|&b| b as u32).sum();
        e[127] = ((256 - (sum % 256)) % 256) as u8;
        e
    }

    #[test]
    fn pnp_id_decodes_known_vendors() {
        // "AAA" = 1,1,1 → (1<<10)|(1<<5)|1 = 0x0421
        assert_eq!(decode_pnp_id(&[0x04, 0x21]), "AAA");
        // "BOE" (BOE Display) = 2,15,5 → 0x09E5
        assert_eq!(decode_pnp_id(&[0x09, 0xE5]), "BOE");
        // "HPH" (HP) = 8,16,8 → (8<<10)|(16<<5)|8 = 0x2208
        assert_eq!(decode_pnp_id(&[0x22, 0x08]), "HPH");
    }

    #[test]
    fn checksum_rejects_corrupt_block() {
        let mut edid = vec![0u8; 128];
        edid[127] = 1;
        assert!(!checksum_valid(&edid));
        edid[127] = 0;
        assert!(checksum_valid(&edid));
        assert!(!checksum_valid(&edid[..100]));
    }

    #[test]
    fn parse_rejects_bad_header_even_with_valid_checksum() {
        let mut edid = vec![0u8; 128];
        let sum: u32 = edid[..127].iter().map(|&b| b as u32).sum();
        edid[127] = ((256 - (sum % 256)) % 256) as u8;
        assert!(!parse_edid(&edid).edid_valid);
    }

    #[test]
    fn parses_realistic_fhd_panel() {
        // 1920x1080, pixel clock 140 MHz, blanking 160/90
        let e = make_edid(&EdidFixture {
            pnp: (0x09, 0xE5),
            week: 12,
            year_offset: 34,
            w_cm: 30,
            h_cm: 18,
            timing: Some(TimingFixture {
                px_clock_10khz: 14_000,
                h_active: 1920,
                h_blank: 160,
                v_active: 1080,
                v_blank: 90,
            }),
        });
        let d = parse_edid(&e);
        assert!(d.edid_valid);
        assert_eq!(d.manufacturer.as_deref(), Some("BOE"));
        assert_eq!(d.product_code, Some(0x1234));
        assert_eq!(d.serial_number, Some(0x12345678));
        assert_eq!(d.manufacture_week, Some(12));
        assert_eq!(d.manufacture_year, Some(2024)); // 1990 + 34
        assert_eq!(d.horizontal_px, Some(1920));
        assert_eq!(d.vertical_px, Some(1080));
        let r = d.preferred_refresh_hz.unwrap();
        assert!(r > 55.0 && r < 75.0, "refresh out of range: {r}");
        let diag = d.diagonal_cm.unwrap();
        assert!(diag > 33.0 && diag < 37.0, "diagonal off: {diag}");
    }

    #[test]
    fn parses_high_refresh_gaming_panel() {
        // 2560x1440 @ ~165Hz-class pixel clock ~586 MHz, blanking 480/150
        let e = make_edid(&EdidFixture {
            pnp: (0x22, 0x08),
            week: 5,
            year_offset: 31,
            w_cm: 60,
            h_cm: 34,
            timing: Some(TimingFixture {
                px_clock_10khz: 58_600,
                h_active: 2560,
                h_blank: 480,
                v_active: 1440,
                v_blank: 150,
            }),
        });
        let d = parse_edid(&e);
        assert_eq!(d.horizontal_px, Some(2560));
        assert_eq!(d.vertical_px, Some(1440));
        let r = d.preferred_refresh_hz.unwrap();
        assert!(r > 155.0 && r < 175.0, "expected ~165Hz, got {r}");
        assert_eq!(d.manufacturer.as_deref(), Some("HPH"));
    }
}
