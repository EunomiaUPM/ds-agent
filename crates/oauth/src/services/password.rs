/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use ymir::errors::{BadFormat, Errors, Outcome};

/// Hash utils for hashing password before persist layers
pub(crate) fn hash_password(password: &str) -> Outcome<(String, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let phc = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| Errors::crazy("password hashing failed", Some(e.to_string().into())))?;
    let salt_str = phc.salt.map(|s| s.to_string()).unwrap_or_default();
    Ok((phc.to_string(), salt_str))
}

/// Hash utils for verify password against password hash
pub(crate) fn verify_password(password: &str, password_hash: &str) -> Outcome<()> {
    let hash = PasswordHash::new(password_hash)
        .map_err(|e| Errors::crazy("password hash error", Some(e.to_string().into())))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .map_err(|_| Errors::format(BadFormat::Received, "invalid credentials", None))
}
