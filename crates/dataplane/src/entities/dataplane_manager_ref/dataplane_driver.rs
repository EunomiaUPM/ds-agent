use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct DataplaneDriverAuth {
    #[serde(default)]
    #[serde(rename = "type")]
    pub(crate) r#type: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DataplaneDriverInteraction {
    #[serde(default)]
    #[serde(rename = "type")]
    pub(crate) r#type: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DataplaneDriver {
    pub auth: DataplaneDriverAuth,
    pub interaction: DataplaneDriverInteraction,
}

impl DataplaneDriver {
    pub fn new(auth: DataplaneDriverAuth, interaction: DataplaneDriverInteraction) -> Self {
        Self { auth, interaction }
    }
}

