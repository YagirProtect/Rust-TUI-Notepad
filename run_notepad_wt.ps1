param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"

function Get-WindowsTerminalSettingsPath {
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA "Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json"),
        (Join-Path $env:LOCALAPPDATA "Packages\Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe\LocalState\settings.json"),
        (Join-Path $env:LOCALAPPDATA "Microsoft\Windows Terminal\settings.json")
    )

    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) {
            return $candidate
        }
    }

    throw "Windows Terminal settings.json not found."
}

function Get-KeyBindingList($settingsObject) {
    if ($settingsObject.PSObject.Properties.Name -contains "keybindings") {
        return "keybindings"
    }

    if ($settingsObject.PSObject.Properties.Name -contains "actions") {
        return "actions"
    }

    $settingsObject | Add-Member -NotePropertyName "keybindings" -NotePropertyValue @()
    return "keybindings"
}

function Test-CtrlVBinding($binding) {
    if (-not ($binding.PSObject.Properties.Name -contains "keys")) {
        return $false
    }

    $keys = $binding.keys
    if ($null -eq $keys) {
        return $false
    }

    if ($keys -is [string]) {
        return $keys.ToLowerInvariant() -eq "ctrl+v"
    }

    foreach ($key in $keys) {
        if ($null -ne $key -and $key.ToString().ToLowerInvariant() -eq "ctrl+v") {
            return $true
        }
    }

    return $false
}

function Set-CtrlVUnbound($settingsPath) {
    $raw = Get-Content $settingsPath -Raw -Encoding UTF8
    $settingsObject = $raw | ConvertFrom-Json
    $listName = Get-KeyBindingList $settingsObject
    $bindings = @($settingsObject.$listName)

    $filtered = @()
    foreach ($binding in $bindings) {
        if (-not (Test-CtrlVBinding $binding)) {
            $filtered += $binding
        }
    }

    $filtered += [pscustomobject]@{
        id = $null
        keys = @("ctrl+v")
    }

    $settingsObject.$listName = $filtered

    $settingsObject |
        ConvertTo-Json -Depth 100 |
        Set-Content -Path $settingsPath -Encoding UTF8

    return $raw
}

$root = Split-Path -Parent $PSCommandPath
$settingsPath = Get-WindowsTerminalSettingsPath
$originalSettings = Set-CtrlVUnbound $settingsPath

try {
    $exePath = if ($Release) {
        Join-Path $root "target\release\NOTEPAD.exe"
    } else {
        Join-Path $root "target\debug\NOTEPAD.exe"
    }

    if (Test-Path $exePath) {
        & $exePath
        $exitCode = $LASTEXITCODE
    } else {
        if ($Release) {
            cargo run --release
        } else {
            cargo run
        }
        $exitCode = $LASTEXITCODE
    }

    exit $exitCode
}
finally {
    Set-Content -Path $settingsPath -Value $originalSettings -Encoding UTF8
}
