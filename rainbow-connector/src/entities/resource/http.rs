use crate::entities::parameters::{TemplateMapString, TemplateString, TemplateVecString};
use serde::{Deserialize, Serialize};

/// HTTP protocol specification.
///
/// All string fields support `{{__PARAM__}}` placeholders.  The `headers` map
/// values and `body_template` body string are each resolved individually.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpSpec {
    /// URL, possibly containing placeholders (e.g. `https://api.example.com/{{__ID__}}`).
    pub url_template: TemplateString,
    /// HTTP method(s).  Typically a single-element list such as `["GET"]`.
    pub method: TemplateVecString,
    /// Optional request headers map.  Values may contain placeholders.
    pub headers: Option<TemplateMapString>,
    /// Optional request body.  May be a JSON template string.
    pub body_template: Option<TemplateString>,
}
