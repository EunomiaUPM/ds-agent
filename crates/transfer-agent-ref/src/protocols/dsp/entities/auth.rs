use common::facades::Mates;
use oauth::entities::user::User;

#[derive(Debug)]
pub struct TransferDSPAuthn {
    pub raw: String,
    pub token_type: String,
    pub token_content: String,
    pub associated_participant: Mates,
}


#[derive(Debug)]
pub struct TransferRPCAuthn {
    pub raw: String,
    pub token_type: String,
    pub token_content: String,
    pub me_participant: Mates,
    pub me_user: User
}