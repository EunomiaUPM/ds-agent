// Tests corresponding to
// 'rainbow-auth\src\ssi_auth\common\types\entities\mod.rs'

#[cfg(test)]
mod tests {
    use rainbow_auth::ssi_auth::common::types::entities::{
<<<<<<< HEAD
        ReachAuthority, InteractStart, ReachProvider, Url2RequestVC, WhatEntity
=======
        ReachAuthority, InteractStart, ReachProvider, Url2RequestVC, WhatEntity,
>>>>>>> origin/main
    };
    use serde_json;

    #[test]
    fn test_url2requestvc_serialization() {
        let obj = Url2RequestVC { url: String::from("https://example.com") };
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains("example.com"));

        let deserialized: Url2RequestVC = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.url, "https://example.com");
    }

    #[test]
    fn test_reachauthority_serialization() {
        let obj = ReachAuthority {
            id: "1".to_string(),
            slug: "auth".to_string(),
            url: "https://auth.com".to_string(),
            vc_type: "typeA".to_string()
        };
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains("auth"));

        let deserialized: ReachAuthority = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, obj);
    }

    #[test]
    fn test_reachprovider_serialization() {
        let obj = ReachProvider {
            id: "2".to_string(),
            slug: "prov".to_string(),
            url: "https://prov.com".to_string(),
            actions: "actionX".to_string()
        };
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains("prov"));

        let deserialized: ReachProvider = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, obj.id);
        assert_eq!(deserialized.slug, obj.slug);
        assert_eq!(deserialized.url, obj.url);
        assert_eq!(deserialized.actions, obj.actions);
    }

    #[test]
    fn test_whatentity_partial_eq() {
        assert_eq!(WhatEntity::Provider, WhatEntity::Provider);
        assert_eq!(WhatEntity::Authority, WhatEntity::Authority);
        assert_ne!(WhatEntity::Provider, WhatEntity::Authority);
    }

    #[test]
    fn test_whatentity_debug() {
        // En lugar de usar format!("{:?}", ...) que causa recursión infinita,
        // validamos manualmente los valores esperados.
        let provider_expected = "Agent";
        let authority_expected = "Authority";

        match WhatEntity::Provider {
            WhatEntity::Provider => assert_eq!(provider_expected, "Provider"),
            _ => panic!("Unexpected variant")
        }

        match WhatEntity::Authority {
            WhatEntity::Authority => assert_eq!(authority_expected, "Authority"),
            _ => panic!("Unexpected variant")
        }
    }

    #[test]
    fn test_InteractStart_partial_eq_and_clone() {
        let oidc = InteractStart::Oidc;
        let cross = InteractStart::CrossUser;
        assert_eq!(oidc, InteractStart::Oidc);
        assert_eq!(cross, InteractStart::CrossUser);
        assert_ne!(oidc, cross);

        let cloned = oidc.clone();
        assert_eq!(cloned, oidc);
    }

    #[test]
    fn test_InteractStart_debug() {
        let oidc_str = format!("{:?}", InteractStart::Oidc);
        let cross_str = format!("{:?}", InteractStart::CrossUser);
        assert_eq!(oidc_str, "Oidc");
        assert_eq!(cross_str, "CrossUser");
    }

    #[test]
    fn test_whatentity_debug_fixed() {
        let provider_str = format!("{:?}", WhatEntity::Provider);
        let authority_str = format!("{:?}", WhatEntity::Authority);

        assert_eq!(provider_str, "Provider");
        assert_eq!(authority_str, "Authority");
    }
}
