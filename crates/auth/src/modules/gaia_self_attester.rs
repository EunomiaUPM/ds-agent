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

use crate::services::{HasGaiaSelfAttester, MayHaveEventBus};
use crate::types::events::GaiaIssuedEvent;
use crate::types::gaia::{ComplianceForm, DispatcherOutcome};
use async_trait::async_trait;
use serde_json::Value;
use ymir::data::entities::wallet::vc;
use ymir::errors::{Errors, Outcome};
use ymir::oauth::Owner;
use ymir::services::{HasIssuer, HasWallet};
use ymir::types::issuance::VcBody;
use ymir::types::oauth::{RoleTrait, UserInfo};
use ymir::types::vcs::VcType;
use ymir::types::wallet::ComplianceRequest;

/// Issuing this participant's own Gaia-X credentials.
#[async_trait]
pub trait GaiaSelfAttesterModule:
    HasGaiaSelfAttester + HasIssuer + HasWallet + MayHaveEventBus + Send + Sync + 'static
{
    /// Attests the connector's shared identity, hence admin only.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn generate_gaia_vcs(&self, user: &UserInfo) -> Outcome<()> {
        user.require_root()?;
        // The legal person links the subject of the newest registration number in the wallet.
        let registration = self
            .newest_registration()
            .await?
            .parsed_document
            .pointer("/credentialSubject/id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| Errors::forbidden("The registration number has no subject", None))?;
        let legal_p = self.gaia().generate_legal_person(&registration).await?;
        let terms = self.gaia().generate_terms_cons_vc().await?;
        let legal_p = self.issuer().sign_claims(&legal_p).await?;
        let terms = self.issuer().sign_claims(&terms).await?;

        let legal_p = vc::Plan {
            vc_body: VcBody::Jwt(legal_p.as_str().to_string()),
        };
        let terms = vc::Plan {
            vc_body: VcBody::Jwt(terms.as_str().to_string()),
        };
        self.wallet().store_vc(legal_p).await?;
        self.wallet().store_vc(terms).await?;

        let payload = GaiaIssuedEvent {
            credentials: vec!["LegalPerson".to_string(), "Issuer".to_string()],
        };
        events::emit_action!(
            self.event_bus(),
            &Owner::connector(),
            crate::EVENT_PREFIX,
            "gaia",
            "issued",
            &payload
        );
        Ok(())
    }

    /// Gaia-X compliance by plain POST to the service at `form.url`: the wallet presents its legal
    /// person, terms and newest registration number, and stores the credential it gets back.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn request_compliance(&self, user: &UserInfo, form: ComplianceForm) -> Outcome<vc::Model> {
        user.require_root()?;
        let registration = self.newest_registration().await?.vc_type;
        let request = ComplianceRequest {
            url: form.url,
            vcid: form.vcid.filter(|vcid| !vcid.trim().is_empty()),
            types: vec![VcType::LegalPerson, VcType::GxIssuer, registration],
        };
        let stored = self.wallet().request_compliance(&request).await?;
        self.compliance_event(&stored).await;
        Ok(stored)
    }

    /// Gaia-X compliance through a Dispatcher: presents to its OID4VP request `uri` and, if it
    /// answers with a credential offer, claims it. Any other next step goes back to the caller.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn present_to_dispatcher(&self, user: &UserInfo, uri: &str) -> Outcome<DispatcherOutcome> {
        user.require_root()?;
        let next = self.wallet().process_oid4vp(uri).await?;
        match next {
            Some(offer) if is_credential_offer(&offer) => {
                self.wallet().process_oid4vci(&offer).await?;
                Ok(DispatcherOutcome { received: true, redirect_uri: None })
            }
            other => Ok(DispatcherOutcome { received: false, redirect_uri: other }),
        }
    }

    /// The newest registration number credential in the wallet.
    async fn newest_registration(&self) -> Outcome<vc::Model> {
        self.wallet()
            .retrieve_all_vcs()
            .await?
            .into_iter()
            .filter(|vc| vc.vc_type.is_legal_registration_number())
            .max_by_key(|vc| vc.added_on)
            .ok_or_else(|| Errors::forbidden("Request a registration number credential first", None))
    }

    async fn compliance_event(&self, stored: &vc::Model) {
        let payload = GaiaIssuedEvent {
            credentials: vec![stored.vc_type.to_string()],
        };
        events::emit_action!(
            self.event_bus(),
            &Owner::connector(),
            crate::EVENT_PREFIX,
            "gaia",
            "compliance",
            &payload
        );
    }
}

/// Whether `uri` is an OpenID4VCI credential offer (by value or by reference).
fn is_credential_offer(uri: &str) -> bool {
    uri.starts_with("openid-credential-offer:")
        || uri.contains("credential_offer=")
        || uri.contains("credential_offer_uri=")
}
