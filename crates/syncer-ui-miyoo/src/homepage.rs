//! The home tab of the UI, used for installing & uninstalling the daemon.
//!
//! Current UI & navigation:
//!
//! ```text
//!  -------------------------------
//! | Daemon installed checkbox |o| |
//!  -------------------------------
//!              ^
//!              |
//!              v
//!  -------------------------------
//! | Daemon running checkbox   |o| |
//!  -------------------------------
//!              ^
//!              |
//!              v
//!  -------------------------------
//! | Poll time             < 30m > |
//!  -------------------------------
//!              ^
//!              |
//!              v
//!  -------------------------------
//! | Enable fsnotify?        |o|   |
//!  -------------------------------
//!     ^                   ^
//!     |                   |
//!     v                   v
//!   -----------       -----------
//!  | Reinstall | <-> | Uninstall |
//!   -----------       -----------
//! ```

use std::time::Duration;

use buoyant::{
    layout::Layout,
    render::EmbeddedGraphicsView,
    view::{HStack, LayoutExtensions, RenderExtensions, Spacer, Text, VStack},
};
use embedded_graphics::{pixelcolor::Rgb888, prelude::RgbColor};
use embedded_vintage_fonts::FONT_24X32;
use futures::future;
use syncer_model::config::{Config, ParseableDuration};

use crate::daemon::{daemon_is_installed, install_daemon, reinstall_daemon, uninstall_daemon};
use crate::{ApplicationState, ViewState};
use crate::{
    components::{button, labeled_checkbox},
    daemon::{daemon_is_running, start_daemon, stop_daemon},
};

const POLL_TIME_OPTIONS: &[Duration] = &[
    Duration::MAX, // Disabled
    Duration::from_secs(60),
    Duration::from_secs(60 * 5),
    Duration::from_secs(60 * 10),
    Duration::from_secs(60 * 15),
    Duration::from_secs(60 * 30),
    Duration::from_secs(60 * 60),
    Duration::from_secs(60 * 60 * 2),
    Duration::from_secs(60 * 60 * 4),
    Duration::from_secs(60 * 60 * 8),
];

const fn cur_poll_idx(duration: Duration) -> usize {
    let mut retvl = 0;
    while POLL_TIME_OPTIONS[retvl].as_secs() < duration.as_secs() {
        retvl += 1;
    }
    if retvl >= POLL_TIME_OPTIONS.len() {
        retvl = POLL_TIME_OPTIONS.len();
    }
    retvl
}

