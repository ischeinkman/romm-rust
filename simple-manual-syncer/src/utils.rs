use std::path::{Path, PathBuf};
use std::io; 

use futures::TryStream;

pub fn async_walkdir(root: &Path) -> impl TryStream<Ok = PathBuf, Error = io::Error> {
    futures::stream::unfold(vec![Ok(root.to_path_buf())], |mut queue| async move {
        let nxt = match queue.pop()? {
            Ok(pt) => pt,
            Err(e) => {
                return Some((Err(e), queue));
            }
        };
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