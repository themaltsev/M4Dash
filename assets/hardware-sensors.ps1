[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$LhmPath)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)

try {
    Add-Type -Path $LhmPath -ErrorAction Stop
    $computer = [LibreHardwareMonitor.Hardware.Computer]::new()
    $computer.IsCpuEnabled = $true
    $computer.IsMotherboardEnabled = $true
    $computer.IsControllerEnabled = $false
    $computer.IsGpuEnabled = $false
    $computer.IsMemoryEnabled = $false
    $computer.IsNetworkEnabled = $false
    $computer.IsStorageEnabled = $false
    $computer.Open()

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

        foreach ($hardware in $computer.Hardware) {
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

        $sample = [ordered]@{
            cpuTemperatureC = $cpuTemp
            cpuTemperatureSensor = $cpuTempName
            cpuName = $cpuName
            motherboardName = $boardName
            motherboardSensors = @($boardSensors.ToArray())
            providerError = $null
        }
        [Console]::WriteLine(($sample | ConvertTo-Json -Depth 5 -Compress))
        Start-Sleep -Milliseconds 1000
    }
} catch {
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
    if ($null -ne $computer) { try { $computer.Close() } catch { } }
}
