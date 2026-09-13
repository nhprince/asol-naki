# Comprehensive Analysis and Production Plan — Asol Naki? (আসল নাকি?)

**Prepared for:** Prince Khan (NH Prince)
**Project:** Asol Naki? — Native Windows Hardware Diagnostic & Fraud Detection App
**Architecture:** Tauri v2 (Rust Core) + React 19 + TypeScript + Vite + Tailwind CSS
**Target Environment:** Windows 10 & 11 (Offline-First Portable `.exe`)
**Development Environment:** Arch Linux (CI-driven execution via GitHub Actions `windows-latest`)

---

## Executive Summary & Deep Finding Report

Following a deep scan and analysis of all code, configuration, workflow, and documentation files (`plan.md`, `ROADMAP.md`, `CLAUDE.md`, `docs/DEVELOPMENT-PLAN.md`, `docs/PROGRESS.md`), here is the complete state of the project and the technical blueprint to take it to production.

---

### 1. Existing System Architecture & Code Base Findings

#### **Frontend (`src/`)**
- **React 19 + TypeScript + Tailwind CSS v4 + `react-i18next`:** Fully structured, type-safe, and internationalized (English and Bengali `bn-BD` localization with Noto Sans Bengali font).
- **Core Hook (`useScan.ts`):** Runs parallel async Tauri commands (`scan_hardware_full`, `scan_battery`, `scan_storage`, `scan_display`, `run_integrity_checks`), gracefully tolerates per-section failures (`Promise.allSettled`), and aggregates weighted sub-scores and critical score caps.
- **Reporting & Export Components:**
  - `ReportCard.tsx`: Formats a concise summary and offers copy-to-clipboard functionality with fallbacks.
  - `PrintButton.tsx` / `print.css`: Utilizes native webview print-to-PDF with clean media queries.

#### **Backend Core (`src-tauri/`)**
- **Hardware Module (`hardware.rs`):** Leverages `sysinfo` and WMI queries for CPU, RAM, GPU, motherboard, BIOS, and OS information.
- **Storage Module (`storage.rs`):** Shells out to bundled `smartctl` with automated UAC elevation handling (`ShellExecuteW runas`) for Windows direct physical drive queries (`PhysicalDrive0..15`).
- **Battery Module (`battery.rs`):** WMI `ROOT\WMI` reader for designed capacity, full-charge capacity, and cycle counts.
- **Display Module (`display.rs`):** EDID reader via Windows registry (`winreg`) with VESA header/PnP checksum parsing for native resolution, refresh rate, diagonal size, and manufacture date.
- **Fraud Detection Engine (`integrity.rs` & `models_db.rs`):** Evaluates reported model identities against physical thread/core counts and capacity metrics. Caps overall score to max `3.0` if any `Critical` flag is raised.
- **Scoring Engine (`scoring.rs`):** Computes normalized 0–10 trust scores based on weighted sub-scores (Storage 25%, CPU/GPU 25%, Battery 20%, Display 15%, Ports/Input 10%).

#### **CI/CD & Testing Infrastructure (`.github/workflows/`)**
- `ci.yml`: TypeScript typechecks, Vitest unit tests, Rust `clippy`/`fmt`, and cross-compilation on `windows-latest`.
- `e2e.yml`: Full automated E2E WebDriver tests via `tauri-driver` running on `windows-latest`.
- `release.yml`: Automatic draft releases with portable NSIS `.exe` and `.msi` installers when tagged `v*`.

---

### 2. Gap Analysis for Production Readiness

To take **Asol Naki?** from its current Phase 1/2 state to a 100% feature-complete, production-grade application, the following additions are required:

1. **Phase 2 Guided Interactive Diagnostic Screens:**
   - **Display Diagnostic Modal:** Interactive full-screen color tester (Red, Green, Blue, White, Black, Grid) for detecting dead pixels, stuck pixels, and backlight bleeding.
   - **Keyboard Diagnostic Matrix:** Interactive keyboard layout component that listens to keydown events and visually highlights tested keys (supporting standard ANSI 60%/TKL/Full laptop layouts).
   - **Port & Connectivity Checklist / Status Screen:** WiFi, Bluetooth, Audio, HDMI, USB port presence and interactive status confirmation.

2. **Phase 3 Polish & Image Export:**
   - **Shareable Image Report Export:** Client-side HTML-to-Canvas / PNG generator (`html-to-image` or HTML5 Canvas rendering) allowing users to save or share a visual diagnostic verdict card.

3. **Phase 3 Optional Modules (Offline-Safe & Opt-In):**
   - **Optional Network Speed Test:** User-triggered ping / download speed test with explicit offline/online checks.
   - **Optional Spec DB Update Fetch:** Capability to check for updated `known_models.json` definitions online without disrupting offline operations.

4. **Phase 4 Production Distribution & Deployment:**
   - Configure `webviewInstallMode: embedBootstrapper` in `tauri.conf.json`.
   - Update all `en.json` and `bn.json` i18n translation keys.
   - Update `ROADMAP.md` and `docs/PROGRESS.md` with complete verification history.

---

## Production Execution Plan

Below is the step-by-step master plan to execute these additions, test them, and release the software.

1. **Guided Manual Diagnostic Screens (Display & Keyboard)**
   - Implement `DisplayTestModal.tsx` for full-screen dead pixel and backlight bleed inspection.
   - Implement `KeyboardTestModal.tsx` for keydown tracking and visual keyboard layout highlighting.
   - Implement `PortTestModal.tsx` for port/connectivity checks.
   - Integrate interactive test completion flags into `useScan.ts` scoring calculations.

2. **Shareable Image Export & Optional Modules**
   - Add image export capability to `ReportCard.tsx` generating downloadable PNG diagnostic cards.
   - Add user-triggered optional Network Speed Test and Spec DB Update modules (strictly optional and offline-friendly).

3. **i18n, Polish, & Distribution Packaging**
   - Add all corresponding translation keys in `en.json` and `bn.json`.
   - Ensure `tauri.conf.json` is configured for single-file portable release (`embedBootstrapper`).

4. **Complete Pre-Commit Instructions**
   - Run local typechecks (`npm run typecheck`), unit tests (`npm test`), and verify all modified files.

5. **Submission & Release Workflow**
   - Commit all changes with standard conventional commit messages, push to GitHub, and let CI/CD verify E2E WebDriver tests on `windows-latest`.
