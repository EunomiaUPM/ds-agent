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

//! Placeholders found in single fields: strings, string lists and string maps.

use super::*;

/// A string that is only a placeholder yields it as a String parameter.
#[test]
fn complete_value_extractor_on_string() {
    let mut extractor = TemplateParametersExtractor::new();
    let mut field: String = "{{__TEST__}}".to_string();
    extractor.on_string(&mut field).unwrap();
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(found_parameters[0].content_type, FoundParameterType::String);
}

/// A placeholder inside a longer string is found too.
#[test]
fn partial_value_extractor_on_string() {
    let mut extractor = TemplateParametersExtractor::new();
    let mut field: String = "http://mi-api.com/{{__TEST__}}".to_string();
    extractor.on_string(&mut field).unwrap();
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(found_parameters[0].content_type, FoundParameterType::String);
}

/// A string without placeholders yields nothing.
#[test]
fn nothing_to_extract_on_string() {
    let mut extractor = TemplateParametersExtractor::new();
    let mut field: String = "http://mi-api.com/no-test".to_string();
    extractor.on_string(&mut field).unwrap();
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 0);
}

/// A templated string list yields a VecString parameter.
#[test]
fn vec_string_parameter_extractor_as_complete() {
    let mut extractor = TemplateParametersExtractor::new();
    let mut field = TemplateVecString::Template("{{__TEST__}}".to_string());
    extractor.on_vec_string(&mut field).unwrap();
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(
        found_parameters[0].content_type,
        FoundParameterType::VecString
    );
}

/// A placeholder inside one item of a string list is found.
#[test]
fn vec_string_parameter_extractor_as_partial() {
    let mut extractor = TemplateParametersExtractor::new();
    let mut field = TemplateVecString::Value(vec!["test".to_string(), "{{__TEST__}}".to_string()]);
    extractor.on_vec_string(&mut field).unwrap();
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(
        found_parameters[0].content_type,
        FoundParameterType::VecString
    );
}

/// A literal string list yields nothing.
#[test]
fn nothing_to_extract_on_vec_string() {
    let mut extractor = TemplateParametersExtractor::new();
    let mut field = TemplateVecString::Value(vec!["test".to_string(), "test".to_string()]);
    extractor.on_vec_string(&mut field).unwrap();
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 0);
}

/// A templated string map yields a MapString parameter.
#[test]
fn map_string_parameter_extractor_as_complete() {
    let mut extractor = TemplateParametersExtractor::new();
    let mut field = TemplateMapString::Template("{{__TEST__}}".to_string());
    extractor.on_map_string(&mut field).unwrap();
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(
        found_parameters[0].content_type,
        FoundParameterType::MapString
    );
}

/// A placeholder in the `__EXTRA__` value of a map is found.
#[test]
fn map_string_parameter_extractor_as_partial() {
    let mut extractor = TemplateParametersExtractor::new();
    let mut field = TemplateMapString::Value(HashMap::from([(
        "__EXTRA__".to_string(),
        "{{__TEST__}}".to_string(),
    )]));
    extractor.on_map_string(&mut field).unwrap();
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(
        found_parameters[0].content_type,
        FoundParameterType::MapString
    );
}

/// A literal string map yields nothing.
#[test]
fn nothing_to_extract_on_map_string() {
    let mut extractor = TemplateParametersExtractor::new();
    let mut field =
        TemplateMapString::Value(HashMap::from([("test".to_string(), "test".to_string())]));
    extractor.on_map_string(&mut field).unwrap();
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 0);
}
