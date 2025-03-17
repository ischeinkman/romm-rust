use std::io;

use interprocess::local_socket::tokio::Stream;
use interprocess::local_socket::traits::tokio::Stream as _;
use interprocess::local_socket::{GenericFilePath, ToFsName};
use tokio::io::AsyncWriteExt;

use syncer_model::commands::DaemonCommand;
use syncer_model::platforms::Platform;

#[derive(Clone, Debug)]
pub struct DaemonSocket {
    _phantom: (),
}

impl DaemonSocket {
    pub async fn new() -> io::Result<Self> {
        Ok(Self { _phantom: () })
    }
    pub async fn send(&self, cmd: &DaemonCommand) -> io::Result<()> {
        let mut inner = Stream::connect(
            Platform::get()
                .socket_path()
                .to_fs_name::<GenericFilePath>()?,
        )
        .await?;
        let payload = cmd.serialize();
        inner.write_all(payload.as_bytes()).await?;
        Ok(())
    }
}
