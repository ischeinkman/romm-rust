//! The tab used for configuring what, specifically, gets synced.
//!
//! Current UI & navigation is a paged scroll list of checkmarks with
//! `[default]` at the top controlling what the default is for new saves
//! following by each of the possible save files, ordered alphabetically.

use std::{
    borrow::Cow,
    io,
    path::{Path, PathBuf},
};

use buoyant::{layout::Layout, render::EmbeddedGraphicsView};
use embedded_graphics::pixelcolor::Rgb888;
use futures::{StreamExt, TryFutureExt, TryStreamExt, future};
use syncer_model::config::Config;
use tracing::error;

use crate::components::labeled_checkbox;
use crate::utils::ForEachDyn;
use crate::{ApplicationState, ViewState};

pub struct SavelistState {
    saves: Vec<(String, bool)>,
    selected: usize,
    pub cfg: ApplicationState,
}

impl SavelistState {
    pub async fn new(cfg: ApplicationState) -> Self {
        let mut retvl = Self {
            saves: Vec::new(),
            selected: 0,
            cfg,
        };
        retvl.reload().await;
        retvl
    }
    pub async fn reload(&mut self) {
        self.saves = saves_from_cfg(&*self.cfg.config().await).await;
        self.selected = 0;
    }
}

async fn saves_from_cfg(cfg: &Config) -> Vec<(String, bool)> {
    let mut saves = cfg
        .possible_saves()
        .filter_map(|res| match res {
            Ok((path, _, _)) => futures::future::ready(Some(path)),
            Err(e) => {
                error!("Error getting save: {e:?}");
                futures::future::ready(None)
            }
        })
        .map(|path| {
            let flag = is_enabled(cfg, &path);
            (path, flag)
        })
        .map(|(path, enabled)| (path.to_string_lossy().into_owned(), enabled))
        .collect::<Vec<_>>()
        .await;
    saves.sort();
    saves.insert(
        0,
        (
            "[default]".to_owned(),
            is_enabled(cfg, Path::new("[default]")),
        ),
    );
    saves
}

impl ViewState for SavelistState {
    fn up(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        self.selected = self.selected.saturating_sub(1);
        future::ready(Ok(()))
    }
    fn down(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        self.selected = (self.saves.len() - 1).min(self.selected + 1);
        future::ready(Ok(()))
    }
    async fn press(&mut self) -> Result<(), anyhow::Error> {
        if self.selected == 0 {
            self.cfg.modify_and_save_cfg(toggle_default).await??;
        } else {
            let &(ref save, prev_enabled) = &self.saves[self.selected];
            let save = PathBuf::from(save);
            self.cfg
                .modify_and_save_cfg(move |cfg: &mut Config| {
                    toggle_single(cfg, save, prev_enabled);
                    future::ready(())
                })
                .await?;
        }
        self.reload().await;
        Ok(())
    }
    fn build_view(&self) -> impl EmbeddedGraphicsView<Rgb888> + Layout + '_ {
        const PER_SCREEN: usize = 10;
        const SPACING: u16 = 4;
        const MAX_CHARACTERS_PER_BUTTON: usize = 24;

        let skip = self.selected.saturating_sub(PER_SCREEN - 1);
        let boxes = self
            .saves
            .iter()
            .enumerate()
            .map(|(idx, (label, is_on))| {
                let num_chars = label.chars().count();
                let label_trunc = if num_chars <= MAX_CHARACTERS_PER_BUTTON {
                    Cow::Borrowed(label.as_str())
                } else {
                    let to_skip = num_chars - MAX_CHARACTERS_PER_BUTTON;
                    let mapped = label
                        .chars()
                        .skip(to_skip)
                        .fold(String::new(), |mut acc, cur| {
                            acc.push(cur);
                            acc
                        });
                    Cow::Owned(mapped)
                };
                labeled_checkbox(label_trunc, self.selected == idx, *is_on)
            })
            .skip(skip)
            .take(PER_SCREEN)
            .collect::<Vec<_>>();
        ForEachDyn::new(boxes).with_spacing(SPACING)
    }
}

fn is_enabled(cfg: &Config, path: &Path) -> bool {
    if path.to_str() == Some("[default]") {
        return cfg.system.allow.is_none();
    }
    if let Some(allow) = cfg.system.allow.as_ref() {
        if !allow.iter().any(|needle| path.ends_with(needle)) {
            return false;
        }
    }
    if cfg.system.deny.iter().any(|needle| path.ends_with(needle)) {
        return false;
    }
    cfg.system
        .saves
        .as_slice()
        .iter()
        .any(|needle| needle.matches_path(path))
}

// TODO: These are only lifted out of the `pressed` function because of
// difficulties with Rust's lifetime construction for async closures. These
// should eventually be moved back once we figure out how to make the borrow
// checker happy.

async fn toggle_default(cfg: &mut Config) -> Result<(), io::Error> {
    let (new_allow, new_deny) = if cfg.system.allow.is_none() {
        cfg.possible_saves()
            .map_ok(|data| data.0)
            .try_filter(|pt| future::ready(is_enabled(cfg, pt)))
            .try_collect::<Vec<_>>()
            .map_ok(|res| (Some(res), Vec::new()))
            .await?
    } else {
        cfg.possible_saves()
            .map_ok(|data| data.0)
            .try_filter(|pt| future::ready(!is_enabled(cfg, pt)))
            .try_collect::<Vec<_>>()
            .map_ok(|res| (None, res))
            .await?
    };
    cfg.system.allow = new_allow;
    cfg.system.deny = new_deny;
    Result::<_, io::Error>::Ok(())
}

fn toggle_single(cfg: &mut Config, save: PathBuf, prev_enabled: bool) {
    if prev_enabled {
        if let Some(allow) = cfg.system.allow.as_mut() {
            if let Some(prev_idx) = allow.iter().position(|pt| pt.ends_with(&save)) {
                allow.remove(prev_idx);
            }
        }
        if !cfg.system.deny.iter().any(|pt| pt.ends_with(&save)) {
            cfg.system.deny.push(save);
        }
    } else {
        if let Some(allow) = cfg.system.allow.as_mut() {
            if !allow.iter().any(|pt| pt.ends_with(&save)) {
                allow.push(save.clone());
            }
        }
        if let Some(prev_idx) = cfg.system.deny.iter().position(|pt| pt.ends_with(&save)) {
            cfg.system.deny.remove(prev_idx);
        }
    }
}
