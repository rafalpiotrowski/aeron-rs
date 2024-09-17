use std::slice;
use std::sync::{Arc, Mutex};

use aeron_archiver_codecs::auth_connect_request_codec::AuthConnectRequestEncoder;
use aeron_archiver_codecs::boolean_type::BooleanType;
use aeron_archiver_codecs::bounded_replay_request_codec::BoundedReplayRequestEncoder;
use aeron_archiver_codecs::challenge_response_codec::ChallengeResponseEncoder;
use aeron_archiver_codecs::close_session_request_codec::CloseSessionRequestEncoder;
use aeron_archiver_codecs::extend_recording_request_2_codec::ExtendRecordingRequest2Encoder;
use aeron_archiver_codecs::extend_recording_request_codec::ExtendRecordingRequestEncoder;
use aeron_archiver_codecs::find_last_matching_recording_request_codec::FindLastMatchingRecordingRequestEncoder;
use aeron_archiver_codecs::keep_alive_request_codec::KeepAliveRequestEncoder;
use aeron_archiver_codecs::list_recording_request_codec::ListRecordingRequestEncoder;
use aeron_archiver_codecs::list_recording_subscriptions_request_codec::ListRecordingSubscriptionsRequestEncoder;
use aeron_archiver_codecs::list_recordings_for_uri_request_codec::ListRecordingsForUriRequestEncoder;
use aeron_archiver_codecs::list_recordings_request_codec::ListRecordingsRequestEncoder;
use aeron_archiver_codecs::purge_recording_request_codec::PurgeRecordingRequestEncoder;
use aeron_archiver_codecs::recording_position_request_codec::RecordingPositionRequestEncoder;
use aeron_archiver_codecs::replay_request_codec::ReplayRequestEncoder;
use aeron_archiver_codecs::replicate_request_2_codec::ReplicateRequest2Encoder;
use aeron_archiver_codecs::source_location::SourceLocation;
use aeron_archiver_codecs::start_position_request_codec::StartPositionRequestEncoder;
use aeron_archiver_codecs::start_recording_request_2_codec::StartRecordingRequest2Encoder;
use aeron_archiver_codecs::start_recording_request_codec::StartRecordingRequestEncoder;
use aeron_archiver_codecs::stop_all_replays_request_codec::StopAllReplaysRequestEncoder;
use aeron_archiver_codecs::stop_position_request_codec::StopPositionRequestEncoder;
use aeron_archiver_codecs::stop_recording_by_identity_request_codec::StopRecordingByIdentityRequestEncoder;
use aeron_archiver_codecs::stop_recording_subscription_request_codec::StopRecordingSubscriptionRequestEncoder;
use aeron_archiver_codecs::stop_replay_request_codec::StopReplayRequestEncoder;
use aeron_archiver_codecs::stop_replication_request_codec::StopReplicationRequestEncoder;
use aeron_archiver_codecs::truncate_recording_request_codec::TruncateRecordingRequestEncoder;
use aeron_archiver_codecs::{message_header_codec, WriteBuf};

use crate::archiver::configuration::{MESSAGE_TIMEOUT_DEFAULT_NS, PROTOCOL_SEMANTIC_VERSION};
use crate::archiver::RecordingId;
use crate::client_conductor::ClientConductor;
use crate::concurrent::agent_invoker::AgentInvoker;
use crate::concurrent::atomic_buffer::{AlignedBuffer, AtomicBuffer};
use crate::concurrent::strategies::{Strategy, YieldingIdleStrategy};
use crate::publication::Publication;
use crate::security::{CredentialSupplier, NoCredentialsSupplier};
use crate::utils::errors::AeronError;
use crate::utils::time::{NanoClock, SystemNanoClock};
use crate::utils::types::Index;
use crate::{context, ChannelUriStr, ControlSessionId, CorrelationId, SessionId, StreamId, SubscriptionId};

use super::ReplayParams;

pub const DEFAULT_RETRY_ATTEMPTS: i32 = 3;

type Pub = Arc<Mutex<Publication>>;

