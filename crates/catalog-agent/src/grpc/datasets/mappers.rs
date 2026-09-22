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

//! Proto ⇄ domain mappers for the dataset RPCs.

use crate::entities::datasets::{DatasetDto, EditDatasetDto, NewDatasetDto};
use crate::entities::filters::DatasetFilter;
use crate::grpc::api::catalog_agent::{
    CreateDatasetRequest, Dataset, DatasetListResponse, DatasetResponse, ListDatasetsRequest,
    PutDatasetRequest,
};
use common::grpc::{ListParams, PageMeta, ProtoField};
use common::paginated_spec::Paginated;
use tonic::Status;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListDatasetsRequest> for ListParams<DatasetFilter> {
    type Error = Status;

    fn try_from(req: ListDatasetsRequest) -> Result<Self, Status> {
        let filter = DatasetFilter {
            tenant_id: None,
            catalog_id: req.catalog_id.non_empty().map(str::to_owned),
            title: req.title.non_empty().map(str::to_owned),
            creator: req.creator.non_empty().map(str::to_owned),
            conforms_to: req.conforms_to.non_empty().map(str::to_owned),
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<CreateDatasetRequest> for NewDatasetDto {
    type Error = Status;

    fn try_from(req: CreateDatasetRequest) -> Result<Self, Status> {
        Ok(Self {
            id: req.id.as_deref().unwrap_or_default().opt_urn("id")?,
            tenant_id: None,
            dct_conforms_to: req.dct_conforms_to,
            dct_creator: req.dct_creator,
            dct_title: req.dct_title,
            dct_description: req.dct_description,
            catalog_id: req.catalog_id.urn("catalog_id")?,
        })
    }
}

impl From<PutDatasetRequest> for EditDatasetDto {
    fn from(req: PutDatasetRequest) -> Self {
        Self {
            dct_conforms_to: req.dct_conforms_to,
            dct_creator: req.dct_creator,
            dct_title: req.dct_title,
            dct_description: req.dct_description,
        }
    }
}

// Domain to Response ──────────────────────────────────────────────────────

impl From<DatasetDto> for Dataset {
    fn from(dto: DatasetDto) -> Self {
        let model = dto.inner;
        Self {
            id: model.id,
            dct_conforms_to: model.dct_conforms_to,
            dct_creator: model.dct_creator,
            dct_identifier: model.dct_identifier,
            dct_issued: model.dct_issued.to_rfc3339(),
            dct_modified: model.dct_modified.map(|d| d.to_rfc3339()),
            dct_title: model.dct_title,
            dct_description: model.dct_description,
            catalog_id: model.catalog_id,
        }
    }
}

impl From<DatasetDto> for DatasetResponse {
    fn from(dto: DatasetDto) -> Self {
        Self {
            dataset: Some(dto.into()),
        }
    }
}

impl From<Paginated<DatasetDto>> for DatasetListResponse {
    fn from(p: Paginated<DatasetDto>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}

/// Batch / by-parent results are a complete, unpaged set: no cursor, total = item count.
impl From<Vec<DatasetDto>> for DatasetListResponse {
    fn from(dtos: Vec<DatasetDto>) -> Self {
        Self {
            total: dtos.len() as u64,
            items: dtos.into_iter().map(Into::into).collect(),
            next_cursor: String::new(),
        }
    }
}
