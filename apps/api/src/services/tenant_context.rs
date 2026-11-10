use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::models::_entities::{memberships, organizations, tenants, users};

/// Tenant scope resolved from an authenticated user and an active membership.
/// Never build this from a request tenant ID without checking membership.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantContext {
    user_id: i64,
    tenant_id: i64,
    tenant_public_id: Uuid,
    organization_id: i64,
    role: String,
}

impl TenantContext {
    pub async fn resolve(
        db: &DatabaseConnection,
        user: &users::Model,
        tenant_public_id: Uuid,
    ) -> ModelResult<Self> {
        if !user.is_active {
            return Err(ModelError::EntityNotFound);
        }

        let Some(tenant) = tenants::Entity::find()
            .filter(tenants::Column::PublicId.eq(tenant_public_id))
            .filter(tenants::Column::IsActive.eq(true))
            .one(db)
            .await?
        else {
            return Err(ModelError::EntityNotFound);
        };

        let Some(membership) = memberships::Entity::find()
            .filter(memberships::Column::UserId.eq(user.id))
            .filter(memberships::Column::TenantId.eq(tenant.id))
            .filter(memberships::Column::IsActive.eq(true))
            .one(db)
            .await?
        else {
            return Err(ModelError::EntityNotFound);
        };

        let Some(organization) = organizations::Entity::find_by_id(tenant.organization_id)
            .one(db)
            .await?
        else {
            return Err(ModelError::EntityNotFound);
        };

        if organization.status != "active" {
            return Err(ModelError::EntityNotFound);
        }

        Ok(Self {
            user_id: user.id,
            tenant_id: tenant.id,
            tenant_public_id: tenant.public_id,
            organization_id: organization.id,
            role: membership.role,
        })
    }

    #[must_use]
    pub const fn user_id(&self) -> i64 {
        self.user_id
    }

    #[must_use]
    pub const fn tenant_id(&self) -> i64 {
        self.tenant_id
    }

    #[must_use]
    pub const fn tenant_public_id(&self) -> Uuid {
        self.tenant_public_id
    }

    #[must_use]
    pub const fn organization_id(&self) -> i64 {
        self.organization_id
    }

    #[must_use]
    pub fn role(&self) -> &str {
        &self.role
    }

    /// Initial write policy; full RBAC permission catalog is a separate issue.
    #[must_use]
    pub fn can_manage_network(&self) -> bool {
        matches!(self.role.as_str(), "owner" | "admin" | "network_admin")
    }

    pub fn ensure_resource_tenant(&self, resource_tenant_id: i64) -> Result<(), TenantScopeError> {
        if self.tenant_id == resource_tenant_id {
            Ok(())
        } else {
            Err(TenantScopeError)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantScopeError;

impl std::fmt::Display for TenantScopeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("resource is outside the active tenant scope")
    }
}

impl std::error::Error for TenantScopeError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_context(role: &str) -> TenantContext {
        TenantContext {
            user_id: 10,
            tenant_id: 20,
            tenant_public_id: Uuid::from_u128(20),
            organization_id: 30,
            role: role.to_owned(),
        }
    }

    #[test]
    fn tenant_scope_accepts_same_tenant() {
        assert!(sample_context("owner").ensure_resource_tenant(20).is_ok());
    }

    #[test]
    fn tenant_scope_rejects_foreign_tenant() {
        assert!(sample_context("owner").ensure_resource_tenant(21).is_err());
    }

    #[test]
    fn network_write_policy_fails_closed_for_read_only_roles() {
        assert!(sample_context("owner").can_manage_network());
        assert!(sample_context("admin").can_manage_network());
        assert!(sample_context("network_admin").can_manage_network());
        assert!(!sample_context("viewer").can_manage_network());
        assert!(!sample_context("unknown").can_manage_network());
    }
}
