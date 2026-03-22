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

use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::parameters::parameters::{TemplateMapString, TemplateVecString};
use crate::entities::parameters::template_parameters_extractor::ParameterExtractorBehavior;
use crate::entities::parameters::template_walker::ConnectorTemplateWalker;
use crate::entities::parameters::TemplateField;
use ymir::errors::Outcome;

/// Walks a [`ConnectorTemplateDto`] and feeds every templatable field to a
/// [`ParameterExtractorBehavior`].
///
/// The visitor implements a read-only tree traversal using the shared
/// [`ConnectorTemplateWalker`] structural routing: it visits the
/// `authentication` section first, then the `interaction` section.
///
/// # API
///
/// Construct with the behavior that should receive each discovered field,
/// then call [`extract`](ParameterExtractorVisitor::extract) with a mutable
/// reference to the template.  The template is not modified.
///
/// ```ignore
/// let mut extractor = TemplateParameterExtractor::new();
/// ParameterExtractorVisitor::new(&mut extractor).extract(&mut template);
/// let params = extractor.found_parameters();
/// ```
///
/// # Current limitations
/// The `BearerToken` authentication arm is **not yet walked** — that field is
/// intentionally opaque (`SecretString`) and never parameterised via placeholders.
pub struct ParameterExtractorVisitor<'a> {
    extractor: &'a mut dyn ParameterExtractorBehavior,
}

impl<'a> ParameterExtractorVisitor<'a> {
    pub fn new(extractor: &'a mut dyn ParameterExtractorBehavior) -> Self {
        Self { extractor }
    }

    /// Walks `template` and feeds every discovered field token to the
    /// extractor supplied at construction time.
    ///
    /// The template is not modified; `&mut` is required only to satisfy the
    /// shared [`ConnectorTemplateWalker`] interface.
    pub fn extract(&mut self, template: &mut ConnectorTemplateDto) {
        let _ = self.walk(template); // Infallible — always succeeds
    }
}

impl ConnectorTemplateWalker for ParameterExtractorVisitor<'_> {
    fn on_string(&mut self, field: &mut String) -> Outcome<()> {
        self.extractor.extract(TemplateField::TemplateString(field));
        Ok(())
    }

    fn on_vec_string(&mut self, field: &mut TemplateVecString) -> Outcome<()> {
        self.extractor
            .extract(TemplateField::TemplateVecString(field));
        Ok(())
    }

    fn on_map_string(&mut self, field: &mut TemplateMapString) -> Outcome<()> {
        self.extractor
            .extract(TemplateField::TemplateMapString(field));
        Ok(())
    }
}

#[cfg(test)]
mod tests;
