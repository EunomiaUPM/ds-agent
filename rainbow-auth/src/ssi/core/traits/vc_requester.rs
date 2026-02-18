/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use std::sync::Arc;

use async_trait::async_trait;
use ymir::data::entities::{mates, req_vc};
use ymir::types::gnap::grant_request::InteractStart;
use ymir::types::gnap::ApprovedCallbackBody;

use crate::ssi::services::callback::CallbackTrait;
use crate::ssi::services::repo::repo_trait::AuthRepoTrait;
use crate::ssi::services::vc_requester::VcRequesterTrait;
use crate::ssi::types::entities::ReachAuthority;

#[async_trait]
pub trait CoreVcRequesterTrait: Send + Sync + 'static {
    fn vc_req(&self) -> Arc<dyn VcRequesterTrait>;
    fn repo(&self) -> Arc<dyn AuthRepoTrait>;
    fn callback(&self) -> Arc<dyn CallbackTrait>;
    async fn beg_vc(
        &self,
        payload: ReachAuthority,
        method: InteractStart,
    ) -> anyhow::Result<Option<String>> {
        let (vc_model, int_model) = self.vc_req().start(payload, method);
        let mut vc_model = self.repo().vc_req().create(vc_model).await?;
        let mut int_model = self.repo().interaction_req().create(int_model).await?;
        let uri = self.vc_req().send_req(&mut vc_model, &mut int_model).await?;
        let _vc_model = self.repo().vc_req().update(vc_model).await?;
        let int_model = self.repo().interaction_req().update(int_model).await?;
        match uri {
            Some(uri) => {
                let ver_model = self.vc_req().save_ver_data(&uri, &int_model.id)?;
                let _ver_model = self.repo().verification_req().create(ver_model).await?;
                Ok(Some(uri))
            }
            None => Ok(None),
        }
    }

    async fn get_all(&self) -> anyhow::Result<Vec<req_vc::Model>> {
        self.repo().vc_req().get_all(None, None).await
    }

    async fn get_by_id(&self, id: String) -> anyhow::Result<req_vc::Model> {
        self.repo().vc_req().get_by_id(&id).await
    }
    async fn continue_req(
        &self,
        id: String,
        payload: ApprovedCallbackBody,
    ) -> anyhow::Result<mates::Model> {
        let mut int_model = self.repo().interaction_req().get_by_id(&id).await?;
        let result = self.callback().check_callback(&mut int_model, &payload);
        let int_model = self.repo().interaction_req().update(int_model).await?;
        result?;
        let response = self.callback().continue_req(&int_model).await?;
        let mut vc_req_model = self.repo().vc_req().get_by_id(&id).await?;
        let mate = self.vc_req().manage_res(&mut vc_req_model, response).await?;
        self.repo().vc_req().update(vc_req_model).await?;
        let mate = self.repo().mates().force_create(mate).await?;
        Ok(mate)
    }
    async fn manage_rejection(&self, id: String) -> anyhow::Result<()> {
        let mut vc_req_model = self.repo().vc_req().get_by_id(&id).await?;
        self.vc_req().manage_rejection(&mut vc_req_model).await?;
        self.repo().vc_req().update(vc_req_model).await?;
        Ok(())
    }
}
