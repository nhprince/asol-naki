# Asol Naki? — আসল নাকি?

**"Is it genuine?"** A portable Windows app that diagnoses second-hand laptops, detects
spec fraud/spoofing via internal consistency checks, and gives a 1–10 trust score with a
plain-language verdict: buy / negotiate / walk away.

> ⚠️ **Early development — Phase 0.** Not usable yet. See [ROADMAP.md](ROADMAP.md).

## Status

| Phase | Scope | Status |
|---|---|---|
| 0 | Scaffold + CI/CD + POC | ✅ done (CI+E2E green; ProBook checklist pending) |
| 1 | MVP diagnostics | 🟢 code complete — ground-truth pending |
| 2 | Fraud detection | 🟢 engine + E2E spoof gate done — display/input screens pending |
| 3 | Reporting & polish | 🟠 model/text/copy/PDF done — shareable image export remains |
| 4 | Distribution readiness | ⚪ pipeline ready, not tagged |
| 5 | Licensing / reseller / Android | future |

## Docs

- [plan.md](plan.md) — product plan ("why")
- [ROADMAP.md](ROADMAP.md) — build order
- [docs/DEVELOPMENT-PLAN.md](docs/DEVELOPMENT-PLAN.md) — dev workflow & CI design
- [CLAUDE.md](CLAUDE.md) — conventions for AI coding sessions

## Tech

Tauri v2 · React + TypeScript + Vite + Tailwind · Rust core (`sysinfo`, `smartctl`,
`powercfg`) · bilingual EN/বাংলা from day one.

## Development (CI-centric)

This project is built **Windows-first on GitHub Actions** — the maintainers develop on
Linux and let CI do all Windows compilation, E2E testing, and release bundling:

- `ci.yml` — typecheck, unit tests, Rust fmt/clippy/tests on every PR
- `e2e.yml` — WebDriver tests against a real compiled build on windows-latest
- `release.yml` — tag `v*` → draft release with Windows installers

Local loop for frontend work: `npm install && npm run dev` (Vite in browser mode;
Tauri APIs unavailable — that's what CI is for).
