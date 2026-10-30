#ifndef AppVersion
  #error AppVersion must be supplied by CI
#endif
#ifndef SourceDir
  #error SourceDir must point at the staged Windows binaries
#endif
#ifndef OutputDir
  #error OutputDir must be supplied by CI
#endif

[Setup]
AppId={{BEE5823B-1635-4B5B-BA07-5E392807F3E0}
AppName=Tuxedo Task
AppVersion={#AppVersion}
AppPublisher=Caracolaracol
AppPublisherURL=https://github.com/Caracolaracol/tuxedo-task
AppSupportURL=https://github.com/Caracolaracol/tuxedo-task/issues
AppUpdatesURL=https://github.com/Caracolaracol/tuxedo-task/releases/latest
DefaultDirName={localappdata}\Programs\TuxedoTask
DefaultGroupName=Tuxedo Task
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
OutputDir={#OutputDir}
OutputBaseFilename=tuxedo-task-v{#AppVersion}-windows-x86_64-setup
SetupIconFile=tuxedo-task.ico
UninstallDisplayIcon={app}\tuxedo-task.ico
LicenseFile=..\..\LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ChangesEnvironment=yes
CloseApplications=no
RestartApplications=no
UsePreviousAppDir=yes
UsePreviousTasks=yes

[Tasks]
Name: "addtopath"; Description: "Add tuxedo and tuxedotask to your user PATH"; Flags: checkedonce
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "{#SourceDir}\tuxedo.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\tuxedotask.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "tuxedo-task.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "Manage-UserPath.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Tuxedo Task"; Filename: "{cmd}"; Parameters: "/D /S /K """"{app}\tuxedo.exe"""""; WorkingDir: "{userdocs}"; IconFilename: "{app}\tuxedo-task.ico"; Comment: "Open your local task manager in a terminal"
Name: "{autodesktop}\Tuxedo Task"; Filename: "{cmd}"; Parameters: "/D /S /K """"{app}\tuxedo.exe"""""; WorkingDir: "{userdocs}"; IconFilename: "{app}\tuxedo-task.ico"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Caracolaracol\TuxedoTask"; ValueType: string; ValueName: "InstallDir"; ValueData: "{app}"; Flags: uninsdeletevalue uninsdeletekeyifempty

[Run]
Filename: "{cmd}"; Parameters: "/D /S /K """"{app}\tuxedo.exe"""""; WorkingDir: "{userdocs}"; Description: "Open Tuxedo Task"; Flags: nowait postinstall skipifsilent unchecked

[Code]
const
  StateKey = 'Software\Caracolaracol\TuxedoTask';
var
  UninstallPathScript: String;

function SameDirectory(A, B: String): Boolean;
begin
  Result := CompareText(RemoveBackslashUnlessRoot(A), RemoveBackslashUnlessRoot(B)) = 0;
end;

function HasContents(Directory: String): Boolean;
var
  Item: TFindRec;
begin
  Result := False;
  if FindFirst(AddBackslash(Directory) + '*', Item) then begin
    try
      repeat
        if (Item.Name <> '.') and (Item.Name <> '..') then begin
          Result := True;
          Break;
        end;
      until not FindNext(Item);
    finally
      FindClose(Item);
    end;
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Previous, Target: String;
begin
  Result := '';
  Target := ExpandConstant('{app}');
  if RegQueryStringValue(HKCU, StateKey, 'InstallDir', Previous) then begin
    if not SameDirectory(Previous, Target) then
      Result := 'To update, keep the existing installation directory: ' + Previous;
  end else if HasContents(Target) then
    Result := 'The selected directory is not empty and is not a recognized Tuxedo Task installation. Choose an empty directory. Existing programs and task files will not be overwritten.';
  if WizardIsTaskSelected('addtopath') and ((Pos(';', Target) > 0) or (Pos('%', Target) > 0)) then
    Result := 'For PATH integration, choose a directory without semicolons or percent signs, or uncheck the PATH option.';
end;

function RunPathScript(Script, Operation: String): Boolean;
var
  ExitCode: Integer;
  Arguments: String;
begin
  Arguments := '-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File ' +
    AddQuotes(Script) + ' -Action ' + Operation + ' -InstallDir ' + AddQuotes(ExpandConstant('{app}'));
  Result := Exec(ExpandConstant('{sys}\WindowsPowerShell\v1.0\powershell.exe'),
    Arguments, '', SW_HIDE, ewWaitUntilTerminated, ExitCode);
  if Result then Result := ExitCode = 0;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  Operation: String;
begin
  if CurStep = ssPostInstall then begin
    if WizardIsTaskSelected('addtopath') then Operation := 'Install' else Operation := 'Remove';
    if not RunPathScript(ExpandConstant('{app}\Manage-UserPath.ps1'), Operation) then
      SuppressibleMsgBox('The application was installed, but user PATH could not be updated. Use the Start menu shortcut. No other installed programs were removed.', mbInformation, MB_OK, IDOK);
  end;
end;

function InitializeUninstall(): Boolean;
begin
  UninstallPathScript := ExpandConstant('{tmp}\tuxedo-manage-path.ps1');
  Result := FileCopy(ExpandConstant('{app}\Manage-UserPath.ps1'), UninstallPathScript, False);
  if not Result then
    SuppressibleMsgBox('Cannot prepare PATH cleanup. Repair this installation with Setup before uninstalling.', mbError, MB_OK, IDOK);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then begin
    if not RunPathScript(UninstallPathScript, 'Remove') then
      SuppressibleMsgBox('The application was removed, but its user PATH entry could not be cleaned up automatically. Your task files and configuration were not removed.', mbInformation, MB_OK, IDOK);
  end;
end;
