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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::entities::parameters::parameters::{SysParameterType, TemplateMapString};
use crate::entities::parameters::system_parameter_extractor::SystemParameterExtractor;
use crate::entities::parameters::template_parameters_extractor::ParameterExtractorBehavior;
use crate::entities::parameters::TemplateField;
use crate::TemplateVecString;
use std::collections::HashMap;

fn extract(field: TemplateField) -> Vec<(String, SysParameterType)> {
    let mut extractor = SystemParameterExtractor::new();
    extractor.extract(field);
    extractor
        .found_sys_parameters()
        .iter()
        .map(|p| (p.name.clone(), p.content_type.clone()))
        .collect()
}

// =========================================================================
// TemplateString
// =========================================================================

#[test]
fn extracts_sys_urn_interpolated_in_string() {
    let field = "https://example.com/{{__SYS_URN__}}".to_string();
    let found = extract(TemplateField::TemplateString(&field));
    assert_eq!(1, found.len());
    assert_eq!("SYS_URN", found[0].0);
    assert!(matches!(found[0].1, SysParameterType::SysUrn));
}

#[test]
fn extracts_sys_token_as_complete_string() {
    let field = "{{__SYS_TOKEN__}}".to_string();
    let found = extract(TemplateField::TemplateString(&field));
    assert_eq!(1, found.len());
    assert!(matches!(found[0].1, SysParameterType::SysToken));
}

#[test]
fn extracts_sys_iso8601_interpolated_in_string() {
    let field = "date={{__SYS_ISO8601__}}&other=value".to_string();
    let found = extract(TemplateField::TemplateString(&field));
    assert_eq!(1, found.len());
    assert!(matches!(found[0].1, SysParameterType::SysIso8601));
}

#[test]
fn extracts_sys_own_url_non_docker() {
    let field = "{{__SYS_OWN_URL__}}".to_string();
    let found = extract(TemplateField::TemplateString(&field));
    assert_eq!(1, found.len());
    assert!(matches!(
        found[0].1,
        SysParameterType::SysOwnUrl {
            host_docker_internal: false
        }
    ));
}

#[test]
fn extracts_sys_own_url_docker() {
    let field = "{{__SYS_OWN_URL_DOCKER__}}".to_string();
    let found = extract(TemplateField::TemplateString(&field));
    assert_eq!(1, found.len());
    assert!(matches!(
        found[0].1,
        SysParameterType::SysOwnUrl {
            host_docker_internal: true
        }
    ));
}

#[test]
fn ignores_non_sys_placeholders_in_string() {
    let field = "https://{{__HOST__}}/{{__SYS_URN__}}".to_string();
    let found = extract(TemplateField::TemplateString(&field));
    assert_eq!(1, found.len());
    assert_eq!("SYS_URN", found[0].0);
}

#[test]
fn ignores_unknown_sys_placeholder_in_string() {
    let field = "{{__SYS_UNKNOWN__}}".to_string();
    let found = extract(TemplateField::TemplateString(&field));
    assert!(found.is_empty());
}

#[test]
fn literal_string_extracts_nothing() {
    let field = "https://api.example.com/data".to_string();
    let found = extract(TemplateField::TemplateString(&field));
    assert!(found.is_empty());
}

// =========================================================================
// TemplateVecString — each element is a string, scanned individually
// =========================================================================

#[test]
fn extracts_sys_urn_from_vec_string_value_item() {
    let field = TemplateVecString::Value(vec!["static".to_string(), "{{__SYS_URN__}}".to_string()]);
    let found = extract(TemplateField::TemplateVecString(&field));
    assert_eq!(1, found.len());
    assert!(matches!(found[0].1, SysParameterType::SysUrn));
}

#[test]
fn extracts_sys_iso8601_interpolated_in_vec_item() {
    let field = TemplateVecString::Value(vec!["since={{__SYS_ISO8601__}}".to_string()]);
    let found = extract(TemplateField::TemplateVecString(&field));
    assert_eq!(1, found.len());
    assert!(matches!(found[0].1, SysParameterType::SysIso8601));
}

#[test]
fn extracts_sys_token_from_vec_string_template() {
    let field = TemplateVecString::Template("{{__SYS_TOKEN__}}".to_string());
    let found = extract(TemplateField::TemplateVecString(&field));
    assert_eq!(1, found.len());
    assert!(matches!(found[0].1, SysParameterType::SysToken));
}

#[test]
fn vec_string_with_no_sys_placeholders_extracts_nothing() {
    let field = TemplateVecString::Value(vec!["GET".to_string(), "POST".to_string()]);
    let found = extract(TemplateField::TemplateVecString(&field));
    assert!(found.is_empty());
}

// =========================================================================
// TemplateMapString — only __EXTRA__ value is dynamic
// =========================================================================

#[test]
fn extracts_sys_token_from_map_extra_key() {
    let field = TemplateMapString::Value(HashMap::from([(
        "__EXTRA__".to_string(),
        "Bearer {{__SYS_TOKEN__}}".to_string(),
    )]));
    let found = extract(TemplateField::TemplateMapString(&field));
    assert_eq!(1, found.len());
    assert!(matches!(found[0].1, SysParameterType::SysToken));
}

#[test]
fn extracts_sys_own_url_docker_from_map_extra_key() {
    let field = TemplateMapString::Value(HashMap::from([(
        "__EXTRA__".to_string(),
        "{{__SYS_OWN_URL_DOCKER__}}/webhook".to_string(),
    )]));
    let found = extract(TemplateField::TemplateMapString(&field));
    assert_eq!(1, found.len());
    assert!(matches!(
        found[0].1,
        SysParameterType::SysOwnUrl {
            host_docker_internal: true
        }
    ));
}

#[test]
fn map_without_extra_key_extracts_nothing() {
    let field = TemplateMapString::Value(HashMap::from([(
        "Authorization".to_string(),
        "Bearer {{__SYS_TOKEN__}}".to_string(),
    )]));
    let found = extract(TemplateField::TemplateMapString(&field));
    assert!(found.is_empty());
}

#[test]
fn extracts_sys_token_from_map_template() {
    let field = TemplateMapString::Template("{{__SYS_TOKEN__}}".to_string());
    let found = extract(TemplateField::TemplateMapString(&field));
    assert_eq!(1, found.len());
    assert!(matches!(found[0].1, SysParameterType::SysToken));
}
