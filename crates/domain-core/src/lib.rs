//! Shared domain identifiers and invariant helpers. No framework or persistence types belong here.

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TenantId(Uuid);

impl TenantId {
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for TenantId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for TenantId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantScope {
    tenant_id: TenantId,
}

impl TenantScope {
    #[must_use]
    pub const fn new(tenant_id: TenantId) -> Self {
        Self { tenant_id }
    }

    #[must_use]
    pub const fn tenant_id(self) -> TenantId {
        self.tenant_id
    }

    pub fn ensure_matches(self, resource_tenant: TenantId) -> Result<(), TenantScopeError> {
        if self.tenant_id == resource_tenant {
            Ok(())
        } else {
            Err(TenantScopeError)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantScopeError;

impl fmt::Display for TenantScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("resource is outside the active tenant scope")
    }
}

impl std::error::Error for TenantScopeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_scope_accepts_matching_tenant() {
        let tenant = TenantId::new();
        assert!(TenantScope::new(tenant).ensure_matches(tenant).is_ok());
    }

    #[test]
    fn tenant_scope_rejects_foreign_tenant() {
        let active = TenantId::new();
        let foreign = TenantId::new();
        assert!(TenantScope::new(active).ensure_matches(foreign).is_err());
    }

    #[test]
    fn tenant_id_round_trips_string_representation() {
        let tenant = TenantId::new();
        let parsed = tenant.to_string().parse::<TenantId>();
        assert!(parsed.is_ok());
        assert_eq!(parsed.ok(), Some(tenant));
    }
}
