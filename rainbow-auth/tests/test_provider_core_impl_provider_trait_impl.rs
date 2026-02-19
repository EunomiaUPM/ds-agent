// Tests corresponding to
// 'rainbow-auth\src\ssi_auth\provider\core\impls\provider_trait_impl.rs'

#[cfg(test)]
mod tests {
<<<<<<< HEAD
    use std::sync::Arc;

=======
>>>>>>> origin/main
    use anyhow::Result;
    use async_trait::async_trait;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use chrono::Utc;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use rainbow_auth::ssi_auth::provider::core::traits::provider_trait::RainbowSSIAuthProviderManagerTrait;
    use rainbow_auth::ssi_auth::{
        common::{
            errors::AuthErrors,
            types::gnap::{
<<<<<<< HEAD
                grant_request::{
                    Access4AT, AccessTokenRequirements4GR, Finish4Interact, Interact4GR
                },
                GrantRequest, GrantResponse
            }
        },
        provider::core::Manager
=======
                grant_request::{Access4AT, AccessTokenRequirements4GR, Finish4Interact, Interact4GR},
                GrantRequest, GrantResponse,
            },
        },
        provider::core::Manager,
>>>>>>> origin/main
    };
    use rainbow_common::{
        config::{
            database::DbType,
            global_config::{DatabaseConfig, HostConfig},
            provider_config::ApplicationProviderConfig,
<<<<<<< HEAD
            ConfigRoles
        },
        errors::CommonErrors,
        ssi::{ClientConfig, DisplayInfo, SSIWalletConfig}
    };
    use rainbow_db::auth_provider::entities::auth_interaction::{
        Model as InteractionModel, NewModel as NewInteractionModel
=======
            ConfigRoles,
        },
        errors::CommonErrors,
        ssi::{ClientConfig, DisplayInfo, SSIWalletConfig},
    };
    use rainbow_db::auth_provider::entities::auth_interaction::{
        Model as InteractionModel, NewModel as NewInteractionModel,
>>>>>>> origin/main
    };
    use rainbow_db::auth_provider::entities::auth_request::Model as RequestModel;
    use rainbow_db::auth_provider::entities::auth_request::{Model, NewModel};
    use rainbow_db::auth_provider::entities::auth_token_requirements::Model as TokenRequirementsModel;
    use rainbow_db::auth_provider::entities::auth_verification::NewModel as NewVerificationModel;
<<<<<<< HEAD
    use rainbow_db::auth_provider::entities::mates::{
        Model as MateModel, NewModel as NewMateModel
    };
=======
    use rainbow_db::auth_provider::entities::mates::{Model as MateModel, NewModel as NewMateModel};
>>>>>>> origin/main
    use rainbow_db::auth_provider::repo_factory::factory_trait::AuthRepoFactoryTrait;
    use rainbow_db::{
        auth_provider::{
            entities::auth_verification::Model as VerificationModel,
            repo_factory::traits::{
                AuthInteractionRepoTrait, AuthRequestRepoTrait, AuthTokenRequirementsRepoTrait,
<<<<<<< HEAD
                AuthVerificationRepoTrait, BusinessMatesRepoTrait, MatesRepoTrait
            }
        },
        common::BasicRepoTrait
    };
    use serde_json::json;
    use serde_json::Value;
=======
                AuthVerificationRepoTrait, BusinessMatesRepoTrait, MatesRepoTrait,
            },
        },
        common::BasicRepoTrait,
    };
    use serde_json::json;
    use serde_json::Value;
    use std::sync::Arc;
>>>>>>> origin/main
    use uuid::Uuid;

    // Mocks

    const PRIVATE_KEY_PEM: &str = r#"
    -----BEGIN RSA PRIVATE KEY-----
    MIIEpAIBAAKCAQEAy6fCcCNmPjERiyH9AwG9WLQyRk98phd6AAAN/PpYTUb3I8Nm
    WflVj0hxldmJzOEKSBtYGznYPr9IUWrtsW+GWTx+QLv7sJoSaJ3uiIfoiJWxC/Cr
    ZCqpzfnN6DDd7ZL7noMrRkW7qUnU1mDftncpzQRnSeBbBHHO3IYceAiP6giofF73
    P0TZbUyG/eILIkGNetlPeoWkEAMYUqyPGcmK0sE/5JG1x9cIhzQ8FT9GrKTXZNz+
    af/2q+TRzG0TfMBRVhwRQ2x9xLEOb7DjCcBfk1NtW4zrSvFrLVAJXXroTauMLwO8
    DQSGgxDxxNkutsDjtI5X4iPb1Vsmx+gwcHOrqwIDAQABAoIBADabnCp/g3nnpGGf
    Un2M6N2xK83ooG7U2rTHTvjnk+fcwIYJbhdwIP4TmclplGaobR5anqxmPVLN7bFP
    L78lPWwOKXhoL5vyJD+DIaPgdXyyTs/5z6tDOVbuxcSd3dRHVkrxtxfXuPMyxexZ
    +4KsvzunE58elqlBbwt4toeFHDnPb6zV+AQh11MX6xbW4vKqyAVt+z0egF0hwd+w
    Ldk5ZFCVMiykhOkaA5rTD9QLCI03aw0CWH7A78VY+P/KJljHpgAQxPpdZMsQaC8i
    usOdoi64CYVubJaRgBuVtlCUKBJ1jSh6hJDSveUE6EHMbhMbSLjDdiVx5mVT+f4w
    gy+HmCkCgYEA9310ZXoBAFn45yqQqLoUMY53c0mpu1tGlmAFSUeYxUYCjeJ5OybY
    9AHtqfi9BmyKYS8NhS6iakIfbpBrXp4GU4QQxOnj/BZ49CqoU3rGAvQrUFM6H+ur
    12sL+TxaYFy/gFOOJgfNEjSdncC9n1m32KSy++5ZEOZPZ8s1kOnBYA0CgYEA0qhy
    bk+h0XUd5UYRn81awcOIu1qDHHiC5FFX5kePTxH5fpnLa/5nAHLSt9NslHpRFk3b
    KgZ6XtsCo+PIKWYeqIBDizyrQ1aXRTydUGbs3/0wTAk2fShhBxOK0f5VEU53p7e+
    aOnakvmjv17XKoCxLDeyd/tszbaAuIf5IDevFJcCgYEA3RfrSthSWrEF5eWls8YA
    UkE2vvx4sBKbna0MK+nVNa8UixJeQRS0TlAGtIisvTdr6+PwDSGbIJgPomNMOAuu
    FR/vJyrXbBXbWi1dkK4mhO8CXtDrJScRrfFIkhBzoJBa19ZNL/ZaIkB06kAMZnXt
    ApIn/15vnORS3aOxJ9Nb2GUCgYEAzZKqkPHPFUt4UCzPuCW7YomBnmvWtOr5uPuU
    jtnYhS4iTqSg+hN6ytpCvjdpp3+yL9TsgpdtxbFuYyc/Rv8r/f2lbHRZIU6YMm3T
    iwnWMUOwZRM8hGjqPvCYMRNESq5LYHWUMGe98F4DqRZRXV3XCDKWTDYk2J84AFxZ
    BlKw3aMCgYAOcHclEmli65qtMpMCkr8bVm0+TEyh20AQeYM57+zLHuepaCZJ3MDw
    FPoPJOKuX21grjSUe4vvkgty+LnCneESpGpWhAGsMpRCq3jG6XJwqGg9lV3lHHGS
    c4qkXDZEm0/HxUI/tN5SUnORMdbNjmaMYTVnwKxPAhm51nykY88HWw==
    -----END RSA PRIVATE KEY-----
    "#;

    #[derive(Clone)]
    pub struct MockBusinessMatesRepo;

    #[async_trait]
    impl
        BasicRepoTrait<
            rainbow_db::auth_provider::entities::business_mates::Model,
