# Asol Naki? — Master Development Plan

**Owner:** Prince Khan (NH Prince) · **Prepared by:** Saturday · **Date:** 2026-08-23
**Status:** Locked v1.1 — decisions made (repo: **PUBLIC**). Ready to execute.

---

## 1. What we're building (my understanding)

A portable Windows `.exe` ("Asol Naki?" / আসল নাকি?) that a second-hand laptop buyer runs
**before handing over money**. One click scans CPU/RAM/GPU/storage/battery/display,
cross-checks every claim against independent signals (fraud/spoof detection = the core IP),
and outputs a 1–10 trust score plus a plain-language verdict: **buy / negotiate / walk away**.
Bilingual EN/বাংলা from day one. Offline-first. Built with Tauri v2 (Rust core + React UI).

**Your real intention (as I read it):**
1. Ship a genuinely *useful* product for the Bangladesh used-laptop market — not a toy demo.
2. Do it professionally: real repo, real CI, real tests, clean phases, revertable history.
3. Work within hard physical constraints: **Arch Linux dev box, weak hardware, Windows-only target.**
4. Never get blocked by the machine: **GitHub Actions absorbs every heavy task** (builds, Windows compilation, E2E).
5. Fraud detection = internal consistency checking (per `plan.md` §6) — NOT benchmark databases. This is the moat; everything else is plumbing around it.

## 2. The core constraint and the workflow it dictates

| Constraint | Consequence |
|---|---|
| Dev OS = Arch Linux | No native Windows APIs. Everything Windows-specific must be verified elsewhere. |
| Target OS = Windows 10/11 only | Every WMI/EDID/powercfg code path compiles and runs ONLY on Windows. |
| Weak dev machine (<16GB RAM assumed) | No full `cargo build` of the Tauri stack locally. Even `cargo check` is slow on cold cache. |
| GitHub Actions available | windows-latest runners = our real build/test machines. |

### The Golden Rule that follows:

> **The local machine writes code and runs light checks only.
> GitHub Actions compiles, verifies, tests on Windows, and produces every artifact.
> Nothing ships that hasn't passed a green Windows CI run.**

Division of labor per task type:

| Task | Where it happens |
|---|---|
| React/TS UI work, Vite dev server, i18n tables | Locally (fast, low RAM) |
| Rust logic authoring (scoring, integrity rules) | Written locally, verified by CI |
| `cargo check` / `clippy` / `test` | CI primarily; locally optional with sccache |
| Windows compilation (`tauri build`) | CI only (windows-latest) |
| E2E tests (real app window) | CI only (windows-latest + tauri-driver) |
| Release artifacts (.exe/.msi) | CI only, attached to GitHub Releases |
| Ground-truth sanity ("does this match reality?") | Prince, manually, on his HP ProBook 440 G11 |

## 3. Key architecture decisions

1. **100% CI for Windows builds.** No local cross-compilation experiment (cargo-xwin/xwin
   on Arch is fragile and eats your day when it breaks). windows-latest runners are free
   on public repos and reliable. Decision: don't fight it locally at all.
   *(If you later want offline local builds, we revisit — timeboxed.)*
2. **Diagnostic provider abstraction (the testability keystone).**
   In Rust, define `trait DiagnosticsProvider` with two implementations:
   - `RealProvider` — sysinfo / smartctl / powercfg / WMI / EDID (used in production).
   - `FixtureProvider` — loads recorded JSON snapshots from real machines (used in ALL automated tests).
   Selected via env var (`ASOL_PROVIDER=fixture`) or cargo feature.
   This single pattern makes fraud-detection rules deterministically testable (feed it a
   synthetic spoofed-device snapshot → assert the Critical flag fires → satisfies the
   Phase 2 exit criterion without owning spoofed hardware) and makes E2E stable.
3. **Fixture capture mode.** A `--capture-fixtures` flag records the raw output of every
   diagnostic into sanitized JSON files. You run it once on your ProBook; we commit the
   fixtures; CI forever replays them. Each new machine tested adds fixtures = growing regression net.
4. **Testing pyramid (honest version):**
   - **Layer 1 — Logic tests (bulk, automated):** `cargo test` for scoring.rs + integrity.rs
     against fixtures; `vitest` for TS utilities. Runs everywhere including your laptop.
     *This is where 90% of test effort goes — it IS the product.*
   - **Layer 2 — E2E smoke (small, automated, CI-only):** `tauri-driver` + WebDriver on a
     windows-latest runner drives the REAL compiled app: launches, scan button → results
     render (fixture provider), language toggle EN↔BN, history persists. Deliberately kept
     to ~3–8 flows. Nightly + on PRs touching src/.
   - **Layer 3 — Ground-truth protocol (human):** scripted checklist per phase on your
     ProBook comparing app numbers vs hand-verified truth (smartctl directly, powercfg
     directly, spec sheet). Cannot and should not be automated. This is the "done" bar
     from CLAUDE.md.
