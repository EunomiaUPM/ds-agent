<<<<<<< HEAD
// Tests corresponding to
// 'rainbow-auth\src\ssi_auth\consumer\core\impls\consumer_trait_impl.rs'

#[cfg(test)]
mod tests {
    use std::sync::Arc;

=======
// Tests corresponding to 'rainbow-auth\src\ssi_auth\consumer\core\impls\consumer_trait_impl.rs'

#[cfg(test)]
mod tests {
>>>>>>> origin/main
    use async_trait::async_trait;
    use base64::Engine;
    use chrono::{self};
    use mockall::mock;
    use mockall::predicate::eq;
    use mockall::predicate::*;
<<<<<<< HEAD
    use rainbow_auth::ssi_auth::common::types::entities::{
        ReachAuthority, InteractStart, WhatEntity
    };
    use rainbow_auth::ssi_auth::consumer::core::traits::consumer_trait::{
        MockRainbowSSIAuthConsumerManagerTrait, RainbowSSIAuthConsumerManagerTrait
=======
    use rainbow_auth::ssi_auth::common::types::entities::{ReachAuthority, InteractStart, WhatEntity};
    use rainbow_auth::ssi_auth::consumer::core::traits::consumer_trait::{
        MockRainbowSSIAuthConsumerManagerTrait, RainbowSSIAuthConsumerManagerTrait,
>>>>>>> origin/main
    };
    use rainbow_auth::ssi_auth::consumer::core::Manager;
    use rainbow_common::config::consumer_config::ApplicationConsumerConfig;
    use rainbow_db::auth_consumer::entities::auth_interaction::{
<<<<<<< HEAD
        Model as InteractionModel, NewModel as NewInteractionModel
=======
        Model as InteractionModel, NewModel as NewInteractionModel,
>>>>>>> origin/main
    };
    use rainbow_db::auth_consumer::entities::auth_token_requirements::Model as TokenModel;
    use rainbow_db::auth_consumer::entities::auth_verification::{Model, NewModel};
    use rainbow_db::auth_consumer::entities::{auth_request, authority_request, mates};
    use rainbow_db::auth_consumer::repo_factory::traits::AuthVerificationRepoTrait;
    use rainbow_db::auth_consumer::repo_factory::traits::AuthorityRequestRepoTrait;
    use rainbow_db::auth_consumer::repo_factory::traits::MatesRepoTrait;
    use rainbow_db::auth_consumer::repo_factory::{
        factory_trait::AuthRepoFactoryTrait,
<<<<<<< HEAD
        traits::{AuthInteractionRepoTrait, AuthRequestRepoTrait, AuthTokenRequirementsRepoTrait}
=======
        traits::{AuthInteractionRepoTrait, AuthRequestRepoTrait, AuthTokenRequirementsRepoTrait},
>>>>>>> origin/main
    };
    use rainbow_db::common::BasicRepoTrait;
    use serde_json::json;
    use sha2::{Digest, Sha256};
<<<<<<< HEAD
=======
    use std::sync::Arc;
>>>>>>> origin/main
    use urn::Urn;

    // === Mocks ===
    mock! {
        pub AuthorityRepo {}

        #[async_trait]
        impl BasicRepoTrait<rainbow_db::auth_consumer::entities::authority_request::Model, rainbow_db::auth_consumer::entities::authority_request::NewModel> for AuthorityRepo {
            async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> anyhow::Result<Vec<rainbow_db::auth_consumer::entities::authority_request::Model>>;
            async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<rainbow_db::auth_consumer::entities::authority_request::Model>>;
            async fn create(&self, model: rainbow_db::auth_consumer::entities::authority_request::NewModel) -> anyhow::Result<rainbow_db::auth_consumer::entities::authority_request::Model>;
            async fn update(&self, model: rainbow_db::auth_consumer::entities::authority_request::Model) -> anyhow::Result<rainbow_db::auth_consumer::entities::authority_request::Model>;
            async fn delete(&self, id: &str) -> anyhow::Result<()>;
        }

        #[async_trait]
        impl AuthorityRequestRepoTrait for AuthorityRepo {}
    }

    mock! {
        pub VerificationRepo {}

        #[async_trait]
        impl BasicRepoTrait<rainbow_db::auth_consumer::entities::auth_verification::Model, rainbow_db::auth_consumer::entities::auth_verification::NewModel> for VerificationRepo {
            async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> anyhow::Result<Vec<rainbow_db::auth_consumer::entities::auth_verification::Model>>;
            async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<rainbow_db::auth_consumer::entities::auth_verification::Model>>;
            async fn create(&self, model: rainbow_db::auth_consumer::entities::auth_verification::NewModel) -> anyhow::Result<rainbow_db::auth_consumer::entities::auth_verification::Model>;
            async fn update(&self, model: rainbow_db::auth_consumer::entities::auth_verification::Model) -> anyhow::Result<rainbow_db::auth_consumer::entities::auth_verification::Model>;
            async fn delete(&self, id: &str) -> anyhow::Result<()>;
        }

        #[async_trait]
        impl AuthVerificationRepoTrait for VerificationRepo {}
    }

    mock! {
        pub MatesRepo {}

        #[async_trait]
        impl BasicRepoTrait<rainbow_db::auth_consumer::entities::mates::Model, rainbow_db::auth_consumer::entities::mates::NewModel> for MatesRepo {
            async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> anyhow::Result<Vec<rainbow_db::auth_consumer::entities::mates::Model>>;
            async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<rainbow_db::auth_consumer::entities::mates::Model>>;
            async fn create(&self, model: rainbow_db::auth_consumer::entities::mates::NewModel) -> anyhow::Result<rainbow_db::auth_consumer::entities::mates::Model>;
            async fn update(&self, model: rainbow_db::auth_consumer::entities::mates::Model) -> anyhow::Result<rainbow_db::auth_consumer::entities::mates::Model>;
            async fn delete(&self, id: &str) -> anyhow::Result<()>;
        }

        #[async_trait]
        impl MatesRepoTrait for MatesRepo {
            async fn get_me(&self) -> anyhow::Result<Option<rainbow_db::auth_consumer::entities::mates::Model>>;
            async fn get_by_token(&self, token: &str) -> anyhow::Result<Option<rainbow_db::auth_consumer::entities::mates::Model>>;
            async fn force_create(&self, model: rainbow_db::auth_consumer::entities::mates::NewModel) -> anyhow::Result<rainbow_db::auth_consumer::entities::mates::Model>;
            async fn get_batch(&self, ids: &Vec<Urn>) -> anyhow::Result<Vec<rainbow_db::auth_consumer::entities::mates::Model>>;
        }
    }

    mock! {
        pub AuthVerificationRepoMock {}

        #[async_trait]
        impl BasicRepoTrait<Model, NewModel> for AuthVerificationRepoMock {
            async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> anyhow::Result<Vec<Model>>;
            async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<Model>>;
            async fn create(&self, model: NewModel) -> anyhow::Result<Model>;
            async fn update(&self, model: Model) -> anyhow::Result<Model>;
            async fn delete(&self, id: &str) -> anyhow::Result<()>;
        }
    }

    mock! {
        pub AuthRequestRepo {}

        // Implementación del trait base
        #[async_trait]
        impl BasicRepoTrait<rainbow_db::auth_consumer::entities::auth_request::Model, rainbow_db::auth_consumer::entities::auth_request::NewModel> for AuthRequestRepo {
            async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> anyhow::Result<Vec<rainbow_db::auth_consumer::entities::auth_request::Model>>;
            async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<rainbow_db::auth_consumer::entities::auth_request::Model>>;
            async fn create(&self, model: rainbow_db::auth_consumer::entities::auth_request::NewModel) -> anyhow::Result<rainbow_db::auth_consumer::entities::auth_request::Model>;
            async fn update(&self, model: rainbow_db::auth_consumer::entities::auth_request::Model) -> anyhow::Result<rainbow_db::auth_consumer::entities::auth_request::Model>;
            async fn delete(&self, id: &str) -> anyhow::Result<()>;
        }

        // Implementación del trait extendido
        impl AuthRequestRepoTrait for AuthRequestRepo {}
    }

    mock! {
        pub InteractionRepo {}

        #[async_trait]
        impl BasicRepoTrait<InteractionModel, NewInteractionModel> for InteractionRepo {
            async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> anyhow::Result<Vec<InteractionModel>>;
            async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<InteractionModel>>;
            async fn create(&self, model: NewInteractionModel) -> anyhow::Result<InteractionModel>;
            async fn update(&self, model: InteractionModel) -> anyhow::Result<InteractionModel>;
            async fn delete(&self, id: &str) -> anyhow::Result<()>;
        }

        impl AuthInteractionRepoTrait for InteractionRepo {}
    }

    mock! {
        pub TokenRequirementsRepo {}

        #[async_trait]
        impl BasicRepoTrait<TokenModel, TokenModel> for TokenRequirementsRepo {
            async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> anyhow::Result<Vec<TokenModel>>;
            async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<TokenModel>>;
            async fn create(&self, model: TokenModel) -> anyhow::Result<TokenModel>;
            async fn update(&self, model: TokenModel) -> anyhow::Result<TokenModel>;
            async fn delete(&self, id: &str) -> anyhow::Result<()>;
        }

        impl AuthTokenRequirementsRepoTrait for TokenRequirementsRepo {}
    }

    mock! {
        pub RepoFactory {}

        impl AuthRepoFactoryTrait for RepoFactory {
            fn request(&self) -> Arc<dyn AuthRequestRepoTrait>;
            fn interaction(&self) -> Arc<dyn AuthInteractionRepoTrait>;
            fn verification(&self) -> Arc<dyn AuthVerificationRepoTrait>;
            fn token_requirements(&self) -> Arc<dyn AuthTokenRequirementsRepoTrait>;
            fn mates(&self) -> Arc<dyn MatesRepoTrait>;
            fn authority(&self) -> Arc<dyn AuthorityRequestRepoTrait>;
        }
    }

    impl Clone for MockRepoFactory {
        fn clone(&self) -> Self { Self::new() }
    }

    #[tokio::test]
    async fn test_request_onboard_provider_success() {
        let mut manager = MockRainbowSSIAuthConsumerManagerTrait::new();

        manager
            .expect_request_onboard_provider()
            .with(
                eq("http://provider.com".to_string()),
                eq("provider_id".to_string()),
                eq("provider_slug".to_string())
            )
            .returning(|_, _, _| Ok("http://example.com/callback".to_string()));

        let result = manager
            .request_onboard_provider(
                "http://provider.com".to_string(),
                "provider_id".to_string(),
<<<<<<< HEAD
                "provider_slug".to_string()
=======
                "provider_slug".to_string(),
>>>>>>> origin/main
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://example.com/callback");
    }

    #[tokio::test]
    async fn test_request_onboard_provider_error_handling() {
        // Create mock
        let mut manager = MockRainbowSSIAuthConsumerManagerTrait::new();

        // Set the expectation for the function to fail, not return OK.
        manager
            .expect_request_onboard_provider()
            .withf(|url, provider_id, provider_slug| {
                url == "http://provider.com" && provider_id == "provider_id" && provider_slug == "provider_slug"
            })
            .times(1)
            .returning(|_, _, _| {
                // Return an error to force the failure branch
                Err(anyhow::anyhow!("Database error"))
            });

        // Call to the function to test
        let result = manager
            .request_onboard_provider(
                "http://provider.com".to_string(),
                "provider_id".to_string(),
                "provider_slug".to_string()
            )
            .await;

        // Verify the result: it should be Err
        assert!(result.is_err(), "Error was expected but it was Ok");

        // Verify that the error message is as expected.
        if let Err(e) = result {
            assert_eq!(e.to_string(), "Database error");
        }
    }

    #[tokio::test]
    async fn test_check_callback_success() {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;

        let mut mock_interaction_repo = MockInteractionRepo::new();

        let id = "test-id".to_string();
        let id_for_mock = id.clone();
        let interact_ref = "mock_ref".to_string();
        let client_nonce = "nonce123";
        let as_nonce = "as_nonce456";
        let grant_endpoint = "https://provider.com/grant";

<<<<<<< HEAD
        let hash_input =
            format!("{}\n{}\n{}\n{}", client_nonce, as_nonce, interact_ref, grant_endpoint);
=======
        let hash_input = format!(
            "{}\n{}\n{}\n{}",
            client_nonce, as_nonce, interact_ref, grant_endpoint
        );
>>>>>>> origin/main
        let mut hasher = Sha256::new();
        hasher.update(hash_input.as_bytes());
        let calculated_hash = URL_SAFE_NO_PAD.encode(hasher.finalize());

        mock_interaction_repo.expect_get_by_id().returning(move |_| {
            Ok(Some(InteractionModel {
                id: id_for_mock.clone(),
                client_nonce: client_nonce.to_string(),
                as_nonce: Some(as_nonce.to_string()),
                interact_ref: None,
                hash: None,
                grant_endpoint: grant_endpoint.to_string(),
                continue_token: None,
                continue_endpoint: None,
                continue_wait: None,
                method: "push".to_string(),
                uri: "https://callback".to_string(),
                hints: None,
                hash_method: "push".to_string(),
                start: vec!["start".to_string()]
            }))
        });

        mock_interaction_repo.expect_update().returning(|model| Ok(model));

        let mut mock_factory = MockRepoFactory::new();
        mock_factory
            .expect_interaction()
            .return_const(Arc::new(mock_interaction_repo) as Arc<dyn AuthInteractionRepoTrait>);

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_factory), config);

        let result = manager.check_callback(id, interact_ref, calculated_hash).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_callback_security_error() {
        let id = "test-id".to_string();
        let interact_ref = "mock_ref".to_string();
        let wrong_hash = "invalid_hash".to_string();

        let mut mock_interaction_repo = MockInteractionRepo::new();
        let id_for_mock = id.clone();

        mock_interaction_repo.expect_get_by_id().returning(move |_| {
            Ok(Some(InteractionModel {
                id: id_for_mock.clone(),
                client_nonce: "nonce123".to_string(),
                as_nonce: Some("as_nonce456".to_string()),
                interact_ref: None,
                hash: None,
                grant_endpoint: "https://provider.com/grant".to_string(),
                continue_token: None,
                continue_endpoint: None,
                continue_wait: None,
                method: "push".to_string(),
                uri: "https://callback".to_string(),
                hints: None,
                hash_method: "push".to_string(),
                start: vec!["start".to_string()]
            }))
        });

        mock_interaction_repo.expect_update().returning(|model| Ok(model));

        let mut mock_factory = MockRepoFactory::new();
        mock_factory
            .expect_interaction()
            .return_const(Arc::new(mock_interaction_repo) as Arc<dyn AuthInteractionRepoTrait>);

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_factory), config);

