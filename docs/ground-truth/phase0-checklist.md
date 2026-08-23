# Ground-Truth Protocol — Phase 0

**Machine:** HP ProBook 440 G11 · **Date:** ____________ · **Tester:** Prince

Run the CI-built exe on your laptop and verify each item against reality.
CI proves code compiles + logic passes; ONLY you can prove it reflects real hardware.

## Get the build

1. Open the green "E2E" workflow run for the latest commit:
   <https://github.com/nhprince/asol-naki/actions/workflows/e2e.yml>
   (or trigger one: Actions → E2E → Run workflow)
2. Download artifact `windows-debug-build` from the run page.
3. Unzip, run `asol-naki.exe` (SmartScreen: More info → Run anyway).

## Verify

| # | Check | Expected | OK? |
|---|---|---|---|
| 1 | App launches, no console window flash | clean window | ☐ |
| 2 | Window title | Asol Naki? — আসল নাকি? | ☐ |
| 3 | Bengali text renders (no boxes □□) | proper বাংলা glyphs | ☐ |
| 4 | Auto-scan completes within ~3 s | results table appears | ☐ |
| 5 | CPU name | matches Task Manager → Performance → CPU | ☐ |
| 6 | Threads (logical CPUs) | match Task Manager | ☐ |
| 7 | Memory MB | ≈ RAM in Settings → About (MB vs GB rounding) | ☐ |
| 8 | OS name/version | match winver | ☐ |
| 9 | Language toggle EN→BN→EN | all strings switch, layout holds | ☐ |

## Result

- All 9 pass → tick ROADMAP.md Phase 0 box 5, phase marker stays Phase 0 until
  nightly E2E also proves stable; then declare Phase 1 start.
- Any fail → note the number + what you saw, tell Saturday. Do NOT guess-fix.

## Notes / observations

_______________________________________________
