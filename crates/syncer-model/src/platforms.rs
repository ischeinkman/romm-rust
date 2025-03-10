//! Tools used for determining the platform we're running on.

use std::path::Path;

/// Different platforms the ROMM sync tool supports, for deriving things like
/// config & socket paths.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Platform {
    /// We're running on Onion OS on the Miyoo Mini
    MiyooMiniOnion,
    /// We're running in Windows
    Windows,
    /// We're running on Linux, or we don't know what the system is and assume
    /// it is Linux
    #[default]
    Linux,
    /// We're running on some sort of Mac
    Mac,
}

impl Platform {
    /// Retrieves the current platform based on what we can see from the
    /// environment.
    #[cfg(target_os = "windows")]
    pub fn get() -> Self {
        Platform::Windows
    }

    /// Retrieves the current platform based on what we can see from the
    /// environment.
    #[cfg(target_os = "macos")]
    pub fn get() -> Self {
        Platform::Mac
    }

    /// Retrieves the current platform based on what we can see from the
    /// environment.
    #[cfg(all(target_os = "linux", target_arch = "arm", target_abi = "eabihf"))]
    pub fn get() -> Self {
        use std::sync::LazyLock;

        // Wrap this in a `LazyLock` so we only need to do the check once
        static CACHE: LazyLock<Platform> =
            LazyLock::new(
                || match std::fs::exists("/mnt/SDCARD/.tmp_update/onionVersion") {
                    Ok(true) => Platform::MiyooMiniOnion,
                    _ => Platform::Linux,
                },
            );
        *CACHE
    }

    /// Retrieves the current platform based on what we can see from the
    /// environment.
    #[cfg(all(
        target_os = "linux",
        not(all(target_arch = "arm", target_abi = "eabihf"))
    ))]
    pub fn get() -> Self {
        Platform::Linux
    }

    /// Retrieves the current platform based on what we can see from the
    /// environment.
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    pub fn get() -> Self {
        Platform::default()
    }

    /// The place(s) to look for the config file(s).
    pub fn config_input_paths(&self) -> impl Iterator<Item = &Path> {
        const MIYOO_PATHS: &[&str] = &["config.toml"];
        const LINUX_PATHS: &[&str] = &[
            "/etc/romm-syncer/config.toml",
            "~/.config/romm-syncer/config.toml",
        ];
        let raw = match self {
            Platform::MiyooMiniOnion => MIYOO_PATHS,
            Platform::Linux => LINUX_PATHS,
            _ => todo!("Platform not yet supported"),
        };
        raw.iter().map(Path::new)
    }

    /// The path on the system new configs should be written to.
    pub fn config_save_path(&self) -> &Path {
        match self {
            Platform::MiyooMiniOnion => Path::new("config.toml"),
            Platform::Linux => Path::new("~/.config.romm-syncer/config.toml"),
            _ => todo!("Platform not yet supported"),
        }
    }

    /// The place to open the named socket on the platform.
    pub fn socket_path(&self) -> String {
        match *self {
            Platform::MiyooMiniOnion => "daemon-socket.socket".into(),
            Platform::Linux => "~/.config/romm-syncer/daemon-socket.socket".into(),
            _ => todo!("Platform not yet supported"),
        }
    }
}
