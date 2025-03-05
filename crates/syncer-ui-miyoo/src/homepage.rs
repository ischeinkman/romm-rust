use buoyant::{
    layout::Layout,
    render::EmbeddedGraphicsView,
    view::{HStack, LayoutExtensions, VStack},
};
use embedded_graphics::pixelcolor::Rgb888;

use crate::{
    ViewState, button, checkbox,
    daemon::{daemon_is_installed, install_daemon, reinstall_daemon, uninstall_daemon},
};

pub struct HomepageState {
    daemon_installed: bool,
    pressed: bool,
    selection: HomePageSelection,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
enum HomePageSelection {
    #[default]
    Nothing,
    DaemonInstalledBox,
    ReinstallDaemon,
    UninstallDaemon,
}

impl HomepageState {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let mut retvl = Self {
            daemon_installed: false,
            pressed: false,
            selection: HomePageSelection::default(),
        };
        retvl.reload().await?;
        Ok(retvl)
    }
    async fn reload(&mut self) -> Result<(), anyhow::Error> {
        self.daemon_installed = daemon_is_installed().await?;
        Ok(())
    }
}

impl ViewState for HomepageState {
    async fn up(&mut self) -> Result<(), anyhow::Error> {
        use HomePageSelection::*;
        let next_selection = match self.selection {
            ReinstallDaemon | UninstallDaemon => DaemonInstalledBox,
            Nothing | DaemonInstalledBox => Nothing,
        };
        self.selection = next_selection;
        Ok(())
    }
    async fn down(&mut self) -> Result<(), anyhow::Error> {
        use HomePageSelection::*;
        let next_selection = match self.selection {
            Nothing => DaemonInstalledBox,
            DaemonInstalledBox => ReinstallDaemon,
            other => other,
        };
        self.selection = next_selection;
        Ok(())
    }
    async fn left(&mut self) -> Result<(), anyhow::Error> {
        use HomePageSelection::*;
        let next_selection = match self.selection {
            UninstallDaemon => ReinstallDaemon,
            other => other,
        };
        self.selection = next_selection;
        Ok(())
    }
    async fn right(&mut self) -> Result<(), anyhow::Error> {
        use HomePageSelection::*;
        let next_selection = match self.selection {
            ReinstallDaemon => UninstallDaemon,
            other => other,
        };
        self.selection = next_selection;
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
            Nothing => {}
        }
        Ok(())
    }
    fn build_view(&self) -> impl EmbeddedGraphicsView<Rgb888> + Layout + Clone + '_ {
        let installed_box = checkbox(
            self.selection == HomePageSelection::DaemonInstalledBox,
            self.daemon_installed,
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

        let btns = HStack::new((reinstall_btn, uninstall_btn));
        VStack::new((installed_box, btns)).frame()
    }
}
