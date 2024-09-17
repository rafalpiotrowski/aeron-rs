pub mod archive;
mod archive_proxy;
pub(crate) mod configuration;

pub use archive_proxy::ArchiveProxy;

microtype! {
    #[derive(Debug, Clone, PartialEq)]
    #[int]
    pub i64 {
        RecordingId
    }
}

/// Fluent API for setting optional replay parameters. Not threadsafe. Allows the user to configure starting position,
/// replay length, bounding counter (for a bounded replay) and the max length for file I/O operations.
#[derive(Default, Clone, Copy, Debug)]
pub struct ReplayParams {
    bounded_limit_counter_id: Option<i32>,
    file_io_max_length: Option<i32>,
    position: Option<i64>,
    length: Option<i64>,
}

impl ReplayParams {
    /// Default, initialise all values to None
    pub fn reset(mut self) -> Self {
        self.bounded_limit_counter_id = None;
        self.file_io_max_length = None;
        self.position = None;
        self.length = None;
        self
    }

    pub fn position(mut self, value: i64) -> Self {
        self.position = Some(value);
        self
    }
    pub fn length(mut self, value: i64) -> Self {
        self.length = Some(value);
        self
    }
    pub fn file_io_max_length(mut self, value: i32) -> Self {
        self.file_io_max_length = Some(value);
        self
    }
    pub fn bounded_limit_counter_id(mut self, value: i32) -> Self {
        self.bounded_limit_counter_id = Some(value);
        self
    }

    pub fn get_position(&self) -> Option<i64> {
        self.position
    }
    pub fn get_length(&self) -> Option<i64> {
        self.length
    }
    pub fn get_file_io_max_length(&self) -> Option<i32> {
        self.file_io_max_length
    }
    pub fn get_bounded_limit_counter_id(&self) -> Option<i32> {
        self.bounded_limit_counter_id
    }

    pub fn is_bounded(&self) -> bool {
        self.bounded_limit_counter_id.is_some()
    }
}