5. **Smartctl distribution:** NOT committed to the repo. CI downloads the official
   smartmontools Windows binaries pinned to a fixed version + SHA256 verification, stages
   into resources before `tauri build`. Keeps repo clean, upgrades auditable, GPL credit
   file shipped alongside (see risk R7).
6. **Repo visibility: PUBLIC — DECIDED (Prince).** Public repos get unlimited free Actions
   minutes INCLUDING Windows runners, so nightly E2E stays free forever. License: no LICENSE
   file for now (default = all-rights-reserved); revisit open-sourcing at Phase 5.
7. **Trunk-based flow, gated by CI:** short-lived feature branches → PR → all CI jobs green
   → squash merge to `main`. `main` is ALWAYS releasable. Tags `v*` trigger release builds.

## 4. Repository layout (extends CLAUDE.md's tree)

```
asol-naki/
├── plan.md, ROADMAP.md, CLAUDE.md      # existing docs (unchanged roles)
├── docs/
│   ├── DEVELOPMENT-PLAN.md             # THIS file
│   └── ground-truth/                   # per-phase manual test checklists + results
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                      # lint + typecheck + tests + Windows build (every PR/push)
│   │   ├── e2e.yml                     # tauri-driver E2E on windows-latest (PRs + nightly)
│   │   └── release.yml                 # tagged builds → GitHub Releases (tauri-action)
│   └── dependabot.yml                  # npm + cargo updates weekly
├── src/                                # React + TS + Tailwind + i18n (en.json/bn.json)
├── e2e/                                # WebDriver E2E specs (run against built app in CI)
├── fixtures/                           # captured hardware JSON snapshots (sanitized)
├── scripts/
│   └── fetch-smartctl.ps1              # CI: pinned download + SHA256 verify + stage
└── src-tauri/
    ├── src/                            # hardware.rs, storage.rs, battery.rs, display.rs,
    │                                   # integrity.rs, scoring.rs, models_db.rs
    ├── src/provider/                   # DiagnosticsProvider trait + Real/Fixture impls
    ├── tests/                          # cargo integration tests (fixtures-driven)
    └── resources/known_models.json
```

## 5. CI/CD design

### ci.yml — every push/PR (fast gate)
Jobs run in parallel:
- **frontend** (ubuntu): `pnpm install`, `tsc --noEmit`, eslint, `vitest run`, `vite build`.
- **rust-tests** (ubuntu): `cargo fmt --check`, `cargo clippy -- -D warnings`,
  `cargo test` (logic tests are pure + fixture-driven → cross-platform safe).
  Aggressive caching: `Swatinem/rust-cache` + pnpm store cache.
- **windows-build** (windows-latest): fetch-smartctl.ps1 → `cargo build` of the Tauri app
  (debug profile — faster). Purpose: prove Windows-only code COMPILES on every change;
  no artifacts kept except on demand.

Concurrency group cancels superseded runs on the same branch (saves minutes).

### e2e.yml — PRs touching `src/**`/`src-tauri/**`/`e2e/**` + nightly cron on main
Single job on windows-latest:
1. Checkout + caches.
2. `fetch-smartctl.ps1`.
3. Build app: `npm run tauri build -- --debug` (debug = much faster, symbols for failures).
4. Download matching `msedgedriver` (tauri-driver on Windows drives WebView2 via Edge driver).
5. Start `tauri-driver`, run `e2e/` suite (WebDriver client in Node via `selenium-webdriver`
   or Rust via `thirtyfour` — decided in Phase 0b, see Q4).
6. On failure: upload screenshot/video/trace + app log as workflow artifacts (you debug
   from the artifacts, never from your laptop).
7. Suite stays SMALL (3–8 flows): launch, scan→results (fixtures), toggle BN, verdict shown.
   Flaky-rate policy: 1 retry, then fail loudly; investigate same day, don't let it rot.

### release.yml — on tag `v*`
- `tauri-apps/tauri-action@v0`: builds Windows bundles (NSIS portable/installer + MSI),
  creates/updates a **draft** GitHub Release with artifacts + generated notes.
- You review the draft, write 3 lines of changelog, publish.
- Unsigned binaries initially → SmartScreen will warn (risk R9) until/unless we buy a
  code-signing cert (costs money — per your no-payment rule, we document the bypass instead).

