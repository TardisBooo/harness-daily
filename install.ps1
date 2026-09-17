# harness-daily installer (Windows PowerShell)
# Usage: irm https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.ps1 | iex

$ErrorActionPreference = "Stop"
$Repo = "TardisBooo/harness-daily"
$InstallDir = Join-Path $env:LOCALAPPDATA "harness-daily"

$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { "x86_64-pc-windows-msvc" }
    "ARM64" { throw "ARM64 Windows build is not published yet; build from source with cargo." }
    default { throw "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
}

$zipName = "harness-daily-$arch.zip"
$api = "https://api.github.com/repos/$Repo/releases/latest"
$rel = Invoke-RestMethod -Uri $api -Headers @{ "User-Agent" = "harness-daily-install" }
$asset = $rel.assets | Where-Object { $_.name -eq $zipName } | Select-Object -First 1
if (-not $asset) {
    throw "Release asset $zipName not found. See https://github.com/$Repo/releases"
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$zipPath = Join-Path $env:TEMP $zipName
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath -UseBasicParsing
Expand-Archive -Path $zipPath -DestinationPath $InstallDir -Force
Remove-Item $zipPath -Force

$exe = Get-ChildItem -Path $InstallDir -Recurse -Filter "harness-daily.exe" | Select-Object -First 1
if (-not $exe) { throw "harness-daily.exe missing after unzip" }
$dest = Join-Path $InstallDir "harness-daily.exe"
if ($exe.FullName -ne $dest) {
    Copy-Item $exe.FullName $dest -Force
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
    $env:Path = "$InstallDir;$env:Path"
    Write-Host "Added $InstallDir to your user PATH (new terminals pick it up automatically)."
}

Write-Host "Installed: $dest"
& $dest --version
Write-Host "Next: harness-daily init --host auto --out <your-logs-dir>"
Write-Host "      harness-daily schedule install --time 08:00"
