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

//! Proto ⇄ domain mappers for the catalog RPCs.

use crate::entities::catalogs::{CatalogDto, EditCatalogDto, NewCatalogDto};
use crate::entities::filters::CatalogFilter;
use crate::grpc::api::catalog_agent::{
    Catalog, CatalogListResponse, CatalogResponse, CreateCatalogRequest, ListCatalogsRequest,
    PutCatalogRequest,
};
use common::grpc::{ListParams, PageMeta, ProtoField};
use common::paginated_spec::Paginated;
use tonic::Status;

// Request to Domain ───────────────────────────────────────────────────────

impl TryFrom<ListCatalogsRequest> for ListParams<CatalogFilter> {
    type Error = Status;

    fn try_from(req: ListCatalogsRequest) -> Result<Self, Status> {
        let filter = CatalogFilter {
            tenant_id: None,
            title: req.title.non_empty().map(str::to_owned),
            creator: req.creator.non_empty().map(str::to_owned),
            participant_id: req.participant_id.non_empty().map(str::to_owned),
            with_main_catalog: req.with_main_catalog,
            created_after: req.created_after.opt_rfc3339("created_after")?,
            created_before: req.created_before.opt_rfc3339("created_before")?,
        };
        Self::new(filter, req.limit, &req.cursor, &req.sort)
    }
}

impl TryFrom<CreateCatalogRequest> for NewCatalogDto {
    type Error = Status;

    fn try_from(req: CreateCatalogRequest) -> Result<Self, Status> {
        Ok(Self {
            id: req.id.as_deref().unwrap_or_default().opt_urn("id")?,
            tenant_id: None,
            foaf_home_page: req.foaf_home_page,
            dct_conforms_to: req.dct_conforms_to,
            dct_creator: req.dct_creator,
            dct_title: req.dct_title,
            dspace_participant_id: req.dspace_participant_id,
        })
    }
}

impl From<PutCatalogRequest> for EditCatalogDto {
    fn from(req: PutCatalogRequest) -> Self {
        Self {
            foaf_home_page: req.foaf_home_page,
            dct_conforms_to: req.dct_conforms_to,
            dct_creator: req.dct_creator,
            dct_title: req.dct_title,
        }
    }
}

// Domain to Response ──────────────────────────────────────────────────────

impl From<CatalogDto> for Catalog {
    fn from(dto: CatalogDto) -> Self {
        let model = dto.inner;
        Self {
            id: model.id,
            foaf_home_page: model.foaf_home_page,
            dct_conforms_to: model.dct_conforms_to,
            dct_creator: model.dct_creator,
            dct_identifier: model.dct_identifier,
            dct_issued: model.dct_issued.to_rfc3339(),
            dct_modified: model.dct_modified.map(|d| d.to_rfc3339()),
            dct_title: model.dct_title,
            dspace_participant_id: model.dspace_participant_id,
            dspace_main_catalog: model.dspace_main_catalog,
        }
    }
}

impl From<CatalogDto> for CatalogResponse {
    fn from(dto: CatalogDto) -> Self {
        Self {
            catalog: Some(dto.into()),
        }
    }
}

impl From<Paginated<CatalogDto>> for CatalogListResponse {
    fn from(p: Paginated<CatalogDto>) -> Self {
        let meta = PageMeta::from(&p);
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            next_cursor: meta.next_cursor,
            total: meta.total,
        }
    }
}

/// Batch results are a complete, unpaged set: no cursor, total = item count.
impl From<Vec<CatalogDto>> for CatalogListResponse {
    fn from(dtos: Vec<CatalogDto>) -> Self {
        Self {
            total: dtos.len() as u64,
            items: dtos.into_iter().map(Into::into).collect(),
            next_cursor: String::new(),
        }
    }
}
