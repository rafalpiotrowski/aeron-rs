#![forbid(unsafe_code)]
#![allow(clippy::upper_case_acronyms)]
#![allow(non_camel_case_types)]
#![allow(ambiguous_glob_reexports)]

use ::core::convert::TryInto;

pub mod attach_segments_request_codec;
pub mod auth_connect_request_codec;
pub mod boolean_type;
pub mod bounded_replay_request_codec;
pub mod catalog_header_codec;
pub mod challenge_codec;
pub mod challenge_response_codec;
pub mod close_session_request_codec;
pub mod connect_request_codec;
pub mod control_response_code;
pub mod control_response_codec;
pub mod delete_detached_segments_request_codec;
pub mod detach_segments_request_codec;
pub mod extend_recording_request_2_codec;
pub mod extend_recording_request_codec;
pub mod find_last_matching_recording_request_codec;
pub mod keep_alive_request_codec;
pub mod list_recording_request_codec;
pub mod list_recording_subscriptions_request_codec;
pub mod list_recordings_for_uri_request_codec;
pub mod list_recordings_request_codec;
pub mod message_header_codec;
pub mod migrate_segments_request_codec;
pub mod purge_recording_request_codec;
pub mod purge_segments_request_codec;
pub mod recording_descriptor_codec;
pub mod recording_descriptor_header_codec;
pub mod recording_position_request_codec;
pub mod recording_progress_codec;
pub mod recording_signal;
pub mod recording_signal_event_codec;
pub mod recording_started_codec;
pub mod recording_state;
pub mod recording_stopped_codec;
pub mod recording_subscription_descriptor_codec;
pub mod replay_request_codec;
pub mod replicate_request_2_codec;
pub mod replicate_request_codec;
pub mod source_location;
pub mod start_position_request_codec;
pub mod start_recording_request_2_codec;
pub mod start_recording_request_codec;
pub mod stop_all_replays_request_codec;
pub mod stop_position_request_codec;
pub mod stop_recording_by_identity_request_codec;
pub mod stop_recording_request_codec;
pub mod stop_recording_subscription_request_codec;
pub mod stop_replay_request_codec;
pub mod stop_replication_request_codec;
pub mod tagged_replicate_request_codec;
pub mod truncate_recording_request_codec;
pub mod var_ascii_encoding_codec;
pub mod var_data_encoding_codec;

pub type SbeResult<T> = core::result::Result<T, SbeErr>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SbeErr {
    ParentNotSet,
}
impl core::fmt::Display for SbeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SbeErr {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub trait Writer<'a>: Sized {
    fn get_buf_mut(&mut self) -> &mut WriteBuf<'a>;
}

pub trait Encoder<'a>: Writer<'a> {
    fn get_limit(&self) -> usize;
    fn set_limit(&mut self, limit: usize);
}

pub trait ActingVersion {
    fn acting_version(&self) -> u16;
}

pub trait Reader<'a>: Sized {
    fn get_buf(&self) -> &ReadBuf<'a>;
}

pub trait Decoder<'a>: Reader<'a> {
    fn get_limit(&self) -> usize;
    fn set_limit(&mut self, limit: usize);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ReadBuf<'a> {
    data: &'a [u8],
}
impl<'a> Reader<'a> for ReadBuf<'a> {
    #[inline]
    fn get_buf(&self) -> &ReadBuf<'a> {
        self
    }
}
#[allow(dead_code)]
impl<'a> ReadBuf<'a> {
    #[inline]
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    #[inline]
    pub(crate) fn get_bytes_at<const N: usize>(slice: &[u8], index: usize) -> [u8; N] {
        slice[index..index + N].try_into().expect("slice with incorrect length")
    }

    #[inline]
    pub fn get_u8_at(&self, index: usize) -> u8 {
        self.data[index]
    }

    #[inline]
    pub fn get_i8_at(&self, index: usize) -> i8 {
        i8::from_le_bytes(Self::get_bytes_at(self.data, index))
    }

