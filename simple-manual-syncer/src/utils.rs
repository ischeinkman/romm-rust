use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use futures::pin_mut;
use futures::{Stream, TryStream, TryStreamExt};
use thiserror::Error;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

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

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EitherError<A, B> {
    #[error(transparent)]
    A(#[from] A),
    #[error(transparent)]
    B(B),
}

pub async fn download<S, B, E>(data: S, dst: &Path) -> Result<(), EitherError<io::Error, E>>
where
    S: Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
{
    let tmp_fname = dst.with_extension(timestamp_now().to_rfc3339());
    let mut fh = File::create_new(&tmp_fname).await?;

    pin_mut!(data);
    while let Some(chunk) = data.try_next().await.map_err(EitherError::B)? {
        fh.write_all(chunk.as_ref()).await?;
    }
    fh.flush().await?;
    drop(fh);
    fs::rename(tmp_fname, dst).await?;
    Ok(())
}

pub fn timestamp_now() -> DateTime<Utc> {
    let dt = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as _;
    DateTime::from_timestamp_nanos(dt)
}
