use std::path::{Path, PathBuf};

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
        Platform::MiyooMini
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

    pub fn socket_path(&self) -> PathBuf {
        match *self {
            Platform::MiyooMini => "daemon-socket.socket".into(),
            Platform::Linux => "~/.config/romm-syncer/daemon-socket.socket".into(),
            _ => todo!("Platform not yet supported"),
        }
    }
}
