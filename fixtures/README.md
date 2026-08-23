# Fixtures — recorded hardware snapshots

Sanitized JSON captures of real diagnostic output, used by the `FixtureProvider`
(Phase 0b+) so fraud-detection logic and E2E can run deterministically in CI
without real hardware.

Rules:
- Raw captures land in `fixtures/raw/` (gitignored) and are NEVER committed.
- Only sanitized files live here: no serial numbers, MACs, usernames.
- Every new device tested = one more fixture file = wider regression net.
- Synthetic/corrupted fixtures (spoofed capacity, mismatched CPU identity) are
  welcome and are how Phase 2's "catches deliberate misrepresentation" gate passes.