#[allow(dead_code)]
pub struct ArchiveProxy {
    retry_attempts: i32,
    publication: Pub,
    buffer: AlignedBuffer,
    retry_idle_strategy: Box<dyn Strategy>,
    connect_timeout_ns: i64,
    credential_supplier: Box<dyn CredentialSupplier>,
    nano_clock: Box<dyn NanoClock>,
}

impl ArchiveProxy {
    pub fn new(publication: Pub) -> Self {
        let retry_idle_strategy = Box::new(YieldingIdleStrategy {});
        Self::new_with_params(
            AlignedBuffer::with_capacity(1024),
            publication,
            retry_idle_strategy,
            DEFAULT_RETRY_ATTEMPTS,
            MESSAGE_TIMEOUT_DEFAULT_NS,
            Box::new(NoCredentialsSupplier {}),
            Box::new(SystemNanoClock {}),
        )
    }

    pub fn new_with_params(
        buffer: AlignedBuffer,
        publication: Pub,
        retry_idle_strategy: Box<dyn Strategy>,
        retry_attempts: i32,
        connect_timeout_ns: i64,
        credential_supplier: Box<dyn CredentialSupplier>,
        nano_clock: Box<dyn NanoClock>,
    ) -> Self {
        Self {
            retry_attempts,
            publication,
            buffer,
            retry_idle_strategy,
            connect_timeout_ns,
            credential_supplier,
            nano_clock,
        }
    }

    pub fn retry_attempts(&self) -> i32 {
        self.retry_attempts
    }

    pub fn pubication(&self) -> Pub {
        Arc::clone(&self.publication)
    }

