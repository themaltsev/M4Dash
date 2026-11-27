param([string]$Configuration = "release", [string]$TargetTriple = "x86_64-pc-windows-gnu")
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root "target\$TargetTriple\$Configuration\M4PCMonitor.exe"
$icon = Join-Path $root 'assets\M4PCMonitor.ico'
$readme = Join-Path $root 'README_Portable.md'
$sensorScript = Join-Path $root 'assets\hardware-sensors.ps1'
$sensorLibrary = Join-Path $root 'assets\vendor\LibreHardwareMonitorLib.dll'
$sensorLicense = Join-Path $root 'assets\vendor\LICENSE-LibreHardwareMonitor.txt'
$out = Join-Path $root 'artifacts'
if (!(Test-Path -LiteralPath $exe)) { throw "Build output not found: $exe" }
if (!(Test-Path -LiteralPath $icon)) { throw "Portable icon not found: $icon" }
if (!(Test-Path -LiteralPath $readme)) { throw "Portable readme not found: $readme" }
foreach ($file in @($sensorScript, $sensorLibrary, $sensorLicense)) { if (!(Test-Path -LiteralPath $file)) { throw "Portable sensor dependency not found: $file" } }
New-Item -ItemType Directory -Force -Path $out | Out-Null
$version = (Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
$stage = Join-Path $out "M4Dash-Portable-v$version-x64"
New-Item -ItemType Directory -Force -Path $stage | Out-Null
Copy-Item -LiteralPath $exe -Destination $stage
Copy-Item -LiteralPath $icon -Destination $stage
Copy-Item -LiteralPath $readme -Destination $stage
Copy-Item -LiteralPath $sensorScript -Destination $stage
Copy-Item -LiteralPath $sensorLibrary -Destination $stage
Copy-Item -LiteralPath $sensorLicense -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'assets\icons') -Destination (Join-Path $stage 'icons') -Recurse -Force
$zip = Join-Path $out "M4Dash-Portable-v$version-x64.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
Get-FileHash -Algorithm SHA256 $zip
