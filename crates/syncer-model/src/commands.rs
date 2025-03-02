use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum DaemonCommandBody {

    /// Perform a full sync of all saves. 
    DoSync,

    /// Sync only a single save.
    SyncSingle(PathBuf),
}
