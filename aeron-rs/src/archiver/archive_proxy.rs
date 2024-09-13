use aeron_archiver_messages::auth_connect_request_codec::{self, AuthConnectRequestEncoder};
use aeron_archiver_messages::{
    message_header_codec::{self, MessageHeaderEncoder},
    WriteBuf,
};
use std::{
    slice,
    sync::{Arc, Mutex},
};

use crate::aeron::Aeron;
use crate::client_conductor::ClientConductor;
use crate::concurrent::agent_invoker::AgentInvoker;
use crate::concurrent::strategies::YieldingIdleStrategy;
use crate::utils::errors::AeronError;
use crate::utils::time::{NanoClock, SystemNanoClock};
use crate::utils::types::Index;
use crate::{
    concurrent::{
        atomic_buffer::{AlignedBuffer, AtomicBuffer},
        strategies::{NoOpIdleStrategy, Strategy},
    },
    publication::Publication,
    security::{CredentialSupplier, NoCredentialsSupplier},
};

use crate::archiver::configuration::{MESSAGE_TIMEOUT_DEFAULT_NS, PROTOCOL_SEMANTIC_VERSION};

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

    pub fn get_write_buf(&self) -> WriteBuf<'_> {
        let slice = unsafe { slice::from_raw_parts_mut(self.buffer.ptr.offset(0 as isize), self.buffer.len as usize) };
        WriteBuf::new(slice)
    }

    pub fn connect(&mut self, response_channel: &str, response_stream_id: i32, correlation_id: i64) -> Result<bool, AeronError> {
        let credentials = self.credential_supplier.encoded_credentials();
        let mut msg = AuthConnectRequestEncoder::default().wrap(self.get_write_buf(), message_header_codec::ENCODED_LENGTH);
        msg.correlation_id(correlation_id);
        msg.response_stream_id(response_stream_id);
        msg.version(PROTOCOL_SEMANTIC_VERSION.version());
        msg.response_channel(response_channel.as_bytes());
        msg.encoded_credentials(&credentials);
        let len = msg.encoded_length();
        // write header
        let _ = msg.header(0);
        self.offer_with_timeout(message_header_codec::ENCODED_LENGTH + len, None)
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
