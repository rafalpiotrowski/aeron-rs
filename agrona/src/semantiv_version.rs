use crate::AgronaError;
use std::result::Result;

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
        if major < 0 || major > 255 {
            return Err(AgronaError::ArgumentOutOfBounds(format!("major must be 0-255: {0}", major)));
        }
        if minor < 0 || minor > 255 {
            return Err(AgronaError::ArgumentOutOfBounds(format!("minor must be 0-255: {0}", minor)));
        }
        if patch < 0 || patch > 255 {
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

impl Into<String> for SemanticVersion {
    fn into(self) -> String {
        format!("{0}.{1}.{2}", self.major(), self.minor(), self.patch()).to_string()
    }
}
