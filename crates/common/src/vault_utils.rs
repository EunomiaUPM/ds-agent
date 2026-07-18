use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::Outcome;
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::vault_rs::RealVaultService;
use ymir::services::vault::VaultService;

/// Builds the [`VaultService`] the config asks for: the real Vault client
/// when `is_vault_real`, the in-memory fake otherwise.
pub fn vault(config: &impl ConnectionConfigTrait) -> Outcome<VaultService> {
    Ok(if config.is_vault_real() {
        VaultService::Real(RealVaultService::new()?)
    } else {
        VaultService::Fake(FakeVaultService::new()?)
    })
}
