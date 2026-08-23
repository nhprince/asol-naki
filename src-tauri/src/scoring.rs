//! Scoring engine (plan.md §7): weighted category sub-scores roll into one
//! overall 0–10 score, with a Critical fraud flag capping the result.
//!
//! Pure functions over input data — no I/O, fully unit-testable. This module
//! IS the product's judgment; every rule here needs a test.

use serde::Serialize;

/// Per-category health score, 0.0–10.0.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CategoryScores {
    pub storage: f64,
    pub cpu_gpu_sanity: f64,
    pub battery: f64,
    pub display: f64,
    pub ports_connectivity: f64,
}

/// Weights per plan.md §7, exactly as specified there. Note they sum to 0.95
/// (fraud consistency is a Gate, not a weighted category); compute_score
/// normalizes by the actual sum so a perfect device still scores 10.
pub const WEIGHTS: StorageWeights = StorageWeights {
    storage: 0.25,
    cpu_gpu_sanity: 0.25,
    battery: 0.20,
    display: 0.15,
    ports_connectivity: 0.10,
};

#[derive(Debug, Clone, Copy)]
pub struct StorageWeights {
    pub storage: f64,
    pub cpu_gpu_sanity: f64,
    pub battery: f64,
    pub display: f64,
    pub ports_connectivity: f64,
}

/// Severity of a fraud/consistency flag (plan.md §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FlagSeverity {
    /// Surfaced in report only; no scoring effect.
    Info,
    /// Reduces relevant category score; no overall cap.
    Warning,
    /// Caps the overall score regardless of everything else.
    Critical,
}

/// A device with any Critical flag must never score above this (plan.md §6).
pub const CRITICAL_CAP: f64 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ScoreResult {
    /// Final score after weighting and cap, clamped to [0, 10].
    pub overall: f64,
    /// True if a Critical flag forced the cap down.
    pub capped_by_critical: bool,
}

/// Compute the weighted overall score from category sub-scores and whether
/// any Critical fraud flag is present.
pub fn compute_score(
    categories: &CategoryScores,
    weights: &StorageWeights,
    critical_flag: bool,
) -> ScoreResult {
    let total_weight = weights.storage
        + weights.cpu_gpu_sanity
        + weights.battery
        + weights.display
        + weights.ports_connectivity;

    // plan.md §7 weights sum to 0.95 (fraud is a gate, not a weighted
    // category). Normalize so the scale stays 0–10; guard against an
    // all-zero weight config.
    let norm = if total_weight > f64::EPSILON {
        1.0 / total_weight
    } else {
        1.0
    };

    let weighted = (categories.storage * weights.storage
        + categories.cpu_gpu_sanity * weights.cpu_gpu_sanity
        + categories.battery * weights.battery
        + categories.display * weights.display
        + categories.ports_connectivity * weights.ports_connectivity)
        * norm;

    let mut score = weighted.clamp(0.0, 10.0);
    let mut capped = false;
    if critical_flag && score > CRITICAL_CAP {
        score = CRITICAL_CAP;
        capped = true;
    }

    ScoreResult {
        overall: (score * 10.0).round() / 10.0, // one decimal
        capped_by_critical: capped,
    }
}

/// Battery health (full-charge ÷ design capacity) → 0–10 sub-score.
/// Calibration is expected to change with real-world testing (plan.md §7).
pub fn battery_subscore(health_percent: f64) -> f64 {
    // >=90% healthy → ~10; linear to 40% floor → 0.
    const FULL_AT: f64 = 90.0;
    const ZERO_AT: f64 = 40.0;
    if health_percent >= FULL_AT {
        return 10.0;
    }
    if health_percent <= ZERO_AT {
        return 0.0;
    }
    ((health_percent - ZERO_AT) / (FULL_AT - ZERO_AT) * 10.0 * 10.0).round() / 10.0
}

/// SMART health percentage (100 = perfect) → 0–10 sub-score.
pub fn storage_subscore(smart_health_percent: f64) -> f64 {
    smart_health_percent.clamp(0.0, 100.0) / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_perfect() -> CategoryScores {
        CategoryScores {
            storage: 10.0,
            cpu_gpu_sanity: 10.0,
            battery: 10.0,
            display: 10.0,
            ports_connectivity: 10.0,
        }
    }

    #[test]
    fn perfect_device_scores_ten() {
        let r = compute_score(&all_perfect(), &WEIGHTS, false);
        assert_eq!(r.overall, 10.0);
        assert!(!r.capped_by_critical);
    }

    #[test]
    fn weights_sum_to_one() {
        let sum = WEIGHTS.storage
            + WEIGHTS.cpu_gpu_sanity
            + WEIGHTS.battery
            + WEIGHTS.display
            + WEIGHTS.ports_connectivity;
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn critical_flag_caps_high_score_at_three() {
        let r = compute_score(&all_perfect(), &WEIGHTS, true);
        assert_eq!(r.overall, CRITICAL_CAP);
        assert!(r.capped_by_critical);
    }

    #[test]
    fn critical_flag_never_raises_low_score() {
        let mut cats = all_perfect();
        cats.battery = 1.0;
        cats.display = 1.0;
        let uncapped = compute_score(&cats, &WEIGHTS, false);
        let capped = compute_score(&cats, &WEIGHTS, true);
        assert!(capped.overall <= uncapped.overall);
        assert_eq!(capped.overall, uncapped.overall.min(CRITICAL_CAP));
    }

    #[test]
    fn plan_example_seven_point_seven_is_good_buy_range() {
        // Roughly healthy device with worn battery should land in negotiate/good-buy band.
        let cats = CategoryScores {
            storage: 9.5,
            cpu_gpu_sanity: 9.0,
            battery: 3.4, // ≈71% health
            display: 9.0,
            ports_connectivity: 9.0,
        };
        let r = compute_score(&cats, &WEIGHTS, false);
        assert!(
            r.overall >= 6.0 && r.overall <= 9.0,
            "unexpected score {r:?}"
        );
    }

    #[test]
    fn battery_subscore_boundaries() {
        assert_eq!(battery_subscore(95.0), 10.0); // ≥90 full marks
        assert_eq!(battery_subscore(30.0), 0.0); // ≤40 dead
        let mid = battery_subscore(65.0); // midpoint → 5
        assert!((mid - 5.0).abs() < 0.11, "mid={mid}");
    }

    #[test]
    fn storage_subscore_maps_percent_to_ten_scale() {
        assert_eq!(storage_subscore(100.0), 10.0);
        assert_eq!(storage_subscore(85.0), 8.5);
        assert_eq!(storage_subscore(-5.0), 0.0); // clamped
    }

    #[test]
    fn score_rounds_to_one_decimal() {
        let cats = CategoryScores {
            storage: 7.77,
            cpu_gpu_sanity: 7.77,
            battery: 7.77,
            display: 7.77,
            ports_connectivity: 7.77,
        };
        let r = compute_score(&cats, &WEIGHTS, false);
        assert_eq!(r.overall, 7.8);
    }
}
