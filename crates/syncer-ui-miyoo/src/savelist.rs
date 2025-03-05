use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use buoyant::{layout::Layout, render::EmbeddedGraphicsView};
use embedded_graphics::pixelcolor::Rgb888;
use futures::{StreamExt, TryFutureExt, TryStreamExt};
use syncer_model::config::Config;

use crate::{ViewState, labeled_checkbox, utils::ForEachDyn};

pub struct SavelistState<'a> {
    saves: Vec<String>,
    selected: usize,
    cfg: &'a mut Config,
}

impl<'a> SavelistState<'a> {
    pub async fn new(cfg: &'a mut Config) -> Self {
        let mut retvl = Self {
            saves: Vec::new(),
            selected: 0,
            cfg,
        };
        retvl.reload().await;
        retvl
    }
    pub async fn reload(&mut self) {
        self.saves = saves_from_cfg(self.cfg).await;
        self.selected = 0;
    }
}

async fn saves_from_cfg(cfg: &Config) -> Vec<String> {
    let mut saves = cfg
        .possible_saves()
        .filter_map(|res| match res {
            Ok((path, _, _)) => futures::future::ready(Some(path)),
            Err(_e) => {
                //TODO: Log this
                futures::future::ready(None)
            }
        })
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .await;
    saves.sort();
    saves.insert(0, "[default]".to_owned());
    saves
}

impl ViewState for SavelistState<'_> {
    fn up(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        self.selected = self.selected.saturating_sub(1);
        futures::future::ready(Ok(()))
    }
    fn down(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        self.selected = self.saves.len().min(self.selected + 1);
        futures::future::ready(Ok(()))
    }
    async fn press(&mut self) -> Result<(), anyhow::Error> {
        if self.selected == 0 {
            let (new_allow, new_deny) = if self.cfg.system.allow.is_none() {
                self.cfg
                    .possible_saves()
                    .map_ok(|data| data.0)
                    .try_filter(|pt| futures::future::ready(is_enabled(self.cfg, pt)))
                    .try_collect::<Vec<_>>()
                    .map_ok(|res| (Some(res), Vec::new()))
                    .await?
            } else {
                self.cfg
                    .possible_saves()
                    .map_ok(|data| data.0)
                    .try_filter(|pt| futures::future::ready(!is_enabled(self.cfg, pt)))
                    .try_collect::<Vec<_>>()
                    .map_ok(|res| (None, res))
                    .await?
            };
            self.cfg.system.allow = new_allow;
            self.cfg.system.deny = new_deny;
        } else {
            let prev_enabled = is_enabled(self.cfg, Path::new(&self.saves[self.selected]));

            if prev_enabled {
                if let Some(allow) = self.cfg.system.allow.as_mut() {
                    if let Some(prev_idx) = allow
                        .iter()
                        .position(|pt| pt.ends_with(&self.saves[self.selected]))
                    {
                        allow.remove(prev_idx);
                    }
                }
                if !self
                    .cfg
                    .system
                    .deny
                    .iter()
                    .any(|pt| pt.ends_with(&self.saves[self.selected]))
                {
                    self.cfg
                        .system
                        .deny
                        .push(PathBuf::from(&self.saves[self.selected]));
                }
            } else {
                if let Some(allow) = self.cfg.system.allow.as_mut() {
                    if !allow
                        .iter()
                        .any(|pt| pt.ends_with(&self.saves[self.selected]))
                    {
                        allow.push(PathBuf::from(&self.saves[self.selected]));
                    }
                }
                if let Some(prev_idx) = self
                    .cfg
                    .system
                    .deny
                    .iter()
                    .position(|pt| pt.ends_with(&self.saves[self.selected]))
                {
                    self.cfg.system.deny.remove(prev_idx);
                }
            }
        }
        //TODO: signal the daemon via the domain socket
        self.cfg.save_current_platform().await?;
        Ok(())
    }
    fn build_view(&self) -> impl EmbeddedGraphicsView<Rgb888> + Layout + Clone + '_ {
        const PER_SCREEN: usize = 7;
        const SPACING: u16 = 4;
        const MAX_CHARACTERS_PER_BUTTON: usize = 24;

        let skip = self.selected.saturating_sub(PER_SCREEN - 1);
        let boxes = self
            .saves
            .iter()
            .enumerate()
            .map(|(idx, label)| {
                let is_on = is_enabled(self.cfg, Path::new(&label));

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
                labeled_checkbox(label_trunc, self.selected == idx, is_on)
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
