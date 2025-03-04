use std::path::{Path, PathBuf};

/// Different platforms the ROMM sync tool supports, for deriving things like
/// config & socket paths.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Platform {
    MiyooMini,
    Windows,
    #[default]
    Linux,
    Mac,
}

impl Platform {
    #[cfg(target_os = "windows")]
    pub fn get() -> Self {
        Platform::Windows
    }

    #[cfg(target_os = "macos")]
    pub fn get() -> Self {
        Platform::Mac
    }

    #[cfg(all(target_os = "linux", target_arch = "arm", target_abi = "eabihf"))]
    pub fn get() -> Self {
        use std::sync::LazyLock;

        // Wrap this in a `LazyLock` so we only need to do the check once
        static CACHE: LazyLock<Platform> = LazyLock::new(|| {
            // TODO: Detect this properly
            //
            // NOTES:
            // * The Miyoo Mini doesn't have any standard OS detection systems
            //   (/etc/os-release, uname, etc)
            // * The Miyoo Mini doesn't have any info under /sys referencing
            //   itself
            // * The only thing I could find currently is a `/etc/fw_printenv`
            //   program that seems to print out environment variables for ...
            //   something, some of which do indeed reference the fact that its
            //   running on a Miyoo Mini
            Platform::MiyooMini
        });
        *CACHE
    }

    #[cfg(all(
        target_os = "linux",
        not(all(target_arch = "arm", target_abi = "eabihf"))
    ))]
    pub fn get() -> Self {
        Platform::Linux
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    pub fn get() -> Self {
        Platform::default()
    }

    /// The place(s) to look for the config file(s).
    pub fn config_paths(&self) -> impl Iterator<Item = &Path> {
        const MIYOO_PATHS: &[&str] = &["sync_config.toml"];
        const LINUX_PATHS: &[&str] = &[
            "/etc/romm-syncer/config.toml",
            "~/.config/romm-syncer/config.toml",
        ];
        let raw = match self {
            Platform::MiyooMini => MIYOO_PATHS,
            Platform::Linux => LINUX_PATHS,
            _ => todo!("Platform not yet supported"),
        };
        raw.iter().map(Path::new)
    }

    /// The place to open the named socket on the platform.
    pub fn socket_path(&self) -> PathBuf {
        match *self {
            Platform::MiyooMini => "daemon-socket.socket".into(),
            Platform::Linux => "~/.config/romm-syncer/daemon-socket.socket".into(),
            _ => todo!("Platform not yet supported"),
        }
    }
}
