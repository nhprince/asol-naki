# CLAUDE.md — project context for Claude Code

This file is read automatically by Claude Code. See `plan.md` for the full product plan and `ROADMAP.md` for build order — read both before starting work if this is a new session.

## What this project is

**Asol Naki?** — a portable Windows desktop app that automatically diagnoses second-hand laptops (CPU, storage, battery, display, ports), detects spec fraud/spoofing, and outputs a 1–10 trust score with a plain-language buy/negotiate/walk-away verdict.

## Who's building this

Prince (the human) is an experienced full-stack JS/TypeScript/React developer and UI/UX designer. He does **not** know Rust and isn't learning it as a separate goal right now — Claude Code should write and own the Rust core, explaining decisions in plain language when they matter, not assuming Prince can read Rust idioms fluently. Prince owns product decisions, UI/UX direction, and testing on his own hardware.

## Stack

- **Shell:** Tauri v2
- **Frontend:** React + TypeScript + Vite + Tailwind CSS + `react-i18next` (English + Bengali)
- **Backend/core:** Rust — `sysinfo` crate for hardware data, `smartctl` subprocess for SMART disk data, Windows' built-in `powercfg /batteryreport` for battery data, WMI/SMBIOS queries for model identity and EDID display data
- **Local data:** flat JSON files for scan history/reports in v1 — do not introduce SQLite until bulk/reseller mode is actually being built
- **Package manager:** npm (matches Prince's other projects)

## Repo structure

```
asol-naki/
├── plan.md              — product plan, read this for "why"
├── ROADMAP.md            — phase-by-phase build order
├── CLAUDE.md              — this file
├── src/                   — React + TS frontend
│   ├── components/
│   ├── i18n/              — en.json, bn.json string tables
│   └── ...
├── src-tauri/             — Rust core
│   ├── src/
│   │   ├── hardware.rs    — sysinfo pulls (CPU/RAM/GPU/OS)
│   │   ├── storage.rs     — SMART data via smartctl
│   │   ├── battery.rs     — powercfg parsing
│   │   ├── display.rs     — EDID reading
│   │   ├── integrity.rs   — fraud/spoof consistency checks
│   │   ├── scoring.rs     — sub-score + overall score calculation
│   │   └── models_db.rs   — loads the bundled "known models" spec JSON
│   ├── resources/
│   │   ├── smartctl/      — bundled smartctl binary
│   │   └── known_models.json
│   └── Cargo.toml
```

## Conventions

**Rust:**
- Use `anyhow` for error handling in application code, `thiserror` only where a typed error actually needs to be matched on.
- Use `serde`/`serde_json` for all data crossing the Rust↔TypeScript boundary — define a shared shape and keep it flat where possible.
- Every Tauri command should be small and single-purpose (one hardware domain per command: `scan_storage`, `scan_battery`, etc.) rather than one giant `run_full_scan` blob — makes partial re-scans and testing easier.
- Never shell out to or bundle closed/restricted-license diagnostic tools (CrystalDiskInfo, HWiNFO, Cinebench, etc.). `smartctl` (GPL, smartmontools) and Windows' own `powercfg` are the approved external/OS tools. If a new external dependency is needed, flag it — don't add it silently.

**TypeScript/React:**
- Functional components + hooks only.
- All user-facing strings go through the `en.json`/`bn.json` i18n tables — never hardcode English strings directly in JSX, even placeholder ones, since retrofitting bilingual support later is much more painful than doing it from the start.
- Tailwind utility classes; no separate CSS-in-JS system.

**Testing:**
- Rust: `cargo test` for scoring logic and consistency-check rules — these are the highest-value tests since they're the actual fraud-detection product, not boilerplate.
- Frontend: Playwright for the scan → results flow once it exists; don't over-invest in UI tests before the UI direction is settled.

**Offline-first:** Every core diagnostic command must work with no network access. Anything that needs the internet (speed test, spec-DB update) must be clearly optional in both the UI and the code path — never a silent dependency.

## What "done" looks like for a feature

Before considering a diagnostic module complete: it runs on Prince's actual HP ProBook 440 G11 and returns real, sane values — not just "compiles and returns a struct." Hardware diagnostics are only as good as their ground-truth testing.

## Current phase

See `ROADMAP.md` — update the "current phase" marker there as work progresses, don't track progress only in chat history.
