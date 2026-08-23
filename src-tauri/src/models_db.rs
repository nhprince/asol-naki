//! Loader for the bundled "known models" spec DB (`resources/known_models.json`).
//! Embedded at compile time via include_str! — zero I/O, offline-first.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct KnownCpu {
    pub expected_cores: u32,
    pub expected_threads: u32,
    pub cache_mb: u32,
    pub family: String,
}

#[derive(Debug, Deserialize)]
pub struct KnownModels {
    #[serde(default)]
    cpus: HashMap<String, KnownCpu>,
}

impl KnownModels {
    /// Parse the embedded JSON. Panics only on malformed bundled data (a
    /// build-time bug, not a runtime condition).
    pub fn embedded() -> Self {
        serde_json::from_str(include_str!("../resources/known_models.json"))
            .expect("bundled known_models.json is valid JSON")
    }

    pub fn from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Case-insensitive lookup by CPU model string.
    pub fn cpu(&self, model: &str) -> Option<&KnownCpu> {
        self.cpus.get(&model.trim().to_lowercase())
    }

    #[cfg(test)]
    pub fn cpu_count(&self) -> usize {
        self.cpus.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_db_loads_and_has_bd_market_cpus() {
        let db = KnownModels::embedded();
        assert!(db.cpu_count() >= 10, "bootstrap set should be populated");
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let db = KnownModels::embedded();
        let a = db.cpu("Intel Core Ultra 5 125H");
        let b = db.cpu("intel core ultra 5 125h");
        assert!(a.is_some() && b.is_some());
        assert_eq!(a.unwrap().expected_threads, b.unwrap().expected_threads);
    }

    #[test]
    fn probook_cpu_is_known() {
        // Prince's dev machine — the primary ground-truth target.
        let db = KnownModels::embedded();
        let cpu = db
            .cpu("Intel Core Ultra 5 125H")
            .expect("Ultra 5 125H in DB");
        assert_eq!(cpu.expected_cores, 14);
        assert_eq!(cpu.expected_threads, 18);
    }

    #[test]
    fn unknown_model_returns_none() {
        let db = KnownModels::embedded();
        assert!(db.cpu("Totally Fake CPU 9999X").is_none());
    }

    #[test]
    fn from_str_rejects_garbage() {
        assert!(KnownModels::from_str("not json").is_err());
    }
}
