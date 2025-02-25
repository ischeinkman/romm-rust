use std::path::PathBuf;
use std::{collections::HashMap, io};

use futures::{future, stream, FutureExt, StreamExt, TryFutureExt, TryStream, TryStreamExt};
use tracing::{debug, error, trace, warn};

use crate::path_format_strings::FormatString;
use crate::utils::async_walkdir;

use super::Config;

pub fn possible_saves(
    config: &Config,
) -> impl TryStream<Ok = (PathBuf, &FormatString, HashMap<String, String>), Error = io::Error> + '_
{
    let skip_hidden = config.system.skip_hidden;
    let full_tree = stream::iter(save_roots(config))
        .map(io::Result::Ok)
        .map_ok(|root| async_walkdir(&root))
        .try_flatten();

    let no_hidden = full_tree.try_filter(move |pt| {
        let is_hidden = pt
            .file_stem()
            .map(|raw| raw.to_string_lossy().starts_with('.'))
            .unwrap_or(false);
        future::ready(!skip_hidden || !is_hidden)
    });

    let matching_paths = no_hidden.try_filter_map(move |path| {
        let span = tracing::info_span!(
            "possible_path_matches",
            path = tracing::field::display(&path.display())
        );
        let _guard = span.enter();
        trace!("Testing path: {path:?}");
        let mut variables = HashMap::new();
        let mut fmt = None;
        for saves in config.system.saves.as_slice().iter() {
            trace!("Trying fmt: {saves:?}");
            let Ok(cur) = saves.resolve(&path) else {
                continue;
            };
            if cur.len() > variables.len() {
                fmt = Some(saves);
                variables = cur;
            }
        }
        trace!("Result: {fmt:?}");

        future::ready(Ok(fmt.map(|fmt| (path, fmt, variables))))
    });
    matching_paths.try_filter(|(path, _, _)| {
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
