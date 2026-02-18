use crate::entities::catalogs::{CatalogDto, EditCatalogDto, NewCatalogDto};
use crate::entities::data_services::{DataServiceDto, EditDataServiceDto, NewDataServiceDto};
use crate::entities::datasets::{DatasetDto, EditDatasetDto, NewDatasetDto};
use crate::entities::distributions::{DistributionDto, EditDistributionDto, NewDistributionDto};
use crate::entities::odrl_policies::{CatalogEntityTypes, NewOdrlPolicyDto, OdrlPolicyDto};
use crate::entities::policy_templates::types::LocalizedText;
use crate::entities::policy_templates::{NewPolicyTemplateDto, PolicyTemplateDto};
use crate::grpc::api::catalog_agent::{
    CatalogEntityType, CreateCatalogRequest, CreateDataServiceRequest, CreateDatasetRequest,
    CreateDistributionRequest, CreateOdrlPolicyRequest, CreatePolicyTemplateRequest, DataService,
    Dataset, Distribution, OdrlPolicy, PolicyTemplate, PutCatalogRequest, PutDataServiceRequest,
    PutDatasetRequest, PutDistributionRequest,
};
use prost_types::Struct;
use rainbow_common::dcat_formats::DctFormats;
use rainbow_common::dsp_common::odrl::OdrlPolicyInfo;
use std::str::FromStr;
use tonic::Status;
use urn::Urn;

