use crate::entities::dataplane_drivers::{DriverAuthenticatorTrait, DriverProxyConfiguratorTrait};
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager::dataplane_proxy::{
    DataplaneProxy, DataplaneProxyEgress, DataplaneProxyIngress, HTTP_LISTENER_PATH,
};
use connector::{InteractionConfig, ProtocolSpec};
use ymir::errors::{Errors, Outcome};

#[derive(Debug)]
pub struct HttpProviderPullConfigurator;

impl HttpProviderPullConfigurator {
    fn configure_ingress(&self, context: &DataplaneContext) -> Outcome<DataplaneProxyIngress> {
        let dataplane_id = context.dataplane_process().inner.id.clone();
        let ingress_path = format!("{}{}", HTTP_LISTENER_PATH, dataplane_id);
        Ok(DataplaneProxyIngress::HttpListener {
            path: ingress_path,
            token_type: None, // TODO create on fly
            token: None,
        })
    }
    fn configure_egress(&self, context: &DataplaneContext) -> Outcome<DataplaneProxyEgress> {
        if let Some(connector) = context.connector_instance() {
            let protocol_spec = match &connector.interaction {
                InteractionConfig::Pull(lc) => lc.data_access.clone(),
                InteractionConfig::Push(_) => {
                    return Err(Errors::crazy(
                        "Connector interaction shouldn't be Push at this point",
                        None,
                    ))
                }
            };
            if let ProtocolSpec::Http(http_spec) = protocol_spec {
                Ok(DataplaneProxyEgress::HttpProxy {
                    path: http_spec.url_template,
                    token_type: None, // TODO fetch from runtime
                    token: None,
                })
            } else {
                return Err(Errors::crazy(
                    "Connector interaction spec should be HTTP at this point",
                    None,
                ));
            }
        } else {
            Err(Errors::crazy("Connector instance not available", None))
        }
    }
}

#[async_trait::async_trait]
impl DriverProxyConfiguratorTrait for HttpProviderPullConfigurator {
    async fn configure_proxy(&self, mut context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let mut proxy = DataplaneProxy::new();
        let ingress = self.configure_ingress(context)?;
        let egress = self.configure_egress(context)?;
        proxy.set_ingress(ingress);
        proxy.set_egress(egress);
        let mut new_context = context.clone();
        new_context.set_proxy(proxy);
        Ok(new_context)
    }
}
