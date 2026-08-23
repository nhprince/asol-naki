# Roadmap — Asol Naki?

No fixed calendar — this is a flexible-pace side project. Phases are ordered by dependency, not by date. **Current phase: Phase 0.**

Update the "Current phase" line above as work progresses.

---

## Phase 0 — Setup

- [ ] Scaffold Tauri v2 + React + TypeScript + Vite + Tailwind project
- [ ] Confirm `npm run tauri dev` runs a blank window on Prince's machine
- [ ] Set up `react-i18next` with empty `en.json`/`bn.json` tables
- [ ] Basic Rust proof-of-concept: one Tauri command that calls `sysinfo` and returns CPU name to the frontend, displayed as plain text
- [ ] Git repo initialized, `.gitignore` for `target/`, `node_modules/`, `dist/`

**Exit criteria:** a running window that shows your real CPU name, pulled from Rust.

## Phase 1 — MVP (personal use only)

- [ ] `hardware.rs` — full CPU/RAM/GPU/OS/motherboard pull via `sysinfo` + WMI
- [ ] `storage.rs` — SMART health via bundled `smartctl`, parsed JSON output
- [ ] `battery.rs` — parse `powercfg /batteryreport` output
- [ ] `scoring.rs` — first-pass scoring formula per §7 of `plan.md`
- [ ] Minimal UI: scan button → results screen showing raw category scores
- [ ] Test against Prince's actual laptop, sanity-check every number by hand

**Exit criteria:** running the app on your own laptop gives numbers you can independently verify are correct.

## Phase 2 — Fraud detection + remaining hardware checks

- [ ] `known_models.json` — bootstrap with common Bangladesh-market laptop models
- [ ] `integrity.rs` — consistency-check rules per §6 of `plan.md` (CPU identity, storage capacity, GPU identity, model cross-reference)
- [ ] Critical/Warning/Info flag severity and score-capping logic
- [ ] `display.rs` — EDID read + guided manual dead-pixel/backlight-bleed test screens
- [ ] Guided keyboard/port/WiFi/Bluetooth test screens
- [ ] Bilingual pass — every string in the app now has an EN and BN entry

**Exit criteria:** the app catches at least one deliberately-misrepresented test case (e.g. manually report a wrong capacity and confirm the flag fires).

## Phase 3 — Reporting & polish

- [ ] Results UI redesign to final aesthetic direction
- [ ] PDF export (try webview print-to-PDF first before adding a new Rust dependency)
- [ ] Shareable image/screenshot export
- [ ] Optional internet speed test module (clearly marked optional, works with wifi off otherwise)
- [ ] Optional online spec-database update fetch

**Exit criteria:** a full scan produces a report you'd be comfortable handing to a stranger to justify a price negotiation.

## Phase 4 — Distribution readiness

- [ ] Package as portable `.exe` via `tauri-bundler`
- [ ] `webviewInstallMode: embedBootstrapper` for broadest compatibility
- [ ] Test on a real Windows 10 machine and a real Windows 11 machine (not just Prince's dev box)
- [ ] Soft-test with 2–3 real second-hand buyers/resellers for scoring calibration feedback

**Exit criteria:** someone other than Prince runs it successfully with zero setup help.

## Phase 5 — Future expansion (post-validation)

- [ ] Licensing: Cloudflare Worker + D1 key validation (one-time purchase model)
- [ ] Reseller bulk-check mode
- [ ] Migrate local storage from JSON to SQLite (`rusqlite`) if bulk mode needs it
- [ ] Android support via Tauri mobile + ADB-based diagnostics
- [ ] Linux support
- [ ] Expand `known_models.json` via online updates, informed by real scan data







analyze all the doc files, understand all of them, understand my real intention, i need to build this software, need to make a solid step by step process and foolproof planning to complete the project in a professional way, for your information i need the software mainly for windows but my development environment is arch linux, also my device is weak so it cant handle heavy loads and builds, we need to use github actions actively for heavy tasks, also i want full automated e2e test using github actions, now understand everything and find the proper solution of all possible problems, figure out the exact process, plan of the development workflow, then make a plan, save a copy of the plan on docs folder, ask me question for clarification and the best result.