    #[inline]
    pub fn get_i16_at(&self, index: usize) -> i16 {
        i16::from_le_bytes(Self::get_bytes_at(self.data, index))
    }

    #[inline]
    pub fn get_i32_at(&self, index: usize) -> i32 {
        i32::from_le_bytes(Self::get_bytes_at(self.data, index))
    }

    #[inline]
    pub fn get_i64_at(&self, index: usize) -> i64 {
        i64::from_le_bytes(Self::get_bytes_at(self.data, index))
    }

    #[inline]
    pub fn get_u16_at(&self, index: usize) -> u16 {
        u16::from_le_bytes(Self::get_bytes_at(self.data, index))
    }

    #[inline]
    pub fn get_u32_at(&self, index: usize) -> u32 {
        u32::from_le_bytes(Self::get_bytes_at(self.data, index))
    }

    #[inline]
    pub fn get_u64_at(&self, index: usize) -> u64 {
        u64::from_le_bytes(Self::get_bytes_at(self.data, index))
    }

    #[inline]
    pub fn get_f32_at(&self, index: usize) -> f32 {
        f32::from_le_bytes(Self::get_bytes_at(self.data, index))
    }

    #[inline]
    pub fn get_f64_at(&self, index: usize) -> f64 {
        f64::from_le_bytes(Self::get_bytes_at(self.data, index))
    }

    #[inline]
    pub fn get_slice_at(&self, index: usize, len: usize) -> &[u8] {
        &self.data[index..index + len]
    }
}

#[derive(Debug, Default)]
pub struct WriteBuf<'a> {
    data: &'a mut [u8],
}
impl<'a> WriteBuf<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        Self { data }
    }

    #[inline]
    pub fn put_bytes_at<const COUNT: usize>(&mut self, index: usize, bytes: &[u8; COUNT]) -> usize {
        self.data[index..index + COUNT].copy_from_slice(bytes);
        COUNT
    }

    #[inline]
    pub fn put_u8_at(&mut self, index: usize, value: u8) {
        self.data[index] = value;
    }

    #[inline]
    pub fn put_i8_at(&mut self, index: usize, value: i8) {
        self.put_bytes_at(index, &i8::to_le_bytes(value));
    }

    #[inline]
    pub fn put_i16_at(&mut self, index: usize, value: i16) {
        self.put_bytes_at(index, &i16::to_le_bytes(value));
    }

    #[inline]
    pub fn put_i32_at(&mut self, index: usize, value: i32) {
        self.put_bytes_at(index, &i32::to_le_bytes(value));
    }

    #[inline]
    pub fn put_i64_at(&mut self, index: usize, value: i64) {
        self.put_bytes_at(index, &i64::to_le_bytes(value));
    }

    #[inline]
    pub fn put_u16_at(&mut self, index: usize, value: u16) {
        self.put_bytes_at(index, &u16::to_le_bytes(value));
    }

    #[inline]
    pub fn put_u32_at(&mut self, index: usize, value: u32) {
        self.put_bytes_at(index, &u32::to_le_bytes(value));
    }

    #[inline]
    pub fn put_u64_at(&mut self, index: usize, value: u64) {
        self.put_bytes_at(index, &u64::to_le_bytes(value));
    }

    #[inline]
    pub fn put_f32_at(&mut self, index: usize, value: f32) {
        self.put_bytes_at(index, &f32::to_le_bytes(value));
    }

    #[inline]
    pub fn put_f64_at(&mut self, index: usize, value: f64) {
        self.put_bytes_at(index, &f64::to_le_bytes(value));
    }

    #[inline]
    pub fn put_slice_at(&mut self, index: usize, src: &[u8]) -> usize {
        let len = src.len();
        let dest = self.data.split_at_mut(index).1.split_at_mut(len).0;
        dest.clone_from_slice(src);
        len
    }
}
