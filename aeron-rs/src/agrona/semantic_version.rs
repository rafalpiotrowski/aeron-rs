use std::result::Result;

use crate::agrona::AgronaError;

#[derive(Debug, Copy, Clone)]
pub struct SemanticVersion {
    version: i32,
}

unsafe impl Sync for SemanticVersion {}
unsafe impl Send for SemanticVersion {}

impl SemanticVersion {
    /// Compose a 4-byte integer with major, minor, and patch version stored in the least significant 3 bytes.
    /// The sum of the components must be greater than zero.
    pub fn compose(major: i32, minor: i32, patch: i32) -> Result<SemanticVersion, AgronaError> {
        if !(0..=255).contains(&major) {
            return Err(AgronaError::ArgumentOutOfBounds(format!("major must be 0-255: {0}", major)));
        }
        if !(0..=255).contains(&minor) {
            return Err(AgronaError::ArgumentOutOfBounds(format!("minor must be 0-255: {0}", minor)));
        }
        if !(0..=255).contains(&patch) {
            return Err(AgronaError::ArgumentOutOfBounds(format!("patch must be 0-255: {0}", patch)));
        }
        Ok(SemanticVersion {
            version: (major << 16) | (minor << 8) | patch,
        })
    }

    pub fn major(&self) -> i32 {
        (self.version >> 16) & 0xFF
    }
    pub fn minor(&self) -> i32 {
        (self.version >> 8) & 0xFF
    }
    pub fn patch(&self) -> i32 {
        self.version & 0xFF
    }

    pub fn version(&self) -> i32 {
        self.version
    }
}

impl From<SemanticVersion> for String {
    fn from(val: SemanticVersion) -> Self {
        format!("{0}.{1}.{2}", val.major(), val.minor(), val.patch()).to_string()
    }
}
