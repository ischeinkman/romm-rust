use std::sync::Arc;

use interprocess::local_socket::tokio::Stream;
use interprocess::local_socket::traits::tokio::Listener as _;
use interprocess::local_socket::{GenericFilePath, ListenerOptions, ToFsName};
use tokio::io::AsyncReadExt;
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use syncer_model::commands::DaemonCommand;
use syncer_model::platforms::Platform;

use crate::DaemonState;

pub fn spawn_command_listen_thread(
    state: Arc<DaemonState>,
) -> Result<JoinHandle<()>, anyhow::Error> {
    let path = Platform::get()
        .socket_path()
        .to_fs_name::<GenericFilePath>()?;
    debug!("Opening command socket at {path:?}");
    let listener = ListenerOptions::new()
        .name(path)
        .reclaim_name(false)
        .create_tokio()?;
    debug!("Command socket spawned.");
    let handle = tokio::task::spawn(async move {
        loop {
            let next_stream = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    error!("Error waiting for connection: {e:?}");
                    continue;
                }
            };
            let state = Arc::clone(&state);
            let _runner = tokio::task::spawn(handle_stream(state, next_stream));
        }
    });
    debug!("Command thread spawned.");
    Ok(handle)
}

async fn handle_stream(state: Arc<DaemonState>, mut stream: Stream) {
    debug!("Received new connection on daemon command socket.");
    let mut buffer = Vec::new();
    loop {
        if let Err(e) = stream.read_buf(&mut buffer).await {
            error!("Error pulling data from stream: {e:?}");
        }
        let mut des =
            serde_json::Deserializer::from_slice(buffer.as_slice()).into_iter::<DaemonCommand>();
        let new_start = loop {
            match des.next() {
                Some(Ok(evt)) => {
                    trace!("Parsed command from socket: {evt:?}");
                    state.run_command(&evt);
                }
                Some(Err(e)) if e.is_eof() => {
                    break des.byte_offset();
                }
                Some(Err(e)) => {
                    error!("Error parsing command: {e:?}");
                    break des.byte_offset();
                }
                None => {
                    break buffer.len();
                }
            }
        };
        let new_buffer = buffer.split_off(new_start);
        buffer = new_buffer;
    }
}
