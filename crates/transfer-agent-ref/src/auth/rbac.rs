use oauth::entities::Role;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::auth::extractor::AuthClaims;

impl AuthClaims {
    /// Caller may read resources belonging to `tenant_id`.
    /// Admin: any tenant. Owner/Reader: own tenant only.
    pub(crate) fn require_read(&self, tenant_id: &str) -> Outcome<()> {
        match self.role {
            Role::Admin => Ok(()),
            Role::Owner | Role::Reader => self.assert_own_tenant(tenant_id),
        }
    }

    /// Caller may write (create/update/delete) resources belonging to `tenant_id`.
    /// Admin: any tenant. Owner: own tenant. Reader: denied.
    pub(crate) fn require_write(&self, tenant_id: &str) -> Outcome<()> {
        match self.role {
            Role::Admin => Ok(()),
            Role::Owner => self.assert_own_tenant(tenant_id),
            Role::Reader => Err(forbidden()),
        }
    }

    fn assert_own_tenant(&self, tenant_id: &str) -> Outcome<()> {
        if self.sub == tenant_id {
            Ok(())
        } else {
            Err(forbidden())
        }
    }
}

fn forbidden() -> Errors {
    Errors::format(BadFormat::Received, "forbidden: insufficient permissions", None)
}