pub struct HomepageState {
    daemon_installed: bool,
    daemon_running: bool,
    fs_notify_enabled: bool,
    pressed: bool,
    selection: HomePageSelection,
    app_state: ApplicationState,
    poll_interval: ParseableDuration,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
enum HomePageSelection {
    #[default]
    Nothing,
    DaemonInstalledBox,
    DaemonRunningBox,
    PollTimeSelection,
    FsnotifyBox,
    ReinstallDaemon,
    UninstallDaemon,
}

impl HomePageSelection {
    const fn up(&self) -> HomePageSelection {
        use HomePageSelection::*;
        match *self {
            UninstallDaemon | ReinstallDaemon => FsnotifyBox,
            FsnotifyBox => PollTimeSelection,
            PollTimeSelection => DaemonRunningBox,
            DaemonRunningBox => DaemonInstalledBox,
            DaemonInstalledBox | Nothing => Nothing,
        }
    }
    const fn down(&self) -> HomePageSelection {
        use HomePageSelection::*;
        match *self {
            Nothing => DaemonInstalledBox,
            DaemonInstalledBox => DaemonRunningBox,
            DaemonRunningBox => PollTimeSelection,
            PollTimeSelection => FsnotifyBox,
            FsnotifyBox => ReinstallDaemon,
            ReinstallDaemon | UninstallDaemon => Nothing,
        }
    }
    const fn left(&self) -> HomePageSelection {
        use HomePageSelection::*;
        match *self {
            ReinstallDaemon => UninstallDaemon,
            UninstallDaemon => ReinstallDaemon,
            other => other,
        }
    }
    const fn right(&self) -> HomePageSelection {
        self.left()
    }
}

impl HomepageState {
    pub async fn new(cfg: ApplicationState) -> Result<Self, anyhow::Error> {
        let mut retvl = Self {
            app_state: cfg,
            daemon_installed: false,
            daemon_running: false,
            fs_notify_enabled: false,
            pressed: false,
            selection: HomePageSelection::default(),
            poll_interval: ParseableDuration::new(Duration::ZERO),
        };
        retvl.reload().await?;
        Ok(retvl)
    }
    async fn reload(&mut self) -> Result<(), anyhow::Error> {
        self.daemon_installed = daemon_is_installed().await?;
        self.daemon_running = daemon_is_running().await?;
        let cfg = self.app_state.config().await;
        self.poll_interval = cfg.system.poll_interval;
        self.fs_notify_enabled = cfg.system.sync_on_file_change;
        Ok(())
    }
}

impl ViewState for HomepageState {
    async fn up(&mut self) -> Result<(), anyhow::Error> {
        self.selection = self.selection.up();
        Ok(())
    }
    async fn down(&mut self) -> Result<(), anyhow::Error> {
        self.selection = self.selection.down();
        Ok(())
    }
    async fn left(&mut self) -> Result<(), anyhow::Error> {
        if self.selection == HomePageSelection::PollTimeSelection {
            let cur_poll_idx = cur_poll_idx(*self.app_state.config().await.system.poll_interval);
            let prev_poll_idx = cur_poll_idx.saturating_sub(1);
            self.app_state
                .modify_and_save_cfg(move |cfg: &mut Config| {
                    cfg.system.poll_interval = POLL_TIME_OPTIONS[prev_poll_idx].into();
                    future::ready(())
                })
                .await?;
        } else {
            self.selection = self.selection.left();
        }
        Ok(())
    }
    async fn right(&mut self) -> Result<(), anyhow::Error> {
        if self.selection == HomePageSelection::PollTimeSelection {
            let cur_poll_idx = cur_poll_idx(*self.app_state.config().await.system.poll_interval);
            let next_poll_idx = (POLL_TIME_OPTIONS.len() - 1).min(cur_poll_idx + 1);
            self.app_state
                .modify_and_save_cfg(move |cfg: &mut Config| {
                    cfg.system.poll_interval = POLL_TIME_OPTIONS[next_poll_idx].into();
                    future::ready(())
                })
                .await?;
        } else {
            self.selection = self.selection.right();
        }
        Ok(())
    }
    async fn press(&mut self) -> Result<(), anyhow::Error> {
        self.pressed = true;
        Ok(())
    }
    async fn release(&mut self) -> Result<(), anyhow::Error> {
        use HomePageSelection::*;
        match self.selection {
            ReinstallDaemon => {
                reinstall_daemon().await?;
                self.reload().await?;
            }
            UninstallDaemon => {
                uninstall_daemon().await?;
                self.reload().await?;
            }
            DaemonInstalledBox if self.daemon_installed => {
                uninstall_daemon().await?;
                self.reload().await?;
            }
            DaemonInstalledBox => {
                install_daemon().await?;
                self.reload().await?;
            }
            DaemonRunningBox if self.daemon_running => {
                stop_daemon().await?;
                self.reload().await?;
            }
            DaemonRunningBox => {
                start_daemon().await?;
                self.reload().await?;
            }
            FsnotifyBox => {
                self.app_state
                    .modify_and_save_cfg(|cfg: &mut Config| {
                        cfg.system.sync_on_file_change = !self.fs_notify_enabled;
                        future::ready(())
                    })
                    .await?;
                self.reload().await?;
            }
            PollTimeSelection | Nothing => {}
        }
        self.pressed = false;
        Ok(())
    }
    fn build_view(&self) -> impl EmbeddedGraphicsView<Rgb888> + Layout + '_ {
        let installed_box = labeled_checkbox(
            "Daemon installed",
            self.selection == HomePageSelection::DaemonInstalledBox,
            self.daemon_installed,
        );
        let running_box = labeled_checkbox(
            "Daemon Running",
            self.selection == HomePageSelection::DaemonRunningBox,
            self.daemon_running,
        );
        let uninstall_btn = button(
            "Uninstall",
            self.selection == HomePageSelection::UninstallDaemon,
            self.selection == HomePageSelection::UninstallDaemon && self.pressed,
        );
        let reinstall_btn = button(
            "Reinstall",
            self.selection == HomePageSelection::ReinstallDaemon,
            self.selection == HomePageSelection::ReinstallDaemon && self.pressed,
        );

        let current_poll_time = self.poll_interval.to_string();
        let poll_time_cfg = labelled_scrollable_options(
            "Poll time",
            current_poll_time,
            self.selection == HomePageSelection::PollTimeSelection,
        );

        let fs_notify_box = labeled_checkbox(
            "Sync on change",
            self.selection == HomePageSelection::FsnotifyBox,
            self.fs_notify_enabled,
        );

        let btns = HStack::new((reinstall_btn, uninstall_btn));
        VStack::new((
            installed_box,
            running_box,
            poll_time_cfg,
            fs_notify_box,
            btns,
        ))
        .frame()
    }
}

fn labelled_scrollable_options<'a>(
    label: impl AsRef<str> + Clone + 'a,
    current_option: impl AsRef<str> + Clone + 'a,
    is_selected: bool,
) -> impl EmbeddedGraphicsView<Rgb888> + Layout + 'a {
    const LABEL_COLOR: Rgb888 = Rgb888::BLACK;
    const LABEL_SELECTED_COLOR: Rgb888 = Rgb888::BLUE;
    const ARROW_COLOR: Rgb888 = Rgb888::BLACK;
    const ARROW_SELECTED_COLOR: Rgb888 = Rgb888::BLUE;
    const OPTION_COLOR: Rgb888 = Rgb888::BLACK;
    const OPTION_SELECTED_COLOR: Rgb888 = Rgb888::BLUE;

    let label_color: Rgb888 = if is_selected {
        LABEL_SELECTED_COLOR
    } else {
        LABEL_COLOR
    };
    let label = Text::new(label, &FONT_24X32).foreground_color(label_color);

    let arrow_color: Rgb888 = if is_selected {
        ARROW_SELECTED_COLOR
    } else {
        ARROW_COLOR
    };
    let option_color: Rgb888 = if is_selected {
        OPTION_SELECTED_COLOR
    } else {
        OPTION_COLOR
    };

    let scrollable = HStack::new((
        Text::new("<", &FONT_24X32).foreground_color(arrow_color),
        Text::new(current_option, &FONT_24X32).foreground_color(option_color),
        Text::new(">", &FONT_24X32).foreground_color(arrow_color),
    ))
    .flex_frame();

    HStack::new((label, Spacer::default(), scrollable))
        .flex_frame()
        .with_infinite_max_width()
}
