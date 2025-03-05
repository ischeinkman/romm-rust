use std::io;
use std::path::{Path, PathBuf};

use futures::TryStream;
use tokio::fs;

pub fn async_walkdir(root: &Path) -> impl TryStream<Ok = PathBuf, Error = io::Error> {
    futures::stream::unfold(vec![Ok(root.to_path_buf())], |mut queue| async move {
        let nxt = match queue.pop()? {
            Ok(pt) => pt,
            Err(e) => {
                return Some((Err(e), queue));
            }
        };
        match fs::symlink_metadata(&nxt).await.map(|meta| meta.is_dir()) {
            Ok(false) => {
                return Some((Ok(nxt), queue));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Some((Ok(nxt), queue));
            }
            Ok(true) => {}
            Err(e) => {
                queue.push(Err(e));
                return Some((Ok(nxt), queue));
            }
        }
        let mut rdr = match tokio::fs::read_dir(&nxt).await {
            Ok(rdr) => rdr,
            Err(e) => {
                queue.push(Err(e));
                return Some((Ok(nxt), queue));
            }
        };
        loop {
            match rdr.next_entry().await {
                Ok(None) => {
                    break;
                }
                Ok(Some(ent)) => {
                    queue.push(Ok(ent.path()));
                }
                Err(e) => {
                    queue.push(Err(e));
                }
            }
        }
        Some((Ok(nxt), queue))
    })
}
