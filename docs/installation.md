# Installation

Choose your system: Homebrew for macOS/Linux, or the Windows installer below.
Neither option needs Rust or Cargo.

## Recommended: Homebrew (macOS and Linux, no Cargo required)

With [Homebrew](https://brew.sh/) available:

```sh
brew install caracolaracol/tap/tuxedotask
```

Homebrew downloads a **precompiled binary** and verifies its SHA-256 checksum.
It does not install Rust or Cargo or compile the application on your machine.
The tap supports Apple Silicon / Intel Macs (macOS 12+) and ARM64 / x86-64 Linux.
Linux binaries are built on Ubuntu 22.04 and require glibc 2.35 or newer.

Both command names are available: **`tuxedo` and `tuxedotask` run the same
application**. After installation, follow [Usage](usage.md#usage) to open or create your
task file.

The config stays under `~/.config/tuxedo` (or your XDG config directory), and
existing task files are not migrated or deleted.

**Already have upstream Tuxedo installed?** `brew install tuxedo` installs the
original application, not this fork. Our formula conflicts with the upstream
Homebrew formula because both provide `tuxedo`; choose which one you want linked
rather than forcing an overwrite. Homebrew does not remove an existing Cargo or
manual copy. Check `command -v tuxedo` / `command -v tuxedotask` to confirm PATH
precedence. Back up your task files and config before switching clients.

## Windows installer (recommended)

1. Download [Tuxedo Task Setup.exe](https://github.com/Caracolaracol/tuxedo-task/releases/latest/download/tuxedo-task-windows-x86_64-setup.exe)
   from our official GitHub release. Windows 10 (1809+) / Windows 11, x86-64.
2. Run the setup wizard. It installs for **your user only** in
   `%LOCALAPPDATA%\Programs\TuxedoTask`, without an administrator prompt.
   Leave the optional user PATH checkbox selected to use `tuxedo` and
   `tuxedotask` in a new terminal; a Start menu shortcut is also created.
3. Open **Tuxedo Task** from Start, or close/reopen your terminal and follow
   [Usage](usage.md#usage) to choose your task file.

The shortcut opens a terminal in your Documents folder, not the program folder.
Installing alone does not create a personal `todo.txt`: choose a path when
opening the app, or choose `c` on the first-run welcome screen. Setup never
removes another Cargo/manual/upstream copy. The shortcut always opens this
installation even if another copy appears earlier on PATH.

To update, close Tuxedo Task and run the newer Setup.exe in the same location.
Uninstall through Windows Settings → Apps. Tasks, configuration, notes and
unregistered files are kept; uninstall removes only this installer's PATH
entry, not entries you already had or changes made by other software.

**Unsigned initial release:** Windows SmartScreen may show an unknown-publisher
warning. Download only from the official release and compare its
[SHA-256 file](https://github.com/Caracolaracol/tuxedo-task/releases/latest/download/tuxedo-task-windows-x86_64-setup.exe.sha256).
Do not disable Windows security. An optional checksum command is in the
[CLI reference](cli.md#verify-a-windows-installer); it is not an installation step.

## Precompiled downloads (portable option, including Windows)

Download your platform archive and its `.sha256` file from
[this fork's Releases](https://github.com/Caracolaracol/tuxedo-task/releases/latest).
Rust and Cargo are **not** needed for these downloads either.

| Platform | Archive target |
| --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| macOS Intel | `x86_64-apple-darwin` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| Linux x86-64 | `x86_64-unknown-linux-gnu` |
| Windows x86-64 | `x86_64-pc-windows-msvc` |

For example, for the v2026.10.4 Apple Silicon archive and checksum in the same
directory (match the version in your own downloaded filenames):

```sh
shasum -a 256 -c tuxedo-task-v2026.10.4-aarch64-apple-darwin.tar.gz.sha256
tar -xzf tuxedo-task-v2026.10.4-aarch64-apple-darwin.tar.gz
```

Extract the matching archive and place `tuxedo` and `tuxedotask` in a directory
on PATH, keeping the Unix symlink beside its target. On Windows, the ZIP contains
`tuxedo.exe` and `tuxedotask.exe`; compare `Get-FileHash -Algorithm SHA256` with the
provided checksum before extracting. Both names share the same tasks and config.

## Optional: install from source with Cargo

Only source builds need **Git**, a **current stable Rust toolchain** (Rust 2024
edition, including Cargo), and native linker/build tools. Use
[rustup](https://rustup.rs/) for Rust; native tools come from Xcode Command Line
Tools on macOS, your distribution's C build tools on Linux, or Visual Studio C++
Build Tools for the MSVC toolchain on Windows.

```sh
cargo install --git https://github.com/Caracolaracol/tuxedo-task --locked
```

The Cargo package/executable remains named **`tuxedo`** for compatibility. The
`tuxedotask` alias is supplied by Homebrew and the precompiled archives, not by
`cargo install`. Cargo installs into `$CARGO_HOME/bin` (normally `~/.cargo/bin`
or `%USERPROFILE%\.cargo\bin`); make sure it is on PATH. To replace an existing
Cargo-installed copy, add `--force`. No install method moves your task files.

## Alternatively: clone and run without installing

```sh
git clone https://github.com/Caracolaracol/tuxedo-task.git
cd tuxedo-task
cargo build --release --locked
```

The executable is in `target/release/`. Follow [Usage](usage.md#usage) to launch the
interface with a task file; replace `tuxedo` with `./target/release/tuxedo` on
macOS/Linux or `.\target\release\tuxedo.exe` on Windows.

## Updates

For Homebrew installations:

```sh
brew update
brew upgrade caracolaracol/tap/tuxedotask
```

For Cargo installations:

```sh
cargo install --git https://github.com/Caracolaracol/tuxedo-task --locked --force
```

For manual downloads, replace the executable and alias from the latest matching
release archive after checking its checksum. For cloned checkouts, run
`git pull --ff-only` and `cargo build --release --locked` in the checkout.

---

[Documentation index](README.md) · [Project overview](../README.md)
