# Download the exact Setup.exe from the latest published release before this one
# and verify its adjacent checksum, so CI exercises a genuine installer upgrade.
[CmdletBinding()]
param([string]$OutputDir = 'dist/test-fixtures')
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$OutputDir = (Resolve-Path -LiteralPath $OutputDir).Path
$name = 'tuxedo-task-windows-x86_64-setup.exe'
$base = 'https://github.com/Caracolaracol/tuxedo-task/releases/download/v2026.10.2'
$installer = Join-Path $OutputDir 'previous-installer.exe'
Invoke-WebRequest -Uri "$base/$name" -OutFile $installer
$checksum = (Invoke-WebRequest -Uri "$base/$name.sha256").Content
if ($checksum -is [byte[]]) { $checksum = [Text.Encoding]::ASCII.GetString($checksum) }
$match = [regex]::Match([string]$checksum, '^([a-fA-F0-9]{64})\s+\*?([^\r\n]+)\s*$')
if (-not $match.Success -or $match.Groups[2].Value.Trim() -ne $name) { throw 'Invalid original installer checksum' }
if ((Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash -ine $match.Groups[1].Value) { throw 'Original installer checksum mismatch' }
