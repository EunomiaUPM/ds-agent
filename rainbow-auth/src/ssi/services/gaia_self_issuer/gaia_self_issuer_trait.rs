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

use async_trait::async_trait;
use serde_json::Value;
use ymir::data::entities::issuing;
use ymir::types::issuing::IssuingToken;
use ymir::types::vcs::VcType;
use ymir::types::wallet::WalletCredentials;

#[async_trait]
pub trait GaiaOwnIssuerTrait: Send + Sync + 'static {
    fn start_basic_vcs(&self) -> issuing::NewModel;
    fn get_token(&self) -> IssuingToken;
    fn get_did(&self) -> String;
    async fn issue_cred(&self, did: &str) -> anyhow::Result<Value>;
    async fn build_vp(
        &self,
        vcs: Vec<WalletCredentials>,
        did: Option<String>
    ) -> anyhow::Result<String>;
    async fn send_req(&self, body: String) -> anyhow::Result<String>;
    fn get_vc_types(&self) -> Vec<VcType>;
}
