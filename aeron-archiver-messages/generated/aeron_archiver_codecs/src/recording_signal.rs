#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum RecordingSignal {
    START = 0_i32,
    STOP = 1_i32,
    EXTEND = 2_i32,
    REPLICATE = 3_i32,
    MERGE = 4_i32,
    SYNC = 5_i32,
    DELETE = 6_i32,
    REPLICATE_END = 7_i32,
    #[default]
    NullVal = -2147483648_i32,
}
impl From<i32> for RecordingSignal {
    #[inline]
    fn from(v: i32) -> Self {
        match v {
            0_i32 => Self::START,
            1_i32 => Self::STOP,
            2_i32 => Self::EXTEND,
            3_i32 => Self::REPLICATE,
            4_i32 => Self::MERGE,
            5_i32 => Self::SYNC,
            6_i32 => Self::DELETE,
            7_i32 => Self::REPLICATE_END,
            _ => Self::NullVal,
        }
    }
}
