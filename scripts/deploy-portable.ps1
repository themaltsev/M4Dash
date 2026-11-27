param([Parameter(Mandatory = $true)][string]$Stage)
$ErrorActionPreference = 'Stop'
$destination = Join-Path $env:LOCALAPPDATA 'M4 PC Monitor Portable'
$exe = Join-Path $destination 'M4PCMonitor.exe'
New-Item -ItemType Directory -Force -Path $destination | Out-Null
$allProcesses = @(Get-CimInstance Win32_Process)
$processTree = @($allProcesses | Where-Object Name -eq 'M4PCMonitor.exe' | ForEach-Object { [uint32]$_.ProcessId })
# Recover only an orphaned LHM bridge from a previous app shutdown; it holds the bundled DLL open.
$portableProvider = Join-Path $destination 'hardware-sensors.ps1'
$processTree += @($allProcesses | Where-Object { $_.Name -eq 'powershell.exe' -and $_.CommandLine -like "*$portableProvider*" } | ForEach-Object { [uint32]$_.ProcessId })
do {
    $children = @($allProcesses | Where-Object { $processTree -contains [uint32]$_.ParentProcessId -and $processTree -notcontains [uint32]$_.ProcessId } | ForEach-Object { [uint32]$_.ProcessId })
    $newCount = $children.Count
    $processTree += $children
    $processTree = @($processTree | Select-Object -Unique)
} while ($newCount -gt 0)
foreach ($processId in $processTree) { Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue }
Start-Sleep -Milliseconds 600
foreach ($name in @('M4PCMonitor.exe', 'M4PCMonitor.ico', 'hardware-sensors.ps1', 'LibreHardwareMonitorLib.dll', 'LICENSE-LibreHardwareMonitor.txt')) {
    $source = Join-Path $Stage $name
    if (!(Test-Path -LiteralPath $source)) { throw "Required portable file missing: $source" }
    Copy-Item -LiteralPath $source -Destination (Join-Path $destination $name) -Force
}
if (!(Test-Path -LiteralPath (Join-Path $destination 'settings.json'))) {
    Set-Content -LiteralPath (Join-Path $destination 'settings.json') -Value '{}' -Encoding UTF8
}

# Resolve the current user's actual Desktop (including OneDrive/Known Folder redirection).
$desktop = [Environment]::GetFolderPath([Environment+SpecialFolder]::Desktop)
$shortcutPath = Join-Path $desktop 'M4Dash.lnk'
$legacyShortcutPath = Join-Path $desktop 'M4 PC Monitor.lnk'
if ((Test-Path -LiteralPath $legacyShortcutPath) -and !(Test-Path -LiteralPath $shortcutPath)) {
    Move-Item -LiteralPath $legacyShortcutPath -Destination $shortcutPath
}
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $exe
$shortcut.WorkingDirectory = $destination
$shortcut.IconLocation = (Join-Path $destination 'M4PCMonitor.ico') + ',0'
$shortcut.Save()

# Leave startup disabled, then start this update through the current interactive desktop token.
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
Remove-ItemProperty -Path $runKey -Name 'M4PCMonitor' -ErrorAction SilentlyContinue
$taskName = 'M4PCMonitor-Portable-LaunchOnce'
$action = New-ScheduledTaskAction -Execute $exe -WorkingDirectory $destination
$principal = New-ScheduledTaskPrincipal -UserId "$env:COMPUTERNAME\$env:USERNAME" -LogonType Interactive -RunLevel Highest
Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal -Force | Out-Null
Start-ScheduledTask -TaskName $taskName
Start-Sleep -Seconds 5
Get-ScheduledTaskInfo -TaskName $taskName | Select-Object LastRunTime, LastTaskResult, NextRunTime
Get-Process -Name M4PCMonitor -ErrorAction SilentlyContinue | Select-Object Id, MainWindowHandle, MainWindowTitle, SessionId, Path
Unregister-ScheduledTask -TaskName $taskName -Confirm:$false

function Get-ShortcutTarget([string]$Path) {
    $link = $shell.CreateShortcut($Path)
    [pscustomobject]@{ TargetPath = $link.TargetPath; IconLocation = $link.IconLocation; Desktop = $desktop }
}
Get-Process -Name M4PCMonitor -ErrorAction SilentlyContinue | Select-Object Id, MainWindowTitle, Path
Get-ShortcutTarget -Path $shortcutPath
Get-Item -LiteralPath (Join-Path $destination 'settings.json') | Select-Object FullName, Length
