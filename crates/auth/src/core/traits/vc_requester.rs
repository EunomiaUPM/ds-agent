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
use std::str::FromStr;
use std::sync::Arc;

use crate::services::callback::CallbackTrait;
use crate::services::repo::repo_trait::AuthRepoTrait;
use crate::services::vc_requester::VcRequesterTrait;
use crate::types::entities::{ReachAuthority, WhoEntity};
use crate::types::wallet_helper::{ProcessUriOid4VCI, ProcessUriOid4VP};
use async_trait::async_trait;
use ymir::data::entities::{mates, req_vc};
use ymir::errors::{Errors, Outcome};
use ymir::services::wallet::WalletTrait;
use ymir::types::gnap::ApprovedCallbackBody;
use ymir::utils::trim_4_base;

#[async_trait]
pub trait CoreVcRequesterTrait: Send + Sync + 'static {
    fn vc_req(&self) -> Arc<dyn VcRequesterTrait>;
    fn repo(&self) -> Arc<dyn AuthRepoTrait>;
    fn callback(&self) -> Arc<dyn CallbackTrait>;
    fn wallet(&self) -> Arc<dyn WalletTrait>;
    async fn beg_vc(&self, payload: ReachAuthority) -> Outcome<Option<String>> {
        let (vc_model, int_model) = self.vc_req().start(&payload)?;
        let mut vc_model = self.repo().vc_req().create(vc_model).await?;
        let mut int_model = self.repo().interaction_req().create(int_model).await?;
        let (approved, uri) = self
            .vc_req()
            .send_req(&mut vc_model, &mut int_model)
            .await?;
        let mut vc_model = self.repo().vc_req().update(vc_model).await?;
        let int_model = self.repo().interaction_req().update(int_model).await?;
        match uri {
            Some(uri) => {
                if approved {
                    if vc_model.auto {
                        vc_model.vc_uri = Some(uri.clone());
                        vc_model.status = "Finalized".to_string();

                        let base_url = trim_4_base(&vc_model.grant_endpoint);
                        let mate = mates::NewModel {
                            participant_id: vc_model.authority_id.clone(),
                            participant_slug: vc_model.authority_slug.clone(),
                            participant_type: "Authority".to_string(),
                            base_url,
                            token: None,
                            extra_fields: None,
                            is_me: false,
                        };
                        self.repo().mates().force_create(mate).await?;
                        self.wallet().process_oidc4vci(&uri).await?;
                        self.repo().vc_req().update(vc_model).await?;
                        return Ok(None);
                    } else {
                        vc_model.vc_uri = Some(uri.clone());
                        vc_model.status = "Approved".to_string();
                        self.repo().vc_req().update(vc_model).await?;
                    }
                    Ok(Some(uri))
                } else {
                    let ver_model = self.vc_req().save_ver_data(&uri, &int_model.id)?;
                    let _ver_model = self.repo().verification_req().create(ver_model).await?;

                    if vc_model.auto {
                        self.wallet().process_oidc4vp(&uri).await?;
                        return Ok(None);
                    }
                    Ok(Some(uri))
                }
            }
            None => Ok(None),
        }
    }

    async fn continue_req(
        &self,
        id: String,
        payload: ApprovedCallbackBody,
    ) -> Outcome<mates::Model> {
        let mut int_model = self.repo().interaction_req().get_by_id(&id).await?;
        let result = self.callback().check_callback(&mut int_model, &payload);
        let int_model = self.repo().interaction_req().update(int_model).await?;
        result?;
        let response = self.callback().continue_req(&int_model).await?;
        let mut vc_req_model = self.repo().vc_req().get_by_id(&id).await?;
        let mate = self
            .vc_req()
            .manage_res(&mut vc_req_model, response)
            .await?;
        let mut vc_req_model = self.repo().vc_req().update(vc_req_model).await?;
        let mate = self.repo().mates().force_create(mate).await?;

        if vc_req_model.auto {
            let uri = vc_req_model
                .vc_uri
                .as_deref()
                .ok_or_else(|| Errors::crazy("Something crazy with auto wallet happened", None))?;
            self.wallet().process_oidc4vci(uri).await?;
            vc_req_model.status = "Finalized".to_string();
            self.repo().vc_req().update(vc_req_model).await?;
        }

        Ok(mate)
    }
    async fn manage_rejection(&self, id: String) -> Outcome<()> {
        let mut vc_req_model = self.repo().vc_req().get_by_id(&id).await?;
        self.vc_req().manage_rejection(&mut vc_req_model).await?;
        self.repo().vc_req().update(vc_req_model).await?;
        Ok(())
    }
    async fn get_all(&self) -> Outcome<Vec<req_vc::Model>> {
        self.repo().vc_req().get_all(None, None).await
    }

    async fn get_by_id(&self, id: String) -> Outcome<req_vc::Model> {
        self.repo().vc_req().get_by_id(&id).await
    }
    async fn process_oid4vci(&self, payload: &ProcessUriOid4VCI) -> Outcome<()> {
        let mut model = self.repo().vc_req().get_by_id(&payload.id).await?;

        self.wallet().process_oidc4vci(&payload.uri).await?;
        model.status = "Finalized".to_string();
        let model = self.repo().vc_req().update(model).await?;
        let base_url = trim_4_base(&model.grant_endpoint);
        let mate = mates::NewModel {
            participant_id: model.authority_id.clone(),
            participant_slug: model.authority_slug.clone(),
            participant_type: "Authority".to_string(),
            base_url,
            token: None,
            extra_fields: None,
            is_me: false,
        };
        self.repo().mates().force_create(mate).await?;

        Ok(())
    }
    async fn process_oid4vp(&self, payload: &ProcessUriOid4VP) -> Outcome<()> {
        self.wallet().process_oidc4vp(&payload.uri).await?;

        let entity = WhoEntity::from_str(&payload.entity)?;
        match entity {
            WhoEntity::Authority => {
                let mut model = self.repo().vc_req().get_by_id(&payload.id).await?;
                model.status = "Approved".to_string();
                self.repo().vc_req().update(model).await?;
            }
            WhoEntity::Provider => {
                let mut model = self.repo().request_req().get_by_id(&payload.id).await?;
                model.status = "Approved".to_string();
                self.repo().request_req().update(model).await?;
            }
        };

        Ok(())
    }
}
