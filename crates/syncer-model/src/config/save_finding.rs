use std::path::PathBuf;
use std::{collections::HashMap, io};

use futures::future::ready;
use futures::{stream, FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use tracing::{debug, error, trace, warn};

use crate::config::Config;
use crate::path_format_strings::FormatString;
use crate::utils::async_walkdir;

impl Config {
    /// Finds all possible local save files based on the given [`Config`].
    ///
    /// Returns:
    /// * The path on the local filesystem to the save file
    /// * The format string within `config.system.saves` that the path matched
    /// * The variables pulled from the path string based on that format string
    pub fn possible_saves(
        &self,
    ) -> impl Stream<Item = Result<(PathBuf, &FormatString, HashMap<String, String>), io::Error>> + '_
    {
        let skip_hidden = self.system.skip_hidden;
        let full_tree = stream::iter(self.save_roots())
            .map(io::Result::Ok)
            .map_ok(|root| async_walkdir(&root))
            .try_flatten();

        let matches_allowdeny = full_tree
            .try_filter(move |pt| {
                let Some(allow) = self.system.allow.as_deref() else {
                    return ready(true);
                };
                let res = allow.iter().any(|prefix| pt.starts_with(prefix));
                ready(res)
            })
            .try_filter(move |pt| {
                let res = !self.system.deny.iter().any(|prefix| pt.starts_with(prefix));
                ready(res)
            });

        let no_hidden = matches_allowdeny.try_filter(move |pt| {
            let is_hidden = pt
                .file_stem()
                .map(|raw| raw.to_string_lossy().starts_with('.'))
                .unwrap_or(false);
            ready(!skip_hidden || !is_hidden)
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
            for saves in self.system.saves.as_slice().iter() {
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

            ready(Ok(fmt.map(|fmt| (path, fmt, variables))))
        });
        matching_paths.try_filter(|(path, _, _)| {
            tokio::fs::metadata(path.to_path_buf())
                .map_ok(|meta| meta.is_file())
                .map(|res| res.unwrap_or(true))
        })
    }

    /// Finds the list of static directories that could possibly contain saves
    /// we need to sync.
    ///
    /// In other words, takes each value in the `saves` list and takes the
    /// longest subpath we can before we hit a component containing a
    /// `$VARIABLE`.
    pub fn save_roots(&self) -> impl Iterator<Item = PathBuf> + '_ {
        let all_fmts = self.system.saves.as_slice().iter();
        let possible = all_fmts.map(|s| s.prefix()).map(PathBuf::from);
        possible
            .filter(|pt| {
                let Some(allowlist) = self.system.allow.as_deref() else {
                    return true;
                };
                allowlist.iter().any(|prefix| pt.starts_with(prefix))
            })
            .filter(|pt| !self.system.deny.iter().any(|prefix| pt.starts_with(prefix)))
            .filter(
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
}
