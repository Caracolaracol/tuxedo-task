[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version,
    [Parameter(Mandatory)][string]$BinaryDir,
    [Parameter(Mandatory)][string]$OutputDir
)
$ErrorActionPreference = 'Stop'
$compiler = Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'
if (-not (Test-Path -LiteralPath $compiler)) { throw 'Inno Setup 6 is required; CI uses the preinstalled windows-2025 compiler. No tools are installed automatically.' }
$BinaryDir = (Resolve-Path -LiteralPath $BinaryDir).Path
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$OutputDir = (Resolve-Path -LiteralPath $OutputDir).Path
foreach ($name in @('tuxedo.exe', 'tuxedotask.exe')) {
    $binary = Join-Path $BinaryDir $name
    $actual = & $binary --version
    if ($LASTEXITCODE -ne 0 -or $actual -ne "Tuxedo Task $Version") { throw "Unexpected binary identity: $name / $actual" }
}
& $compiler "/DAppVersion=$Version" "/DSourceDir=$BinaryDir" "/DOutputDir=$OutputDir" "$PSScriptRoot\tuxedo-task.iss"
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup compilation failed' }
$name = "tuxedo-task-v$Version-windows-x86_64-setup.exe"
$hash = (Get-FileHash -LiteralPath (Join-Path $OutputDir $name) -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText((Join-Path $OutputDir "$name.sha256"), "$hash  $name`n", [Text.Encoding]::ASCII)
