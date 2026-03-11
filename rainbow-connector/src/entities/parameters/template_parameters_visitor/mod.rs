use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::parameters::parameters::{TemplateMapString, TemplateVecString};
use crate::entities::parameters::template_parameters_extractor::ParameterExtractorBehavior;
use crate::entities::parameters::template_walker::ConnectorTemplateWalker;
use crate::entities::parameters::TemplateField;
use std::convert::Infallible;

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
    type Error = Infallible;

    fn on_string(&mut self, field: &mut String) -> Result<(), Infallible> {
        self.extractor.extract(TemplateField::TemplateString(field));
        Ok(())
    }

    fn on_vec_string(&mut self, field: &mut TemplateVecString) -> Result<(), Infallible> {
        self.extractor.extract(TemplateField::TemplateVecString(field));
        Ok(())
    }

    fn on_map_string(&mut self, field: &mut TemplateMapString) -> Result<(), Infallible> {
        self.extractor.extract(TemplateField::TemplateMapString(field));
        Ok(())
    }
}

#[cfg(test)]
mod tests;
