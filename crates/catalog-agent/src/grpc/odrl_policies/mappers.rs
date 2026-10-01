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

//! Proto ⇄ domain mappers for the ODRL policy RPCs.

use crate::entities::filters::OdrlPolicyFilter;
use crate::entities::odrl_policies::{CatalogEntityTypes, NewOdrlPolicyDto, OdrlPolicyDto};
use crate::grpc::api::catalog_agent::{
    CatalogEntityType, CreateOdrlPolicyRequest, ListOdrlPoliciesRequest, OdrlPolicy,
    OdrlPolicyListResponse, OdrlPolicyResponse,
};
use common::dsp_common::odrl::OdrlPolicyInfo;
use common::grpc::{
    InvalidField, JsonStructExt, JsonValueExt, ListParams, PageMeta, ProtoEnum, ProtoField,
};
use common::paginated_spec::Paginated;
use tonic::Status;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListOdrlPoliciesRequest> for ListParams<OdrlPolicyFilter> {
    type Error = Status;

    fn try_from(req: ListOdrlPoliciesRequest) -> Result<Self, Status> {
        let entity_type = match req
            .entity_type
            .proto_enum::<CatalogEntityType>("entity_type")?
        {
            CatalogEntityType::Unspecified => None,
            other => Some(CatalogEntityTypes::try_from(other)?.to_string()),
        };
        let filter = OdrlPolicyFilter {
            tenant_id: None,
            entity: req.entity_id.non_empty().map(str::to_owned),
            entity_type,
            source_template_id: req.source_template_id.non_empty().map(str::to_owned),
            source_template_version: req.source_template_version.non_empty().map(str::to_owned),
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<CreateOdrlPolicyRequest> for NewOdrlPolicyDto {
    type Error = Status;

    fn try_from(req: CreateOdrlPolicyRequest) -> Result<Self, Status> {
        let offer = req
            .odrl_offer
            .ok_or_else(|| InvalidField::status("odrl_offer", "is required"))?;
        if offer.fields.is_empty() {
            return Err(InvalidField::status("odrl_offer", "cannot be empty"));
        }
        let entity_type = req
            .entity_type
            .proto_enum::<CatalogEntityType>("entity_type")?;
        Ok(Self {
            id: req.id.as_deref().unwrap_or_default().opt_urn("id")?,
            tenant_id: None,
            odrl_offer: offer.into_typed::<OdrlPolicyInfo>("odrl_offer")?,
            entity_id: req.entity_id.urn("entity_id")?,
            entity_type: CatalogEntityTypes::try_from(entity_type)?,
            source_template_id: req.source_template_id,
            source_template_version: req.source_template_version,
            instantiation_parameters: req.instantiation_parameters.map(JsonStructExt::into_json),
            description: req.description,
        })
    }
}

// Domain to Response ──────────────────────────────────────────────────────

/// An entity type stored with an unknown label is reported as `UNSPECIFIED`.
impl From<OdrlPolicyDto> for OdrlPolicy {
    fn from(dto: OdrlPolicyDto) -> Self {
        let model = dto.inner;
        let entity_type = model
            .entity_type
            .parse::<CatalogEntityTypes>()
            .map(CatalogEntityType::from)
            .unwrap_or(CatalogEntityType::Unspecified);
        Self {
            id: model.id,
            odrl_offer: Some(model.odrl_offer.into_prost_struct()),
            entity_id: model.entity,
            entity_type: entity_type as i32,
            created_at: model.created_at.to_rfc3339(),
            source_template_id: model.source_template_id,
            source_template_version: model.source_template_version,
            instantiation_parameters: model
                .instantiation_parameters
                .map(JsonValueExt::into_prost_struct),
            description: model.description,
        }
    }
}

impl From<OdrlPolicyDto> for OdrlPolicyResponse {
    fn from(dto: OdrlPolicyDto) -> Self {
        Self {
            policy: Some(dto.into()),
        }
    }
}

impl From<Paginated<OdrlPolicyDto>> for OdrlPolicyListResponse {
    fn from(p: Paginated<OdrlPolicyDto>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}

/// Batch / by-entity results are a complete, unpaged set: no cursor, total = item count.
impl From<Vec<OdrlPolicyDto>> for OdrlPolicyListResponse {
    fn from(dtos: Vec<OdrlPolicyDto>) -> Self {
        Self {
            total: dtos.len() as u64,
            items: dtos.into_iter().map(Into::into).collect(),
            next_cursor: String::new(),
        }
    }
}

// Proto enums ⇄ domain enums ──────────────────────────────────────────────

impl TryFrom<CatalogEntityType> for CatalogEntityTypes {
    type Error = Status;

    fn try_from(value: CatalogEntityType) -> Result<Self, Status> {
        match value {
            CatalogEntityType::Distribution => Ok(CatalogEntityTypes::Distribution),
            CatalogEntityType::DataService => Ok(CatalogEntityTypes::DataService),
            CatalogEntityType::Catalog => Ok(CatalogEntityTypes::Catalog),
            CatalogEntityType::Dataset => Ok(CatalogEntityTypes::Dataset),
            CatalogEntityType::Unspecified => {
                Err(InvalidField::status("entity_type", "must be specified"))
            }
        }
    }
}

impl From<CatalogEntityTypes> for CatalogEntityType {
    fn from(value: CatalogEntityTypes) -> Self {
        match value {
            CatalogEntityTypes::Distribution => CatalogEntityType::Distribution,
            CatalogEntityTypes::DataService => CatalogEntityType::DataService,
            CatalogEntityTypes::Catalog => CatalogEntityType::Catalog,
            CatalogEntityTypes::Dataset => CatalogEntityType::Dataset,
        }
    }
}
