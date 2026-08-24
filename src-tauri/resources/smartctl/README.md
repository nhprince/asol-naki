smartctl.exe is NOT committed here (binary, GPL).

It is downloaded at build time:
- Windows CI/release: scripts/fetch-smartctl.ps1 (choco, pinned version)
- This placeholder exists so tauri.conf.json's resources glob
  ("resources/smartctl/*") always matches, even on non-Windows builds
  where the binary is absent.
