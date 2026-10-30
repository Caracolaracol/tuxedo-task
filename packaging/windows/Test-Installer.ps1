[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$InstallerPath,
    [Parameter(Mandatory)][string]$ExpectedVersion
)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:OS -ne 'Windows_NT') {
    throw 'Installation tests are authorized only on a disposable Windows GitHub Actions runner.'
}
$InstallerPath = (Resolve-Path -LiteralPath $InstallerPath).Path
$checksum = [IO.File]::ReadAllText("$InstallerPath.sha256")
if ($checksum.Contains("`r")) { throw 'Checksum files must use portable LF line endings.' }
$expectedHash = ([regex]::Match($checksum, '^([a-f0-9]{64})  ')).Groups[1].Value
if (-not $expectedHash -or (Get-FileHash -LiteralPath $InstallerPath -Algorithm SHA256).Hash -ine $expectedHash) { throw 'Installer checksum mismatch.' }
$output = Split-Path $InstallerPath
$previous = Join-Path $output 'test-fixtures/previous-installer.exe'
if (-not (Test-Path -LiteralPath $previous)) { throw 'Build the synthetic upgrade fixture first.' }
$statePath = 'Software\Caracolaracol\TuxedoTask'
$uninstallKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{BEE5823B-1635-4B5B-BA07-5E392807F3E0}_is1'
if ([Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($uninstallKey)) { throw 'Refusing to touch an existing installation.' }
$root = Join-Path $env:RUNNER_TEMP ('tuxedo-installer-' + [guid]::NewGuid().ToString('N'))
$directory = Join-Path $root 'Tuxedo Task & QA'
New-Item -ItemType Directory -Force -Path $root | Out-Null
$environment = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Environment')
$originalProcessPath = $env:Path
$originalHasPath = $environment.GetValueNames() -icontains 'Path'
$originalPath = [string]$environment.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
$originalKind = [Microsoft.Win32.RegistryValueKind]::ExpandString
if ($originalHasPath) { $originalKind = $environment.GetValueKind('Path') }
$configDir = Join-Path $env:APPDATA 'tuxedo'
$configFile = Join-Path $configDir 'config.toml'
$configExisted = Test-Path -LiteralPath $configFile
$originalConfig = if ($configExisted) { [IO.File]::ReadAllBytes($configFile) } else { $null }
New-Item -ItemType Directory -Force -Path $configDir | Out-Null
$sentinel = "# keep this user configuration`nmouse = false`n"
[IO.File]::WriteAllText($configFile, $sentinel)
$tasks = Join-Path $root 'user-tasks/todo.txt'
New-Item -ItemType Directory -Force -Path (Split-Path $tasks) | Out-Null
[IO.File]::WriteAllText($tasks, "Keep this task +installer-test`n")
$taskText = [IO.File]::ReadAllText($tasks)
$baseline = '%USERPROFILE%\Original Tools;;C:\Windows\System32;'
$logNumber = 0

function Assert([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}
function UserPath {
    return [string]$environment.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
}
function PathCount {
    return @((UserPath).Split(';') | Where-Object { $_ -ieq $directory }).Count
}
function RunSetup([string]$File, [string]$Dir, [bool]$Success = $true) {
    $script:logNumber++
    $log = Join-Path $output "installer-test-$script:logNumber.log"
    $arguments = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP- /TASKS=addtopath /DIR="' + $Dir + '" /LOG="' + $log + '"'
    $process = Start-Process -FilePath $File -ArgumentList $arguments -PassThru
    if (-not $process.WaitForExit(90000)) { $process.Kill(); throw 'Installer timed out.' }
    if ($Success) { Assert ($process.ExitCode -eq 0) "Installer failed: $($process.ExitCode); see $log" }
    else { Assert ($process.ExitCode -ne 0) 'Installer unexpectedly accepted a foreign nonempty directory.' }
}
function Uninstall {
    $uninstaller = Join-Path $directory 'unins000.exe'
    Assert (Test-Path -LiteralPath $uninstaller) 'Uninstaller missing.'
    $script:logNumber++
    $log = Join-Path $output "installer-test-$script:logNumber.log"
    $process = Start-Process -FilePath $uninstaller -ArgumentList ('/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /LOG="' + $log + '"') -PassThru
    if (-not $process.WaitForExit(90000)) { $process.Kill(); throw 'Uninstaller timed out.' }
    Assert ($process.ExitCode -eq 0) 'Uninstaller failed.'
    # The original process/file can disappear before the temporary second phase
    # finishes usPostUninstall (including our PATH helper). Wait for that worker,
    # not just for unins000.exe to vanish, before checking registry state.
    $deadline = [DateTime]::UtcNow.AddSeconds(25)
    do {
        $workers = @(Get-CimInstance Win32_Process -Filter "Name = '_unins.tmp'" | Where-Object {
            $_.CommandLine -and $_.CommandLine.Contains($directory, [StringComparison]::OrdinalIgnoreCase)
        })
        $pending = (Test-Path -LiteralPath $uninstaller) -or $workers.Count -gt 0
        if ($pending) { Start-Sleep -Milliseconds 100 }
    } while ($pending -and [DateTime]::UtcNow -lt $deadline)
    Assert (-not $pending) 'Uninstaller second phase did not finish.'
}
function AssertData {
    Assert ([IO.File]::ReadAllText($configFile) -ceq $sentinel) 'User configuration changed.'
    Assert ([IO.File]::ReadAllText($tasks) -ceq $taskText) 'User tasks changed.'
}
function AssertVersion([string]$Version) {
    foreach ($name in @('tuxedo.exe', 'tuxedotask.exe')) {
        $actual = & (Join-Path $directory $name) --version
        Assert ($LASTEXITCODE -eq 0 -and $actual -eq "Tuxedo Task $Version") "Wrong installed version: $actual"
    }
}
try {
    $environment.SetValue('Path', $baseline, [Microsoft.Win32.RegistryValueKind]::ExpandString)
    $foreign = Join-Path $root 'foreign-program'
    New-Item -ItemType Directory -Path $foreign | Out-Null
    [IO.File]::WriteAllText((Join-Path $foreign 'tuxedo.exe'), 'Do not overwrite this other installation')
    RunSetup $InstallerPath $foreign $false
    Assert ([IO.File]::ReadAllText((Join-Path $foreign 'tuxedo.exe')) -eq 'Do not overwrite this other installation') 'Foreign executable overwritten.'
    Assert ((UserPath) -ceq $baseline) 'Rejected installation modified PATH.'

    RunSetup $previous $directory
    AssertVersion '2026.10.1'
    Assert ((PathCount) -eq 1) 'Initial PATH entry missing or duplicated.'
    $extraData = Join-Path $directory 'my-personal-todo.txt'
    [IO.File]::WriteAllText($extraData, 'Keep unregistered files even inside the program directory')
    RunSetup $InstallerPath $directory
    AssertVersion $ExpectedVersion
    AssertData
    Assert ((PathCount) -eq 1) 'Upgrade duplicated PATH.'
    RunSetup $InstallerPath $directory
    AssertVersion $ExpectedVersion
    Assert ((PathCount) -eq 1) 'Repair/reinstall duplicated PATH.'
    $env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::ExpandEnvironmentVariables((UserPath))
    foreach ($command in @('tuxedo.exe', 'tuxedotask.exe')) {
        Assert ((Get-Command $command -CommandType Application).Source -ieq (Join-Path $directory $command)) 'New terminal PATH does not resolve the installed command.'
    }
    Assert ((UserPath).StartsWith($baseline, [StringComparison]::Ordinal)) 'Original PATH text/variables changed.'
    Assert ($environment.GetValueKind('Path') -eq [Microsoft.Win32.RegistryValueKind]::ExpandString) 'PATH registry type changed.'

    $registration = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($uninstallKey)
    Assert ($null -ne $registration) 'Per-user uninstall registration missing.'
    Assert ($registration.GetValue('DisplayVersion') -eq $ExpectedVersion) 'Uninstall registration was not upgraded.'
    $registration.Dispose()
    $shortcutPath = Join-Path ([Environment]::GetFolderPath('Programs')) 'Tuxedo Task.lnk'
    Assert (Test-Path -LiteralPath $shortcutPath) 'Start menu shortcut missing.'
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($shortcutPath)
    Assert ($shortcut.TargetPath -ieq (Join-Path $env:WINDIR 'System32/cmd.exe')) 'Shortcut must open a terminal.'
    Assert ($shortcut.Arguments -ceq ('/D /S /K ""' + (Join-Path $directory 'tuxedo.exe') + '""')) 'Unsafe shortcut quoting.'
    Assert ($shortcut.WorkingDirectory -ieq [Environment]::GetFolderPath('MyDocuments')) 'Shortcut may create tasks in the program directory.'

    $external = ';%USERPROFILE%\Added Later'
    $environment.SetValue('Path', ((UserPath) + $external), [Microsoft.Win32.RegistryValueKind]::ExpandString)
    Uninstall
    Assert (-not (Test-Path -LiteralPath (Join-Path $directory 'tuxedo.exe'))) 'Executable survived uninstall.'
    Assert ((UserPath) -ceq ($baseline + $external)) ("Uninstall PATH mismatch. Expected: [$($baseline + $external)] Actual: [$(UserPath)]")
    AssertData
    Assert (Test-Path -LiteralPath $extraData) 'Uninstall removed an unregistered personal file.'
    Remove-Item -LiteralPath $extraData
    if (Test-Path -LiteralPath $directory) { Remove-Item -LiteralPath $directory }

    # A preexisting PATH entry is user-owned; uninstall must leave it alone.
    $preexisting = $baseline + ';' + $directory
    $environment.SetValue('Path', $preexisting, [Microsoft.Win32.RegistryValueKind]::String)
    RunSetup $InstallerPath $directory
    Assert ((PathCount) -eq 1) 'Preexisting PATH entry duplicated.'
    Uninstall
    Assert ((UserPath) -ceq $preexisting) 'User-owned PATH entry removed.'
    Assert ($environment.GetValueKind('Path') -eq [Microsoft.Win32.RegistryValueKind]::String) 'REG_SZ was not preserved.'
    AssertData
    $environment.SetValue('Path', $baseline, [Microsoft.Win32.RegistryValueKind]::ExpandString)
    RunSetup $InstallerPath $directory
    Uninstall
    Assert ((UserPath) -ceq $baseline) 'Uninstall did not restore the exact original PATH.'
    AssertData
    Write-Host 'PASS: foreign-folder protection, install, actual older-binary upgrade, repair, aliases, shortcuts, PATH ownership/type, tasks/config and uninstall preservation.'
} finally {
    # Cleanup affects only this ephemeral runner and the unique test directory.
    if (Test-Path -LiteralPath (Join-Path $directory 'unins000.exe')) {
        try { Uninstall } catch { Write-Warning $_ }
    }
    if ($originalHasPath) { $environment.SetValue('Path', $originalPath, $originalKind) }
    else { $environment.DeleteValue('Path', $false) }
    $environment.Dispose()
    $env:Path = $originalProcessPath
    if ($configExisted) { [IO.File]::WriteAllBytes($configFile, $originalConfig) }
    else { Remove-Item -LiteralPath $configFile -ErrorAction SilentlyContinue }
    if (Test-Path -LiteralPath $root) { Remove-Item -LiteralPath $root -Recurse -Force }
}
