# Releasing Tuxedo Task

End users install precompiled binaries. Rust/Cargo is needed by maintainers and
GitHub Actions, **not** for a Homebrew install or a release download.

## 1. Prepare the source on main

- Bump the package version in `Cargo.toml` and its own entry in `Cargo.lock`.
  Keep dependencies locked. Versions use CalVer, for example `2026.10.1`.
- Update version-specific download examples in `README.md`.
- Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`,
  and `cargo test --all-targets --locked`. If the status bar's version changes,
  review and update the rendered snapshots and regenerate the screenshots.
- Run the website's `npm run check`, `npm run build`, and `npm test`.
- Integrate the reviewed source into `main` and wait for CI. The website deploys
  from `main`; feature branches do not publish production directly.

## 2. Publish a version tag

```sh
git tag v2026.10.1
git push mine v2026.10.1
```

Use the actual version, not the example above. Do not move or reuse published
version tags. `.github/workflows/release.yml` validates the tag against Cargo,
runs the Rust checks, and builds on native runners for:

- macOS Apple Silicon and Intel (deployment target macOS 12).
- Linux ARM64 and x86-64 (Ubuntu 22.04 / glibc 2.35 baseline).
- Windows x86-64.

Every archive includes the executable, `tuxedotask` alias, README and MIT license,
plus an adjacent `.sha256` checksum. Unix archives use a relative symlink;
Windows ZIPs include a second copy of the executable. Packaging smoke-tests the
alias before uploading. Release publication waits for **all** targets.

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

- Verify all five release archives and checksums are present.
- On a machine authorized for installation, test the formula and both command
  names. Confirm `--version` identifies Tuxedo Task and `update` recommends the
  fork's formula, not upstream.
- Exercise CLI mutations with isolated sample files/config, not personal tasks.
- Check the website and documentation use the new Homebrew/download routes.
- Homebrew users update with `brew update` and
  `brew upgrade caracolaracol/tap/tuxedotask`.
