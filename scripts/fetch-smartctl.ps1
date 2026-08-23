# Fetches pinned smartmontools smartctl.exe for bundling with the app.
# Uses Chocolatey (available on GitHub Actions windows runners) so version
# pinning + package integrity are handled by the choco ecosystem.
#
# Usage (CI):  powershell -File scripts/fetch-smartctl.ps1 -Version 7.5
# Copies smartctl.exe into src-tauri/resources/smartctl/
param(
    [string]$Version = "7.5"
)

$ErrorActionPreference = "Stop"

Write-Host "== Installing smartmontools $Version via Chocolatey =="
choco install smartmontools --version "$Version" -y --no-progress | Out-Null
if ($LASTEXITCODE -ne 0) { throw "choco install failed with $LASTEXITCODE" }

$src = "C:\Program Files\smartmontools\bin\smartctl.exe"
if (-not (Test-Path $src)) {
    # Some choco versions land elsewhere; search defensively.
    $src = (Get-ChildItem "C:\Program Files\smartmontools" -Recurse -Filter smartctl.exe |
        Select-Object -First 1).FullName
}
if (-not $src -or -not (Test-Path $src)) { throw "smartctl.exe not found after install" }

$destDir = Join-Path $PSScriptRoot "..\src-tauri\resources\smartctl"
New-Item -ItemType Directory -Force -Path $destDir | Out-Null
Copy-Item $src (Join-Path $destDir "smartctl.exe") -Force

# Ship the GPL license alongside the binary (redistribution requirement).
$licenseSrc = "C:\Program Files\smartmontools\doc\COPYING"
if (Test-Path $licenseSrc) {
    Copy-Item $licenseSrc (Join-Path $destDir "LICENSE.smartmontools.txt") -Force
} else {
    Write-Host "License file not found at expected path — writing source-offer notice."
    Set-Content -Path (Join-Path $destDir "LICENSE.smartmontools.txt") -Value @"
This application bundles smartctl from smartmontools (https://www.smartmontools.org),
licensed under GPL v2+. Source is available at the project website.
Free use of this software is granted under the terms of the GNU General Public License.
"@
}

& (Join-Path $destDir "smartctl.exe") --version
if ($LASTEXITCODE -ne 0) { throw "bundled smartctl failed to run" }
Write-Host "== smartctl staged into src-tauri/resources/smartctl =="
