//! User directories shared by configuration, themes, notes and the update cache.

use std::path::PathBuf;

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn absolute_override(name: &str) -> Option<PathBuf> {
    let path = env_path(name)?;
    if path.is_absolute() {
        Some(path)
    } else {
        eprintln!(
            "Tuxedo Task: ignoring non-absolute {name}={:?}",
            path.display()
        );
        None
    }
}

/// Preserve explicit HOME; native Windows shells normally provide USERPROFILE.
pub fn home_dir() -> Option<PathBuf> {
    choose_home(env_path("HOME"), env_path("USERPROFILE"), cfg!(windows))
}

fn choose_home(home: Option<PathBuf>, profile: Option<PathBuf>, windows: bool) -> Option<PathBuf> {
    home.or(if windows { profile } else { None })
}

/// XDG overrides win. Keep an existing legacy ~/.config/tuxedo directory on
/// Windows; otherwise use APPDATA. Unix keeps the existing XDG/HOME behavior.
pub fn config_home() -> Option<PathBuf> {
    if let Some(path) = absolute_override("XDG_CONFIG_HOME") {
        return Some(path);
    }
    let home = home_dir();
    let legacy_exists = home
        .as_ref()
        .is_some_and(|path| path.join(".config/tuxedo").is_dir());
    config_base(
        home,
        absolute_override("APPDATA"),
        cfg!(windows),
        legacy_exists,
    )
}

fn config_base(
    home: Option<PathBuf>,
    roaming: Option<PathBuf>,
    windows: bool,
    legacy_exists: bool,
) -> Option<PathBuf> {
    let legacy = home.map(|path| path.join(".config"));
    if windows && !legacy_exists {
        roaming.or(legacy)
    } else {
        legacy
    }
}

/// Cache is disposable; native Windows uses LOCALAPPDATA rather than requiring
/// HOME. Explicit XDG_CACHE_HOME remains supported on every platform.
pub fn cache_home() -> Option<PathBuf> {
    absolute_override("XDG_CACHE_HOME")
        .or_else(|| cache_base(home_dir(), absolute_override("LOCALAPPDATA"), cfg!(windows)))
}

fn cache_base(home: Option<PathBuf>, local: Option<PathBuf>, windows: bool) -> Option<PathBuf> {
    if windows {
        local.or_else(|| home.map(|path| path.join(".cache")))
    } else {
        home.map(|path| path.join(".cache"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(value: &str) -> Option<PathBuf> {
        Some(PathBuf::from(value))
    }

    #[cfg(windows)]
    #[test]
    fn native_windows_directories_without_home_in_isolated_process() {
        const CHILD: &str = "TUXEDO_WINDOWS_PATH_TEST";
        if let Some(mode) = std::env::var_os(CHILD) {
            let home = PathBuf::from(std::env::var_os("USERPROFILE").expect("profile"));
            assert_eq!(home_dir(), Some(home.clone()));
            let base = if mode == "legacy" {
                home.join(".config")
            } else {
                PathBuf::from(std::env::var_os("APPDATA").expect("roaming"))
            };
            assert_eq!(config_home(), Some(base.clone()));
            assert_eq!(
                cache_home(),
                std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
            );
            let config = crate::config::Config::path().expect("native Windows config");
            assert_eq!(config, base.join("tuxedo/config.toml"));
            let settings = crate::config::Config {
                mouse: false,
                ..Default::default()
            };
            settings
                .save_to(&config)
                .expect("persist native configuration");
            assert!(
                !crate::config::Config::load_strict(&config)
                    .expect("read native config")
                    .mouse
            );
            return;
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "tuxedo-windows-paths-{}-{stamp}",
            std::process::id()
        ));
        for mode in ["native", "legacy"] {
            let profile = root.join(mode).join("profile");
            std::fs::create_dir_all(&profile).expect("profile directory");
            if mode == "legacy" {
                std::fs::create_dir_all(profile.join(".config/tuxedo")).expect("legacy directory");
            }
            let output =
                std::process::Command::new(std::env::current_exe().expect("test executable"))
                    .args([
                        "--exact",
                        "xdg::tests::native_windows_directories_without_home_in_isolated_process",
                        "--nocapture",
                    ])
                    .env(CHILD, mode)
                    .env_remove("HOME")
                    .env_remove("XDG_CONFIG_HOME")
                    .env_remove("XDG_CACHE_HOME")
                    .env("USERPROFILE", &profile)
                    .env("APPDATA", root.join(mode).join("roaming"))
                    .env("LOCALAPPDATA", root.join(mode).join("local"))
                    .output()
                    .expect("isolated Windows test process");
            assert!(output.status.success(), "{output:?}");
        }
        std::fs::remove_dir_all(root).expect("cleanup isolated fixtures");
    }

    #[test]
    fn native_windows_home_does_not_require_home_environment_variable() {
        assert_eq!(choose_home(None, path("profile"), true), path("profile"));
        assert_eq!(
            choose_home(path("home"), path("profile"), true),
            path("home")
        );
        assert_eq!(choose_home(None, path("profile"), false), None);
    }

    #[test]
    fn windows_uses_roaming_config_but_keeps_existing_legacy_directory() {
        assert_eq!(
            config_base(path("home"), path("roaming"), true, false),
            path("roaming")
        );
        assert_eq!(
            config_base(path("home"), path("roaming"), true, true),
            Some(PathBuf::from("home").join(".config"))
        );
        assert_eq!(
            config_base(None, path("roaming"), true, false),
            path("roaming")
        );
        assert_eq!(
            config_base(path("home"), None, true, false),
            Some(PathBuf::from("home").join(".config"))
        );
    }

    #[test]
    fn unix_config_and_cache_stay_unchanged() {
        assert_eq!(
            config_base(path("home"), path("roaming"), false, false),
            Some(PathBuf::from("home").join(".config"))
        );
        assert_eq!(
            cache_base(path("home"), path("local"), false),
            Some(PathBuf::from("home").join(".cache"))
        );
        assert_eq!(config_base(None, path("roaming"), false, false), None);
    }

    #[test]
    fn windows_cache_uses_local_appdata_without_home() {
        assert_eq!(cache_base(None, path("local"), true), path("local"));
        assert_eq!(cache_base(path("home"), path("local"), true), path("local"));
        assert_eq!(
            cache_base(path("home"), None, true),
            Some(PathBuf::from("home").join(".cache"))
        );
    }
}
