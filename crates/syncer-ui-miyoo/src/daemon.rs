use std::{io, path::Path};

use procfs::{ProcError, process};
use thiserror::Error;
use tokio::{fs, process::Command};

const SERVICE_INSTALL_PATH: &str = "/mnt/SDCARD/.tmp_update/startup/start-daemon.sh";
const DAEMON_EXE_PATH: &str = "./syncer-daemon";
const SERVICE_PATH: &str = "./start-daemon.sh";

/// Installs the daemon into the Miyoo Mini, including:
///
/// * Moving all files to the correct locations.
/// * Telling the operating system to start the daemon on boot.
/// * Starting the daemon now.
pub async fn install_daemon() -> Result<(), DaemonError> {
    fs::copy(SERVICE_PATH, SERVICE_INSTALL_PATH).await?;
    start_daemon().await?;
    Ok(())
}

/// Uninstalls the daemon from the Miyoo Mini, including:
///
/// * Stopping the daemon.
/// * Telling the operating system to NOT start the daemon on boot.
/// * Removing any stray files in the OS.
pub async fn uninstall_daemon() -> Result<(), DaemonError> {
    stop_daemon().await?;
    match fs::remove_file(SERVICE_INSTALL_PATH).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn restart_daemon() -> Result<(), DaemonError> {
    stop_daemon().await?;
    start_daemon().await?;
    Ok(())
}

pub async fn start_daemon() -> Result<(), DaemonError> {
    let res = Command::new(SERVICE_PATH).output().await?;
    if res.status.success() {
        return Ok(());
    }
    let stdout = String::from_utf8_lossy(&res.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&res.stderr).into_owned();
    Err(DaemonError::Subprocess {
        process: res.status.code(),
        stdout,
        stderr,
    })
}

pub async fn stop_daemon() -> Result<bool, DaemonError> {
    let res = Command::new("killall")
        .arg("syncer-daemon")
        .output()
        .await?;
    Ok(res.status.success())
}

pub async fn reinstall_daemon() -> Result<(), DaemonError> {
    install_daemon().await?;
    restart_daemon().await?;
    Ok(())
}

pub async fn daemon_is_installed() -> Result<bool, DaemonError> {
    match fs::try_exists(SERVICE_INSTALL_PATH).await {
        Ok(b) => Ok(b),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

pub async fn daemon_is_running() -> Result<bool, DaemonError> {
    let task = || {
        let expected = Path::new(DAEMON_EXE_PATH).canonicalize()?;
        let procs = process::all_processes()?;
        let skip_not_found = procs.filter(|p| !matches!(p, Err(ProcError::NotFound(_))));
        for proc in skip_not_found {
            let proc = proc?;
            let exe_path = match proc.exe() {
                Ok(pt) => pt,

                // Not all processes have an /exe symlink; skip those without
                // one.
                Err(ProcError::NotFound(_)) => {
                    continue;
                }
                Err(ProcError::Io(e, _)) if e.kind() == io::ErrorKind::NotFound => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            };
            if exe_path == expected {
                return Ok(true);
            }
        }
        Ok(false)
    };
    tokio::task::spawn_blocking(task).await.unwrap()
}

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error(
        "Subprocess failed ({}). Stdout: {stdout}. Stderr: {stderr}",
        process.map(|n| n.to_string()).unwrap_or("<UNKNOWN>".to_owned())
    )]
    Subprocess {
        process: Option<i32>,
        stdout: String,
        stderr: String,
    },
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    ProcFs(#[from] ProcError),
}
