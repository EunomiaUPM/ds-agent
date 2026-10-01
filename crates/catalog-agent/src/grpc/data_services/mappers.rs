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

//! Proto ⇄ domain mappers for the data-service RPCs.

use crate::entities::data_services::{DataServiceDto, EditDataServiceDto, NewDataServiceDto};
use crate::entities::filters::DataServiceFilter;
use crate::grpc::api::catalog_agent::{
    CreateDataServiceRequest, DataService, DataServiceListResponse, DataServiceResponse,
    ListDataServicesRequest, PutDataServiceRequest,
};
use common::grpc::{ListParams, PageMeta, ProtoField};
use common::paginated_spec::Paginated;
use tonic::Status;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListDataServicesRequest> for ListParams<DataServiceFilter> {
    type Error = Status;

    fn try_from(req: ListDataServicesRequest) -> Result<Self, Status> {
        let filter = DataServiceFilter {
            tenant_id: None,
            catalog_id: req.catalog_id.non_empty().map(str::to_owned),
            endpoint_url: req.endpoint_url.non_empty().map(str::to_owned),
            title: req.title.non_empty().map(str::to_owned),
            creator: req.creator.non_empty().map(str::to_owned),
            main_data_service: req.main_data_service,
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<CreateDataServiceRequest> for NewDataServiceDto {
    type Error = Status;

    fn try_from(req: CreateDataServiceRequest) -> Result<Self, Status> {
        Ok(Self {
            id: req.id.as_deref().unwrap_or_default().opt_urn("id")?,
            tenant_id: None,
            dcat_endpoint_description: req.dcat_endpoint_description,
            dcat_endpoint_url: req.dcat_endpoint_url,
            dct_conforms_to: req.dct_conforms_to,
            dct_creator: req.dct_creator,
            dct_title: req.dct_title,
            dct_description: req.dct_description,
            catalog_id: req.catalog_id.urn("catalog_id")?,
        })
    }
}

impl From<PutDataServiceRequest> for EditDataServiceDto {
    fn from(req: PutDataServiceRequest) -> Self {
        Self {
            dcat_endpoint_description: req.dcat_endpoint_description,
            dcat_endpoint_url: req.dcat_endpoint_url,
            dct_conforms_to: req.dct_conforms_to,
            dct_creator: req.dct_creator,
            dct_title: req.dct_title,
            dct_description: req.dct_description,
        }
    }
}

// Domain to Response ──────────────────────────────────────────────────────

impl From<DataServiceDto> for DataService {
    fn from(dto: DataServiceDto) -> Self {
        let model = dto.inner;
        Self {
            id: model.id,
            dcat_endpoint_description: model.dcat_endpoint_description,
            dcat_endpoint_url: model.dcat_endpoint_url,
            dct_conforms_to: model.dct_conforms_to,
            dct_creator: model.dct_creator,
            dct_identifier: model.dct_identifier,
            dct_issued: model.dct_issued.to_rfc3339(),
            dct_modified: model.dct_modified.map(|d| d.to_rfc3339()),
            dct_title: model.dct_title,
            dct_description: model.dct_description,
            catalog_id: model.catalog_id,
            dspace_main_data_service: model.dspace_main_data_service,
        }
    }
}

impl From<DataServiceDto> for DataServiceResponse {
    fn from(dto: DataServiceDto) -> Self {
        Self {
            data_service: Some(dto.into()),
        }
    }
}

impl From<Paginated<DataServiceDto>> for DataServiceListResponse {
    fn from(p: Paginated<DataServiceDto>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}

/// Batch / by-parent results are a complete, unpaged set: no cursor, total = item count.
impl From<Vec<DataServiceDto>> for DataServiceListResponse {
    fn from(dtos: Vec<DataServiceDto>) -> Self {
        Self {
            total: dtos.len() as u64,
            items: dtos.into_iter().map(Into::into).collect(),
            next_cursor: String::new(),
        }
    }
}
