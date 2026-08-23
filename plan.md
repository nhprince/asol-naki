# Asol Naki? (আসল নাকি?) — Project Plan

**Tagline:** "Is it genuine?" — a native Windows diagnostic and fraud-detection tool for second-hand device buyers.

**Owner:** Prince Khan (NH Prince)
**Status:** Planning → Phase 0
**Doc purpose:** Single source of truth for what this product is and why. Paired with `CLAUDE.md` (dev conventions for Claude Code) and `ROADMAP.md` (build order).

---

## 1. The problem

Buying a second-hand laptop in Bangladesh (and most used-device markets) means trusting the seller's word, or juggling five separate tools yourself: CrystalDiskInfo for the drive, a battery report tool, Cinebench for CPU sanity, manual key-by-key keyboard checks, and no real way to catch a relabeled CPU, a fake-capacity SSD, or a spoofed spec sheet. There's no single, automatic, trustworthy answer to "is this actually what they say it is, and is it worth the price?"

## 2. The product

A portable Windows `.exe`. Plug in nothing extra, run the app, click scan. It reads the machine's real hardware data directly from the OS and firmware, cross-checks it against itself for signs of fraud or spoofing, runs a few short guided checks for things software can't see (screen, keys, ports), and produces one clear score out of 10 with a plain-language verdict — "good buy," "negotiate the price," or "walk away, this is misrepresented."

**Primary user (v1):** Prince, on his own hardware, to validate the tool works.
**Primary users (launch):** everyday buyers checking a laptop before purchase.
**Future users:** resellers/shops running bulk checks across inventory.

## 3. Platform & scope

| Dimension | Decision |
|---|---|
| Device type (v1) | Laptops |
| Device type (future) | Smartphones (Android first — much more accessible than iOS) |
| OS target | Windows 10 & 11 (fully supported) |
| OS legacy | Windows 7/8.1 — best-effort only. Microsoft ended WebView2 support for these in Dec 2022; the runtime is frozen and unpatched. The app may run, but this is not a promised or tested target — don't spend development time chasing Win7-specific bugs. |
| Future platforms | Android (native, via Tauri mobile), Linux |
| Distribution | Portable single `.exe`, no install required for v1 |
| Connectivity | **Offline-first.** All core diagnostics run with zero internet. Internet is used only for: (1) optional internet speed test, (2) optional spec-database lookups/updates. The app must be fully useful with wifi off. |
| Language | Bilingual — English and Bengali from v1, toggle in-app |
| Monetization | Free during build/validation. Future: one-time purchase (~few hundred taka), no subscription. Licensing infrastructure is a v2+ concern (see §7). |

## 4. Tech stack

**Shell:** Tauri v2 (stable, v2.10.1+). Chosen over Electron for size/speed (~8MB vs ~150MB, ~0.3s vs ~2.5s startup) and because the same Rust core can extend to Android later without a rewrite — Electron has no mobile path at all.

**Frontend:** React + TypeScript + Vite + Tailwind CSS. This is Prince's existing skillset — no new language here. UI aesthetic: AI-directed based on best practice for a *trust-focused diagnostic tool* (leans cleaner/more clinical than pure glassmorphism, since the product's whole value prop is credibility — but should still feel premium and polished, not sterile). Bilingual via `react-i18next`.

**Backend/core:** Rust, written primarily by Claude Code with Prince reviewing/directing. Key crates and tools:
- `sysinfo` — CPU, RAM, OS, network interface data
- `smartctl` (smartmontools, GPL, bundled binary) — SMART disk health, invoked as a subprocess, parsed from its JSON output. This is the same underlying engine CrystalDiskInfo itself uses.
- `powercfg /batteryreport` (built into Windows, no bundling needed) — battery design capacity, full-charge capacity, cycle count
- WMI / SMBIOS queries (via `wmi` or `windows-rs` crate) — motherboard/BIOS model strings, EDID for real display specs
- Local JSON — scan history and reports for v1 (simple, fast to build). SQLite (`rusqlite`) is a clean v2 upgrade once bulk/reseller mode exists — not needed for v1.
- Local static JSON "known models" database — bundled spec sheet for common Bangladesh-market laptop models (HP ProBook, Dell Latitude/Inspiron, Lenovo ThinkPad/IdeaPad, Asus, Acer), used by the fraud-detection layer. Online-updatable later.

**Never bundle:** CrystalDiskInfo, HWiNFO, Cinebench, or other closed/restricted-license tools inside the shipped product. `smartctl` is fine (GPL, redistributable with credit); Windows' own `powercfg` is fine (OS-native).

