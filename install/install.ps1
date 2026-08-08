# Astral — install via terminal (Windows).
# Downloads the latest GitHub release MSI and installs it (elevates if needed).
#
#   irm https://raw.githubusercontent.com/nguyenthanhthe/astral/main/install/install.ps1 | iex
#
# Requires: Windows 10/11 x64, PowerShell 5.1+.

$ErrorActionPreference = 'Stop'

$repo = 'nguyenthanhthe/astral'
$api = "https://api.github.com/repos/$repo/releases/latest"
$ua = 'astral-install-script'

Write-Host 'astral  resolving latest release...' -ForegroundColor Cyan
$release = Invoke-RestMethod -Uri $api -Headers @{ 'User-Agent' = $ua }
$version = $release.tag_name.TrimStart('v')

switch ($env:PROCESSOR_ARCHITECTURE) {
  { $_ -in @('AMD64', 'x86_64') } { break }
  'ARM64' { throw 'no release build for Windows ARM64 yet' }
  default { throw "unsupported architecture: $_" }
}

$asset = "astral_${version}_x64_en-US.msi"
$url = "https://github.com/$repo/releases/download/v${version}/$asset"
$tmp = Join-Path $env:TEMP $asset

Write-Host "astral  downloading $asset ..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $url -OutFile $tmp -Headers @{ 'User-Agent' = $ua }

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).
  IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

Write-Host 'astral  installing (msiexec)...' -ForegroundColor Cyan
if ($isAdmin) {
  Start-Process msiexec -ArgumentList '/i', "`"$tmp`"", '/qn', '/norestart' -Wait
} else {
  Start-Process msiexec -ArgumentList '/i', "`"$tmp`"", '/qn', '/norestart' -Wait -Verb RunAs
}

Write-Host 'astral  done — launch Astral from the Start menu.' -ForegroundColor Green
