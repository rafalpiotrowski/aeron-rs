#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum ControlResponseCode {
    OK = 0_i32, 
    ERROR = 1_i32, 
    RECORDING_UNKNOWN = 2_i32, 
    SUBSCRIPTION_UNKNOWN = 3_i32, 
    #[default]
    NullVal = -2147483648_i32, 
}
impl From<i32> for ControlResponseCode {
    #[inline]
    fn from(v: i32) -> Self {
        match v {
            0_i32 => Self::OK, 
            1_i32 => Self::ERROR, 
            2_i32 => Self::RECORDING_UNKNOWN, 
            3_i32 => Self::SUBSCRIPTION_UNKNOWN, 
            _ => Self::NullVal,
        }
    }
}
