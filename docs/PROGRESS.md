# Progress Log — Asol Naki?

**Rule:** update this file after EVERY change (one row per commit, newest first).
ROADMAP checkboxes get ticked only when CI is green AND ground-truth verified.

**Current phase:** 1 — MVP diagnostics (in progress)

---

| Date (UTC) | Commit | Change | Verification |
|---|---|---|---|
| 2026-08-24 | ccd487f | **FIX:** CI was red on Ubuntu job since smartctl bundling — tauri resources glob `resources/smartctl/*` matched nothing on Linux. Added README placeholder; Windows builds unaffected | CI 32688664344 ✅ (both jobs) |
| 2026-08-23 | e912c7f | feat(report): ScanReport model + shareable summary text (Phase 3 groundwork); 4 tests; capacity GiB-floor to match Explorer | local tsc/vitest/build ✅ · E2E 32662622834 ✅ (CI had unrelated Linux glob failure, see above) |
| 2026-08-23 | eeebd03 | docs: Phase 2 exit gate proven on Windows (5/5 E2E) | E2E 32666949583 ✅ (nightly) |
| 2026-08-23 | aa88a49 → 7ea8905 | smartctl bundling: fetch-smartctl.ps1 (choco, pinned 7.5, GPL license shipped), tauri.conf resources, artifact includes smartctl/ | E2E runs ✅ |
| 2026-08-23 | 5f3d7ee | **P2 UI wiring:** useScan → run_integrity_checks, Critical cap applied to displayed score (3.0), fraud-flags panel with severity badges EN/BN | CI 32659583205 ✅ · local tsc/vitest/build ✅ |
| 2026-08-23 | 3c9eb30 | **PHASE 2 START — fraud engine:** integrity.rs (CPU thread/core identity vs OS counts → Critical; fake-NVMe-capacity + absurd-capacity heuristics), models_db.rs (11 BD-market CPUs embedded, case-insensitive lookup), `run_integrity_checks` command (JSON-in → E2E can feed spoofed fixtures). **Exit gate proven in tests:** spoofed CPU fixture raises Critical, score capped at 3.0 | CI 32659287426 ✅ (26 rust tests) |
| 2026-08-23 | cd9376e | **UI:** full scan (parallel `Promise.allSettled`, per-section errors), score card + verdict colors, storage/battery/WMI sections, BN digits | CI 32655584137 ✅ · local tsc/vitest/build ✅ |
| 2026-08-23 | cecaf16 | **hardware.rs full pull:** WMI GPU/motherboard/BIOS/physical-cores (Windows-gated, degrades to None honestly), `scan_hardware_full` command. Fixes: rustfmt, unused_mut allow | CI 32655208207 ✅ |
| 2026-08-23 | 5b3d50e | **storage.rs:** smartctl JSON parser (NVMe+ATA), wear scoring (realloc/pending/media/percentage_used), `scan_storage` command, 5 tests. Fixes: percentage_used field name, clippy lints, f64 annotation | CI 32654628579 ✅ (19 rust tests) |
| 2026-08-23 | c58d01a | scoring.rs: weighted 0-10 engine, critical-fraud cap @3.0, battery/storage sub-score curves (9 tests). Fix: normalize by weight sum — plan §7 weights total 0.95 (fraud = gate), test now locks spec | CI 32653725221 ✅ (14 rust tests) |
| 2026-08-23 | 73f3c47 → 85f96d5 | battery.rs: powercfg /XML parser (fixture-tested, tolerant); Windows shell-out isolated; `scan_battery` command; rustfmt applied | CI ✅ after fmt + test fixes |
| 2026-08-23 | a6ce732 | docs/PROGRESS.md created; session discipline added to CLAUDE.md | push ✅ |
| 2026-08-23 | a06d5ee | ROADMAP Phase 0 boxes marked; CI+E2E declared green | CI 32632023384 ✅ · E2E 32631550731 ✅ (3/3 on Windows) |
| 2026-08-23 | 03186a3 | e2e: deterministic JS-click i18n toggle test | E2E ✅ first fully-green WebDriver run |
| 2026-08-23 | c435626 → 5adcdce | e2e fixes: screenshots dir, capability shape (`browserName:"tauri"`), named export, lockfile, scoped service pkg | each verified by its CI run |
| 2026-08-23 | 2820b99 | e2e: switch to @wdio/tauri-service **embedded mode**; feature-gate `wdio` cargo feature (test driver never ships in releases) | CI ✅ |
| 2026-08-23 | 9443a11 | ROADMAP progress + debug-exe artifact upload for ground-truth testing | CI ✅ |
| 2026-08-23 | ee01a58 | fix: drop `cpu_cores` (removed in sysinfo 0.37); WMI takes over in Phase 1 | CI ✅ after red clippy run |
| 2026-08-23 | da43dea | Scaffold: Tauri v2 + React 19 + Tailwind v4 + i18n EN/BN; POC `scan_hardware_basic`; 3 workflows; icons; docs trio + master plan | CI ✅ after clippy fix |

## Phase status

- **Phase 0 — DONE** (pending Prince's ground-truth checklist on ProBook; exe artifact downloadable from any green E2E run)
- **Phase 1 — IN PROGRESS**: full hardware pull (WMI GPU/motherboard), storage.rs (smartctl JSON), battery.rs (powercfg XML), scoring.rs (weighted formula), minimal results UI
