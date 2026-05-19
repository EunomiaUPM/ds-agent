use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Version(u64);

impl Version {
    pub const INITIAL: Version = Version(1);
    pub fn new(v: u64) -> Self {
        Version(v)
    }
    pub fn value(self) -> u64 {
        self.0
    }
    pub fn next(self) -> Self {
        Version(self.0 + 1)
    }
}
