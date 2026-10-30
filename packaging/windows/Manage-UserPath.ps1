[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateSet('Install', 'Remove')][string]$Action,
    [Parameter(Mandatory)][string]$InstallDir
)
$ErrorActionPreference = 'Stop'
$statePath = 'Software\Caracolaracol\TuxedoTask'

function Normalize-Directory([string]$Value, [bool]$Expand = $false) {
    $value = $Value.Trim().Trim('"')
    if ($Expand) { $value = [Environment]::ExpandEnvironmentVariables($value) }
    if (-not $value) { return '' }
    try { $value = [IO.Path]::GetFullPath($value) } catch { return $value }
    if ($value.Length -gt 3) { $value = $value.TrimEnd([char[]]'\/') }
    return $value
}

$directory = Normalize-Directory $InstallDir
if (-not [IO.Path]::IsPathRooted($directory) -or $directory.Contains(';') -or $directory.Contains('%')) {
    throw 'The PATH directory must be absolute and cannot contain ; or %.'
}
$environment = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Environment')
$state = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey($statePath)
try {
    $owned = [string]$state.GetValue('OwnedPath', '')
    $hadPath = $environment.GetValueNames() -icontains 'Path'
    $raw = [string]$environment.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
    $kind = [Microsoft.Win32.RegistryValueKind]::ExpandString
    if ($hadPath) { $kind = $environment.GetValueKind('Path') }
    if ($kind -notin @([Microsoft.Win32.RegistryValueKind]::String, [Microsoft.Win32.RegistryValueKind]::ExpandString)) {
        throw 'Unexpected user PATH registry type; refusing to modify it.'
    }
    if ($Action -eq 'Install') {
        $present = @($raw.Split(';') | Where-Object { (Normalize-Directory $_ $true) -ieq $directory }).Count -gt 0
        if ($owned -and (Normalize-Directory $owned) -ine $directory) {
            throw 'A different installation owns a PATH entry. Keep the existing install location.'
        }
        if (-not $present) {
            $after = if ($raw.Length) { $raw + ';' + $directory } else { $directory }
            $state.SetValue('PathBefore', $raw, [Microsoft.Win32.RegistryValueKind]::String)
            $state.SetValue('PathAfter', $after, [Microsoft.Win32.RegistryValueKind]::String)
            $state.SetValue('PathKindBefore', [int]$kind, [Microsoft.Win32.RegistryValueKind]::DWord)
            $state.SetValue('PathExistedBefore', [int]$hadPath, [Microsoft.Win32.RegistryValueKind]::DWord)
            $state.SetValue('OwnedPath', $directory, [Microsoft.Win32.RegistryValueKind]::String)
            $environment.SetValue('Path', $after, $kind)
        }
        # An existing user-supplied entry is not owned and must survive uninstall.
    } elseif ($owned -and (Normalize-Directory $owned) -ieq $directory) {
        $after = [string]$state.GetValue('PathAfter', '')
        if ($raw -ceq $after) {
            if ([int]$state.GetValue('PathExistedBefore', 1)) {
                $previousKind = [Microsoft.Win32.RegistryValueKind][int]$state.GetValue('PathKindBefore', [int]$kind)
                $environment.SetValue('Path', [string]$state.GetValue('PathBefore', ''), $previousKind)
            } else { $environment.DeleteValue('Path', $false) }
        } else {
            # Other software/user edits happened after installation. Remove just
            # our first matching entry; never restore a stale complete PATH.
            $removed = $false
            $remaining = [Collections.Generic.List[string]]::new()
            foreach ($entry in $raw.Split(';')) {
                if (-not $removed -and (Normalize-Directory $entry $true) -ieq $directory) { $removed = $true }
                else { $remaining.Add($entry) }
            }
            if ($removed) { $environment.SetValue('Path', ($remaining -join ';'), $kind) }
        }
        foreach ($name in @('OwnedPath', 'PathBefore', 'PathAfter', 'PathKindBefore', 'PathExistedBefore')) {
            $state.DeleteValue($name, $false)
        }
    }
} finally {
    $environment.Dispose()
    $state.Dispose()
}
if ($Action -eq 'Remove') {
    $remainingState = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($statePath)
    if ($remainingState) {
        $empty = $remainingState.ValueCount -eq 0 -and $remainingState.SubKeyCount -eq 0
        $remainingState.Dispose()
        if ($empty) { [Microsoft.Win32.Registry]::CurrentUser.DeleteSubKey($statePath, $false) }
    }
}
