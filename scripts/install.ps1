param(
    [Parameter(Mandatory = $false)]
    [string] $Url = "<FILL_ME_GITHUB_DOWNLOAD_URL>",

    [Parameter(Mandatory = $false)]
    [string] $InstallDir = "",

    [switch] $Force
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\SilliReminder"
}

if ($Url -eq "<FILL_ME_GITHUB_DOWNLOAD_URL>") {
    throw "Set -Url to your GitHub release asset URL (direct .exe download)."
}

Write-Host "Installing SilliReminder to: $InstallDir"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$exePath = Join-Path $InstallDir "SilliReminder.exe"

if ((Test-Path $exePath) -and (-not $Force)) {
    throw "Already installed at $exePath. Re-run with -Force to overwrite."
}

$tmp = Join-Path $env:TEMP ("SilliReminder_" + [Guid]::NewGuid().ToString("N") + ".exe")

Write-Host "Downloading: $Url"
Invoke-WebRequest -Uri $Url -OutFile $tmp

Move-Item -Force -Path $tmp -Destination $exePath

# Start Menu shortcut (per-user)
$startMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$shortcutPath = Join-Path $startMenu "SilliReminder.lnk"

$wsh = New-Object -ComObject WScript.Shell
$sc = $wsh.CreateShortcut($shortcutPath)
$sc.TargetPath = $exePath
$sc.WorkingDirectory = $InstallDir
$sc.IconLocation = $exePath
$sc.Save()

Write-Host "Installed: $exePath"
Write-Host "Start Menu shortcut: $shortcutPath"
Write-Host "Run the app once and enable 'Start with system' inside the app if desired."
