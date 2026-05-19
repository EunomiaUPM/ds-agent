use secrecy::{ExposeSecret, SecretString};

#[derive(Clone)]
pub struct SecretValue(secrecy::SecretString);

impl SecretValue {
    pub fn new(s: String) -> Self {
        Self(SecretString::new(s.into()))
    }
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl std::fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretValue(*****)")
    }
}

impl serde::Serialize for SecretValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("*****")
    }
}

impl<'de> serde::Deserialize<'de> for SecretValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(SecretValue::new(s))
    }
}
