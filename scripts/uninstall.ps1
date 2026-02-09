param(
    [Parameter(Mandatory = $false)]
    [string] $InstallDir = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\SilliReminder"
}

Write-Host "Uninstalling from: $InstallDir"

# Remove Start Menu shortcut
$startMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$shortcutPath = Join-Path $startMenu "SilliReminder.lnk"
if (Test-Path $shortcutPath) {
    Remove-Item -Force $shortcutPath
}

# Remove installed files
if (Test-Path $InstallDir) {
    Remove-Item -Recurse -Force $InstallDir
}

Write-Host "Uninstalled."
