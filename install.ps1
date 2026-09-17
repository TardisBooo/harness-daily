# harness-daily installer (Windows PowerShell)
# Usage: irm https://raw.githubusercontent.com/TardisBooo/harness-daily/main/install.ps1 | iex

$ErrorActionPreference = "Stop"
$Repo = "TardisBooo/harness-daily"
$InstallDir = Join-Path $env:LOCALAPPDATA "harness-daily"

$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
  "AMD64" { "x86_64-pc-windows-msvc" }
  "ARM64" { throw "ARM64 Windows build not published yet; build from source with cargo." }
  default { throw "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
}

$null = gh api "repos/$Repo/releases/latest" 2>&1
if ($LASTEXITCODE -ne 0) { throw "GitHub CLI (gh) is required: https://cli.github.com" }
$tag = (gh api "repos/$Repo/releases/latest" --jq .tag_name)

$zip = "harness-daily-$arch.zip"
$tmp = New-TemporaryFile
Remove-Item $tmp
gh release download $tag --repo $Repo --pattern $zip --output "$tmp.zip" | Out-Null

Expand-Archive "$tmp.zip" -DestinationPath $InstallDir -Force
Remove-Item "$tmp.zip" -Force

$exe = Join-Path $InstallDir "harness-daily.exe"
Write-Host "Installed: $exe"
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
  [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
  Write-Host "Added $InstallDir to your user PATH (new terminals only)."
}
& $exe --version
Write-Host "Next: harness-daily init --host grok --out <your-logs-dir>"
