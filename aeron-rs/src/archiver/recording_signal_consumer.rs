use std::sync::{Arc, LazyLock, Mutex};

use aeron_archiver_codecs::recording_signal::RecordingSignal;

use crate::{ControlSessionId, CorrelationId};

use super::RecordingId;

pub trait RecordingSignalHandler {
    fn on_signal(
        &self,
        control_session_id: ControlSessionId,
        correlation_id: CorrelationId,
        recording_id: RecordingId,
        subscription_id: i64,
        signal: RecordingSignal,
    );
}

pub struct NoOpRecodingSignalConsumer {}
impl RecordingSignalHandler for NoOpRecodingSignalConsumer {
    fn on_signal(
        &self,
        _control_session_id: ControlSessionId,
        _correlation_id: CorrelationId,
        _recording_id: RecordingId,
        _subscription_id: i64,
        _signal: RecordingSignal,
    ) {
        // no op
    }
}