<<<<<<< HEAD
            rainbow_db::auth_provider::entities::business_mates::NewModel
=======
            rainbow_db::auth_provider::entities::business_mates::NewModel,
>>>>>>> origin/main
        > for MockBusinessMatesRepo
    {
        async fn get_all(
            &self,
            _limit: Option<u64>,
<<<<<<< HEAD
            _offset: Option<u64>
        ) -> anyhow::Result<Vec<rainbow_db::auth_provider::entities::business_mates::Model>>
        {
=======
            _offset: Option<u64>,
        ) -> anyhow::Result<Vec<rainbow_db::auth_provider::entities::business_mates::Model>> {
>>>>>>> origin/main
            Ok(vec![])
        }

        async fn get_by_id(
            &self,
<<<<<<< HEAD
            id: &str
        ) -> anyhow::Result<Option<rainbow_db::auth_provider::entities::business_mates::Model>>
        {
            if id == "test-state" {
                Ok(Some(rainbow_db::auth_provider::entities::business_mates::Model {
                    id: id.to_string(),
                    participant_id: "test-holder".to_string(),
                    token: Some("token-abc".to_string()),
                    saved_at: Utc::now().naive_utc(),
                    last_interaction: Utc::now().naive_utc()
                }))
            } else if id == "test-state-fail" {
                Ok(Some(rainbow_db::auth_provider::entities::business_mates::Model {
                    id: id.to_string(),
                    participant_id: "unknown-holder".to_string(),
                    token: Some("token-abc".to_string()),
                    saved_at: Utc::now().naive_utc(),
                    last_interaction: Utc::now().naive_utc()
                }))
=======
            id: &str,
        ) -> anyhow::Result<Option<rainbow_db::auth_provider::entities::business_mates::Model>> {
            if id == "test-state" {
                Ok(Some(
                    rainbow_db::auth_provider::entities::business_mates::Model {
                        id: id.to_string(),
                        participant_id: "test-holder".to_string(),
                        token: Some("token-abc".to_string()),
                        saved_at: Utc::now().naive_utc(),
                        last_interaction: Utc::now().naive_utc(),
                    },
                ))
            } else if id == "test-state-fail" {
                Ok(Some(
                    rainbow_db::auth_provider::entities::business_mates::Model {
                        id: id.to_string(),
                        participant_id: "unknown-holder".to_string(),
                        token: Some("token-abc".to_string()),
                        saved_at: Utc::now().naive_utc(),
                        last_interaction: Utc::now().naive_utc(),
                    },
                ))
>>>>>>> origin/main
            } else {
                Ok(None)
            }
        }

        async fn create(
            &self,
<<<<<<< HEAD
            _model: rainbow_db::auth_provider::entities::business_mates::NewModel
=======
            _model: rainbow_db::auth_provider::entities::business_mates::NewModel,
>>>>>>> origin/main
        ) -> anyhow::Result<rainbow_db::auth_provider::entities::business_mates::Model> {
            unimplemented!()
        }

        async fn update(
            &self,
<<<<<<< HEAD
            _model: rainbow_db::auth_provider::entities::business_mates::Model
=======
            _model: rainbow_db::auth_provider::entities::business_mates::Model,
>>>>>>> origin/main
        ) -> anyhow::Result<rainbow_db::auth_provider::entities::business_mates::Model> {
            unimplemented!()
        }

        async fn delete(&self, _id: &str) -> anyhow::Result<()> { Ok(()) }
    }

    #[async_trait]
    impl BusinessMatesRepoTrait for MockBusinessMatesRepo {
        async fn get_by_token(
            &self,
<<<<<<< HEAD
            _token: &str
        ) -> anyhow::Result<Option<rainbow_db::auth_provider::entities::business_mates::Model>>
        {
=======
            _token: &str,
        ) -> anyhow::Result<Option<rainbow_db::auth_provider::entities::business_mates::Model>> {
>>>>>>> origin/main
            unimplemented!()
        }

        async fn force_create(
            &self,
<<<<<<< HEAD
            _mate: rainbow_db::auth_provider::entities::business_mates::NewModel
=======
            _mate: rainbow_db::auth_provider::entities::business_mates::NewModel,
>>>>>>> origin/main
        ) -> anyhow::Result<rainbow_db::auth_provider::entities::business_mates::Model> {
            unimplemented!()
        }
    }

    struct TestManager {
        inner: Manager<MockAuthRepoFactory>
    }

    impl TestManager {
        fn new(repo: Arc<MockAuthRepoFactory>, config: ApplicationProviderConfig) -> Self {
            Self { inner: Manager::new(repo, config) }
        }
    }

    #[async_trait]
    impl RainbowSSIAuthProviderManagerTrait for TestManager {
<<<<<<< HEAD
        async fn verify_vp(
            &self,
            _model: VerificationModel,
            vp_token: String
        ) -> Result<(Vec<String>, String)> {
=======
        async fn verify_vp(&self, _model: VerificationModel, vp_token: String) -> Result<(Vec<String>, String)> {
>>>>>>> origin/main
            if vp_token == "valid-vp-token" {
                Ok((
                    vec!["valid-vc-token".to_string()],
                    "test-holder".to_string(),
                ))
            } else {
                Err(AuthErrors::security_new(Some("VPT signature is incorrect".to_string())).into())
            }
        }

        async fn verify_vc(&self, vc_token: String, vp_holder: String) -> Result<()> {
            if vc_token == "valid-vc-token" && vp_holder == "test-holder" {
                Ok(())
            } else {
                Err(AuthErrors::security_new(Some("VC verification failed".to_string())).into())
            }
        }

        async fn verify_all(&self, state: String, vp_token: String) -> Result<String> {
            // Simulates recovery of the verification model
            let verification_model = VerificationModel {
                id: "mock-id".to_string(),
                state: state.clone(),
                nonce: "test-nonce".to_string(),
                audience: "test-client-id".to_string(),
                holder: None,
                vpt: None,
                success: None,
                status: "Pending".to_string(),
                created_at: Utc::now().naive_utc(),
                ended_at: None
            };

            // Use mock of verify_vp
            let (vcts, holder) = self.verify_vp(verification_model.clone(), vp_token).await?;

            // Use mock of verify_vc foreach credential
            for cred in vcts {
                self.verify_vc(cred, holder.clone()).await?;
            }

            // Simulate status update
            let mut updated_verification = verification_model.clone();
            updated_verification.ended_at = Some(Utc::now().naive_utc());

            // Simulates request update
            let _updated_request = RequestModel {
                id: verification_model.id.clone(),
                consumer_slug: "test-class".to_string(),
                token: Some("token-abc".to_string()),
                status: "Processing".to_string(),
                created_at: Utc::now().naive_utc(),
                ended_at: None
            };

            // Simulate success
            Ok(verification_model.id)
        }

        // Delegated or unused methods in the current test
        async fn generate_uri(&self, _: VerificationModel) -> Result<String> { todo!() }

        async fn manage_access(&self, _: GrantRequest) -> Result<GrantResponse> { todo!() }

<<<<<<< HEAD
        async fn validate_continue_request(
            &self,
            _: String,
            _: String,
            _: String
        ) -> Result<InteractionModel> {
=======
        async fn validate_continue_request(&self, _: String, _: String, _: String) -> Result<InteractionModel> {
>>>>>>> origin/main
            todo!()
        }

        async fn continue_req(&self, _: InteractionModel) -> Result<RequestModel> { todo!() }

        async fn retrieve_data(
            &self,
            _: RequestModel,
            _: InteractionModel
        ) -> Result<NewMateModel> {
            todo!()
        }

        async fn save_mate(&self, _: NewMateModel) -> Result<MateModel> { todo!() }

        async fn generate_vp_def(&self, _: String) -> Result<Value> { todo!() }

        async fn end_verification(&self, _: String) -> Result<Option<String>> { todo!() }

        async fn fast_login(&self, _: String) -> Result<String> { todo!() }

        async fn verify_token(&self, _: String) -> Result<MateModel> { todo!() }

        async fn retrieve_business_token(&self, _: String) -> Result<Value> { todo!() }
    }

    #[async_trait]
    impl BasicRepoTrait<MateModel, NewMateModel> for MockMatesRepo {
        async fn get_all(
            &self,
            _limit: Option<u64>,
            _offset: Option<u64>
        ) -> anyhow::Result<Vec<MateModel>> {
            Ok(vec![])
        }

        async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<MateModel>> {
            if id == "test-holder" {
                Ok(Some(MateModel {
                    participant_id: "test-holder".to_string(),
                    participant_slug: "test-consumer".to_string(),
                    participant_type: "Agent".to_string(),
                    base_url: Some("https://example.com".to_string()),
                    token: Some("token-abc".to_string()),
                    saved_at: Utc::now().naive_utc(),
                    last_interaction: Utc::now().naive_utc(),
                    is_me: false
                }))
            } else {
                Ok(None)
            }
        }

        async fn create(&self, _model: NewMateModel) -> anyhow::Result<MateModel> {
            unimplemented!()
        }

        async fn update(&self, _model: MateModel) -> anyhow::Result<MateModel> { unimplemented!() }

        async fn delete(&self, _id: &str) -> anyhow::Result<()> { Ok(()) }
    }

    #[derive(Clone)]
    pub struct MockMatesRepo;

    #[async_trait]
<<<<<<< HEAD
    impl rainbow_db::auth_provider::repo_factory::traits::mates_trait::MatesRepoTrait
        for MockMatesRepo
    {
=======
    impl rainbow_db::auth_provider::repo_factory::traits::mates_trait::MatesRepoTrait for MockMatesRepo {
>>>>>>> origin/main
        async fn force_create(&self, mate: NewMateModel) -> anyhow::Result<MateModel> {
            Ok(MateModel {
                participant_id: mate.participant_id,
                participant_slug: mate.participant_slug,
                participant_type: mate.participant_type,
                base_url: mate.base_url,
                token: mate.token,
                saved_at: chrono::Utc::now().naive_utc(),
                last_interaction: chrono::Utc::now().naive_utc(),
                is_me: mate.is_me
            })
        }

<<<<<<< HEAD
        async fn get_me(&self) -> anyhow::Result<Option<MateModel>> { unimplemented!() }
=======
        async fn get_me(&self) -> anyhow::Result<Option<MateModel>> {
            unimplemented!()
        }
>>>>>>> origin/main

        async fn get_by_token(&self, token: &str) -> anyhow::Result<Option<MateModel>> {
            if token == "valid-token" {
                Ok(Some(MateModel {
                    participant_id: "test-holder".to_string(),
                    participant_slug: "test-consumer".to_string(),
                    participant_type: "Agent".to_string(),
                    base_url: Some("https://example.com".to_string()),
                    token: Some(token.to_string()),
                    saved_at: Utc::now().naive_utc(),
                    last_interaction: Utc::now().naive_utc(),
                    is_me: false
                }))
            } else {
                Ok(None)
            }
        }

        async fn get_batch(&self, _ids: &Vec<String>) -> anyhow::Result<Vec<MateModel>> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl BasicRepoTrait<VerificationModel, NewVerificationModel> for MockVerificationRepo {
        async fn get_all(
            &self,
            _limit: Option<u64>,
            _offset: Option<u64>
        ) -> anyhow::Result<Vec<VerificationModel>> {
            Ok(vec![])
        }

        async fn get_by_id(&self, _id: &str) -> anyhow::Result<Option<VerificationModel>> {
            if _id == "mock-id" {
                Ok(Some(VerificationModel {
                    id: _id.to_string(),
                    state: "test-state".to_string(),
                    nonce: "test-nonce".to_string(),
                    audience: "test-client-id".to_string(),
                    holder: Some("test-holder".to_string()), // ← necesario
                    vpt: None,
                    success: None,
                    status: "Pending".to_string(),
                    created_at: Utc::now().naive_utc(),
                    ended_at: None
                }))
            } else {
                Ok(None)
            }
        }

        async fn create(&self, model: NewVerificationModel) -> anyhow::Result<VerificationModel> {
            Ok(VerificationModel {
                id: model.id,
                state: "test-state".to_string(),
                nonce: "test-nonce".to_string(),
                audience: model.audience,
                holder: None,
                vpt: None,
                success: None,
                status: "Pending".to_string(),
                created_at: Utc::now().naive_utc(),
                ended_at: None
            })
        }

        async fn update(&self, model: VerificationModel) -> anyhow::Result<VerificationModel> {
            Ok(model)
        }

        async fn delete(&self, _id: &str) -> anyhow::Result<()> { Ok(()) }
    }

    #[derive(Clone)]
    pub struct MockVerificationRepo;

    #[async_trait]
    impl rainbow_db::auth_provider::repo_factory::traits::auth_verification_trait::AuthVerificationRepoTrait
        for MockVerificationRepo
    {
        async fn get_by_state(
            &self,
            _state: &str,
<<<<<<< HEAD
        ) -> anyhow::Result<Option<rainbow_db::auth_provider::entities::auth_verification::Model,>,> {
            if _state == "test-state"
            {
=======
        ) -> anyhow::Result<Option<rainbow_db::auth_provider::entities::auth_verification::Model>> {
            if _state == "test-state" {
>>>>>>> origin/main
                Ok(Some(VerificationModel {
                    id: "mock-id".to_string(),
                    state: _state.to_string(),
                    nonce: "test-nonce".to_string(),
                    audience: "test-client-id".to_string(),
                    holder: Some("test-holder".to_string(),),
                    vpt: None,
                    success: None,
                    status: "Pending".to_string(),
                    created_at: Utc::now().naive_utc(),
                    ended_at: None,
                },),)
            }
            else
            {
                Ok(None,)
            }
        }

<<<<<<< HEAD

        async fn create_extra(
            &self,
            model: rainbow_db::auth_provider::entities::auth_verification::Model,
        ) -> anyhow::Result<rainbow_db::auth_provider::entities::auth_verification::Model,> {
            if model.state == "fail-verification"
            {
                Err(anyhow::anyhow!("Simulated DB failure in verification"),)
            }
            else
            {
                Ok(model,)
=======
        async fn create_extra(
            &self,
            model: rainbow_db::auth_provider::entities::auth_verification::Model,
        ) -> anyhow::Result<rainbow_db::auth_provider::entities::auth_verification::Model> {
            if model.state == "fail-verification" {
                Err(anyhow::anyhow!("Simulated DB failure in verification"))
            } else {
                Ok(model)
>>>>>>> origin/main
            }
        }
    }

    #[derive(Clone)]
    struct MockAuthRepoFactory;

    impl Default for MockAuthRepoFactory {
        fn default() -> Self { MockAuthRepoFactory }
    }

    impl AuthRepoFactoryTrait for MockAuthRepoFactory {
        fn request(
            &self,
<<<<<<< HEAD
        ) -> Arc<dyn rainbow_db::auth_provider::repo_factory::traits::auth_request_trait::AuthRequestRepoTrait,>
=======
        ) -> Arc<dyn rainbow_db::auth_provider::repo_factory::traits::auth_request_trait::AuthRequestRepoTrait>
>>>>>>> origin/main
        {
            Arc::new(MockRequestRepo)
        }

        fn interaction(
            &self,
<<<<<<< HEAD
        ) -> Arc<dyn rainbow_db::auth_provider::repo_factory::traits::auth_interaction_trait::AuthInteractionRepoTrait,>
=======
        ) -> Arc<dyn rainbow_db::auth_provider::repo_factory::traits::auth_interaction_trait::AuthInteractionRepoTrait>
>>>>>>> origin/main
        {
            Arc::new(MockInteractionRepo)
        }

        fn verification(
            &self,
<<<<<<< HEAD
        ) -> Arc<dyn rainbow_db::auth_provider::repo_factory::traits::auth_verification_trait::AuthVerificationRepoTrait,>
=======
        ) -> Arc<dyn rainbow_db::auth_provider::repo_factory::traits::auth_verification_trait::AuthVerificationRepoTrait>
>>>>>>> origin/main
        {
            Arc::new(MockVerificationRepo)
        }

        fn mates(
            &self
        ) -> Arc<dyn rainbow_db::auth_provider::repo_factory::traits::mates_trait::MatesRepoTrait>
        {
            Arc::new(MockMatesRepo)
        }

        fn token_requirements(&self) -> Arc<dyn rainbow_db::auth_provider::repo_factory::traits::auth_token_requirements_trait::AuthTokenRequirementsRepoTrait>{
            Arc::new(MockTokenRequirementsRepo)
        }

        fn business_mates(&self) -> Arc<dyn BusinessMatesRepoTrait> {
            Arc::new(MockBusinessMatesRepo)
        }
    }

    #[derive(Clone)]
    pub struct MockTokenRequirementsRepo;

    #[async_trait]
    impl BasicRepoTrait<TokenRequirementsModel, TokenRequirementsModel> for MockTokenRequirementsRepo {
        async fn get_all(
            &self,
            _limit: Option<u64>,
            _offset: Option<u64>
        ) -> Result<Vec<TokenRequirementsModel>> {
            Ok(vec![])
        }

        async fn get_by_id(&self, _id: &str) -> Result<Option<TokenRequirementsModel>> { Ok(None) }

        async fn create(&self, model: TokenRequirementsModel) -> Result<TokenRequirementsModel> {
            Ok(model)
        }

        async fn update(&self, model: TokenRequirementsModel) -> Result<TokenRequirementsModel> {
            Ok(model)
        }

        async fn delete(&self, _id: &str) -> Result<()> { Ok(()) }
    }

    #[async_trait]
    impl rainbow_db::auth_provider::repo_factory::traits::auth_token_requirements_trait::AuthTokenRequirementsRepoTrait
        for MockTokenRequirementsRepo
    {
    }

    #[derive(Clone)]
    pub struct MockRequestRepo;

    #[async_trait]
    impl BasicRepoTrait<Model, NewModel> for MockRequestRepo {
        async fn get_all(
            &self,
            _limit: Option<u64>,
            _offset: Option<u64>
        ) -> anyhow::Result<Vec<Model>> {
            unimplemented!()
        }

        async fn get_by_id(&self, _id: &str) -> anyhow::Result<Option<Model>> {
            if _id == "mock-id" {
                Ok(Some(Model {
                    id: _id.to_string(),
                    consumer_slug: "test-class".to_string(),
                    token: None,
                    status: "Pending".to_string(),
                    created_at: Utc::now().naive_utc(),
                    ended_at: None
                }))
            } else {
                Ok(None)
            }
        }

        async fn create(&self, model: NewModel) -> anyhow::Result<Model> {
            Ok(Model {
                id: model.id,
                consumer_slug: model.consumer_slug,
                token: None,
                status: "Pending".to_string(),
                created_at: chrono::Utc::now().naive_utc(),
                ended_at: None
            })
        }

        async fn update(&self, _model: Model) -> anyhow::Result<Model> { Ok(_model) }

        async fn delete(&self, _id: &str) -> anyhow::Result<()> { unimplemented!() }
    }

    impl AuthRequestRepoTrait for MockRequestRepo {}

    #[derive(Clone)]
    pub struct MockInteractionRepo;

    #[async_trait]
    impl BasicRepoTrait<InteractionModel, NewInteractionModel> for MockInteractionRepo {
        async fn get_all(
            &self,
            _limit: Option<u64>,
            _offset: Option<u64>
        ) -> anyhow::Result<Vec<InteractionModel>> {
            unimplemented!()
        }

        async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<InteractionModel>> {
            if id == "cont-id-123" {
                Ok(Some(InteractionModel {
                    id: id.to_string(),
                    method: "redirect".to_string(),
                    uri: "https://example.com/callback".to_string(),
                    hash: "hash789".to_string(),
                    interact_ref: "ref-456".to_string(),
                    // otros campos también deben estar presentes aunque no se usen
                    start: vec!["login".to_string()],
                    client_nonce: "nonce123".to_string(),
                    hash_method: "S256".to_string(),
                    hints: Some("hint".to_string()),
                    grant_endpoint: "https://example.com/grant".to_string(),
                    continue_endpoint: "https://example.com/continue".to_string(),
                    continue_id: id.to_string(),
                    continue_token: "token-abc".to_string(),
                    as_nonce: "as-nonce-xyz".to_string()
                }))
            } else if id == "invalid-method-id" {
                Ok(Some(InteractionModel {
                    id: id.to_string(),
                    start: vec!["login".to_string()],
                    method: "unknown".to_string(), // método no soportado
                    uri: "https://example.com/callback".to_string(),
                    client_nonce: "nonce123".to_string(),
                    hash_method: "S256".to_string(),
                    hints: Some("hint".to_string()),
                    grant_endpoint: "https://example.com/grant".to_string(),
                    continue_endpoint: "https://example.com/continue".to_string(),
                    continue_id: id.to_string(),
                    continue_token: "token-abc".to_string(),
                    as_nonce: "as-nonce-xyz".to_string(),
                    interact_ref: "ref-456".to_string(),
                    hash: "hash789".to_string()
                }))
            } else {
                Ok(None)
            }
        }

        async fn create(&self, model: NewInteractionModel) -> anyhow::Result<InteractionModel> {
            Ok(InteractionModel {
                id: model.id,
                start: model.start,
                method: model.method,
                uri: model.uri,
                client_nonce: model.client_nonce,
                hash_method: model.hash_method.unwrap_or_else(|| "S256".to_string()),
                hints: model.hints,
                grant_endpoint: model.grant_endpoint,
                continue_endpoint: model.continue_endpoint,
                continue_id: Uuid::new_v4().to_string(),
                continue_token: model.continue_token,
                as_nonce: Uuid::new_v4().to_string(),
                interact_ref: Uuid::new_v4().to_string(),
                hash: "mocked-hash".to_string()
            })
        }

        async fn update(&self, _model: InteractionModel) -> anyhow::Result<InteractionModel> {
            Ok(_model)
        }

        async fn delete(&self, _id: &str) -> anyhow::Result<()> { Ok(()) }
    }

    #[async_trait]
    impl AuthInteractionRepoTrait for MockInteractionRepo {
        async fn get_by_reference(
            &self,
            _reference: &str
        ) -> anyhow::Result<Option<InteractionModel>> {
            unimplemented!()
        }

        async fn get_by_cont_id(&self, cont_id: &str) -> anyhow::Result<Option<InteractionModel>> {
            if cont_id == "cont-id-123" {
                Ok(Some(InteractionModel {
                    id: "mock-id".to_string(),
                    start: vec!["login".to_string()],
                    method: "redirect".to_string(),
                    uri: "https://example.com/callback".to_string(),
                    client_nonce: "nonce123".to_string(),
                    hash_method: "S256".to_string(),
                    hints: Some("hint".to_string()),
                    grant_endpoint: "https://example.com/grant".to_string(),
                    continue_endpoint: "https://example.com/continue".to_string(),
                    continue_id: cont_id.to_string(),
                    continue_token: "token-abc".to_string(),
                    as_nonce: "as-nonce-xyz".to_string(),
                    interact_ref: "ref-456".to_string(),
                    hash: "hash789".to_string()
                }))
            } else {
                Ok(None)
            }
        }
    }

    fn build_test_config() -> ApplicationProviderConfig {
        ApplicationProviderConfig {
            transfer_process_host: None,
            business_system_host: None,
            catalog_host: None,
            catalog_as_datahub: false,
            datahub_host: None,
            datahub_token: "dummy-token".to_string(),
            contract_negotiation_host: None,
            auth_host: None,
            ssi_auth_host: Some(HostConfig {
                protocol: "http".to_string(),
                url: "localhost".to_string(),
                port: "8080".to_string()
            }),
            gateway_host: None,
            is_gateway_in_production: false,
            database_config: DatabaseConfig {
                db_type: DbType::Sqlite,
                user: "user".to_string(),
                password: "pass".to_string(),
                url: "localhost".to_string(),
                port: 5432.to_string(),
                name: "test_db".to_string()
            },
            ssh_user: None,
            ssh_private_key_path: None,
            ssi_wallet_config: SSIWalletConfig {
                wallet_id: Some("wallet-id".to_string()),
                wallet_type: "mock".to_string(),
                wallet_name: "test-wallet".to_string(),
                wallet_email: "test@example.com".to_string(),
                wallet_password: "password".to_string(),
                wallet_api_protocol: "http".to_string(),
                wallet_api_url: "localhost".to_string(),
                wallet_api_port: Some("3000".to_string())
            },
            client_config: ClientConfig {
                class_id: "test-class-id".to_string(),
                cert_path: "test-cert-path".to_string(),
                display: Some(DisplayInfo {
                    name: "test-display".to_string(),
                    uri: Some("localhost".to_string()),
<<<<<<< HEAD
                    logo_uri: Some("localhost".to_string())
                })
=======
                    logo_uri: Some("localhost".to_string()),
                }),
>>>>>>> origin/main
            },
            role: ConfigRoles::Provider,
            is_local: true
        }
    }

    pub fn generate_valid_vp_token_for_test() -> String {
        // JWK público correspondiente a la clave privada
        let jwk = json!({
            "kty": "RSA",
            "alg": "RS256",
            "n": "y6fCcCNmPjERiyH9AwG9WLQyRk98phd6AAAN_PpYTUb3I8NmWflVj0hxldmJzOEKSBtYGznYPr9IUWrtsW-GWTx-QLv7sJoSaJ3uiIfoiJWxC_CrZCqpzfnN6DDd7ZL7noMrRkW7qUnU1mDftncpzQRnSeBbBHHO3IYceAiP6giofF73P0TZbUyG_eILIkGNetlPeoWkEAMYUqyPGcmK0sE_5JG1x9cIhzQ8FT9GrKTXZNz-af_2q-TRzG0TfMBRVhwRQ2x9xLEOb7DjCcBfk1NtW4zrSvFrLVAJXXroTauMLwO8DQSGgxDxxNkutsDjtI5X4iPb1Vsmx-gwcHOrqw",
            "e": "AQAB"
        });

        let jwk_encoded = URL_SAFE_NO_PAD.encode(jwk.to_string());
        let kid = format!("did:jwk:{}", jwk_encoded);

        let claims = json!({
            "nonce": "test-nonce",
            "sub": kid,
            "iss": kid,
            "vp": {
                "id": "mock-id",
                "holder": kid,
                "verifiableCredential": ["vc1", "vc2"]
            },
            "nbf": Utc::now().timestamp(),
            "aud": "http://localhost:8080/api/v1/verify/test-state"
        });

        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(kid.clone());

        // Clave RSA privada válida (2048 bits)
        let private_key_pem = PRIVATE_KEY_PEM;

        encode(
            &header,
            &claims,
<<<<<<< HEAD
            &EncodingKey::from_rsa_pem(private_key_pem.as_bytes()).unwrap()
=======
            &EncodingKey::from_rsa_pem(private_key_pem.as_bytes()).unwrap(),
>>>>>>> origin/main
        )
        .unwrap()
    }

    // Tests

    #[tokio::test]
    async fn test_generate_uri_success() -> anyhow::Result<()> {
        // Mock config
        let config = build_test_config();

        // Mock repo
        let mock_repo = Arc::new(MockAuthRepoFactory::default());

        // Create manager with mocks
        let manager = Manager::new(mock_repo, config);

        // Test verification model
        let ver_model = VerificationModel {
            id: "test-id".to_string(),
            state: "test-state".to_string(),
            nonce: "test-nonce".to_string(),
            audience: "test-client-id".to_string(),
            holder: None,
            vpt: None,
            success: None,
            status: "Pending".to_string(),
            created_at: Utc::now().naive_utc(),
            ended_at: None
        };

        // Execute function
        let uri = manager.generate_uri(ver_model).await?;

        // Validations
        assert!(uri.starts_with("openid4vp://authorize"));
        assert!(uri.contains("response_type=vp_token"));
        assert!(uri.contains("client_id=test-client-id"));
        assert!(uri.contains("presentation_definition_uri"));
        assert!(uri.contains("response_uri"));
        assert!(uri.contains("nonce=test-nonce"));

        Ok(())
    }

    #[test]
    fn test_generate_uri_panics_when_host_url_is_none() {
        let result = std::panic::catch_unwind(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut config = build_test_config();
                config.ssi_auth_host = None;

                let mock_repo = Arc::new(MockAuthRepoFactory::default());
                let manager = Manager::new(mock_repo, config);

                let ver_model = VerificationModel {
                    id: "test-id".to_string(),
                    state: "test-state".to_string(),
                    nonce: "test-nonce".to_string(),
                    audience: "test-client-id".to_string(),
                    holder: None,
                    vpt: None,
                    success: None,
                    status: "Pending".to_string(),
                    created_at: Utc::now().naive_utc(),
                    ended_at: None
                };

                let _ = manager.generate_uri(ver_model).await;
            });
        });

        assert!(result.is_err(), "Expected panic due to unwrap on None");
    }

    #[tokio::test]
    async fn test_manage_access_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        let payload = GrantRequest {
            access_token: AccessTokenRequirements4GR {
                access: Access4AT {
                    r#type: "read".to_string(),
                    actions: Some(vec!["read".to_string()]),
                    locations: None,
                    datatypes: None,
                    identifier: None,
                    privileges: None
                },
                label: None,
                flags: None
            },
            subject: None,
            client: json!({ "class_id": "test-class" }),
            user: None,
            interact: Some(Interact4GR {
                start: vec!["oidc4vp".to_string()],
                finish: Finish4Interact {
                    method: "redirect".to_string(),
                    uri: Some("http://localhost/redirect".to_string()),
                    nonce: "nonce123".to_string(),
                    hash_method: None
                },
                hints: None
            })
        };

        let result = manager.manage_access(payload).await;

        assert!(result.is_ok(), "Expected success, got error: {:?}", result);
        let response = result.unwrap();

