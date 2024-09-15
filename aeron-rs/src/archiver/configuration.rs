use std::sync::LazyLock;

use crate::agrona::SemanticVersion;

/// Major version of the network protocol from client to archive. If these don't match then client and archive
/// are not compatible.
pub const PROTOCOL_MAJOR_VERSION: i32 = 1;

/// Minor version of the network protocol from client to archive. If these don't match then some features may
/// not be available.
pub const PROTOCOL_MINOR_VERSION: i32 = 10;

/// Patch version of the network protocol from client to archive. If these don't match then bug fixes may not
/// have been applied.
pub const PROTOCOL_PATCH_VERSION: i32 = 0;

pub static PROTOCOL_SEMANTIC_VERSION: LazyLock<SemanticVersion> = LazyLock::new(|| {
    SemanticVersion::compose(PROTOCOL_MAJOR_VERSION, PROTOCOL_MINOR_VERSION, PROTOCOL_PATCH_VERSION)
        .expect("SemanticVersion issue")
});

/// Timeout when waiting on a message to be sent or received.
pub static MESSAGE_TIMEOUT_DEFAULT_NS: i64 = 10_000_000_000;
