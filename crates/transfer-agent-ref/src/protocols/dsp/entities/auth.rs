use common::facades::Mates;
use oauth::entities::user::User;

/// Authentication info por Context

/// Common view over the auth attached to any transfer context, whatever the
/// source. Lets the shared context / command code read identity and token
/// without knowing whether it came from an inbound DSP peer or an outbound RPC
/// caller.
pub trait TransferAuthn {
    fn raw(&self) -> &str;
    fn token_type(&self) -> &str;
    fn token_content(&self) -> &str;
    fn participant(&self) -> &Mates;
}

/// Inbound DSP: a peer authenticated to us. `associated_participant` is that
/// remote peer, resolved by the auth middleware before we ran.
#[derive(Debug)]
pub struct TransferDSPAuthn {
    pub raw: String,
    pub token_type: String,
    pub token_content: String,
    pub associated_participant: Mates,
}

impl TransferAuthn for TransferDSPAuthn {
    fn raw(&self) -> &str {
        &self.raw
    }
    fn token_type(&self) -> &str {
        &self.token_type
    }
    fn token_content(&self) -> &str {
        &self.token_content
    }
    fn participant(&self) -> &Mates {
        &self.associated_participant
    }
}

/// Outbound RPC: our own app driving a transfer. `me_participant` is us, and
/// `me_user` is the acting user behind the RPC call.
#[derive(Debug)]
pub struct TransferRPCAuthn {
    pub raw: String,
    pub token_type: String,
    pub token_content: String,
    pub me_participant: Mates,
    pub me_user: User,
}

impl TransferAuthn for TransferRPCAuthn {
    fn raw(&self) -> &str {
        &self.raw
    }
    fn token_type(&self) -> &str {
        &self.token_type
    }
    fn token_content(&self) -> &str {
        &self.token_content
    }
    fn participant(&self) -> &Mates {
        &self.me_participant
    }
}