### Minutes budget (non-issue — repo is public)
Public repos get unlimited free Actions minutes, Windows included. Typical PR ≈ 10 min
wall-clock across 3 parallel jobs; nightly E2E ≈ 15 min. Zero budget pressure. If the repo
ever flips to private: 3,000 min/mo cap with Windows billed 2× — downgrade nightly E2E to
weekly FIRST, and keep concurrency-cancel + aggressive caching permanently.

## 6. Development workflow (the daily loop)

```
Pick one bite-sized task (from phase checklist / issue)
  → branch feat/<slug> off main
  → write failing test FIRST for logic (cargo test / vitest)   [where applicable]
  → implement minimal code
  → cheap local checks only:  pnpm vitest run  /  tsc --noEmit
    (Rust: optionally `cargo check -p <crate>` if machine tolerates; otherwise skip)
  → commit conventional style (feat:/fix:/test:/chore:) — small commits, often
  → push branch, open PR → CI runs the FULL gate (incl. Windows compile + E2E if touched)
  → green? squash-merge. red? read logs/artifacts, fix, re-push.
  → phase complete? run ground-truth checklist on ProBook, tick ROADMAP.md boxes,
    bump "Current phase", tag prerelease vX.Y.Z-phase.N
```

Rules that keep this foolproof:
- **Never merge red.** main must stay green — it's your rollback point (your known-good-commit habit, institutionalized).
- **One concern per PR.** Small diffs = CI failures are obvious.
- **Claude Code owns Rust implementation details; you own product calls + ground-truth testing** (as CLAUDE.md says). I (Saturday) own orchestration, CI, and keeping docs truthful.
- **ROADMAP.md is the live tracker** — checkbox ticked only when BOTH CI-green AND ground-truth-verified.

## 7. Phase-by-step execution plan (maps onto ROADMAP.md)

### Phase 0a — Foundation (new session #1, ~half a day of wall-clock, mostly waiting on CI)
1. Local (Arch): `sudo pacman -S --needed nodejs npm git github-cli`; enable `pnpm` via corepack.
   **No Rust toolchain installed locally at all** (decision D1 — CI compiles).
   *(Reversible: `pacman -S rustup` later if you want local cargo check.)*