        let result = manager.check_callback(id, interact_ref, wrong_hash).await;

        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("Security Error"));
    }

    #[tokio::test]
    async fn test_continue_request_success() {
        use serde_json::json;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Simulates a valid JSON response from the external server
        let token_response = json!({
            "value": "access_token_value",
            "label": "label",
            "access": ["read", "write"],
            "expires_in": 3600,
            "flags": ["flag1"]
        });

        Mock::given(method("POST"))
            .and(path("/continue"))
            .respond_with(ResponseTemplate::new(200).set_body_json(token_response))
            .mount(&mock_server)
            .await;

        let id = "test-id".to_string();
        let interact_ref = "mock_ref".to_string();
        let id_for_mock = id.clone();

        let mut mock_repo_factory = MockRepoFactory::new();
        let mut mock_interaction_repo = MockInteractionRepo::new();
        let mut mock_request_repo = MockAuthRequestRepo::new();
        let mut mock_mates_repo = MockMatesRepo::new();

        let continue_url = format!("{}/continue", &mock_server.uri());

        // Mock interaction
        let id_interaction = id_for_mock.clone();
        mock_interaction_repo.expect_get_by_id().returning(move |_| {
            Ok(Some(InteractionModel {
                id: id_interaction.clone(),
                client_nonce: "nonce123".to_string(),
                as_nonce: Some("as_nonce456".to_string()),
                interact_ref: None,
                hash: None,
                grant_endpoint: "https://provider.com/grant".to_string(),
                continue_token: Some("token123".to_string()),
                continue_endpoint: Some(continue_url.clone()),
                continue_wait: None,
                method: "push".to_string(),
                uri: "https://callback".to_string(),
                hints: None,
                hash_method: "push".to_string(),
                start: vec!["start".to_string()]
            }))
        });

        // Application mockup
        let id_request = id_for_mock.clone();
        mock_request_repo.expect_get_by_id().returning(move |_| {
<<<<<<< HEAD
            Ok(Some(rainbow_db::auth_consumer::entities::auth_request::Model {
                id: id_request.clone(),
                provider_id: "provider123".to_string(),
                provider_slug: "provider-slug".to_string(),
                grant_endpoint: "https://provider.com/grant".to_string(),
                assigned_id: Some("assigned-id-456".to_string()),
                token: None,
                status: "Pending".to_string(),
                created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap(),
                ended_at: None
            }))
=======
            Ok(Some(
                rainbow_db::auth_consumer::entities::auth_request::Model {
                    id: id_request.clone(),
                    provider_id: "provider123".to_string(),
                    provider_slug: "provider-slug".to_string(),
                    grant_endpoint: "https://provider.com/grant".to_string(),
                    assigned_id: Some("assigned-id-456".to_string()),
                    token: None,
                    status: "Pending".to_string(),
                    created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(12, 0, 0).unwrap(),
                    ended_at: None,
                },
            ))
>>>>>>> origin/main
        });

        mock_request_repo.expect_update().returning(|model| Ok(model));

        mock_mates_repo.expect_force_create().returning(|model| {
            Ok(rainbow_db::auth_consumer::entities::mates::Model {
                participant_id: model.participant_id,
                participant_slug: model.participant_slug,
                participant_type: model.participant_type,
                base_url: model.base_url,
                token: model.token,
<<<<<<< HEAD
                saved_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                last_interaction: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                is_me: model.is_me
=======
                saved_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                last_interaction: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                is_me: model.is_me,
>>>>>>> origin/main
            })
        });

        // Inject the repos into the factory
        mock_repo_factory
            .expect_interaction()
            .return_const(Arc::new(mock_interaction_repo) as Arc<dyn AuthInteractionRepoTrait>);
<<<<<<< HEAD
        mock_repo_factory
            .expect_request()
            .return_const(Arc::new(mock_request_repo) as Arc<dyn AuthRequestRepoTrait>);
        mock_repo_factory
            .expect_mates()
            .return_const(Arc::new(mock_mates_repo) as Arc<dyn MatesRepoTrait>);
=======
        mock_repo_factory.expect_request().return_const(Arc::new(mock_request_repo) as Arc<dyn AuthRequestRepoTrait>);
        mock_repo_factory.expect_mates().return_const(Arc::new(mock_mates_repo) as Arc<dyn MatesRepoTrait>);
>>>>>>> origin/main

        let config = ApplicationConsumerConfig::default();
        let mut manager = Manager::new(Arc::new(mock_repo_factory), config);

        manager.client = reqwest::Client::new();

        let result = manager.continue_request(id, interact_ref).await;

        if let Err(e) = &result {
            println!("Error en continue_request: {}", e);
        }

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["token"], "access_token_value");
    }

    #[tokio::test]
    async fn test_continue_request_error_response() {
        use serde_json::json;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Simulates server error response
        let error_response = json!({
            "error": "invalid_request"
        });

        Mock::given(method("POST"))
            .and(path("/continue"))
            .respond_with(ResponseTemplate::new(400).set_body_json(error_response))
            .mount(&mock_server)
            .await;

        let id = "test-id".to_string();
        let interact_ref = "mock_ref".to_string();
        let id_for_mock = id.clone();

        let mut mock_repo_factory = MockRepoFactory::new();
        let mut mock_interaction_repo = MockInteractionRepo::new();
        let mut mock_request_repo = MockAuthRequestRepo::new();
        let mut mock_mates_repo = MockMatesRepo::new();

        let continue_url = format!("{}/continue", &mock_server.uri());

        // Mock interaction
        let id_interaction = id_for_mock.clone();
        mock_interaction_repo.expect_get_by_id().returning(move |_| {
            Ok(Some(InteractionModel {
                id: id_interaction.clone(),
                client_nonce: "nonce123".to_string(),
                as_nonce: Some("as_nonce456".to_string()),
                interact_ref: None,
                hash: None,
                grant_endpoint: "https://provider.com/grant".to_string(),
                continue_token: Some("token123".to_string()),
                continue_endpoint: Some(continue_url.clone()),
                continue_wait: None,
                method: "push".to_string(),
                uri: "https://callback".to_string(),
                hints: None,
                hash_method: "push".to_string(),
                start: vec!["start".to_string()]
            }))
        });

        // Application mockup
        let id_request = id_for_mock.clone();
        mock_request_repo.expect_get_by_id().returning(move |_| {
<<<<<<< HEAD
            Ok(Some(rainbow_db::auth_consumer::entities::auth_request::Model {
                id: id_request.clone(),
                provider_id: "provider123".to_string(),
                provider_slug: "provider-slug".to_string(),
                grant_endpoint: "https://provider.com/grant".to_string(),
                assigned_id: Some("assigned-id-456".to_string()),
                token: None,
                status: "Pending".to_string(),
                created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                ended_at: None
            }))
=======
            Ok(Some(
                rainbow_db::auth_consumer::entities::auth_request::Model {
                    id: id_request.clone(),
                    provider_id: "provider123".to_string(),
                    provider_slug: "provider-slug".to_string(),
                    grant_endpoint: "https://provider.com/grant".to_string(),
                    assigned_id: Some("assigned-id-456".to_string()),
                    token: None,
                    status: "Pending".to_string(),
                    created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                    ended_at: None,
                },
            ))
>>>>>>> origin/main
        });

        mock_request_repo.expect_update().returning(|model| Ok(model));

        // Mock dunks (will not be used in this test because it fails before)
<<<<<<< HEAD
        mock_mates_repo
            .expect_force_create()
            .returning(|_| unreachable!("No debería llamarse en caso de error"));
=======
        mock_mates_repo.expect_force_create().returning(|_| unreachable!("No debería llamarse en caso de error"));
>>>>>>> origin/main

        mock_repo_factory
            .expect_interaction()
            .return_const(Arc::new(mock_interaction_repo) as Arc<dyn AuthInteractionRepoTrait>);
<<<<<<< HEAD
        mock_repo_factory
            .expect_request()
            .return_const(Arc::new(mock_request_repo) as Arc<dyn AuthRequestRepoTrait>);
        mock_repo_factory
            .expect_mates()
            .return_const(Arc::new(mock_mates_repo) as Arc<dyn MatesRepoTrait>);
=======
        mock_repo_factory.expect_request().return_const(Arc::new(mock_request_repo) as Arc<dyn AuthRequestRepoTrait>);
        mock_repo_factory.expect_mates().return_const(Arc::new(mock_mates_repo) as Arc<dyn MatesRepoTrait>);
>>>>>>> origin/main

        let config = ApplicationConsumerConfig::default();
        let mut manager = Manager::new(Arc::new(mock_repo_factory), config);

        manager.client = reqwest::Client::new();

        let result = manager.continue_request(id, interact_ref).await;

        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Provider Error"));
    }

    #[tokio::test]
    async fn test_save_mate_success() {
        let mut mock_repo_factory = MockRepoFactory::new();
        let mut mock_mates_repo = MockMatesRepo::new();

        let new_mate = mates::NewModel {
            participant_id: "provider123".to_string(),
            participant_slug: "provider-slug".to_string(),
            participant_type: "Agent".to_string(),
            base_url: "https://provider.com".to_string(),
            token: Some("access_token_value".to_string()),
            is_me: false
        };

        let expected_model = mates::Model {
            participant_id: new_mate.participant_id.clone(),
            participant_slug: new_mate.participant_slug.clone(),
            participant_type: new_mate.participant_type.clone(),
            base_url: new_mate.base_url.clone(),
            token: new_mate.token.clone(),
            is_me: new_mate.is_me,
            saved_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            last_interaction: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        };

        mock_mates_repo.expect_force_create().returning(move |_| Ok(expected_model.clone()));

<<<<<<< HEAD
        mock_repo_factory
            .expect_mates()
            .return_const(Arc::new(mock_mates_repo) as Arc<dyn MatesRepoTrait>);
=======
        mock_repo_factory.expect_mates().return_const(Arc::new(mock_mates_repo) as Arc<dyn MatesRepoTrait>);
>>>>>>> origin/main

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_repo_factory), config);

        let result = manager.save_mate(new_mate).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            mates::Model {
                participant_id: "provider123".to_string(),
                participant_slug: "provider-slug".to_string(),
                participant_type: "Agent".to_string(),
                base_url: "https://provider.com".to_string(),
                token: Some("access_token_value".to_string()),
                is_me: false,
<<<<<<< HEAD
                saved_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                last_interaction: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
=======
                saved_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                last_interaction: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
>>>>>>> origin/main
            }
        );
    }

    #[tokio::test]
    async fn test_save_mate_error() {
        let mut mock_repo_factory = MockRepoFactory::new();
        let mut mock_mates_repo = MockMatesRepo::new();

        let new_mate = mates::NewModel {
            participant_id: "provider123".to_string(),
            participant_slug: "provider-slug".to_string(),
            participant_type: "Agent".to_string(),
            base_url: "https://provider.com".to_string(),
            token: Some("access_token_value".to_string()),
            is_me: false
        };

        mock_mates_repo.expect_force_create().returning(|_| Err(anyhow::anyhow!("DB error")));

<<<<<<< HEAD
        mock_repo_factory
            .expect_mates()
            .return_const(Arc::new(mock_mates_repo) as Arc<dyn MatesRepoTrait>);
=======
        mock_repo_factory.expect_mates().return_const(Arc::new(mock_mates_repo) as Arc<dyn MatesRepoTrait>);
>>>>>>> origin/main

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_repo_factory), config);

        let result = manager.save_mate(new_mate).await;

        println!("Resul: {:?}", result);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Database Error"));
    }

    #[tokio::test]
    async fn test_complete_ver_process_success() {
        let id = "ver-id".to_string();
        let url = "https://provider.com/verify".to_string();
        let uri = Some("openid4vp://example.com?response_type=id_token&client_id=client123&response_mode=post&presentation_definition_uri=https://pd.com/def&client_id_scheme=did&nonce=abc123&response_uri=https://provider.com/response".to_string());

        let mut mock_verification_repo = MockVerificationRepo::new();

        mock_verification_repo.expect_create().returning(|model| {
<<<<<<< HEAD
            Ok(rainbow_db::auth_consumer::entities::auth_verification::Model {
                id: model.id,
                status: "pending".to_string(),
                created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                ended_at: Some(
                    chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                ),
                uri: model.uri,
                scheme: model.scheme,
                response_type: model.response_type,
                client_id: model.client_id,
                response_mode: model.response_mode,
                pd_uri: model.pd_uri,
                client_id_scheme: model.client_id_scheme,
                nonce: model.nonce,
                response_uri: model.response_uri
            })
=======
            Ok(
                rainbow_db::auth_consumer::entities::auth_verification::Model {
                    id: model.id,
                    status: "pending".to_string(),
                    created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                    ended_at: Some(chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
                    uri: model.uri,
                    scheme: model.scheme,
                    response_type: model.response_type,
                    client_id: model.client_id,
                    response_mode: model.response_mode,
                    pd_uri: model.pd_uri,
                    client_id_scheme: model.client_id_scheme,
                    nonce: model.nonce,
                    response_uri: model.response_uri,
                },
            )
>>>>>>> origin/main
        });

        let mut mock_repo_factory = MockRepoFactory::new();
        mock_repo_factory
            .expect_verification()
            .return_const(Arc::new(mock_verification_repo) as Arc<dyn AuthVerificationRepoTrait>);

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_repo_factory), config);

        let result = manager.complete_ver_proccess(uri.clone(), url.clone(), id.clone()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), uri.unwrap());
    }

    #[tokio::test]
    async fn test_complete_ver_process_missing_response_type_error() {
        let id = "ver-id".to_string();
        let url = "https://provider.com/verify".to_string();
        let uri = Some("openid4vp://example.com?client_id=client123&response_mode=post&presentation_definition_uri=https://pd.com/def&client_id_scheme=did&nonce=abc123&response_uri=https://provider.com/response".to_string());

        let mut mock_verification_repo = MockVerificationRepo::new();

        // No se espera que se llame a create porque debe fallar antes
        mock_verification_repo.expect_create().never();

        let mut mock_repo_factory = MockRepoFactory::new();
        mock_repo_factory
            .expect_verification()
            .return_const(Arc::new(mock_verification_repo) as Arc<dyn AuthVerificationRepoTrait>);

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_repo_factory), config);

        let result = manager.complete_ver_proccess(uri.clone(), url.clone(), id.clone()).await;

        println!("{:?}", result);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();

        assert!(
            error_msg.contains("Provider Error"),
            "Mensaje de error inesperado: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_beg_credential_success_until_http() {
        let payload = ReachAuthority {
            id: "auth-id".to_string(),
            slug: "auth-slug".to_string(),
            url: "https://authority.com/grant".to_string(),
            vc_type: "VerifiableCredential".to_string()
        };

        let method = InteractStart::Oidc;

        let mut mock_authority_repo = MockAuthorityRepo::new();
        mock_authority_repo.expect_create().returning(|model| {
<<<<<<< HEAD
            Ok(rainbow_db::auth_consumer::entities::authority_request::Model {
                id: model.id.clone(),
                authority_id: model.authority_id.clone(),
                authority_slug: model.authority_slug.clone(),
                grant_endpoint: model.grant_endpoint.clone(),
                vc_type: model.vc_type.clone(),
                status: "Created".to_string(),
                assigned_id: None,
                vc_uri: Some("vc_uri".to_string()),
                created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                ended_at: Some(
                    chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                )
            })
=======
            Ok(
                rainbow_db::auth_consumer::entities::authority_request::Model {
                    id: model.id.clone(),
                    authority_id: model.authority_id.clone(),
                    authority_slug: model.authority_slug.clone(),
                    grant_endpoint: model.grant_endpoint.clone(),
                    vc_type: model.vc_type.clone(),
                    status: "Created".to_string(),
                    assigned_id: None,
                    vc_uri: Some("vc_uri".to_string()),
                    created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                    ended_at: Some(chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
                },
            )
>>>>>>> origin/main
        });
        mock_authority_repo.expect_update().returning(|model| Ok(model));

        let mut mock_interaction_repo = MockInteractionRepo::new();
        mock_interaction_repo.expect_create().returning(|model| {
<<<<<<< HEAD
            Ok(rainbow_db::auth_consumer::entities::auth_interaction::Model {
                id: model.id.clone(),
                start: model.start.clone(),
                method: model.method.clone(),
                uri: model.uri.clone(),
                hash_method: "hash_method".to_string(),
                hints: None,
                grant_endpoint: model.grant_endpoint.clone(),
                client_nonce: "nonce123".to_string(),
                as_nonce: Some("as_nonce".to_string()),
                continue_token: None,
                continue_endpoint: None,
                continue_wait: Some(5),
                interact_ref: Some("interact_ref".to_string()),
                hash: Some("hash".to_string())
            })
=======
            Ok(
                rainbow_db::auth_consumer::entities::auth_interaction::Model {
                    id: model.id.clone(),
                    start: model.start.clone(),
                    method: model.method.clone(),
                    uri: model.uri.clone(),
                    hash_method: "hash_method".to_string(),
                    hints: None,
                    grant_endpoint: model.grant_endpoint.clone(),
                    client_nonce: "nonce123".to_string(),
                    as_nonce: Some("as_nonce".to_string()),
                    continue_token: None,
                    continue_endpoint: None,
                    continue_wait: Some(5),
                    interact_ref: Some("interact_ref".to_string()),
                    hash: Some("hash".to_string()),
                },
            )
>>>>>>> origin/main
        });
        mock_interaction_repo.expect_update().returning(|model| Ok(model));

        let mut mock_repo_factory = MockRepoFactory::new();
        mock_repo_factory
            .expect_authority()
            .return_const(Arc::new(mock_authority_repo) as Arc<dyn AuthorityRequestRepoTrait>);
        mock_repo_factory
            .expect_interaction()
            .return_const(Arc::new(mock_interaction_repo) as Arc<dyn AuthInteractionRepoTrait>);

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_repo_factory), config);

        let result = manager.beg_credential(payload, method).await;

        // Verificamos que el flujo llegue hasta la petición HTTP
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Petition Error") || error_msg.contains("authority"),
            "Error esperado en la petición HTTP, pero se obtuvo: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_beg_credential_authority_create_error() {
        let payload = ReachAuthority {
            id: "auth-id".to_string(),
            slug: "auth-slug".to_string(),
            url: "https://authority.com/grant".to_string(),
            vc_type: "VerifiableCredential".to_string()
        };

        let method = InteractStart::Oidc;

        let mut mock_authority_repo = MockAuthorityRepo::new();