    fn get_write_buf(&self) -> WriteBuf<'_> {
        let slice = unsafe { slice::from_raw_parts_mut(self.buffer.ptr.offset(0), self.buffer.len as usize) };
        WriteBuf::new(slice)
    }

    /// Connect to an archive on its control interface providing the response stream details.
    pub fn connect(
        &mut self,
        response_channel: &str,
        response_stream_id: i32,
        correlation_id: i64,
        agen_invoker: Option<&AgentInvoker<ClientConductor>>,
    ) -> Result<bool, AeronError> {
        let credentials = self.credential_supplier.encoded_credentials();
        let mut msg = AuthConnectRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.correlation_id(correlation_id);
        msg.response_stream_id(response_stream_id);
        msg.version(PROTOCOL_SEMANTIC_VERSION.version());
        msg.response_channel(response_channel.as_bytes());
        msg.encoded_credentials(credentials);
        let len = msg.encoded_length();
        // write header
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer_with_timeout(length, agen_invoker)
    }

    /// Try and connect to an archive on its control interface providing the response stream details. Only one attempt will
    /// be made to offer the request.
    pub fn try_connect(
        &mut self,
        response_channel: ChannelUriStr,
        response_stream_id: StreamId,
        correlation_id: CorrelationId,
    ) -> Result<bool, AeronError> {
        let credentials = self.credential_supplier.encoded_credentials();
        let mut msg = AuthConnectRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.correlation_id(*correlation_id);
        msg.response_stream_id(*response_stream_id);
        msg.version(PROTOCOL_SEMANTIC_VERSION.version());
        msg.response_channel(response_channel.as_bytes());
        msg.encoded_credentials(credentials);
        let len = msg.encoded_length();
        // write header
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.publication
            .lock()
            .unwrap()
            .offer_part(AtomicBuffer::from_aligned(&self.buffer), 0, length as Index)
            .map(|r| r > 0)
    }

    /// Keep this archive session alive by notifying the archive.
    pub fn keep_alive(
        &mut self,
        control_session_id: ControlSessionId,
        correlation_id: CorrelationId,
    ) -> Result<bool, AeronError> {
        let mut msg = KeepAliveRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Close this control session with the archive.
    pub fn close_session(&mut self, control_session_id: ControlSessionId) -> Result<bool, AeronError> {
        let mut msg = CloseSessionRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Try and send a ChallengeResponse to an archive on its control interface providing the credentials. Only one
    /// attempt will be made to offer the request.
    pub fn try_challange_response(
        &mut self,
        encoded_credentials: &[u8],
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = ChallengeResponseEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.encoded_credentials(encoded_credentials);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.publication
            .lock()
            .unwrap()
            .offer_part(AtomicBuffer::from_aligned(&self.buffer), 0, length as Index)
            .map(|r| r > 0)
    }

    /// Start recording streams for a given channel and stream id pairing.
    pub fn start_recording(
        &mut self,
        channel: ChannelUriStr,
        stream_id: StreamId,
        source_location: SourceLocation,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
        auto_stop: Option<bool>,
    ) -> Result<bool, AeronError> {
        if let Some(auto_stop) = auto_stop {
            let mut msg =
                StartRecordingRequest2Encoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
            msg.control_session_id(*control_session_id);
            msg.correlation_id(*correlation_id);
            msg.channel(channel.as_bytes());
            msg.stream_id(*stream_id);
            msg.source_location(source_location);
            let auto_stop = match auto_stop {
                true => BooleanType::TRUE,
                false => BooleanType::FALSE,
            };
            msg.auto_stop(auto_stop);
            let len = msg.encoded_length();
            let _ = msg.header(0);
            let length = message_header_codec::ENCODED_LENGTH + len;
            self.offer(length)
        } else {
            let mut msg =
                StartRecordingRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
            msg.control_session_id(*control_session_id);
            msg.correlation_id(*correlation_id);
            msg.channel(channel.as_bytes());
            msg.stream_id(*stream_id);
            msg.source_location(source_location);
            let len = msg.encoded_length();
            let _ = msg.header(0);
            let length = message_header_codec::ENCODED_LENGTH + len;
            self.offer(length)
        }
    }

    /// Stop a recording by the [crate::subscription::Subscription::registration_id] it was registered with.
    pub fn stop_recording_subscription(
        &mut self,
        subscription_id: SubscriptionId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg =
            StopRecordingSubscriptionRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.subscription_id(*subscription_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Stop an active recording by the recording id. This is not the [crate::subscription::Subscription::registration_id]
    /// but the id in the archive.
    pub fn stop_recording_by_id(
        &mut self,
        recording_id: RecordingId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg =
            StopRecordingByIdentityRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.recording_id(*recording_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    pub fn reply(
        &mut self,
        recording_id: RecordingId,
        reply_channel: ChannelUriStr,
        reply_stream_id: StreamId,
        reply_params: ReplayParams,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        if reply_params.is_bounded() {
            self.bounded_reply(
                recording_id,
                reply_channel,
                reply_stream_id,
                reply_params,
                correlation_id,
                control_session_id,
            )
        } else {
            self.unbounded_reply(
                recording_id,
                reply_channel,
                reply_stream_id,
                reply_params,
                correlation_id,
                control_session_id,
            )
        }
    }

    /// called from reply() only
    fn bounded_reply(
        &mut self,
        recording_id: RecordingId,
        reply_channel: ChannelUriStr,
        reply_stream_id: StreamId,
        reply_params: ReplayParams,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = BoundedReplayRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.recording_id(*recording_id);
        msg.replay_channel(reply_channel.as_bytes());
        msg.replay_stream_id(*reply_stream_id);
        msg.file_io_max_length(reply_params.get_file_io_max_length().unwrap_or(context::NULL_VALUE));
        msg.length(reply_params.get_length().unwrap_or(context::NULL_VALUE as i64));
        msg.position(reply_params.get_position().unwrap_or(context::NULL_VALUE as i64));
        msg.limit_counter_id(reply_params.get_bounded_limit_counter_id().unwrap_or(context::NULL_VALUE));
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// called from reply() only
    fn unbounded_reply(
        &mut self,
        recording_id: RecordingId,
        reply_channel: ChannelUriStr,
        reply_stream_id: StreamId,
        reply_params: ReplayParams,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = ReplayRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.recording_id(*recording_id);
        msg.replay_channel(reply_channel.as_bytes());
        msg.replay_stream_id(*reply_stream_id);
        msg.file_io_max_length(reply_params.get_file_io_max_length().unwrap_or(context::NULL_VALUE));
        msg.length(reply_params.get_length().unwrap_or(context::NULL_VALUE as i64));
        msg.position(reply_params.get_position().unwrap_or(context::NULL_VALUE as i64));
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Stop an existing replay session.
    pub fn stop_replay(
        &mut self,
        replay_session_id: ControlSessionId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = StopReplayRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.replay_session_id(*replay_session_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Stop any existing replay sessions for recording Id or all replay sessions regardless of recording Id.
    pub fn stop_all_replay(
        &mut self,
        recording_id: RecordingId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = StopAllReplaysRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.recording_id(*recording_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// List a range of recording descriptors.
    pub fn list_recordings(
        &mut self,
        from_recording_id: RecordingId,
        record_count: i32,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = ListRecordingsRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.from_recording_id(*from_recording_id);
        msg.record_count(record_count);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// List a range of recording descriptors which match a channel URI fragment and stream id.
    pub fn list_recordings_from_uri(
        &mut self,
        from_recording_id: RecordingId,
        record_count: i32,
        channel: ChannelUriStr,
        stream_id: StreamId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg =
            ListRecordingsForUriRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.from_recording_id(*from_recording_id);
        msg.record_count(record_count);
        msg.channel(channel.as_bytes());
        msg.stream_id(*stream_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// List a recording descriptor for a given recording id.
    pub fn list_recording(
        &mut self,
        recording_id: RecordingId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = ListRecordingRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.recording_id(*recording_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    // Extend an existing, non-active, recorded stream for a the same channel and stream id.
    ///
    /// The channel must be configured for the initial position from which it will be extended. This can be done
    /// with [crate::channel_uri_string_builder::ChannelUriStringBuilder]. The details required to initialise can
    /// be found by calling [crate::archiver::ArchiveProxy::list_recording]
    #[allow(clippy::too_many_arguments)]
    pub fn extend_recording(
        &mut self,
        channel: ChannelUriStr,
        stream_id: StreamId,
        source_location: SourceLocation,
        recording_id: RecordingId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
        auto_stop: Option<bool>,
    ) -> Result<bool, AeronError> {
        if let Some(auto_stop) = auto_stop {
            let mut msg =
                ExtendRecordingRequest2Encoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
            msg.control_session_id(*control_session_id);
            msg.correlation_id(*correlation_id);
            msg.recording_id(*recording_id);
            msg.channel(channel.as_bytes());
            msg.stream_id(*stream_id);
            msg.source_location(source_location);
            let auto_stop = match auto_stop {
                true => BooleanType::FALSE,
                false => BooleanType::TRUE,
            };
            msg.auto_stop(auto_stop);
            let len = msg.encoded_length();
            let _ = msg.header(0);
            let length = message_header_codec::ENCODED_LENGTH + len;
            self.offer(length)
        } else {
            let mut msg =
                ExtendRecordingRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
            msg.control_session_id(*control_session_id);
            msg.correlation_id(*correlation_id);
            msg.recording_id(*recording_id);
            msg.channel(channel.as_bytes());
            msg.stream_id(*stream_id);
            msg.source_location(source_location);
            let len = msg.encoded_length();
            let _ = msg.header(0);
            let length = message_header_codec::ENCODED_LENGTH + len;
            self.offer(length)
        }
    }

    /// Get the recorded position of an active recording.
    pub fn get_recording_position(
        &mut self,
        recording_id: RecordingId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = RecordingPositionRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.recording_id(*recording_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Truncate a stopped recording to a given position that is less than the stopped position. The provided position
    /// must be on a fragment boundary. Truncating a recording to the start position effectively deletes the recording.
    pub fn truncate_recording(
        &mut self,
        recording_id: RecordingId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
        position: i64,
    ) -> Result<bool, AeronError> {
        let mut msg = TruncateRecordingRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.recording_id(*recording_id);
        msg.position(position);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Purge a stopped recording, i.e. mark recording as [aeron_archiver_codecs::recording_state::RecordingState::INVALID]
    /// and delete the corresponding segment files. The space in the Catalog will be reclaimed upon compaction.
    pub fn purge_recording(
        &mut self,
        recording_id: RecordingId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = PurgeRecordingRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.recording_id(*recording_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Get the start position of a recording.
    pub fn get_start_position(
        &mut self,
        recording_id: RecordingId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = StartPositionRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.recording_id(*recording_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Get the stop position of a recording.
    pub fn get_stop_position(
        &mut self,
        recording_id: RecordingId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = StopPositionRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.recording_id(*recording_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Find the last recording that matches the given criteria.
    pub fn find_last_matching_recording(
        &mut self,
        min_recording_id: RecordingId,
        channel: ChannelUriStr,
        stream_id: StreamId,
        session_id: SessionId,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg =
            FindLastMatchingRecordingRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.min_recording_id(*min_recording_id);
        msg.channel(channel.as_bytes());
        msg.session_id(*session_id);
        msg.stream_id(*stream_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// List registered subscriptions in the archive which have been used to record streams.
    #[allow(clippy::too_many_arguments)]
    pub fn list_recording_subscriptions(
        &mut self,
        pseudo_index: i32,
        subscription_count: i32,
        channel: ChannelUriStr,
        stream_id: StreamId,
        apply_stream_id: bool,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg =
            ListRecordingSubscriptionsRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.pseudo_index(pseudo_index);
        msg.channel(channel.as_bytes());
        msg.subscription_count(subscription_count);
        let apply_stream_id = match apply_stream_id {
            true => BooleanType::FALSE,
            false => BooleanType::TRUE,
        };
        msg.apply_stream_id(apply_stream_id);
        msg.stream_id(*stream_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Replicate a recording from a source archive to a destination which can be considered a backup for a primary
    /// archive. The source recording will be replayed via the provided replay channel and use the original stream id.
    /// If the destination recording id is None then a new destination recording is created,
    /// otherwise the provided destination recording id will be extended. The details of the source recording
    /// descriptor will be replicated.
    ///
    /// For a source recording that is still active the replay can merge with the live stream and then follow it
    /// directly and no longer require the replay from the source. This would require a multicast live destination.
    ///
    /// Errors will be reported asynchronously and can be checked for with [crate::archiver::AeronArchive::poll_for_error_response]
    /// or [crate::archiver::AeronArchive::check_for_error_response].
    #[allow(clippy::too_many_arguments)]
    pub fn replicate(
        &mut self,
        src_recording_id: RecordingId,
        dst_recording_id: RecordingId,
        src_control_stream_id: StreamId,
        src_control_channel: ChannelUriStr,
        live_destination: ChannelUriStr,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        self.inner_replicate(
            src_recording_id,
            dst_recording_id,
            src_control_stream_id,
            src_control_channel,
            live_destination,
            correlation_id,
            control_session_id,
            None,
            None,
            None,
            None,
            None,
        )
    }

    /// Replicate a recording from a source archive to a destination which can be considered a backup for a primary
    /// archive. The source recording will be replayed via the provided replay channel and use the original stream id.
    /// If the destination recording id is None then a new destination recording is created,
    /// otherwise the provided destination recording id will be extended. The details of the source recording
    /// descriptor will be replicated. The subscription used in the archive will be tagged with the provided tags.
    ///
    /// For a source recording that is still active the replay can merge with the live stream and then follow it
    /// directly and no longer require the replay from the source. This would require a multicast live destination.
    ///
    /// Errors will be reported asynchronously and can be checked for with [crate::archiver::AeronArchive::poll_for_error_response]
    /// or [crate::archiver::AeronArchive::check_for_error_response].
    #[allow(clippy::too_many_arguments)]
    pub fn taged_replicate(
        &mut self,
        src_recording_id: RecordingId,
        dst_recording_id: RecordingId,
        src_control_stream_id: StreamId,
        src_control_channel: ChannelUriStr,
        live_destination: ChannelUriStr,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
        channel_tag_id: i64,
        subscription_tag_id: i64,
    ) -> Result<bool, AeronError> {
        self.inner_replicate(
            src_recording_id,
            dst_recording_id,
            src_control_stream_id,
            src_control_channel,
            live_destination,
            correlation_id,
            control_session_id,
            None,
            Some(channel_tag_id),
            Some(subscription_tag_id),
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_replicate(
        &mut self,
        src_recording_id: RecordingId,
        dst_recording_id: RecordingId,
        src_control_stream_id: StreamId,
        src_control_channel: ChannelUriStr,
        live_destination: ChannelUriStr,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
        stop_position: Option<i64>,
        channel_tag_id: Option<i64>,
        subscription_tag_id: Option<i64>,
        replication_channel: Option<ChannelUriStr>,
        file_io_max_length: Option<i32>,
    ) -> Result<bool, AeronError> {
        let mut msg = ReplicateRequest2Encoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.src_control_channel(src_control_channel.as_bytes());
        msg.live_destination(live_destination.as_bytes());
        msg.src_recording_id(*src_recording_id);
        msg.dst_recording_id(*dst_recording_id);
        msg.src_control_stream_id(*src_control_stream_id);
        // optional
        msg.stop_position(stop_position.unwrap_or(context::NULL_VALUE as i64));
        msg.channel_tag_id(channel_tag_id.unwrap_or(context::NULL_VALUE as i64));
        msg.subscription_tag_id(subscription_tag_id.unwrap_or(context::NULL_VALUE as i64));
        if let Some(uri) = replication_channel {
            msg.replication_channel(uri.as_bytes());
        } else {
            msg.replication_channel(&vec![]);
        }
        msg.file_io_max_length(file_io_max_length.unwrap_or(context::NULL_VALUE));
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    /// Stop an active replication by the registration id it was registered with.
    pub fn stop_replication(
        &mut self,
        replication_id: i64,
        correlation_id: CorrelationId,
        control_session_id: ControlSessionId,
    ) -> Result<bool, AeronError> {
        let mut msg = StopReplicationRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.control_session_id(*control_session_id);
        msg.correlation_id(*correlation_id);
        msg.replication_id(replication_id);
        let len = msg.encoded_length();
        let _ = msg.header(0);
        let length = message_header_codec::ENCODED_LENGTH + len;
        self.offer(length)
    }

    fn offer(&mut self, length: usize) -> Result<bool, AeronError> {
        self.retry_idle_strategy.reset();
        let mut attempts = self.retry_attempts;
        loop {
            let result =
                self.publication
                    .lock()
                    .unwrap()
                    .offer_part(AtomicBuffer::from_aligned(&self.buffer), 0, length as Index);
            match result {
                Err(e) => match e {
                    AeronError::MaxPositionExceeded => {
                        return Err(e);
                    },
                    AeronError::NotConnected => {
                        return Err(e);
                    },
                    AeronError::PublicationClosed => {
                        return Err(e);
                    },
                    _ => {
                        // we want to handle ony the two error cases above
                        // otherwise if timeout has not been reached we will
                        // continue to retry
                        attempts -= 1;
                        if attempts <= 0 {
                            return Ok(false);
                        }
                    },
                },
                Ok(result) => {
                    if result > 0 {
                        return Ok(true);
                    }
                },
            }

            self.retry_idle_strategy.idle();
        }
    }

    fn offer_with_timeout(
        &self,
        length: usize,
        agen_invoker: Option<&AgentInvoker<ClientConductor>>,
    ) -> Result<bool, AeronError> {
        self.retry_idle_strategy.reset();
        let dead_line_ns = self.nano_clock.nano_time() + self.connect_timeout_ns;

        loop {
            let result =
                self.publication
                    .lock()
                    .unwrap()
                    .offer_part(AtomicBuffer::from_aligned(&self.buffer), 0, length as Index);
            match result {
                Err(e) => match e {
                    AeronError::MaxPositionExceeded => {
                        return Err(e);
                    },
                    AeronError::PublicationClosed => {
                        return Err(e);
                    },
                    _ => {
                        // we want to handle ony the two error cases above
                        // otherwise if timeout has not been reached we will
                        // continue to retry
                        if (dead_line_ns - self.nano_clock.nano_time()) < 0 {
                            return Ok(false);
                        }
                        agen_invoker.map(|invoker| invoker.invoke());
                    },
                },
                Ok(result) => {
                    if result > 0 {
                        return Ok(true);
                    }
                },
            }

            self.retry_idle_strategy.idle();
        }
    }
}
