use std::sync::LazyLock;

use crate::agrona::SemanticVersion;

use super::recording_signal_consumer::NoOpRecodingSignalConsumer;

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

/// Timeout in nanoseconds when waiting on a message to be sent or received.
pub const MESSAGE_TIMEOUT_PROP_NAME: &str = "aeron.archive.message.timeout";

/// Timeout when waiting on a message to be sent or received.
pub static MESSAGE_TIMEOUT_DEFAULT_NS: i64 = 10_000_000_000;

/// Channel for sending control messages to an archive.
pub const CONTROL_CHANNEL_PROP_NAME: &str = "aeron.archive.control.channel";

/// Stream id within a channel for sending control messages to an archive.
pub const CONTROL_STREAM_ID_PROP_NAME: &str = "aeron.archive.control.stream.id";

/// Stream id within a channel for sending control messages to an archive.
pub const CONTROL_STREAM_ID_DEFAULT: i32 = 10;

/// Channel for sending control messages to a driver local archive.
pub const LOCAL_CONTROL_CHANNEL_PROP_NAME: &str = "aeron.archive.local.control.channel";

/// Channel for sending control messages to a driver local archive. Default to IPC.
pub const LOCAL_CONTROL_CHANNEL_DEFAULT: &str = crate::channel_uri::IPC_CHANNEL;

/// Stream id within a channel for sending control messages to a driver local archive.
pub const LOCAL_CONTROL_STREAM_ID_PROP_NAME: &str = "aeron.archive.local.control.stream.id";

/// Stream id within a channel for sending control messages to a driver local archive.
pub const LOCAL_CONTROL_STREAM_ID_DEFAULT: i32 = CONTROL_STREAM_ID_DEFAULT;

/// Channel for receiving control response messages from an archive.
///
/// Channel's <em>endpoint</em> can be specified explicitly (i.e. by providing address and port pair) or
/// by using zero as a port number. Here is an example of valid response channels:
/// <ul>
///     <li><code>aeron:udp?endpoint=localhost:8020</code> - listen on port <code>8020</code> on localhost.</li>
///     <li><code>aeron:udp?endpoint=192.168.10.10:8020</code> - listen on port <code>8020</code> on
///     <code>192.168.10.10</code>.</li>
///     <li><code>aeron:udp?endpoint=localhost:0</code> - in this case the port is unspecified and the OS
///     will assign a free port from the
///     <a href="https://en.wikipedia.org/wiki/Ephemeral_port">ephemeral port range</a>.</li>
/// </ul>
pub const CONTROL_RESPONSE_CHANNEL_PROP_NAME: &str = "aeron.archive.control.response.channel";

/// Stream id within a channel for receiving control messages from an archive.
pub const CONTROL_RESPONSE_STREAM_ID_PROP_NAME: &str = "aeron.archive.control.response.stream.id";

/// Stream id within a channel for receiving control messages from an archive.
pub const CONTROL_RESPONSE_STREAM_ID_DEFAULT: i32 = 20;

/// Channel for receiving progress events of recordings from an archive.
pub const RECORDING_EVENTS_CHANNEL_PROP_NAME: &str = "aeron.archive.recording.events.channel";

/// Stream id within a channel for receiving progress of recordings from an archive.
pub const RECORDING_EVENTS_STREAM_ID_PROP_NAME: &str = "aeron.archive.recording.events.stream.id";

/// Stream id within a channel for receiving progress of recordings from an archive.
pub const RECORDING_EVENTS_STREAM_ID_DEFAULT: i32 = 30;

/// Is channel enabled for recording progress events of recordings from an archive.
pub const RECORDING_EVENTS_ENABLED_PROP_NAME: &str = "aeron.archive.recording.events.enabled";

/// Channel enabled for recording progress events of recordings from an archive which defaults to true.
pub const RECORDING_EVENTS_ENABLED_DEFAULT: bool = false;

/// Sparse term buffer indicator for control streams.
pub const CONTROL_TERM_BUFFER_SPARSE_PROP_NAME: &str = "aeron.archive.control.term.buffer.sparse";

/// Overrides driver's sparse term buffer indicator for control streams.
pub const CONTROL_TERM_BUFFER_SPARSE_DEFAULT: bool = true;

/// Term length for control streams.
pub const CONTROL_TERM_BUFFER_LENGTH_PROP_NAME: &str = "aeron.archive.control.term.buffer.length";

/// Low term length for control channel reflects expected low bandwidth usage.
pub const CONTROL_TERM_BUFFER_LENGTH_DEFAULT: i32 = 64 * 1024;

/// MTU length for control streams.
pub const CONTROL_MTU_LENGTH_PROP_NAME: &str = "aeron.archive.control.mtu.length";

///  MTU to reflect default for the control streams.
pub const CONTROL_MTU_LENGTH_DEFAULT: i32 = 1408;

pub static NO_OP_RECORDING_SIGNAL_CONSUMER: LazyLock<NoOpRecodingSignalConsumer> =
    LazyLock::new(|| NoOpRecodingSignalConsumer {});
