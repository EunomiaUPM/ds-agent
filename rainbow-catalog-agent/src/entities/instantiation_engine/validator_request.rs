use crate::entities::instantiation_engine::validators::ValidatorFactory;
use crate::entities::instantiation_engine::NewPolicyInstantiationDto;
use crate::entities::policy_templates::PolicyTemplateDto;
use rainbow_common::errors::CommonErrors;
use tracing::error;
use ymir::errors::{Errors, Outcome};

impl NewPolicyInstantiationDto {
    pub(crate) fn validate_instantiation_request(
        &self,
        policy_template: &PolicyTemplateDto,
    ) -> Outcome<()> {
        for req_key in self.parameters.keys() {
            if !policy_template.parameters.contains_key(req_key) {
                let err = Errors::parse(&format!(
                    "Validation Error: Unknown parameter '{}' provided. It is not defined in the template.",
                    req_key
                ), None);
                return Err(err);
            }
        }

        for (param_key, param_def) in &policy_template.parameters {
            let value_to_validate = match self.parameters.get(param_key) {
                Some(val) => val,
                None => {
                    if let Some(default_val) = &param_def.default_value {
                        default_val
                    } else {
                        let err = Errors::parse(&format!(
                            "Validation Error: Missing required parameter '{}'",
                            param_key
                        ), None);
                        return Err(err);
                    }
                }
            };

            let validator = ValidatorFactory::get_validator(param_def.data_type);
            validator.validate(value_to_validate, &param_def.restrictions).map_err(|e| {
                Errors::parse(&format!(
                    "Validation Error for '{}': {:?}",
                    param_key, e
                ), None)
            })?;
        }

        Ok(())
    }
}
