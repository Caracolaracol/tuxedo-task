# Windows installer

`Build-Installer.ps1` builds a per-user Inno Setup 6 installer from the two
precompiled Windows executables. The `windows-2025` GitHub image already has
Inno Setup: scripts fail rather than installing tools automatically.

```powershell
./packaging/windows/Build-Installer.ps1 -Version 2026.10.4 -BinaryDir dist/stage -OutputDir dist
```

## Guarantees and limits

- Windows 10 1809+ / Windows 11, x86-64; no administrator prompt, Rust or Cargo.
- Default destination: `%LOCALAPPDATA%\Programs\TuxedoTask`.
- Both `tuxedo.exe` and `tuxedotask.exe` are installed.
- User PATH integration is optional. It appends an entry, never replaces other
  programs or forces this app ahead of an existing `tuxedo` installation.
- Raw PATH text, environment references and registry value type are preserved.
  An already-present entry stays user-owned. Uninstall removes only the entry
  the installer added and preserves edits made by other software afterwards.
- An unrecognized nonempty destination is refused. Updates keep the original
  directory, avoiding orphaned PATH entries or overwriting a portable install.
- Start menu / optional desktop shortcuts launch `cmd /D /S /K` with a quoted
  executable and Documents as working directory, never the program directory.
- Uninstall removes only registered program files, shortcuts and installer
  metadata. Tasks, configuration, notes and unregistered files are not deleted.
- Native configuration falls back to `%APPDATA%\tuxedo` without `HOME`.
  Explicit XDG paths and an existing legacy `~/.config/tuxedo` are respected.
- The native Windows binaries use static CRT linkage; CI checks PE imports for
  accidental Visual C++ redistributable dependencies.
- This first installer is **unsigned**. SmartScreen may warn. No security
  settings are disabled. Code signing and a WinGet submission are future work,
  not claims of an accepted package or a trusted publisher.
- PATH changes require a new terminal. If another `tuxedo` wins PATH lookup,
  use the shortcut/full path and inspect `Get-Command tuxedo -All`.

## Validation

The `Windows installer` workflow runs all Rust tests natively, builds the
installer, and executes `Test-Installer.ps1` only on an ephemeral Windows runner.
It tests foreign-directory protection, first install, upgrade, repair, command
aliases, shortcut quoting/working directory, PATH ownership and both registry
string types, and data preservation across uninstall. Tests snapshot and restore
runner state even on failure; the test script refuses normal local execution.

`Build-UpgradeFixture.ps1` downloads the actual published v2026.10.3 Setup.exe
and verifies its adjacent checksum. CI upgrades that installation to the
candidate release, exercising the same setup-to-setup path users will take.
The fixture is only used on the disposable Windows runner and is never included
in release assets.

Release builds publish `tuxedo-task-windows-x86_64-setup.exe` plus its adjacent
SHA-256 file. The filename is stable within each versioned GitHub release, so
`releases/latest/download/tuxedo-task-windows-x86_64-setup.exe` stays usable.
CI test artifacts retain the version in their filenames.

The ICO is generated from the project's original MIT-licensed mask SVG
(`src/serve/assets/icon.svg`), not third-party character artwork. It contains
16, 32, 48 and 256-pixel PNG icon images.
