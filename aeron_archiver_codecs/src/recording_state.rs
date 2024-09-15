#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum RecordingState {
    INVALID = 0_i32, 
    VALID = 1_i32, 
    #[default]
    NullVal = -2147483648_i32, 
}
impl From<i32> for RecordingState {
    #[inline]
    fn from(v: i32) -> Self {
        match v {
            0_i32 => Self::INVALID, 
            1_i32 => Self::VALID, 
            _ => Self::NullVal,
        }
    }
}
