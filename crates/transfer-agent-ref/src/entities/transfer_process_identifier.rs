pub(crate) struct TransferProcessIdentifier {
    pub key: String,
    pub value: Option<String>,
}

impl TransferProcessIdentifier {
    pub fn new(key: impl Into<String>, value: impl Into<Option<String>>) -> Self {
        Self { key: key.into(), value: value.into() }
    }
}