fn proto_value_to_json(v: prost_types::Value) -> serde_json::Value {
    match v.kind {
        Some(prost_types::value::Kind::NullValue(_)) => serde_json::Value::Null,
        Some(prost_types::value::Kind::NumberValue(n)) => serde_json::Number::from_f64(n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Some(prost_types::value::Kind::StringValue(s)) => serde_json::Value::String(s),
        Some(prost_types::value::Kind::BoolValue(b)) => serde_json::Value::Bool(b),
        Some(prost_types::value::Kind::StructValue(s)) => proto_struct_to_json(s),
        Some(prost_types::value::Kind::ListValue(l)) => {
            let array = l.values.into_iter().map(proto_value_to_json).collect();
            serde_json::Value::Array(array)
        }
        None => serde_json::Value::Null,
    }
}

fn proto_struct_to_json(s: prost_types::Struct) -> serde_json::Value {
    let map = s.fields.into_iter().map(|(k, v)| (k, proto_value_to_json(v))).collect();
    serde_json::Value::Object(map)
}

fn json_to_proto_value(v: serde_json::Value) -> prost_types::Value {
    let kind = match v {
        serde_json::Value::Null => Some(prost_types::value::Kind::NullValue(0)),
        serde_json::Value::Bool(b) => Some(prost_types::value::Kind::BoolValue(b)),
        serde_json::Value::Number(n) => {
            // Proto usa f64 para números genéricos
            Some(prost_types::value::Kind::NumberValue(n.as_f64().unwrap_or(0.0)))
        }
        serde_json::Value::String(s) => Some(prost_types::value::Kind::StringValue(s)),
        serde_json::Value::Array(a) => {
            let values = a.into_iter().map(json_to_proto_value).collect();
            Some(prost_types::value::Kind::ListValue(prost_types::ListValue { values }))
        }
        serde_json::Value::Object(_) => {
            Some(prost_types::value::Kind::StructValue(json_to_proto_struct(v)))
        }
    };
    prost_types::Value { kind }
}

fn json_to_proto_struct(v: serde_json::Value) -> prost_types::Struct {
    match v {
        serde_json::Value::Object(map) => {
            let fields = map.into_iter().map(|(k, v)| (k, json_to_proto_value(v))).collect();
            prost_types::Struct { fields }
        }
        _ => prost_types::Struct::default(),
    }
}

fn localized_text_to_proto(txt: Option<LocalizedText>) -> Option<prost_types::Value> {
    txt.and_then(|t| serde_json::to_value(t).ok()).map(|json| json_to_proto_value(json))
}

fn proto_to_localized_text(val: Option<prost_types::Value>) -> Option<LocalizedText> {
    val.map(proto_value_to_json).and_then(|json| serde_json::from_value(json).ok())
}

impl From<CatalogDto> for crate::grpc::api::catalog_agent::Catalog {
    fn from(dto: CatalogDto) -> Self {
        let model = dto.inner;
        let dct_issued = model.dct_issued.to_rfc3339();
        let dct_modified = model.dct_modified.map(|d| d.to_rfc3339());

        Self {
            id: model.id,
            foaf_home_page: model.foaf_home_page,
            dct_conforms_to: model.dct_conforms_to,
            dct_creator: model.dct_creator,
            dct_identifier: model.dct_identifier,
            dct_issued,
            dct_modified,
            dct_title: model.dct_title,
            dspace_participant_id: model.dspace_participant_id,
            dspace_main_catalog: model.dspace_main_catalog,
        }
    }
}

impl TryFrom<CreateCatalogRequest> for NewCatalogDto {
    type Error = tonic::Status;

    fn try_from(req: CreateCatalogRequest) -> Result<Self, Self::Error> {
        let id = match req.id {
            Some(s) => Some(
                Urn::from_str(&s)
                    .map_err(|_| Status::invalid_argument("Invalid URN format for ID"))?,
            ),
            None => None,
        };

        Ok(Self {
            id,
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

impl From<DataServiceDto> for DataService {
    fn from(dto: DataServiceDto) -> Self {
        let model = dto.inner;
        let dct_issued = model.dct_issued.to_rfc3339();
        let dct_modified = model.dct_modified.map(|d| d.to_rfc3339());

        Self {
            id: model.id,
            dcat_endpoint_description: model.dcat_endpoint_description,
            dcat_endpoint_url: model.dcat_endpoint_url,
            dct_conforms_to: model.dct_conforms_to,
            dct_creator: model.dct_creator,
            dct_identifier: model.dct_identifier,
            dct_issued,
            dct_modified,
            dct_title: model.dct_title,
            dct_description: model.dct_description,
            catalog_id: model.catalog_id,
        }
    }
}

impl TryFrom<CreateDataServiceRequest> for NewDataServiceDto {
    type Error = Status;

    fn try_from(req: CreateDataServiceRequest) -> Result<Self, Self::Error> {
        let id = match req.id {
            Some(s) => Some(
                Urn::from_str(&s)
                    .map_err(|_| Status::invalid_argument("Invalid URN format for ID"))?,
            ),
            None => None,
        };

        let catalog_id = Urn::from_str(&req.catalog_id)
            .map_err(|_| Status::invalid_argument("Invalid URN format for Catalog ID"))?;

        Ok(Self {
            id,
            dcat_endpoint_description: req.dcat_endpoint_description,
            dcat_endpoint_url: req.dcat_endpoint_url,
            dct_conforms_to: req.dct_conforms_to,
            dct_creator: req.dct_creator,
            dct_title: req.dct_title,
            dct_description: req.dct_description,
            catalog_id,
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

impl From<DatasetDto> for Dataset {
    fn from(dto: DatasetDto) -> Self {
        let model = dto.inner;
        let dct_issued = model.dct_issued.to_rfc3339();
        let dct_modified = model.dct_modified.map(|d| d.to_rfc3339());

        Self {
            id: model.id,
            dct_conforms_to: model.dct_conforms_to,
            dct_creator: model.dct_creator,
            dct_identifier: model.dct_identifier,
            dct_issued,
            dct_modified,
            dct_title: model.dct_title,
            dct_description: model.dct_description,
            catalog_id: model.catalog_id,
        }
    }
}

impl TryFrom<CreateDatasetRequest> for NewDatasetDto {
    type Error = Status;

    fn try_from(req: CreateDatasetRequest) -> Result<Self, Self::Error> {
        let id = match req.id {
            Some(s) => Some(
                Urn::from_str(&s)
                    .map_err(|_| Status::invalid_argument("Invalid URN format for ID"))?,
            ),
            None => None,
        };

        let catalog_id = Urn::from_str(&req.catalog_id)
            .map_err(|_| Status::invalid_argument("Invalid URN format for Catalog ID"))?;

        Ok(Self {
            id,
            dct_conforms_to: req.dct_conforms_to,
            dct_creator: req.dct_creator,
            dct_title: req.dct_title,
            dct_description: req.dct_description,
            catalog_id,
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

impl From<DistributionDto> for Distribution {
    fn from(dto: DistributionDto) -> Self {
        let model = dto.inner;
        let dct_issued = model.dct_issued.to_rfc3339();
        let dct_modified = model.dct_modified.map(|d| d.to_rfc3339());

        // Serializamos el struct complejo a String JSON para devolverlo en el Proto
        let dct_format = model.dct_format; // Asumiendo que en DB ya es String o Option<String>

        Self {
            id: model.id,
            dct_issued,
            dct_modified,
            dct_title: model.dct_title,
            dct_description: model.dct_description,
            dcat_access_service: model.dcat_access_service,
            dataset_id: model.dataset_id,
            dct_format,
        }
    }
}

impl TryFrom<CreateDistributionRequest> for NewDistributionDto {
    type Error = Status;

    fn try_from(req: CreateDistributionRequest) -> Result<Self, Self::Error> {
        let id = match req.id {
            Some(s) => Some(
                Urn::from_str(&s)
                    .map_err(|_| Status::invalid_argument("Invalid URN format for ID"))?,
            ),
            None => None,
        };

        let dataset_id = Urn::from_str(&req.dataset_id)
            .map_err(|_| Status::invalid_argument("Invalid URN format for Dataset ID"))?;

        let dct_formats = req.dct_formats;

        Ok(Self {
            id,
            dct_title: req.dct_title,
            dct_description: req.dct_description,
            dct_formats: Some(dct_formats),
            dcat_access_service: req.dcat_access_service,
            dataset_id,
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

impl From<OdrlPolicyDto> for OdrlPolicy {
    fn from(dto: OdrlPolicyDto) -> Self {
        let model = dto.inner;

        let offer_val = json_to_proto_value(model.odrl_offer);
        let odrl_offer = match offer_val.kind {
            Some(prost_types::value::Kind::StructValue(s)) => Some(s),
            _ => None,
        };

        let inst_params = model.instantiation_parameters.map(|json| {
            let val = json_to_proto_value(json);
            match val.kind {
                Some(prost_types::value::Kind::StructValue(s)) => s,
                _ => Struct::default(),
            }
        });

        Self {
            id: model.id,
            entity_id: model.entity,
            entity_type: CatalogEntityType::from_str_name(&model.entity_type)
                .unwrap_or(CatalogEntityType::Unspecified)
                .into(),
            odrl_offer,
            created_at: model.created_at.to_rfc3339(),
            source_template_id: model.source_template_id,
            source_template_version: model.source_template_version,
            instantiation_parameters: inst_params,
        }
    }
}

impl TryFrom<CreateOdrlPolicyRequest> for NewOdrlPolicyDto {
    type Error = Status;

    fn try_from(req: CreateOdrlPolicyRequest) -> Result<Self, Self::Error> {
        let id = match &req.id {
            Some(s) => Some(
                Urn::from_str(&s)
                    .map_err(|_| Status::invalid_argument("Invalid URN format for ID"))?,
            ),
            None => None,
        };

        let entity_id = Urn::from_str(&req.entity_id)
            .map_err(|_| Status::invalid_argument("Invalid URN format for Entity ID"))?;

        let proto_struct = req
            .odrl_offer
            .clone()
            .ok_or_else(|| Status::invalid_argument("odrl_offer (Struct) is required"))?;
        if proto_struct.fields.is_empty() {
            return Err(Status::invalid_argument("odrl_offer cannot be empty"));
        }
        let json_value = proto_struct_to_json(proto_struct);
        let odrl_offer: OdrlPolicyInfo = serde_json::from_value(json_value).map_err(|e| {
            Status::invalid_argument(format!("Invalid OdrlPolicyInfo structure: {}", e))
        })?;

        let entity_type = match req.entity_type() {
            CatalogEntityType::Distribution => CatalogEntityTypes::Distribution,
            CatalogEntityType::DataService => CatalogEntityTypes::DataService,
            CatalogEntityType::Catalog => CatalogEntityTypes::Catalog,
            CatalogEntityType::Dataset => CatalogEntityTypes::Dataset,
            _ => return Err(Status::invalid_argument("Invalid or Unspecified Entity Type")),
        };

        Ok(Self {
            id,
            odrl_offer,
            entity_id,
            entity_type,
            source_template_id: None,
            source_template_version: None,
            instantiation_parameters: None,
        })
    }
}

impl From<PolicyTemplateDto> for PolicyTemplate {
    fn from(dto: PolicyTemplateDto) -> Self {
        let model = dto;

        let content_json = serde_json::to_value(model.content).unwrap_or(serde_json::Value::Null);
        let content_val = json_to_proto_value(content_json);
        let content = match content_val.kind {
            Some(prost_types::value::Kind::StructValue(s)) => s,
            _ => Struct::default(),
        };

        let parameters_json =
            serde_json::to_value(model.parameters).unwrap_or(serde_json::Value::Null);
        let parameters_val = json_to_proto_value(parameters_json);
        let parameters = match parameters_val.kind {
            Some(prost_types::value::Kind::StructValue(s)) => s,
            _ => Struct::default(),
        };

        Self {
            id: model.id,
            version: model.version,
            date: model.date.to_rfc3339(),
            author: model.author,
            title: localized_text_to_proto(model.title),
            description: localized_text_to_proto(model.description),
            content: Some(content),
            parameters: Some(parameters),
        }
    }
}

impl TryFrom<CreatePolicyTemplateRequest> for NewPolicyTemplateDto {
    type Error = Status;

    fn try_from(req: CreatePolicyTemplateRequest) -> Result<Self, Self::Error> {
        let content_struct =
            req.content.ok_or_else(|| Status::invalid_argument("content is required"))?;
        let content_prost_val = prost_types::Value {
            kind: Some(prost_types::value::Kind::StructValue(content_struct)),
        };
        let content_json = proto_value_to_json(content_prost_val);

        let content: OdrlPolicyInfo = serde_json::from_value(content_json)
            .map_err(|e| Status::invalid_argument(format!("Invalid ODRL content: {}", e)))?;

        let parameters = if let Some(params_struct) = req.parameters {
            let val = prost_types::Value {
                kind: Some(prost_types::value::Kind::StructValue(params_struct)),
            };
            let json = proto_value_to_json(val);
            serde_json::from_value(json).unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

        let date = if let Some(d) = req.date {
            Some(
                chrono::DateTime::parse_from_rfc3339(&d)
                    .map_err(|_| Status::invalid_argument("Invalid date format"))?
                    .into(),
            )
        } else {
            None
        };

        Ok(Self {
            id: req.id,
            version: req.version,
            date,
            author: req.author,
            title: proto_to_localized_text(req.title),
            description: proto_to_localized_text(req.description),
            content,
            parameters,
        })
    }
}
