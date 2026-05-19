use crate::utils::validate_key;
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Key(String);
impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Key {
    pub fn new(s: impl Into<String>) -> Outcome<Self> {
        let s = s.into();
        validate_key(&s)?;
        Ok(Self(s))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct KeyPrefix(String);
