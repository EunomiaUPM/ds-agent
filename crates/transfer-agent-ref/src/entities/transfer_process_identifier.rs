use urn::Urn;

#[derive(Clone)]
pub(crate) struct TransferProcessIdentifier {
    pub transfer_process_id: Urn,
    pub key: String,
    pub value: Option<String>,
}

impl TransferProcessIdentifier {
    pub fn new(
        transfer_process_id: Urn,
        key: impl Into<String>,
        value: impl Into<Option<String>>,
    ) -> Self {
        Self {
            transfer_process_id,
            key: key.into(),
            value: value.into(),
        }
    }
}