## 5. Core features (v1 scope)

1. **System scan** — CPU, GPU, RAM, motherboard/BIOS model, OS version, all pulled automatically on launch.
2. **Storage diagnostics** — SMART health, real vs. claimed capacity (catches fake-capacity scams), TBW/wear, bad sector count, power-on hours.
3. **Battery diagnostics** — design vs. full-charge capacity, cycle count, wear percentage.
4. **Fraud/spoof detection layer** — internal consistency checks: does the reported CPU string match its actual core/thread/cache profile? Does reported SSD capacity match actual addressable sectors? Does the SMBIOS model number match what that model is supposed to have? This catches spoofing without needing an external performance-benchmark database (see §6).
5. **Display check** — real panel spec via EDID (catches "claimed 144Hz IPS, actually 60Hz TN"), plus a guided manual test for dead pixels and backlight bleed.
6. **Input/port checks** — guided keyboard key-press tester, USB/HDMI/audio port test, WiFi/Bluetooth adapter presence and basic function check.
7. **Optional online checks** — internet speed test, spec-database update fetch. Both skippable; app works fully without them.
8. **Scoring** — sub-scores per category rolled into one overall 1–10 score with a plain-language verdict (see §6).
9. **Reporting** — results shown in-app, exportable as a PDF, and exportable as a shareable image/screenshot. All three, not just one.
10. **Bilingual UI** — English/Bengali toggle, all strings externalized.

## 6. Fraud detection design (the core value prop)

**Do not build this as "benchmark score vs. expected score for model X."** That requires an external performance database (PassMark/UserBenchmark-style) that's either legally murky to source or takes years to build from real user data. It's a trap that delays v1 for no real gain.

**Build this instead as internal consistency checking:** spoofing tools typically fake *one* data source a buyer might glance at, but rarely fake every independent signal consistently. The app cross-references multiple signals against each other and against a small local spec-sheet database:

- CPU: reported model string vs. actual core count, thread count, and cache size
- Storage: reported capacity vs. actual addressable sector count
- GPU: reported model vs. actual VRAM and driver-reported identifiers
- Model identity: SMBIOS/BIOS model number cross-checked against the bundled "known models" JSON (what RAM type, display spec, and chipset that model is *supposed* to ship with)

**Flag severity:**
- **Critical** (e.g., spoofed storage capacity, CPU identity mismatch) — caps the overall score regardless of how good other categories look. A device with a critical fraud flag should never score above ~3/10, even with perfect battery and display.
- **Warning** (e.g., high battery wear, minor spec mismatch) — reduces the relevant category score but doesn't cap the overall.
- **Info** — surfaced in the report but doesn't affect scoring.

A basic custom CPU/GPU stress test can exist as a *secondary* signal (does it thermal-throttle immediately, does it hang) — useful, but not the fraud-detection backbone.

## 7. Scoring model (draft — expect to tune this with real-world testing)

Weighted category scores roll into one overall score:

| Category | Weight |
|---|---|
| Storage health | 25% |
| CPU/GPU performance sanity | 25% |
| Battery health | 20% |
| Display accuracy | 15% |
| Ports/connectivity | 10% |
| Fraud consistency | Gate (caps score on Critical flag; doesn't add positive weight) |

Output isn't just a number — it's a short breakdown per category plus one actionable sentence, e.g. *"7.7/10 — good buy, but battery is at 71% health, worth asking for ৳2–3k off."* Getting this genuinely trustworthy takes calibration against real devices over time; don't treat the initial weights as final.

## 8. Licensing & monetization (future, not v1)

Plan: one-time purchase, low price point (few hundred taka), no subscription. A purely offline license key is easy to build but trivial to crack. Given Prince already runs Cloudflare Workers + D1 for another project (Koshagar), a lightweight license-validation API on that same stack (Worker + D1, checks key + hardware fingerprint once, works offline after) is close to zero new infrastructure to learn. **Not needed until the product is validated** — build this in Phase 5, not before.

## 9. Open items to revisit as building progresses

- Exact "known models" list to bootstrap the fraud-detection spec database (start with common Bangladesh-market models, expand from there)
- Whether PDF export uses a Rust PDF crate or the webview's native print-to-PDF (simpler, worth trying first)
- Final scoring weights, once tested against real devices
- UI direction — confirm the AI-chosen aesthetic once a first draft exists, adjust from there
