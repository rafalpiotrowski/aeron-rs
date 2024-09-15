#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum BooleanType {
    FALSE = 0_i32,
    TRUE = 1_i32,
    #[default]
    NullVal = -2147483648_i32,
}
impl From<i32> for BooleanType {
    #[inline]
    fn from(v: i32) -> Self {
        match v {
            0_i32 => Self::FALSE,
            1_i32 => Self::TRUE,
            _ => Self::NullVal,
        }
    }
}
