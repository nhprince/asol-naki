# Progress Log — Asol Naki?

**Rule:** update this file after EVERY change (one row per commit, newest first).
ROADMAP checkboxes get ticked only when CI is green AND ground-truth verified.

**Current phase:** 1 — MVP diagnostics (in progress)

---

| Date (UTC) | Commit | Change | Verification |
|---|---|---|---|
| 2026-08-26 | 6a1f49c | **Ground-truth fixes #2 (ProBook round 2):** (1) battery — dropped powercfg entirely (XML flag still produced blank file); now queries WMI `ROOT\WMI` directly (BatteryStaticData.DesignedCapacity, BatteryFullChargedCapacity.FullChargedCapacity, BatteryCycleCount.CycleCount) via `WMIConnection::with_namespace_path`; (2) storage — keep output when smartctl exits non-zero (failing SMART sets exit code), explicit "run as Administrator" error; (3) artifact zip now stages `smartctl/` as sibling of exe so the resolver finds it | E2E 32962604136 ✅ 5/5 · artifact layout confirmed: `asol-naki.exe` + `smartctl/smartctl.exe` |
| 2026-08-26 | 2d79bfc | **Ground-truth fixes #1 (Prince's ProBook):** (1) storage — smartctl path candidates for artifact-zip layouts (`smartctl/`, flattened, `resources/smartctl/`, PATH); correct `--scan --json` then `--all` per device instead of `--scan --json --all`; (2) battery — `/XML` before `/output` (flag order bug made powercfg emit HTML), defensive HTML-detection error | E2E 32959775823 ✅ 5/5 on windows-latest · local tsc/vitest(8) ✅ |
| 2026-08-26 | ec01ff0 | **Windows-only build fix:** winreg has no `Vec<u8>` FromRegValue — EDID now read via `get_raw_value` + REG_BINARY type check. Caught only by the Windows E2E compile (Linux CI blind spot, by design) | E2E 32957204571 ✅ **5/5 WebDriver on windows-latest** |
| 2026-08-26 | 137e900 | **P2 display.rs COMPLETE:** EDID registry reader (winreg, Windows-gated) + VESA parser (checksum/header validation, PnP vendor decode, product/serial, week/year, diagonal, preferred-timing → native res + refresh Hz). `scan_display` command; UI DisplayCard (vendor/resolution/refresh/diagonal/manufacture week-year) EN/BN; useScan 4th parallel section (silent-skip off Windows). Test-fixture journey: PnP slice-index bug → decode_pnp_value(); gaming fixture needed true CVT-RB math (64_612 units ≤ u16); fixture u32→u16 LE write fix. 7 display unit tests | CI 32955672413 ✅ **38 rust tests** · local tsc/vitest(8)/build ✅ |
| 2026-08-24 | 3e64461 | **DEEP SCAN fixes:** (1) score-cap now mirrors Rust exactly — round after cap, single setState; (2) dead template `App.css` removed; (3) `icon-master.png` moved out of repo root → docs/design/; (4) README phase-status table synced with reality; i18n EN↔BN parity + Rust→i18n message_key contract verified programmatically | CI 32690609136 ✅ · local tsc/vitest(8)/build ✅ |
| 2026-08-24 | c9a6206 | **P3 PDF:** Save-as-PDF via WebView2 print pipeline (zero new deps) + print stylesheet (paper-friendly report layout) | CI 32689390378 ✅ · local tsc/build ✅ |
| 2026-08-24 | 7cfc83f | **P3 UI:** ReportCard — copy-to-clipboard share summary (clipboard API + execCommand fallback) | CI 32689017941 ✅ · local tsc/vitest ✅ |
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