<<<<<<< HEAD
        let uri = response
            .interact
            .as_ref()
            .and_then(|i| i.oidc4vp.clone())
            .expect("Expected oidc4vp URI in response");
=======
        let uri = response.interact.as_ref().and_then(|i| i.oidc4vp.clone()).expect("Expected oidc4vp URI in response");
>>>>>>> origin/main

        assert!(uri.starts_with("openid4vp://authorize"));

        Ok(())
    }

    #[tokio::test]
    async fn test_manage_access_error_missing_interact() {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        let payload = GrantRequest {
            access_token: AccessTokenRequirements4GR {
                access: Access4AT {
                    r#type: "read".to_string(),
                    actions: None,
                    locations: None,
                    datatypes: None,
                    identifier: None,
                    privileges: None
                },
                label: None,
                flags: None
            },
            subject: None,
            client: json!({ "class_id": "test-class" }),
            user: None,
            interact: None // ← ERROR
        };

        let result = manager.manage_access(payload).await;

        assert!(result.is_err());

        let err = result.unwrap_err();
        let common_err = err.downcast_ref::<CommonErrors>().expect("Expected CommonErrors");

        match common_err {
            CommonErrors::FeatureNotImplError { feature, .. } => {
                assert!(feature.contains("Only petitions with an 'interact field'"));
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_validate_continue_request_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        // Simulated values
        let cont_id = "cont-id-123".to_string();
        let interact_ref = "ref-456".to_string();
        let token = "token-abc".to_string();

        // Execute
        let result = manager
            .validate_continue_request(cont_id.clone(), interact_ref.clone(), token.clone())
            .await;

        // Validate
        assert!(result.is_ok());
        let model = result.unwrap();
        assert_eq!(model.continue_id, cont_id);
        assert_eq!(model.interact_ref, interact_ref);
        assert_eq!(model.continue_token, token);

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_continue_request_error_wrong_interact_ref() {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        let cont_id = "cont-id-123".to_string();
        let wrong_interact_ref = "wrong-ref".to_string(); // ← incorrecto
        let token = "token-abc".to_string();

        let result = manager.validate_continue_request(cont_id, wrong_interact_ref, token).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        let auth_err = err.downcast_ref::<AuthErrors>().expect("Expected AuthErrors");

        match auth_err {
            AuthErrors::SecurityError { cause, .. } => {
                let cause_text = cause.as_ref().expect("Expected cause");
                assert!(cause_text.contains("Interact reference"));
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_continue_req_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        let interaction_model = InteractionModel {
            id: "mock-id".to_string(),
            start: vec!["login".to_string()],
            method: "redirect".to_string(),
            uri: "https://example.com/callback".to_string(),
            client_nonce: "nonce123".to_string(),
            hash_method: "S256".to_string(),
            hints: Some("hint".to_string()),
            grant_endpoint: "https://example.com/grant".to_string(),
            continue_endpoint: "https://example.com/continue".to_string(),
            continue_id: "cont-id-123".to_string(),
            continue_token: "token-abc".to_string(),
            as_nonce: "as-nonce-xyz".to_string(),
            interact_ref: "ref-456".to_string(),
            hash: "hash789".to_string()
        };

        let result = manager.continue_req(interaction_model.clone()).await;

        assert!(result.is_ok());
        let updated_model = result.unwrap();
        assert_eq!(updated_model.id, interaction_model.id);
        assert_eq!(updated_model.status, "Approved");
        assert!(updated_model.token.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_continue_req_error_missing_request() {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        let interaction_model = InteractionModel {
            id: "non-existent-id".to_string(), // ← ERROR
            start: vec!["login".to_string()],
            method: "redirect".to_string(),
            uri: "https://example.com/callback".to_string(),
            client_nonce: "nonce123".to_string(),
            hash_method: "S256".to_string(),
            hints: Some("hint".to_string()),
            grant_endpoint: "https://example.com/grant".to_string(),
            continue_endpoint: "https://example.com/continue".to_string(),
            continue_id: "cont-id-123".to_string(),
            continue_token: "token-abc".to_string(),
            as_nonce: "as-nonce-xyz".to_string(),
            interact_ref: "ref-456".to_string(),
            hash: "hash789".to_string()
        };

        let result = manager.continue_req(interaction_model).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        let common_err = err.downcast_ref::<CommonErrors>().expect("Expected CommonErrors");

        match common_err {
            CommonErrors::MissingResourceError { resource_id, .. } => {
                assert_eq!(resource_id, "non-existent-id");
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_retrieve_data_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        let req_model = Model {
            id: "mock-id".to_string(),
            consumer_slug: "test-consumer".to_string(),
            token: Some("token-abc".to_string()),
            status: "Pending".to_string(),
            created_at: Utc::now().naive_utc(),
            ended_at: None
        };

        let int_model = InteractionModel {
            id: "mock-id".to_string(),
            start: vec!["login".to_string()],
            method: "redirect".to_string(),
            uri: "https://example.com/callback".to_string(),
            client_nonce: "nonce123".to_string(),
            hash_method: "S256".to_string(),
            hints: Some("hint".to_string()),
            grant_endpoint: "https://example.com/grant".to_string(),
            continue_endpoint: "https://example.com/continue".to_string(),
            continue_id: "cont-id-123".to_string(),
            continue_token: "token-abc".to_string(),
            as_nonce: "as-nonce-xyz".to_string(),
            interact_ref: "ref-456".to_string(),
            hash: "hash789".to_string()
        };

        let result = manager.retrieve_data(req_model.clone(), int_model.clone()).await;

        println!();

        assert!(result.is_ok());
        let mate = result.unwrap();
        assert_eq!(mate.participant_id, "test-holder");
        assert_eq!(mate.participant_slug, req_model.consumer_slug);
        assert_eq!(mate.token, req_model.token);
        assert_eq!(mate.base_url, Some("https://example.com".to_string()));
        assert!(!mate.is_me);

        Ok(())
    }

    #[tokio::test]
    async fn test_retrieve_data_error_missing_verification() {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        let req_model = Model {
            id: "non-existent-id".to_string(),
            consumer_slug: "test-consumer".to_string(),
            token: Some("token-abc".to_string()),
            status: "Pending".to_string(),
            created_at: Utc::now().naive_utc(),
            ended_at: None
        };

        let int_model = InteractionModel {
            id: "non-existent-id".to_string(),
            start: vec!["login".to_string()],
            method: "redirect".to_string(),
            uri: "https://example.com/callback".to_string(),
            client_nonce: "nonce123".to_string(),
            hash_method: "S256".to_string(),
            hints: Some("hint".to_string()),
            grant_endpoint: "https://example.com/grant".to_string(),
            continue_endpoint: "https://example.com/continue".to_string(),
            continue_id: "cont-id-123".to_string(),
            continue_token: "token-abc".to_string(),
            as_nonce: "as-nonce-xyz".to_string(),
            interact_ref: "ref-456".to_string(),
            hash: "hash789".to_string()
        };

        let result = manager.retrieve_data(req_model, int_model).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        let common_err = err.downcast_ref::<CommonErrors>().expect("Expected CommonErrors");

        match common_err {
            CommonErrors::MissingResourceError { resource_id, .. } => {
                assert_eq!(resource_id, "non-existent-id");
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_save_mate_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        let new_mate = NewMateModel {
            participant_id: "test-holder".to_string(),
            participant_slug: "test-consumer".to_string(),
            participant_type: "Agent".to_string(),
            base_url: Some("https://example.com".to_string()),
            token: Some("token-abc".to_string()),
            is_me: false
        };

        let result = manager.save_mate(new_mate.clone()).await;

        assert!(result.is_ok());
        let model = result.unwrap();
        assert_eq!(model.participant_id, new_mate.participant_id);
        assert_eq!(model.participant_slug, new_mate.participant_slug);
        assert_eq!(model.token, new_mate.token);
        assert_eq!(model.base_url, new_mate.base_url);
        assert_eq!(model.is_me, new_mate.is_me);
        Ok(())
    }

    #[tokio::test]
    async fn test_save_mate_error_database() {
        #[derive(Clone)]
        struct FailingMatesRepo;

        #[async_trait]
        impl BasicRepoTrait<MateModel, NewMateModel> for FailingMatesRepo {
            async fn get_all(
                &self,
                _limit: Option<u64>,
                _offset: Option<u64>
            ) -> anyhow::Result<Vec<MateModel>> {
                Ok(vec![])
            }

            async fn get_by_id(&self, _id: &str) -> anyhow::Result<Option<MateModel>> { Ok(None) }

            async fn create(&self, _model: NewMateModel) -> anyhow::Result<MateModel> {
                unimplemented!()
            }

            async fn update(&self, _model: MateModel) -> anyhow::Result<MateModel> {
                unimplemented!()
            }

            async fn delete(&self, _id: &str) -> anyhow::Result<()> { Ok(()) }
        }

        #[async_trait]
        impl MatesRepoTrait for FailingMatesRepo {
            async fn force_create(&self, _mate: NewMateModel) -> anyhow::Result<MateModel> {
                Err(anyhow::anyhow!("DB failure"))
            }

            async fn get_me(&self) -> anyhow::Result<Option<MateModel>> { unimplemented!() }

            async fn get_by_token(&self, _token: &str) -> anyhow::Result<Option<MateModel>> {
                unimplemented!()
            }

            async fn get_batch(&self, _ids: &Vec<String>) -> anyhow::Result<Vec<MateModel>> {
                unimplemented!()
            }
        }

        #[derive(Clone)]
        struct FailingRepoFactory;

        impl AuthRepoFactoryTrait for FailingRepoFactory {
            fn request(&self) -> Arc<dyn AuthRequestRepoTrait> { Arc::new(MockRequestRepo) }
            fn interaction(&self) -> Arc<dyn AuthInteractionRepoTrait> {
                Arc::new(MockInteractionRepo)
            }
            fn verification(&self) -> Arc<dyn AuthVerificationRepoTrait> {
                Arc::new(MockVerificationRepo)
            }
            fn token_requirements(&self) -> Arc<dyn AuthTokenRequirementsRepoTrait> {
                Arc::new(MockTokenRequirementsRepo)
            }
            fn mates(&self) -> Arc<dyn MatesRepoTrait> { Arc::new(FailingMatesRepo) }
            fn business_mates(&self) -> Arc<dyn BusinessMatesRepoTrait> { todo!() }
        }

        let config = build_test_config();
        let manager = Manager::new(Arc::new(FailingRepoFactory), config);

        let new_mate = NewMateModel {
            participant_id: "test-holder".to_string(),
            participant_slug: "test-consumer".to_string(),
            participant_type: "Agent".to_string(),
            base_url: Some("https://example.com".to_string()),
            token: Some("token-abc".to_string()),
            is_me: false
        };

        let result = manager.save_mate(new_mate).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        let common_err = err.downcast_ref::<CommonErrors>().expect("Expected CommonErrors");

        match common_err {
            CommonErrors::DatabaseError { cause, .. } => {
                assert!(cause.as_ref().unwrap().contains("DB failure"));
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_generate_vp_def_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        let state = "test-state".to_string();

        let result = manager.generate_vp_def(state.clone()).await;

        assert!(result.is_ok());
        let json = result.unwrap();

        assert_eq!(json["id"], "mock-id");
        assert_eq!(
            json["input_descriptors"][0]["id"],
            "DataspaceParticipantCredential"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_vp_def_error_missing_state() {
        let config = build_test_config();
        let mock_repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(mock_repo, config);

        let invalid_state = "non-existent-state".to_string();

        let result = manager.generate_vp_def(invalid_state.clone()).await;

        assert!(result.is_err());

        let err = result.unwrap_err();
        let common_err = err.downcast_ref::<CommonErrors>().expect("Expected CommonErrors");

        match common_err {
            CommonErrors::MissingResourceError { resource_id, .. } => {
                assert_eq!(resource_id, &invalid_state);
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_verify_all_success() -> Result<()> {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = TestManager::new(repo, config);

        let result =
            manager.verify_all("test-state".to_string(), "valid-vp-token".to_string()).await;

        assert!(result.is_ok());
        let id = result.unwrap();
        assert_eq!(id, "mock-id");

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_all_error_invalid_vp() {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = TestManager::new(repo, config);

        let result =
            manager.verify_all("test-state".to_string(), "invalid-vp-token".to_string()).await;

        assert!(result.is_err());

        let err = result.unwrap_err();
        let auth_err = err.downcast_ref::<AuthErrors>().expect("Expected AuthErrors");

        match auth_err {
            AuthErrors::SecurityError { cause, .. } => {
                assert!(cause.as_ref().unwrap().contains("VPT signature is incorrect"));
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_verify_vp_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo, config);

        // Simulate a verification model
        let model = VerificationModel {
            id: "mock-id".to_string(),
            state: "test-state".to_string(),
            nonce: "test-nonce".to_string(),
            audience: "test-client-id".to_string(),
            holder: None,
            vpt: None,
            success: None,
            status: "Pending".to_string(),
            created_at: Utc::now().naive_utc(),
            ended_at: None
        };

        // Valid VP token JWT (you must generate one that meets the expected claims)
        let vp_token = generate_valid_vp_token_for_test();
        let result = manager.verify_vp(model.clone(), vp_token).await;

        assert!(result.is_ok());
        let (vct, holder) = result.unwrap();
        assert!(!vct.is_empty());
        assert!(
            holder.starts_with("did:jwk:"),
            "Holder should start with 'did:jwk:'"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_vp_invalid_token() {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo, config);

        let model = VerificationModel {
            id: "mock-id".to_string(),
            state: "test-state".to_string(),
            nonce: "test-nonce".to_string(),
            audience: "http://localhost:8080/api/v1/verify/test-state".to_string(),
            holder: None,
            vpt: None,
            success: None,
            status: "Pending".to_string(),
            created_at: Utc::now().naive_utc(),
            ended_at: None
        };

        let vp_token = "invalid.jwt.token".to_string(); //<-- ERROR

        let result = manager.verify_vp(model, vp_token).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{:?}", err);

        // Verify that the error is coming from the JWT library
        assert!(
            err_msg.contains("Base64") || err_msg.contains("Invalid"),
            "Expected JWT decoding error, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_verify_vc_success() -> anyhow::Result<()> {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        use chrono::Utc;
        use jsonwebtoken::{encode, EncodingKey, Header};
        use serde_json::json;

        let jwk = json!({
            "kty": "RSA",
            "alg": "RS256",
            "n": "y6fCcCNmPjERiyH9AwG9WLQyRk98phd6AAAN_PpYTUb3I8NmWflVj0hxldmJzOEKSBtYGznYPr9IUWrtsW-GWTx-QLv7sJoSaJ3uiIfoiJWxC_CrZCqpzfnN6DDd7ZL7noMrRkW7qUnU1mDftncpzQRnSeBbBHHO3IYceAiP6giofF73P0TZbUyG_eILIkGNetlPeoWkEAMYUqyPGcmK0sE_5JG1x9cIhzQ8FT9GrKTXZNz-af_2q-TRzG0TfMBRVhwRQ2x9xLEOb7DjCcBfk1NtW4zrSvFrLVAJXXroTauMLwO8DQSGgxDxxNkutsDjtI5X4iPb1Vsmx-gwcHOrqw",
            "e": "AQAB"
        });

        let jwk_encoded = URL_SAFE_NO_PAD.encode(jwk.to_string());
        let kid = format!("did:jwk:{}", jwk_encoded);

        let claims = json!({
            "iss": kid,
            "sub": kid,
            "jti": "vc-id-123",
            "vc": {
                "id": "vc-id-123",
                "issuer": { "id": kid },
                "credentialSubject": { "id": kid },
                "validFrom": Utc::now().to_rfc3339(),
                "validUntil": (Utc::now() + chrono::Duration::days(1)).to_rfc3339()
            },
            "nbf": Utc::now().timestamp()
        });

        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(kid.clone());

        let private_key_pem = PRIVATE_KEY_PEM;

        let token = encode(
            &header,
            &claims,
<<<<<<< HEAD
            &EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?
=======
            &EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?,
>>>>>>> origin/main
        )?;

        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo, config);

        let result = manager.verify_vc(token.clone(), kid.clone()).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_verify_vc_invalid_holder() {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        use chrono::Utc;
        use jsonwebtoken::{encode, EncodingKey, Header};
        use serde_json::json;

        let jwk = json!({
            "kty": "RSA",
            "alg": "RS256",
            "n": "y6fCcCNmPjERiyH9AwG9WLQyRk98phd6AAAN_PpYTUb3I8NmWflVj0hxldmJzOEKSBtYGznYPr9IUWrtsW-GWTx-QLv7sJoSaJ3uiIfoiJWxC_CrZCqpzfnN6DDd7ZL7noMrRkW7qUnU1mDftncpzQRnSeBbBHHO3IYceAiP6giofF73P0TZbUyG_eILIkGNetlPeoWkEAMYUqyPGcmK0sE_5JG1x9cIhzQ8FT9GrKTXZNz-af_2q-TRzG0TfMBRVhwRQ2x9xLEOb7DjCcBfk1NtW4zrSvFrLVAJXXroTauMLwO8DQSGgxDxxNkutsDjtI5X4iPb1Vsmx-gwcHOrqw",
            "e": "AQAB"
        });

        let jwk_encoded = URL_SAFE_NO_PAD.encode(jwk.to_string());
        let kid = format!("did:jwk:{}", jwk_encoded);
        let wrong_holder = "did:jwk:wrongholder";

        let claims = json!({
            "iss": kid,
            "sub": wrong_holder,
            "jti": "vc-id-123",
            "vc": {
                "id": "vc-id-123",
                "issuer": { "id": kid },
                "credentialSubject": { "id": wrong_holder },
                "validFrom": Utc::now().to_rfc3339(),
                "validUntil": (Utc::now() + chrono::Duration::days(1)).to_rfc3339()
            },
            "nbf": Utc::now().timestamp()
        });

        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(kid.clone());

        let private_key_pem = PRIVATE_KEY_PEM;

        let token = encode(
            &header,
            &claims,
<<<<<<< HEAD
            &EncodingKey::from_rsa_pem(private_key_pem.as_bytes()).unwrap()
=======
            &EncodingKey::from_rsa_pem(private_key_pem.as_bytes()).unwrap(),
>>>>>>> origin/main
        )
        .unwrap();

        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo, config);

        let result = manager.verify_vc(token.clone(), kid.clone()).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let auth_err = err.downcast_ref::<AuthErrors>().expect("Expected AuthErrors");

        match auth_err {
            AuthErrors::SecurityError { cause, .. } => {
                let cause_text = cause.as_ref().expect("Expected cause");
                assert!(
                    cause_text
                        .contains("VCT token sub, credential subject & VP Holder do not match"),
                    "Expected holder mismatch error, got: {}",
                    cause_text
                );
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_end_verification_redirect_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo.clone(), config);

        // ID que existe en el mock y tiene método "redirect"
        let id = "cont-id-123".to_string();

        let result = manager.end_verification(id.clone()).await;
        assert!(result.is_ok());

        let redirect_uri = result.unwrap();
        assert!(redirect_uri.is_some());

        let uri = redirect_uri.unwrap();
        assert!(uri.starts_with("https://example.com/callback"));
        assert!(uri.contains("hash=hash789"));
        assert!(uri.contains("interact_ref=ref-456"));

        Ok(())
    }

    #[tokio::test]
    async fn test_end_verification_invalid_method() {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo.clone(), config);

        let id = "invalid-method-id".to_string();

        let result = manager.end_verification(id.clone()).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let common_err = err.downcast_ref::<CommonErrors>().expect("Expected CommonErrors");

        match common_err {
            CommonErrors::FeatureNotImplError { cause, .. } => {
                let cause_text = cause.as_ref().expect("Expected cause");
                assert!(
                    cause_text.contains("Interact method unknown not supported"),
                    "Expected cause to mention unsupported method, got: {}",
                    cause_text
                );
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_fast_login_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo.clone(), config);

        let state = "test-state".to_string();
        let result = manager.fast_login(state.clone()).await;

        assert!(result.is_ok());
        let uri = result.unwrap();

        assert!(uri.starts_with("openid4vp://authorize"));
        assert!(uri.contains("response_type=vp_token"));
        assert!(uri.contains("presentation_definition_uri"));
        assert!(uri.contains("response_uri"));
        assert!(uri.contains(&state));
        Ok(())
    }

    #[tokio::test]
    async fn test_fast_login_verification_db_error() {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo.clone(), config);

        let result = manager.fast_login("fail-verification".to_string()).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let common_err = err.downcast_ref::<CommonErrors>().expect("Expected CommonErrors");

        match common_err {
            CommonErrors::DatabaseError { cause, .. } => {
                assert!(cause.as_ref().unwrap().contains("Simulated DB failure in verification"));
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_verify_token_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo.clone(), config);

        let result = manager.verify_token("valid-token".to_string()).await;
        assert!(result.is_ok());

        let mate = result.unwrap();
        assert_eq!(mate.participant_id, "test-holder");
        assert_eq!(mate.token.unwrap(), "valid-token");
        Ok(())
    }

    #[tokio::test]
    async fn test_verify_token_invalid() {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo.clone(), config);

        let result = manager.verify_token("invalid-token".to_string()).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let auth_err = err.downcast_ref::<CommonErrors>().expect("Expected CommonErrors");

        match auth_err {
            CommonErrors::UnauthorizedError { cause, .. } => {
                assert!(cause.as_ref().unwrap().contains("Invalid token"));
            }
            _ => panic!("Unexpected error type")
        }
    }

    #[tokio::test]
    async fn test_retrieve_business_token_success() -> anyhow::Result<()> {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo.clone(), config);

        let id = "test-state".to_string(); // debe coincidir con el mock

        let result = manager.retrieve_business_token(id.clone()).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["token"], "token-abc");
        assert_eq!(json["mate"]["participant_id"], "test-holder");
        assert_eq!(json["mate"]["participant_slug"], "test-consumer");

        Ok(())
    }

    #[tokio::test]
    async fn test_retrieve_business_token_missing_mate() {
        let config = build_test_config();
        let repo = Arc::new(MockAuthRepoFactory::default());
        let manager = Manager::new(repo.clone(), config);

        let id = "test-state-fail".to_string(); // Business mate exists, but mate doesn't.

        let result = manager.retrieve_business_token(id.clone()).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let common_err = err.downcast_ref::<CommonErrors>().expect("Expected CommonErrors");

        match common_err {
            CommonErrors::MissingActionError { action, .. } => {
                assert_eq!(action, "Onboarding");
            }
            _ => panic!("Unexpected error type")
        }
    }
}
