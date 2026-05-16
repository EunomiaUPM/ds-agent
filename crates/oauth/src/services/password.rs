use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use ymir::errors::{BadFormat, Errors, Outcome};

pub(crate) fn hash_password(password: &str) -> Outcome<(String, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let phc = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| Errors::crazy("password hashing failed", Some(e.to_string().into())))?;
    let salt_str = phc.salt.map(|s| s.to_string()).unwrap_or_default();
    Ok((phc.to_string(), salt_str))
}

pub(crate) fn verify_password(password: &str, password_hash: &str) -> Outcome<()> {
    let hash = PasswordHash::new(password_hash)
        .map_err(|e| Errors::crazy("password hash error", Some(e.to_string().into())))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .map_err(|_| Errors::format(BadFormat::Received, "invalid credentials", None))
}
