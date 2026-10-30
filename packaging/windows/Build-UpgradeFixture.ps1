# v2026.10.1 had a ZIP, not an installer. Wrap those checksum-verified original
# binaries in a synthetic older installer to exercise a genuine version upgrade.
[CmdletBinding()]
param([string]$OutputDir = 'dist/test-fixtures')
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$OutputDir = (Resolve-Path -LiteralPath $OutputDir).Path
$name = 'tuxedo-task-v2026.10.1-x86_64-pc-windows-msvc.zip'
$base = 'https://github.com/Caracolaracol/tuxedo-task/releases/download/v2026.10.1'
$archive = Join-Path $OutputDir $name
Invoke-WebRequest -Uri "$base/$name" -OutFile $archive
$checksum = (Invoke-WebRequest -Uri "$base/$name.sha256").Content
if ($checksum -is [byte[]]) { $checksum = [Text.Encoding]::ASCII.GetString($checksum) }
$match = [regex]::Match([string]$checksum, '^([a-fA-F0-9]{64})\s+\*?([^\r\n]+)\s*$')
if (-not $match.Success -or $match.Groups[2].Value.Trim() -ne $name) { throw 'Invalid original release checksum' }
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ine $match.Groups[1].Value) { throw 'Original release checksum mismatch' }
$stage = Join-Path $OutputDir 'previous-binaries'
Expand-Archive -LiteralPath $archive -DestinationPath $stage -Force
& "$PSScriptRoot/Build-Installer.ps1" -Version '2026.10.1' -BinaryDir $stage -OutputDir $OutputDir
Move-Item -LiteralPath (Join-Path $OutputDir 'tuxedo-task-v2026.10.1-windows-x86_64-setup.exe') -Destination (Join-Path $OutputDir 'previous-installer.exe') -Force