<<<<<<< HEAD
        mock_authority_repo
            .expect_create()
            .returning(|_model| Err(anyhow::anyhow!("Error al crear autoridad")));
=======
        mock_authority_repo.expect_create().returning(|_model| Err(anyhow::anyhow!("Error al crear autoridad")));
>>>>>>> origin/main

        let mut mock_repo_factory = MockRepoFactory::new();
        mock_repo_factory
            .expect_authority()
            .return_const(Arc::new(mock_authority_repo) as Arc<dyn AuthorityRequestRepoTrait>);

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_repo_factory), config);

        let result = manager.beg_credential(payload, method).await;

        print!("{:?}", result);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Database Error"));
    }

    #[tokio::test]
    async fn test_who_is_it_provider() {
        let id = "provider-id".to_string();

        let mut mock_request_repo = MockAuthRequestRepo::new();
        mock_request_repo
            .expect_get_by_id()
            .withf(|input_id| *input_id.to_string() == "provider-id".to_string())
            .returning(|_| {
                Ok(Some(auth_request::Model {
                    id: "provider-id".to_string(),
                    provider_id: "provider123".to_string(),
                    provider_slug: "slug".to_string(),
                    grant_endpoint: "https://provider.com".to_string(),
                    assigned_id: Some("assigned".to_string()),
                    token: Some("token".to_string()),
                    status: "Pending".to_string(),
<<<<<<< HEAD
                    created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                    ended_at: None
=======
                    created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                    ended_at: None,
>>>>>>> origin/main
                }))
            });

        let mut mock_authority_repo = MockAuthorityRepo::new();
        mock_authority_repo.expect_get_by_id().returning(|_| Ok(None));

        let mut mock_repo_factory = MockRepoFactory::new();
<<<<<<< HEAD
        mock_repo_factory
            .expect_request()
            .return_const(Arc::new(mock_request_repo) as Arc<dyn AuthRequestRepoTrait>);
=======
        mock_repo_factory.expect_request().return_const(Arc::new(mock_request_repo) as Arc<dyn AuthRequestRepoTrait>);
>>>>>>> origin/main
        mock_repo_factory
            .expect_authority()
            .return_const(Arc::new(mock_authority_repo) as Arc<dyn AuthorityRequestRepoTrait>);

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_repo_factory), config);

        let result = manager.who_is_it(id.clone()).await;

        assert!(result.is_ok());
        let (entity, provider_model, authority_model) = result.unwrap();
        assert_eq!(entity, WhatEntity::Provider);
        assert!(provider_model.is_some());
        assert!(authority_model.is_none());
    }

    #[tokio::test]
    async fn test_who_is_it_authority() {
        let id = "authority-id".to_string();

        let mut mock_request_repo = MockAuthRequestRepo::new();
        mock_request_repo.expect_get_by_id().returning(|_| Ok(None));

        let mut mock_authority_repo = MockAuthorityRepo::new();
        mock_authority_repo
            .expect_get_by_id()
            .withf(|input_id| *input_id.to_string() == "authority-id".to_string())
            .returning(|_| {
                Ok(Some(authority_request::Model {
                    id: "authority-id".to_string(),
                    authority_id: "auth123".to_string(),
                    authority_slug: "slug".to_string(),
                    grant_endpoint: "https://authority.com".to_string(),
                    vc_type: "VerifiableCredential".to_string(),
                    status: "Pending".to_string(),
                    assigned_id: Some("assigned".to_string()),
                    vc_uri: Some("vc_uri".to_string()),
<<<<<<< HEAD
                    created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                    ended_at: None
=======
                    created_at: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                    ended_at: None,
>>>>>>> origin/main
                }))
            });

        let mut mock_repo_factory = MockRepoFactory::new();
<<<<<<< HEAD
        mock_repo_factory
            .expect_request()
            .return_const(Arc::new(mock_request_repo) as Arc<dyn AuthRequestRepoTrait>);
=======
        mock_repo_factory.expect_request().return_const(Arc::new(mock_request_repo) as Arc<dyn AuthRequestRepoTrait>);
>>>>>>> origin/main
        mock_repo_factory
            .expect_authority()
            .return_const(Arc::new(mock_authority_repo) as Arc<dyn AuthorityRequestRepoTrait>);

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_repo_factory), config);

        let result = manager.who_is_it(id.clone()).await;

        assert!(result.is_ok());
        let (entity, provider_model, authority_model) = result.unwrap();
        assert_eq!(entity, WhatEntity::Authority);
        assert!(provider_model.is_none());
        assert!(authority_model.is_some());
    }

    #[tokio::test]
    async fn test_who_is_it_missing_resource_error() {
        let id = "non-existent-id".to_string();

        let mut mock_request_repo = MockAuthRequestRepo::new();
        mock_request_repo
            .expect_get_by_id()
            .withf(|input_id| *input_id.to_string() == "non-existent-id".to_string())
            .returning(|_| Ok(None));

        let mut mock_authority_repo = MockAuthorityRepo::new();
        mock_authority_repo
            .expect_get_by_id()
            .withf(|input_id| *input_id.to_string() == "non-existent-id".to_string())
            .returning(|_| Ok(None));

        let mut mock_repo_factory = MockRepoFactory::new();
<<<<<<< HEAD
        mock_repo_factory
            .expect_request()
            .return_const(Arc::new(mock_request_repo) as Arc<dyn AuthRequestRepoTrait>);
=======
        mock_repo_factory.expect_request().return_const(Arc::new(mock_request_repo) as Arc<dyn AuthRequestRepoTrait>);
>>>>>>> origin/main
        mock_repo_factory
            .expect_authority()
            .return_const(Arc::new(mock_authority_repo) as Arc<dyn AuthorityRequestRepoTrait>);

        let config = ApplicationConsumerConfig::default();
        let manager = Manager::new(Arc::new(mock_repo_factory), config);

        let result = manager.who_is_it(id.clone()).await;

        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Missing Resource Error"),
            "Mensaje de error inesperado: {}",
            error_msg
        );
    }
}
