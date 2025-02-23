use std::io;
use std::path::PathBuf;

use futures::{StreamExt, TryStream, TryStreamExt};
use tracing::{debug, error, warn};

use crate::utils::async_walkdir;

use super::save_formats::{format_root, path_matches};
use super::Config;

pub fn possible_saves(config: &Config) -> impl TryStream<Ok = PathBuf, Error = io::Error> + '_ {
    futures::stream::iter(save_roots(config))
        .map(io::Result::Ok)
        .map_ok(|root| async_walkdir(&root))
        .try_flatten()
        .try_filter(move |path| {
            let has_matching_format = config
                .system
                .saves
                .as_slice()
                .iter()
                .any(|fmt| path_matches(fmt, path));
            futures::future::ready(has_matching_format)
        })
}

fn save_roots(config: &Config) -> impl Iterator<Item = PathBuf> + '_ {
    let all_fmts = config.system.saves.as_slice().iter();
    let possible = all_fmts.map(|s| format_root(s)).map(PathBuf::from);
    possible.filter(
        |pt| match std::fs::symlink_metadata(pt).map(|meta| meta.is_dir()) {
            Ok(true) => true,
            Ok(false) => {
                warn!("Configured save path {} is not a path!?", pt.display());
                false
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                debug!(
                    "Configured save path {} was not found; skipping.",
                    pt.display()
                );
                false
            }
            Err(e) => {
                error!(
                    "Error looking for save directory {}: {:?}.",
                    pt.display(),
                    e
                );
                false
            }
        },
    )
}
