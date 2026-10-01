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

//! Proto ⇄ domain mappers for the distribution RPCs.

use crate::entities::distributions::{DistributionDto, EditDistributionDto, NewDistributionDto};
use crate::entities::filters::DistributionFilter;
use crate::grpc::api::catalog_agent::{
    CreateDistributionRequest, Distribution, DistributionListResponse, DistributionResponse,
    ListDistributionsRequest, PutDistributionRequest,
};
use common::grpc::{ListParams, PageMeta, ProtoField};
use common::paginated_spec::Paginated;
use tonic::Status;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListDistributionsRequest> for ListParams<DistributionFilter> {
    type Error = Status;

    fn try_from(req: ListDistributionsRequest) -> Result<Self, Status> {
        let filter = DistributionFilter {
            tenant_id: None,
            dataset_id: req.dataset_id.non_empty().map(str::to_owned),
            access_service: req.access_service.non_empty().map(str::to_owned),
            format: req.format.non_empty().map(str::to_owned),
            title: req.title.non_empty().map(str::to_owned),
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<CreateDistributionRequest> for NewDistributionDto {
    type Error = Status;

    fn try_from(req: CreateDistributionRequest) -> Result<Self, Status> {
        Ok(Self {
            id: req.id.as_deref().unwrap_or_default().opt_urn("id")?,
            tenant_id: None,
            dct_title: req.dct_title,
            dct_description: req.dct_description,
            dct_formats: req.dct_formats.non_empty().map(str::to_owned),
            dcat_access_service: req.dcat_access_service,
            dataset_id: req.dataset_id.urn("dataset_id")?,
        })
    }
}

impl From<PutDistributionRequest> for EditDistributionDto {
    fn from(req: PutDistributionRequest) -> Self {
        Self {
            dct_title: req.dct_title,
            dct_description: req.dct_description,
            dcat_access_service: req.dcat_access_service,
        }
    }
}

// Domain to Response ──────────────────────────────────────────────────────

impl From<DistributionDto> for Distribution {
    fn from(dto: DistributionDto) -> Self {
        let model = dto.inner;
        Self {
            id: model.id,
            dct_issued: model.dct_issued.to_rfc3339(),
            dct_modified: model.dct_modified.map(|d| d.to_rfc3339()),
            dct_title: model.dct_title,
            dct_description: model.dct_description,
            dcat_access_service: model.dcat_access_service,
            dataset_id: model.dataset_id,
            dct_format: model.dct_format,
        }
    }
}

impl From<DistributionDto> for DistributionResponse {
    fn from(dto: DistributionDto) -> Self {
        Self {
            distribution: Some(dto.into()),
        }
    }
}

impl From<Paginated<DistributionDto>> for DistributionListResponse {
    fn from(p: Paginated<DistributionDto>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}

/// Batch / by-parent results are a complete, unpaged set: no cursor, total = item count.
impl From<Vec<DistributionDto>> for DistributionListResponse {
    fn from(dtos: Vec<DistributionDto>) -> Self {
        Self {
            total: dtos.len() as u64,
            items: dtos.into_iter().map(Into::into).collect(),
            next_cursor: String::new(),
        }
    }
}
