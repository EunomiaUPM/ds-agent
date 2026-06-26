use crate::entities::dataplane_drivers::{DriverAuthenticatorTrait, DriverProxyConfiguratorTrait};
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager::dataplane_proxy::{
    DataplaneProxy, DataplaneProxyEgress, DataplaneProxyIngress, HTTP_LISTENER_PATH,
};
use connector::{InteractionConfig, ProtocolSpec};
use crate::errors::DataplaneError;
use ymir::errors::Outcome;

#[derive(Debug)]
pub struct HttpProviderPushConfigurator;

impl HttpProviderPushConfigurator {
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
        if let Some(dataplane_address) = context.forward_dataplane_address() {
            Ok(DataplaneProxyEgress::HttpProxy {
                path: dataplane_address.endpoint.clone(),
                token_type: dataplane_address.authorization_type.clone(),
                token: dataplane_address.authorization.clone(),
            })
        } else {
            Err(DataplaneError::MissingTransferContext {
                detail: "forward dataplane address".to_string(),
            }
            .into())
        }
    }
}

#[async_trait::async_trait]
impl DriverProxyConfiguratorTrait for HttpProviderPushConfigurator {
    async fn configure_proxy(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
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
