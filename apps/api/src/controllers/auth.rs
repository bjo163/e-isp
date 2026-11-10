use crate::{
    models::{
        _entities::{memberships, organizations, tenants},
        users::Model as User,
    },
    views::auth::{CurrentResponse, LoginResponse, MembershipResponse, UserResponse},
};
use loco_rs::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LoginParams {
    email: String,
    password: String,
}

#[debug_handler]
async fn login(
    State(ctx): State<AppContext>,
    Json(params): Json<LoginParams>,
) -> Result<Response> {
    let email = params.email.trim().to_ascii_lowercase();
    if email.len() > 254 || !email.contains('@') || params.password.is_empty() {
        return bad_request("invalid login request");
    }

    let user = match User::find_by_email(&ctx.db, &email).await {
        Ok(user) => user,
        Err(_) => return unauthorized("invalid credentials"),
    };

    if !user.is_active || !user.verify_password(&params.password) {
        return unauthorized("invalid credentials");
    }

    let jwt = ctx.config.get_jwt_config()?;
    let token = user
        .generate_jwt(&jwt.secret, jwt.expiration)
        .map_err(|_| unauthorized("authentication failed"))?;

    format::json(LoginResponse {
        token,
        expires_in: jwt.expiration,
        user: UserResponse::from(&user),
    })
}

#[debug_handler]
async fn current(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let user = match User::find_by_pid(&ctx.db, &auth.claims.pid).await {
        Ok(user) if user.is_active => user,
        _ => return unauthorized("session is no longer valid"),
    };

    let rows = memberships::Entity::find()
        .filter(memberships::Column::UserId.eq(user.id))
        .filter(memberships::Column::IsActive.eq(true))
        .all(&ctx.db)
        .await?;

    let mut accessible_tenants = Vec::with_capacity(rows.len());
    for membership in rows {
        let Some(tenant) = tenants::Entity::find_by_id(membership.tenant_id)
            .one(&ctx.db)
            .await?
        else {
            continue;
        };
        if !tenant.is_active {
            continue;
        }

        let Some(organization) = organizations::Entity::find_by_id(tenant.organization_id)
            .one(&ctx.db)
            .await?
        else {
            continue;
        };

        accessible_tenants.push(MembershipResponse {
            tenant_id: tenant.public_id.to_string(),
            tenant_name: tenant.name,
            tenant_slug: tenant.slug,
            organization_id: organization.public_id.to_string(),
            organization_name: organization.name,
            role: membership.role,
        });
    }

    format::json(CurrentResponse {
        user: UserResponse::from(&user),
        tenants: accessible_tenants,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/v1/auth")
        .add("/login", post(login))
        .add("/me", get(current))
}
