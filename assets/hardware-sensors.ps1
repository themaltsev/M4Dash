[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$LhmPath)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$diagnosticPath = Join-Path $PSScriptRoot 'sensor-provider.log'
function Write-ProviderDiagnostic([string]$message) {
    try { Add-Content -LiteralPath $diagnosticPath -Value "$(Get-Date -Format o) $message" -Encoding UTF8 } catch { }
}
Write-ProviderDiagnostic 'Provider starting'

try {
    Add-Type -Path $LhmPath -ErrorAction Stop
    Write-ProviderDiagnostic 'LibreHardwareMonitor assembly loaded'
    $computer = [LibreHardwareMonitor.Hardware.Computer]::new()
    $computer.IsCpuEnabled = $true
    $computer.IsMotherboardEnabled = $true
    $computer.IsControllerEnabled = $false
    $computer.IsGpuEnabled = $false
    $computer.IsMemoryEnabled = $false
    $computer.IsNetworkEnabled = $false
    $computer.IsStorageEnabled = $false
    $computer.Open()
    Write-ProviderDiagnostic "Computer.Open completed; hardware count=$($computer.Hardware.Count)"

    $memoryType = $null
    $memorySpeedMhz = $null
    try {
        $modules = @(Get-CimInstance -ClassName Win32_PhysicalMemory -ErrorAction Stop)
        $speeds = @($modules | ForEach-Object {
            if ($_.ConfiguredClockSpeed -gt 0) { [double]$_.ConfiguredClockSpeed }
            elseif ($_.Speed -gt 0) { [double]$_.Speed }
        } | Where-Object { $_ -gt 0 })
        if ($speeds.Count -gt 0) {
            $memorySpeedMhz = [uint32][Math]::Round(($speeds | Measure-Object -Average).Average)
        }
        $memoryCode = [int]($modules | Select-Object -First 1 -ExpandProperty SMBIOSMemoryType -ErrorAction SilentlyContinue)
        $memoryType = switch ($memoryCode) {
            24 { 'DDR3' }
            26 { 'DDR4' }
            34 { 'DDR5' }
            default { $null }
        }
    } catch { Write-ProviderDiagnostic "Memory SMBIOS query failed: $($_.Exception.Message)" }

    function Update-BoardHardware($hardware, $sensors) {
        try {
            $hardware.Update()
            foreach ($sensor in $hardware.Sensors) {
                if ($null -eq $sensor.Value) { continue }
                $kind = $sensor.SensorType.ToString()
                if ($kind -in @('Temperature', 'Fan')) {
                    $sensors.Add([ordered]@{
                        name = [string]$sensor.Name
                        kind = $kind
                        value = [double]$sensor.Value
                    })
                }
            }
            foreach ($child in $hardware.SubHardware) {
                Update-BoardHardware $child $sensors
            }
        } catch { }
    }

    while ($true) {
        $cpuName = $null
        $cpuTemp = $null
        $cpuTempName = $null
        $cpuFallbacks = [System.Collections.Generic.List[object]]::new()
        $boardName = $null
        $boardSensors = [System.Collections.Generic.List[object]]::new()
        $hardwareCount = 0

        foreach ($hardware in $computer.Hardware) {
            $hardwareCount++
            $type = $hardware.HardwareType.ToString()
            if ($type -eq 'Cpu') {
                $cpuName = [string]$hardware.Name
                try {
                    $hardware.Update()
                    foreach ($sensor in $hardware.Sensors) {
                        if ($sensor.SensorType.ToString() -ne 'Temperature' -or $null -eq $sensor.Value) { continue }
                        $value = [double]$sensor.Value
                        if ($value -lt 20 -or $value -gt 120) { continue }
                        if ($sensor.Name -match 'Tctl/Tdie') {
                            $cpuTemp = $value
                            $cpuTempName = [string]$sensor.Name
                            break
                        }
                        if ($sensor.Name -match 'CCD|Package|Core') {
                            $cpuFallbacks.Add([pscustomobject]@{ name = [string]$sensor.Name; value = $value })
                        }
                    }
                } catch { }
            } elseif ($type -eq 'Motherboard') {
                $boardName = [string]$hardware.Name
                Update-BoardHardware $hardware $boardSensors
            }
        }

        if ($null -eq $cpuTemp -and $cpuFallbacks.Count -gt 0) {
            $fallback = $cpuFallbacks | Sort-Object value -Descending | Select-Object -First 1
            $cpuTemp = [double]$fallback.value
            $cpuTempName = [string]$fallback.name
        }
        if ($null -eq $cpuTemp) {
            $boardCpu = $boardSensors | Where-Object { $_.kind -eq 'Temperature' -and $_.name -match 'CPU' } | Select-Object -First 1
            if ($null -ne $boardCpu) {
                $cpuTemp = [double]$boardCpu.value
                $cpuTempName = "Motherboard: $($boardCpu.name)"
            }
        }

        $sample = [ordered]@{
            cpuTemperatureC = $cpuTemp
            cpuTemperatureSensor = $cpuTempName
            memoryType = $memoryType
            memorySpeedMhz = $memorySpeedMhz
            cpuName = $cpuName
            motherboardName = $boardName
            motherboardSensors = @($boardSensors.ToArray())
            providerError = $null
        }
        [Console]::WriteLine(($sample | ConvertTo-Json -Depth 5 -Compress))
        [Console]::Out.Flush()
        if (-not $script:LoggedSample) {
            Write-ProviderDiagnostic "First sample: cpu='$cpuName' temperature=$cpuTemp source='$cpuTempName' board='$boardName' sensors=$($boardSensors.Count) hardware=$hardwareCount RAM='$memoryType' speedMHz=$memorySpeedMhz"
            $script:LoggedSample = $true
        }
        Start-Sleep -Milliseconds 1000
    }
} catch {
    Write-ProviderDiagnostic "Provider failed: $($_.Exception.Message)"
    $sample = [ordered]@{
        cpuTemperatureC = $null
        cpuTemperatureSensor = $null
        cpuName = $null
        motherboardName = $null
        motherboardSensors = @()
        providerError = [string]$_.Exception.Message
    }
    [Console]::WriteLine(($sample | ConvertTo-Json -Depth 5 -Compress))
    exit 1
} finally {
    Write-ProviderDiagnostic 'Provider stopping'
    if ($null -ne $computer) { try { $computer.Close() } catch { } }
}
