use std::io;
use std::sync::Arc;

use interprocess::local_socket::tokio::Stream;
use interprocess::local_socket::traits::tokio::Stream as _;
use interprocess::local_socket::{GenericFilePath, ToFsName};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use syncer_model::commands::DaemonCommand;
use syncer_model::platforms::Platform;

#[derive(Clone, Debug)]
pub struct DaemonSocket {
    inner: Arc<Mutex<Stream>>,
}

impl DaemonSocket {
    pub async fn new() -> io::Result<Self> {
        let inner = Stream::connect(
            Platform::get()
                .socket_path()
                .to_fs_name::<GenericFilePath>()?,
        )
        .await?;
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }
    pub async fn send(&self, cmd: &DaemonCommand) -> io::Result<()> {
        let payload = cmd.serialize();
        let mut lock = self.inner.lock().await;
        lock.write_all(payload.as_bytes()).await?;
        Ok(())
    }
}
