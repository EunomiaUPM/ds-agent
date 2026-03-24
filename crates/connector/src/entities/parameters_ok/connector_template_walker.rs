//use super::{TemplateMapString, TemplateString, TemplateVecString};
use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::interaction::{InteractionConfig, PullLifecycle, PushLifecycle};
use crate::entities::resource::{HttpSpec, KafkaSpec};
use crate::{ProtocolSpec, TemplateVecString};
use ymir::errors::Outcome;
use crate::entities::parameters::{TemplateMapString, TemplateString};

pub trait ConnectorTemplateWalker {
    fn on_string(&mut self, field: &mut TemplateString) -> Outcome<()>;
    fn on_vec_string(&mut self, field: &mut TemplateVecString) -> Outcome<()>;
    fn on_map_string(&mut self, field: &mut TemplateMapString) -> Outcome<()>;

    fn walk(&mut self, template: &mut ConnectorTemplateDto) -> Outcome<()> {
        self.walk_auth(&mut template.authentication)?;
        self.walk_interaction(&mut template.interaction)?;
        Ok(())
    }

    fn walk_auth(&mut self, auth: &mut AuthenticationConfig) -> Outcome<()> {
        match auth {
            AuthenticationConfig::NoAuth => {}
            AuthenticationConfig::BasicAuth(c) => {
                self.on_string(&mut c.username)?;
            }
            AuthenticationConfig::BearerToken { .. } => {}
            AuthenticationConfig::ApiKey { key, .. } => {
                self.on_string(key)?;
            }
            AuthenticationConfig::OAuth2 {
                token_url,
                client_id,
                scopes,
                ..
            } => {
                self.on_string(token_url)?;
                self.on_string(client_id)?;
                self.on_vec_string(scopes)?;
            }
        }
        Ok(())
    }

    fn walk_interaction(&mut self, interaction: &mut InteractionConfig) -> Outcome<()> {
        match interaction {
            InteractionConfig::Pull(lc) => self.walk_interaction_pull(lc),
            InteractionConfig::Push(lc) => self.walk_interaction_push(lc),
        }
    }

    fn walk_interaction_pull(&mut self, lc: &mut PullLifecycle) -> Outcome<()> {
        self.walk_protocol_spec(&mut lc.data_access)
    }

    fn walk_interaction_push(&mut self, lc: &mut PushLifecycle) -> Outcome<()> {
        self.walk_protocol_spec(&mut lc.subscribe)?;
        if let Some(unsub) = &mut lc.unsubscribe {
            self.walk_protocol_spec(unsub)?;
        }
        Ok(())
    }

    fn walk_protocol_spec(&mut self, spec: &mut ProtocolSpec) -> Outcome<()> {
        match spec {
            ProtocolSpec::Http(s) => self.walk_http(s),
            ProtocolSpec::Kafka(s) => self.walk_kafka(s),
        }
    }

    fn walk_http(&mut self, spec: &mut HttpSpec) -> Outcome<()> {
        self.on_string(&mut spec.url_template)?;
        self.on_vec_string(&mut spec.method)?;
        if let Some(headers) = &mut spec.headers {
            self.on_map_string(headers)?;
        }
        if let Some(body) = &mut spec.body_template {
            self.on_string(body)?;
        }
        Ok(())
    }

    fn walk_kafka(&mut self, spec: &mut KafkaSpec) -> Outcome<()> {
        self.on_vec_string(&mut spec.brokers)?;
        self.on_string(&mut spec.topic)?;
        if let Some(group_id) = &mut spec.group_id {
            self.on_string(group_id)?;
        }
        Ok(())
    }
}
