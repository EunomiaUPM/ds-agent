
use crate::entities::parameters::parameters::{TemplateBoolean, TemplateInt, TemplateMapString};
use crate::entities::parameters::template_parameters_extractor::{
    ParameterExtractorBehavior, TemplateParameterExtractor,
};
use crate::entities::parameters::{FoundParameterType, TemplateField};
use crate::TemplateVecString;
use std::collections::HashMap;

#[test]
fn complete_value_extractor_on_int() {
    let mut extractor = TemplateParameterExtractor::new();
    let field = TemplateInt::Template("{{__TIMEOUT__}}".to_string());
    extractor.extract(TemplateField::TemplateInt(&field));
    let found = extractor.found_parameters();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "TIMEOUT");
    assert_eq!(found[0].content_type, FoundParameterType::Int);
}
#[test]
fn complete_value_extractor_on_boolean() {
    let mut extractor = TemplateParameterExtractor::new();
    let field = TemplateBoolean::Template("{{__TEST__}}".to_string());
    extractor.extract(TemplateField::TemplateBoolean(&field));
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(found_parameters[0].content_type, FoundParameterType::Boolean);
}
#[test]
fn complete_value_extractor_on_string() {
    let mut extractor = TemplateParameterExtractor::new();
    let field: String = "{{__TEST__}}".to_string();
    extractor.extract(TemplateField::TemplateString(&field));
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(found_parameters[0].content_type, FoundParameterType::String);
}
#[test]
fn partial_value_extractor_on_string() {
    let mut extractor = TemplateParameterExtractor::new();
    let field: String = "http://mi-api.com/{{__TEST__}}".to_string();
    extractor.extract(TemplateField::TemplateString(&field));
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(found_parameters[0].content_type, FoundParameterType::String);
}
#[test]
fn nothing_to_extract_on_string() {
    let mut extractor = TemplateParameterExtractor::new();
    let field: String = "http://mi-api.com/no-test".to_string();
    extractor.extract(TemplateField::TemplateString(&field));
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 0);
}
#[test]
fn vec_string_parameter_extractor_as_complete() {
    let mut extractor = TemplateParameterExtractor::new();
    let field = TemplateVecString::Template("{{__TEST__}}".to_string());
    extractor.extract(TemplateField::TemplateVecString(&field));
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(found_parameters[0].content_type, FoundParameterType::VecString);
}
#[test]
fn vec_string_parameter_extractor_as_partial() {
    let mut extractor = TemplateParameterExtractor::new();
    let field = TemplateVecString::Value(vec!["test".to_string(), "{{__TEST__}}".to_string()]);
    extractor.extract(TemplateField::TemplateVecString(&field));
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(found_parameters[0].content_type, FoundParameterType::VecString);
}
#[test]
fn nothing_to_extract_on_vec_string() {
    let mut extractor = TemplateParameterExtractor::new();
    let field = TemplateVecString::Value(vec!["test".to_string(), "test".to_string()]);
    extractor.extract(TemplateField::TemplateVecString(&field));
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 0);
}
#[test]
fn map_string_parameter_extractor_as_complete() {
    let mut extractor = TemplateParameterExtractor::new();
    let field = TemplateMapString::Template("{{__TEST__}}".to_string());
    extractor.extract(TemplateField::TemplateMapString(&field));
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(found_parameters[0].content_type, FoundParameterType::MapString);
}
#[test]
fn map_string_parameter_extractor_as_partial() {
    let mut extractor = TemplateParameterExtractor::new();
    let field = TemplateMapString::Value(HashMap::from([(
        "__EXTRA__".to_string(),
        "{{__TEST__}}".to_string(),
    )]));
    extractor.extract(TemplateField::TemplateMapString(&field));
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 1);
    assert_eq!(found_parameters[0].name, "TEST");
    assert_eq!(found_parameters[0].content_type, FoundParameterType::MapString);
}
#[test]
fn nothing_to_extract_on_map_string() {
    let mut extractor = TemplateParameterExtractor::new();
    let field = TemplateMapString::Value(HashMap::from([("test".to_string(), "test".to_string())]));
    extractor.extract(TemplateField::TemplateMapString(&field));
    let found_parameters = extractor.found_parameters();
    assert_eq!(found_parameters.len(), 0);
}
