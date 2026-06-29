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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::services::HasGaiaSelfAttester;
use async_trait::async_trait;
use ymir::data::entities::wallet::vc;
use ymir::errors::Outcome;
use ymir::services::{HasIssuer, HasWallet};
use ymir::types::issuance::VcBody;

#[async_trait]
pub trait GaiaSelfAttesterModule:
    HasGaiaSelfAttester + HasIssuer + HasWallet + Send + Sync + 'static
{
    async fn generate_gaia_vcs(&self) -> Outcome<()> {
        let legal_p = self.gaia().generate_legal_person().await?;
        let terms = self.gaia().generate_terms_cons_vc().await?;
        let legal_p = self.issuer().sign_claims(&legal_p).await?;
        let terms = self.issuer().sign_claims(&terms).await?;

        let legal_p = vc::Plan {
            vc_body: VcBody::Jwt(legal_p),
        };
        let terms = vc::Plan {
            vc_body: VcBody::Jwt(terms),
        };
        self.wallet().store_vc(legal_p).await?;
        self.wallet().store_vc(terms).await?;

        Ok(())
    }
}
