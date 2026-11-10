use serde::Serialize;

use crate::models::_entities::users;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
}

impl From<&users::Model> for UserResponse {
    fn from(user: &users::Model) -> Self {
        Self {
            id: user.pid.to_string(),
            email: user.email.clone(),
            name: user.name.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MembershipResponse {
    pub tenant_id: String,
    pub tenant_name: String,
    pub tenant_slug: String,
    pub organization_id: String,
    pub organization_name: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: u64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct CurrentResponse {
    pub user: UserResponse,
    pub tenants: Vec<MembershipResponse>,
}
