# Releasing Tuxedo Task

End users install precompiled binaries. Rust/Cargo is needed by maintainers and
GitHub Actions, **not** for a Homebrew install or a release download.

## 1. Prepare the source on main

- Bump the package version in `Cargo.toml` and its own entry in `Cargo.lock`.
  Keep dependencies locked. `2026.10.4` is the CalVer-to-SemVer bridge release;
  the following release starts at SemVer `1.0.0`. After that, increment MAJOR
  for incompatible todo.txt/CLI/config changes, MINOR for compatible features,
  and PATCH for compatible fixes.
- Update version-specific download examples in `docs/installation.md`.
- Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`,
  and `cargo test --all-targets --locked`. If the status bar's version changes,
  review and update the rendered snapshots and regenerate the screenshots.
- Run the website's `npm run check`, `npm run build`, and `npm test`.
- Integrate the reviewed source into `main` and wait for CI. The website deploys
  from `main`; feature branches do not publish production directly.

## 2. Publish a version tag

```sh
git tag v2026.10.4
git push mine v2026.10.4
```

Use the actual version, not the example above. Do not move or reuse published
version tags. `.github/workflows/release.yml` validates the tag against Cargo,
runs the Rust checks, and builds on native runners for:

- macOS Apple Silicon and Intel (deployment target macOS 12).
- Linux ARM64 and x86-64 (Ubuntu 22.04 / glibc 2.35 baseline).
- Windows x86-64.

Every archive includes the executable, `tuxedotask` alias, README and MIT license,
plus an adjacent `.sha256` checksum. The README links to the online guides and
artwork using absolute URLs; only its bundled `LICENSE` link is local. It does
not require a `docs/` folder beside the executable or promise offline guides.
Unix archives use a relative symlink; Windows ZIPs include a second copy of the executable. Packaging smoke-tests the
alias before uploading. Release publication waits for **all** targets.

### Windows installer gate

Before tagging, wait for `.github/workflows/windows.yml` to pass on the exact
source revision. It runs native Rust tests, static-CRT import checks and the
explicitly authorized installer lifecycle tests in disposable Windows. It uses
preinstalled Inno Setup 6; no local installer tools are required. See
[`packaging/windows/README.md`](../packaging/windows/README.md) for safety rules
and the real previous-release installer fixture. Never publish a test fixture.

The release's Windows job also compiles the installer and publishes its stable
filename plus SHA-256, alongside the portable ZIP. Initial releases are unsigned:
keep the SmartScreen notice accurate; do not imply code signing or WinGet
acceptance. Test installation, upgrade/repair and uninstall without deleting
any task files/configuration. The user PATH entry must be reversible and must
not remove a preexisting entry or other software's subsequent edits.

After release publication, update the website/README to offer the installer
through `releases/latest/download/tuxedo-task-windows-x86_64-setup.exe`.
Do not advertise a download that has not been published yet.

## 3. Update the Homebrew tap

The formula lives in `caracolaracol/homebrew-tap`, not in the upstream Webstone
tap. After the GitHub release is available:

```sh
gh workflow run update.yml --repo caracolaracol/homebrew-tap
```

The tap's workflow also checks for updates on a schedule. It downloads the
latest stable release metadata and checksums, verifies the expected four archive
names, generates `Formula/tuxedotask.rb`, and commits changes using that tap's
own short-lived GitHub token. No cross-repository PAT is stored in this repo.

The formula downloads the matching binary and declares **no Rust/Cargo build
dependency**. It installs `tuxedo` and a `tuxedotask` symlink pointing to it. It
explicitly conflicts with the upstream `tuxedo` Homebrew formula. Never use
`brew link --overwrite` or silently remove an existing installation to hide a
conflict. Task data and `~/.config/tuxedo` stay unchanged.

## 4. Verify delivery

- Verify all five release archives/checksums and the Windows Setup.exe/checksum
  are present. The installer asset is `tuxedo-task-windows-x86_64-setup.exe`.
- On a machine authorized for installation, test the formula and both command
  names. Confirm `--version` identifies Tuxedo Task and `update` recommends the
  fork's formula, not upstream.
- Exercise CLI mutations with isolated sample files/config, not personal tasks.
- Check the website and documentation use the new Homebrew/download routes.
- Homebrew users update with `brew update` and
  `brew upgrade caracolaracol/tap/tuxedotask`.
