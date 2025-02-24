use std::io;
use std::path::PathBuf;

use futures::{future, stream, FutureExt, StreamExt, TryFutureExt, TryStream, TryStreamExt};
use tracing::{debug, error, trace, warn};

use crate::utils::async_walkdir;

use super::Config;

pub fn possible_saves(config: &Config) -> impl TryStream<Ok = PathBuf, Error = io::Error> + '_ {
    let skip_hidden = config.system.skip_hidden;
    let full_tree = stream::iter(save_roots(config))
        .map(io::Result::Ok)
        .map_ok(|root| async_walkdir(&root))
        .try_flatten();

    let no_hidden = full_tree.try_filter(move |pt| {
        trace!("HIDDEN CHECK: {pt:?}");
        let is_hidden = pt
            .file_stem()
            .map(|raw| raw.to_string_lossy().starts_with('.'))
            .unwrap_or(false);
        trace!("HIDDEN RES: {pt:?} => {is_hidden:?}");
        future::ready(!skip_hidden || !is_hidden)
    });

    let matching_paths = no_hidden.try_filter(move |path| {
        future::ready(
            config
                .system
                .saves
                .as_slice()
                .iter()
                .any(|fmt| fmt.matches_path(path)),
        )
    });
    matching_paths.try_filter(|path| {
        tokio::fs::metadata(path.to_path_buf())
            .map_ok(|meta| meta.is_file())
            .map(|res| res.unwrap_or(true))
    })
}

fn save_roots(config: &Config) -> impl Iterator<Item = PathBuf> + '_ {
    let all_fmts = config.system.saves.as_slice().iter();
    let possible = all_fmts.map(|s| s.prefix()).map(PathBuf::from);
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
