param([string]$Configuration = "release", [string]$TargetTriple = "x86_64-pc-windows-gnu")
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$version = (Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
$portableBuilder = Join-Path $PSScriptRoot 'package-portable.ps1'
& $portableBuilder -Configuration $Configuration -TargetTriple $TargetTriple
$makensisCommand = Get-Command 'makensis.exe' -ErrorAction SilentlyContinue
if (!$makensisCommand) { $makensisCommand = Get-Command 'makensis' -ErrorAction SilentlyContinue }
$makensisPath = if ($makensisCommand) { $makensisCommand.Source } else { $null }
if (!$makensisPath) {
  $knownPath = Join-Path ${env:ProgramFiles(x86)} 'NSIS\makensis.exe'
  if (Test-Path -LiteralPath $knownPath) { $makensisPath = $knownPath }
}
if (!$makensisPath) { throw 'NSIS 3 is required. Install it from https://nsis.sourceforge.io/Download and rerun this script.' }
Push-Location $root
try {
  & $makensisPath "/DAPP_VERSION=$version" 'installer\M4PCMonitor.nsi'
  if ($LASTEXITCODE -ne 0) { throw "makensis failed with exit code $LASTEXITCODE" }
} finally { Pop-Location }
$setup = Join-Path $root "artifacts\M4-PC-Monitor-Setup-$version-x64.exe"
$hash = (Get-FileHash -LiteralPath $setup -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -LiteralPath "$setup.sha256" -Value "$hash  $(Split-Path -Leaf $setup)" -Encoding ascii
Get-Item -LiteralPath $setup | Select-Object FullName, Length
Get-FileHash -LiteralPath $setup -Algorithm SHA256
