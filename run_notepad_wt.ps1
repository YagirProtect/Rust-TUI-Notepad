param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$notepadSchemeName = "One Half Dark (Copy)"

function Get-NotepadScheme {
    return [pscustomobject]@{
        background = "#282C34"
        black = "#282C34"
        blue = "#61AFEF"
        brightBlack = "#5A6374"
        brightBlue = "#61AFEF"
        brightCyan = "#56B6C2"
        brightGreen = "#98C379"
        brightPurple = "#C678DD"
        brightRed = "#E06C75"
        brightWhite = "#DCDFE4"
        brightYellow = "#E5C07B"
        cursorColor = "#FFFFFF"
        cyan = "#56B6C2"
        foreground = "#DCDFE4"
        green = "#98C379"
        name = $notepadSchemeName
        purple = "#C678DD"
        red = "#E06C75"
        selectionBackground = "#FFFFFF"
        white = "#DCDFE4"
        yellow = "#E5C07B"
    }
}

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
    
    return $null
}

function Get-SettingsBackupPath($settingsPath) {
    return "$settingsPath.notepad.bak"
}

function Write-AtomicUtf8($path, $content) {
    $tempPath = "$path.notepad.tmp"
    Set-Content -Path $tempPath -Value $content -Encoding UTF8
    Move-Item -Path $tempPath -Destination $path -Force
}

function Try-ParseSettings($raw) {
    try {
        return ($raw | ConvertFrom-Json)
    } catch {
        return $null
    }
}

function Repair-SettingsRaw($raw) {
    $parsed = Try-ParseSettings $raw
    if ($null -ne $parsed) {
        return $raw
    }

    $matches = [regex]::Matches($raw, '(?ms)\{\s*"\$help"')
    foreach ($match in $matches) {
        $candidate = $raw.Substring($match.Index)
        if ($null -ne (Try-ParseSettings $candidate)) {
            return $candidate
        }
    }

    return $null
}

function Get-UsableSettingsRaw($settingsPath) {
    $raw = Get-Content $settingsPath -Raw -Encoding UTF8
    $repaired = Repair-SettingsRaw $raw
    if ($null -ne $repaired) {
        if ($repaired -ne $raw) {
            Write-AtomicUtf8 $settingsPath $repaired
        }
        return $repaired
    }

    $backupPath = Get-SettingsBackupPath $settingsPath
    if (Test-Path $backupPath) {
        $backupRaw = Get-Content $backupPath -Raw -Encoding UTF8
        $backupRepaired = Repair-SettingsRaw $backupRaw
        if ($null -ne $backupRepaired) {
            Write-AtomicUtf8 $settingsPath $backupRepaired
            return $backupRepaired
        }
    }

    throw "Windows Terminal settings.json is invalid and no recoverable backup was found."
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
    $raw = Get-UsableSettingsRaw $settingsPath
    $settingsObject = $raw | ConvertFrom-Json
    Write-AtomicUtf8 (Get-SettingsBackupPath $settingsPath) $raw
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

    if (-not ($settingsObject.PSObject.Properties.Name -contains "schemes")) {
        $settingsObject | Add-Member -NotePropertyName "schemes" -NotePropertyValue @()
    }

    $schemes = @($settingsObject.schemes)
    $updatedSchemes = @()
    foreach ($scheme in $schemes) {
        if ($scheme.name -ne $notepadSchemeName) {
            $updatedSchemes += $scheme
        }
    }
    $updatedSchemes += Get-NotepadScheme
    $settingsObject.schemes = $updatedSchemes

    if (-not ($settingsObject.PSObject.Properties.Name -contains "profiles")) {
        $settingsObject | Add-Member -NotePropertyName "profiles" -NotePropertyValue ([pscustomobject]@{})
    }

    if (-not ($settingsObject.profiles.PSObject.Properties.Name -contains "defaults")) {
        $settingsObject.profiles | Add-Member -NotePropertyName "defaults" -NotePropertyValue ([pscustomobject]@{})
    }

    if ($settingsObject.profiles.defaults.PSObject.Properties.Name -contains "colorScheme") {
        $settingsObject.profiles.defaults.colorScheme = $notepadSchemeName
    } else {
        $settingsObject.profiles.defaults | Add-Member -NotePropertyName "colorScheme" -NotePropertyValue $notepadSchemeName
    }

    $json = $settingsObject | ConvertTo-Json -Depth 100
    Write-AtomicUtf8 $settingsPath $json

    return $raw
}

$root = Split-Path -Parent $PSCommandPath
$settingsPath = Get-WindowsTerminalSettingsPath
$originalSettings = $null

if ($null -ne $settingsPath) {
    $originalSettings = Set-CtrlVUnbound $settingsPath
}

try {
    $localExePath = Join-Path $root "NOTEPAD.exe"
    if (Test-Path $localExePath) {
        $exePath = $localExePath
    } elseif ($Release) {
        $exePath = Join-Path $root "target\release\NOTEPAD.exe"
    } else {
        $exePath = Join-Path $root "target\debug\NOTEPAD.exe"
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
    if ($null -ne $settingsPath -and $null -ne $originalSettings) {
        Write-AtomicUtf8 $settingsPath $originalSettings
    }
}
