//! Fraud/spoof consistency checks (plan.md §6) — the core value prop.
//!
//! Spoofing tools typically fake ONE data source; we cross-reference
//! independent signals against each other and the known-models DB. Pure
//! functions over collected data: every rule is deterministically testable,
//! and the Phase 2 exit gate (catch a deliberate misrepresentation) is proven
//! in CI with synthetic spoofed fixtures — no special hardware needed.

use crate::hardware::FullHardwareInfo;
use crate::models_db::KnownModels;
use crate::scoring::FlagSeverity;
use crate::storage::StorageInfo;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FraudFlag {
    pub severity: FlagSeverity,
    /// Machine-readable check id, e.g. "cpu_identity_mismatch".
    pub check_id: String,
    /// i18n key for display; parameters via frontend interpolation.
    pub message_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IntegrityReport {
    pub flags: Vec<FraudFlag>,
    pub has_critical: bool,
    pub has_warning: bool,
}

impl IntegrityReport {
    fn from_flags(flags: Vec<FraudFlag>) -> Self {
        let has_critical = flags.iter().any(|f| f.severity == FlagSeverity::Critical);
        let has_warning = flags.iter().any(|f| f.severity == FlagSeverity::Warning);
        IntegrityReport {
            flags,
            has_critical,
            has_warning,
        }
    }
}

/// Run every consistency check over the collected diagnostics.
///
/// Tauri commands must stay small and single-purpose per CLAUDE.md, but this
/// one is intentionally a thin orchestrator over pure check functions that
/// are individually exported and tested. It takes pre-collected data as JSON
/// from the frontend rather than scanning itself — so E2E can feed synthetic
/// spoofed fixtures through the exact same production code path.
#[tauri::command]
pub fn run_integrity_checks(
    hardware_json: String,
    storage_json: Vec<String>,
) -> Result<IntegrityReport, String> {
    let hw: FullHardwareInfo = serde_json::from_str(&hardware_json).map_err(|e| e.to_string())?;
    let storage: Vec<StorageInfo> = storage_json
        .iter()
        .map(|s| serde_json::from_str(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let db = KnownModels::embedded();
    Ok(run_checks(&hw, &storage, &db))
}

/// Pure entry point used by tests and the command above.
pub fn run_checks(
    hw: &FullHardwareInfo,
    storage: &[StorageInfo],
    db: &KnownModels,
) -> IntegrityReport {
    let mut flags = Vec::new();
    flags.extend(check_cpu_identity(hw, db));
    flags.extend(check_storage_capacity(storage));
    IntegrityReport::from_flags(flags)
}

/// CPU: reported model string vs actual threads (and cores when WMI provided).
///
/// Threads are the strongest independent signal — a relabeled i3 cannot fake
/// 8 threads because the OS counts them. Only fires when the exact model is
/// in the DB; unknown models produce no flag (we never guess).
pub fn check_cpu_identity(hw: &FullHardwareInfo, db: &KnownModels) -> Vec<FraudFlag> {
    let Some(known) = db.cpu(&hw.cpu_name) else {
        return vec![];
    };

    let mut flags = vec![];

    // Thread-count mismatch beyond tolerance → Critical (identity fraud).
    if hw.cpu_threads != known.expected_threads as usize {
        flags.push(FraudFlag {
            severity: FlagSeverity::Critical,
            check_id: "cpu_thread_count_mismatch".into(),
            message_key: "fraud.cpuThreadMismatch".into(),
            detail: Some(format!(
                "reported '{}' claims {} threads but OS counts {}",
                hw.cpu_name, known.expected_threads, hw.cpu_threads
            )),
        });
    }

    // Physical cores (WMI) mismatch → Critical.
    if let Some(actual_cores) = hw.cpu_cores_physical {
        if actual_cores != known.expected_cores {
            flags.push(FraudFlag {
                severity: FlagSeverity::Critical,
                check_id: "cpu_core_count_mismatch".into(),
                message_key: "fraud.cpuCoreMismatch".into(),
                detail: Some(format!(
                    "reported '{}' claims {} cores but WMI reports {}",
                    hw.cpu_name, known.expected_cores, actual_cores
                )),
            });
        }
    }

    flags
}

/// Storage: real addressable capacity vs what the label/price implies.
///
/// Fake-capacity signature v1: a drive claiming NVMe protocol with ≥400 GB
/// but no endurance telemetry (real NVMe drives always report percentage_used),
/// or an absurdly tiny "NVMe SSD". SATA drives are exempt from heuristic #1.
pub fn check_storage_capacity(storage: &[StorageInfo]) -> Vec<FraudFlag> {
    let mut flags = vec![];
    for d in storage {
        let Some(bytes) = d.total_capacity_bytes else {
            continue;
        };
        let gb = bytes as f64 / 1024.0 / 1024.0 / 1024.0;

        if d.protocol.as_deref() == Some("nvme") && d.nvme_percentage_used.is_none() && gb >= 400.0
        {
            flags.push(FraudFlag {
                severity: FlagSeverity::Critical,
                check_id: "fake_nvme_capacity".into(),
                message_key: "fraud.fakeNvmeCapacity".into(),
                detail: Some(format!(
                    "'{}' reports {:.0} GB over NVMe but no endurance data",
                    d.model_name.as_deref().unwrap_or("unknown"),
                    gb
                )),
            });
        }

        if d.protocol.as_deref() == Some("nvme") && gb < 16.0 {
            flags.push(FraudFlag {
                severity: FlagSeverity::Critical,
                check_id: "absurd_nvme_capacity".into(),
                message_key: "fraud.absurdCapacity".into(),
                detail: Some(format!("{gb:.1} GB advertised as NVMe SSD")),
            });
        }
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models_db::KnownModels;

    fn honest_hw() -> FullHardwareInfo {
        FullHardwareInfo {
            cpu_name: "Intel Core Ultra 5 125H".into(),
            cpu_threads: 18,
            cpu_cores_physical: Some(14),
            total_memory_mb: 16384,
            os_name: "Windows".into(),
            os_version: "11".into(),
            kernel_version: "10.0".into(),
            hostname: "PROBOOK".into(),
            motherboard: None,
            bios_vendor: None,
            bios_version: None,
            gpus: None,
        }
    }

    fn spoofed_hw() -> FullHardwareInfo {
        // Scammer flashed an i3-1005G1's firmware string to claim Ultra 5;
        // the OS thread/core counts leak the truth.
        FullHardwareInfo {
            cpu_name: "Intel Core Ultra 5 125H".into(),
            cpu_threads: 4,
            cpu_cores_physical: Some(2),
            ..honest_hw()
        }
    }

    fn nvme_1tb_healthy() -> StorageInfo {
        parse_helper(
            r#"{
            "device_protocol": "NVMe",
            "model_name": "Samsung SSD 980 PRO 1TB",
            "user_capacity": {"bytes": 1000204886016},
            "smart_status": {"passed": true},
            "nvme_smart_health_information_log": {"percentage_used": 3}
        }"#,
        )
    }

    fn parse_helper(json: &str) -> StorageInfo {
        crate::storage::parse_smartctl_json(json)
    }

    #[test]
    fn honest_probook_profile_passes_clean() {
        let db = KnownModels::embedded();
        let r = run_checks(&honest_hw(), &[nvme_1tb_healthy()], &db);
        assert!(r.flags.is_empty(), "flags: {:?}", r.flags);
        assert!(!r.has_critical);
    }

    #[test]
    fn phase2_exit_gate_spoofed_cpu_is_caught() {
        // The deliberate-misrepresentation case from ROADMAP Phase 2:
        // wrong thread + core counts under a known model string → Critical.
        let db = KnownModels::embedded();
        let r = run_checks(&spoofed_hw(), &[nvme_1tb_healthy()], &db);
        assert!(r.has_critical, "spoof must raise Critical");
        assert!(r
            .flags
            .iter()
            .any(|f| f.check_id == "cpu_thread_count_mismatch"));
        assert!(r
            .flags
            .iter()
            .any(|f| f.check_id == "cpu_core_count_mismatch"));
    }

    #[test]
    fn critical_cap_math_holds_for_caught_spoof() {
        // A caught spoof caps the score at 3.0 even with perfect other parts.
        let db = KnownModels::embedded();
        let r = run_checks(&spoofed_hw(), &[nvme_1tb_healthy()], &db);
        assert!(r.has_critical);
        let cats = crate::scoring::CategoryScores {
            storage: 10.0,
            cpu_gpu_sanity: 10.0,
            battery: 10.0,
            display: 10.0,
            ports_connectivity: 10.0,
        };
        let s = crate::scoring::compute_score(&cats, &crate::scoring::WEIGHTS, r.has_critical);
        assert_eq!(s.overall, 3.0);
        assert!(s.capped_by_critical);
    }

    #[test]
    fn unknown_cpu_never_flagged() {
        let mut hw = spoofed_hw();
        hw.cpu_name = "Mystery Chip 9000".into(); // not in DB
        let db = KnownModels::embedded();
        let r = run_checks(&hw, &[], &db);
        assert!(!r.has_critical);
    }

    #[test]
    fn fake_nvme_missing_endurance_flagged() {
        let db = KnownModels::embedded();
        let fake = parse_helper(
            r#"{
            "device_protocol": "NVMe",
            "model_name": "Generic SSD 1TB",
            "user_capacity": {"bytes": 512110190592},
            "smart_status": {"passed": true}
        }"#,
        );
        let r = run_checks(&honest_hw(), &[fake], &db);
        assert!(r.has_critical);
        assert!(r.flags.iter().any(|f| f.check_id == "fake_nvme_capacity"));
    }

    #[test]
    fn tiny_nvme_flagged_absurd() {
        let db = KnownModels::embedded();
        let tiny = parse_helper(
            r#"{
            "device_protocol": "NVMe",
            "model_name": "SuperSpeed 8GB",
            "user_capacity": {"bytes": 8000000000},
            "nvme_smart_health_information_log": {"percentage_used": 1}
        }"#,
        );
        let r = run_checks(&honest_hw(), &[tiny], &db);
        assert!(r.has_critical);
        assert!(r.flags.iter().any(|f| f.check_id == "absurd_nvme_capacity"));
    }

    #[test]
    fn sata_drive_without_nvme_endurance_not_fake_flagged() {
        // The fake-capacity heuristic only applies to NVMe protocol.
        let db = KnownModels::embedded();
        let sata = parse_helper(
            r#"{
            "device_protocol": "ATA",
            "model_name": "WDC WD10SPZX",
            "user_capacity": {"bytes": 1000204886016},
            "smart_status": {"passed": true}
        }"#,
        );
        let r = run_checks(&honest_hw(), &[sata], &db);
        assert!(!r.has_critical, "flags: {:?}", r.flags);
    }

    #[test]
    fn command_entry_rejects_malformed_json() {
        let err = run_integrity_checks("not json".into(), vec![]).unwrap_err();
        assert!(!err.is_empty());
    }
}
