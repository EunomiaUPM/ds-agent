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

//! Proto ⇄ domain mappers for the policy-template RPCs.

use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_templates::types::LocalizedText;
use crate::entities::policy_templates::{NewPolicyTemplateDto, PolicyTemplateDto};
use crate::grpc::api::catalog_agent::{
    CreatePolicyTemplateRequest, ListPolicyTemplatesRequest, PolicyTemplate,
    PolicyTemplateListResponse, PolicyTemplateResponse,
};
use common::dsp_common::odrl::OdrlPolicyInfo;
use common::grpc::{InvalidField, JsonStruct, JsonStructExt, ListParams, PageMeta, ProtoField};
use common::paginated_spec::Paginated;
use tonic::Status;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListPolicyTemplatesRequest> for ListParams<PolicyTemplateFilter> {
    type Error = Status;

    fn try_from(req: ListPolicyTemplatesRequest) -> Result<Self, Status> {
        let filter = PolicyTemplateFilter {
            tenant_id: None,
            id: req.id.non_empty().map(str::to_owned),
            version: req.version.non_empty().map(str::to_owned),
            author: req.author.non_empty().map(str::to_owned),
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<CreatePolicyTemplateRequest> for NewPolicyTemplateDto {
    type Error = Status;

    fn try_from(req: CreatePolicyTemplateRequest) -> Result<Self, Status> {
        let content = req
            .content
            .ok_or_else(|| InvalidField::status("content", "is required"))?
            .into_typed::<OdrlPolicyInfo>("content")?;
        Ok(Self {
            id: req.id,
            tenant_id: None,
            version: req.version,
            date: req
                .date
                .as_deref()
                .unwrap_or_default()
                .opt_rfc3339("date")?
                .map(Into::into),
            title: req
                .title
                .map(|v| v.into_typed::<LocalizedText>("title"))
                .transpose()?,
            description: req
                .description
                .map(|v| v.into_typed::<LocalizedText>("description"))
                .transpose()?,
            author: req.author,
            content,
            parameters: req
                .parameters
                .map(|s| s.into_typed("parameters"))
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

// Domain to Response ──────────────────────────────────────────────────────

/// Fallible because ODRL content and parameters are re-serialized into `Struct`.
impl TryFrom<PolicyTemplateDto> for PolicyTemplate {
    type Error = Status;

    fn try_from(dto: PolicyTemplateDto) -> Result<Self, Status> {
        Ok(Self {
            id: dto.id,
            version: dto.version,
            date: dto.date.to_rfc3339(),
            author: dto.author,
            title: dto
                .title
                .map(|t| JsonStruct::value_from_typed(&t))
                .transpose()?,
            description: dto
                .description
                .map(|t| JsonStruct::value_from_typed(&t))
                .transpose()?,
            content: Some(JsonStruct::from_typed(&dto.content)?),
            parameters: Some(JsonStruct::from_typed(&dto.parameters)?),
        })
    }
}

impl TryFrom<PolicyTemplateDto> for PolicyTemplateResponse {
    type Error = Status;

    fn try_from(dto: PolicyTemplateDto) -> Result<Self, Status> {
        Ok(Self {
            policy_template: Some(dto.try_into()?),
        })
    }
}

impl TryFrom<Paginated<PolicyTemplateDto>> for PolicyTemplateListResponse {
    type Error = Status;

    fn try_from(p: Paginated<PolicyTemplateDto>) -> Result<Self, Status> {
        let meta = PageMeta::from(&p);
        Ok(Self {
            items: p
                .items
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            next_cursor: meta.next_cursor,
            total: meta.total,
        })
    }
}

/// Batch / by-id results are a complete, unpaged set: no cursor, total = item count.
impl TryFrom<Vec<PolicyTemplateDto>> for PolicyTemplateListResponse {
    type Error = Status;

    fn try_from(dtos: Vec<PolicyTemplateDto>) -> Result<Self, Status> {
        Ok(Self {
            total: dtos.len() as u64,
            items: dtos
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            next_cursor: String::new(),
        })
    }
}
