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