2. Auth: `gh auth login` (device flow) — fixes the unauthenticated gh I found today.
3. Scaffold repo `asol-naki/` exactly per §4 tree: `pnpm create tauri-app` (React-TS template),
   add Tailwind, react-i18next with en.json/bn.json stubs, .gitignore, .gitattributes. No LICENSE file (Decision #3).
4. Write the three workflow files (§5). First push → watch all green.
5. POC command `scan_hardware_basic` returning CPU name → displayed in UI (original Phase 0 goal).
   On YOUR laptop: verify by running the CI-built debug exe (CI uploads it as an artifact on demand).
6. Commit docs (plan.md, ROADMAP.md, CLAUDE.md, docs/DEVELOPMENT-PLAN.md).

**Exit criteria (extended):** all three workflows exist & green; artifact exe runs on your
laptop showing your real CPU name; repo pushed to GitHub.

### Phase 0b — Test harness skeleton (session #2)
1. Introduce `DiagnosticsProvider` trait + `FixtureProvider` (empty fixtures OK) behind env var.
2. `--capture-fixtures` mode; run on ProBook → commit first real fixtures (sanitized: no serials/MACs).
3. e2e.yml with ONE smoke test: app launches → window title/main view visible (via tauri-driver).
4. dependabot on.
**Exit:** nightly + PR E2E green; fixtures replay through provider in a unit test.

### Phase 1 — MVP (unchanged scope, new discipline)
hardware.rs / storage.rs / battery.rs / scoring.rs + minimal UI. Each module:
logic tests against fixtures FIRST, Windows compile proof in CI, then ground-truth
hand-verification on your ProBook before its ROADMAP checkbox ticks.

### Phase 2 — Fraud detection (the heart)
integrity.rs rules are PURE functions over provider data → exhaustively unit-tested with
synthetic corrupted fixtures (spoofed capacity, mismatched CPU identity, model mismatch).
Score-cap logic property-tested. E2E adds one flow: fixture-provider device with critical
flag → verdict shows walk-away. Guided display/keyboard/port screens + full bilingual pass.

### Phase 3 — Reporting & polish
PDF via webview print-to-PDF first (per plan.md §9), shareable image export, optional speed
test module clearly quarantined behind explicit user action (offline-first rule enforced
by a lint-level convention: no network imports outside `optional_online/` module).

### Phase 4 — Distribution
Portable exe via release.yml; embedBootstrapper webview mode; test on real Win10 + Win11
machines; soft-test with 2–3 buyers/resellers; feed their real devices back as fixtures.

### Phase 5 — Future (unchanged from ROADMAP.md)

## 8. Risk register — every problem I foresee, with mitigations

| # | Risk | Mitigation |
|---|------|-----------|
| R1 | Windows-only code breaks Ubuntu CI jobs (WMI etc.) | All platform-specific code behind `#[cfg(windows)]` with trait-absttracted providers; ubuntu job runs ONLY pure logic; windows job proves compilation. |
| R2 | Local machine too slow to even `cargo check` | Accepted: CI is the compiler. Optional escape hatch: `rustup + sccache` later, timeboxed. Frontend never touches Rust toolchain → instant local UI iteration. |
| R3 | tauri-driver/E2E flaky on CI | Keep suite ≤8 deterministic flows; fixtures not real hardware; 1 retry; artifacts on failure; nightly separate from PR gate so flake doesn't block merges. |
| R4 | WebView2/Edge driver version drift breaks msedgeddriver pairing | CI resolves driver version from installed WebView2 runtime each run (documented tauri-driver Windows flow); pinned runner image = stable. |
| R5 | Actions minute overrun | RESOLVED — repo is public (unlimited minutes incl. Windows runners). If ever flipped private: nightly E2E → weekly first, plus concurrency-cancel + aggressive caching. |
| R6 | smartctl GPL/redistribution concerns | Ship LICENSE + source-offer text alongside bundled binary; pin version+SHA256 in CI script; never vendor modified smartctl. |
| R7 | Antivirus/SmartScreen flags unsigned portable exe | Expected & documented in README + in-app FAQ ("More info → Run anyway"); signing cert deferred (costs money — your no-pay rule); revisit if soft-testers complain loudly. |
| R8 | powercfg/smartctl output varies across Windows versions/drives | Parse defensively (serde with tolerant types); every unparseable real-world sample becomes a fixture + regression test. |
| R9 | Bengali rendering broken on stock Win10 (font gap) | Bundle Noto Sans Bengali in app assets from Phase 0a; never rely on system fonts for BN strings. |
| R10 | Scope creep / motivation decay on side project | Phase gates with exit criteria; no Phase N+1 work before gate passes; ROADMAP.md is the single progress truth; every session starts by reading the 3 docs (already your CLAUDE.md rule). |
| R11 | Secrets leak via fixtures (serial numbers, MACs) | Sanitizer pass in capture mode (drop serials, MACs, usernames); fixtures reviewed before commit. |
| R12 | Claude Code context loss between sessions | Already solved by your doc trio + this plan; sessions start with "read plan.md + ROADMAP.md + CLAUDE.md"; phase marker kept current. |

## 9. Immediate action checklist (decisions locked — go time)

- [ ] `gh auth login` on this box
- [ ] Create GitHub repo (name: `asol-naki` unless you say otherwise)
- [ ] Scaffold Tauri v2 + React-TS + Tailwind + i18n skeleton
- [ ] Write ci.yml / e2e.yml / release.yml + fetch-smartctl.ps1
- [ ] First green CI run
- [ ] POC: CPU name from Rust shown in window (artifact exe tested on your ProBook)
- [ ] Update ROADMAP.md Phase 0 checkboxes + current-phase marker

## 10. What deliberately does NOT happen (YAGNI guards)

No SQLite, no licensing API, no updater server, no code-signing purchases, no benchmark
database, no macOS/Linux targets, no local cross-compiling, no monorepo tooling, no
component-library dependency — all deferred per plan.md §8/§9 and ROADMAP Phase 5.

## 11. Success metric for the PLAN itself

You can start any session, read only the three project docs + this file, and know exactly
what to do next without asking anyone anything. Every heavy operation has an owner (CI),
every artifact has a path (Releases/artifacts), every phase has a non-fakeable exit gate.

## 12. DECISIONS LOG

| # | Question | Decision |
|---|----------|----------|
| 1 | Repo visibility | **PUBLIC** (Prince's call, 2026-08-23). Unlimited Windows CI minutes; nightly E2E stays. |
| 2 | 100%-CI Windows builds, no local Rust toolchain | Confirmed — was Prince's own requirement (weak device; Actions absorbs all heavy tasks). |
| 3 | License | No LICENSE file for now → default all-rights-reserved. Revisit at Phase 5 if open-sourcing desired. |
| 4 | E2E WebDriver client | Node (`selenium-webdriver`) — lives in Prince's JS/TS comfort zone. |
| 5 | Repo name | `asol-naki`. App display name unchanged: "Asol Naki? আসল নাকি?" |
| 6 | Phase ordering / exit criteria | Approved as-is — no objections raised. |
