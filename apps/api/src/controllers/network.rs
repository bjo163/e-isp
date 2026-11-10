use crate::{
    models::{
        _entities::{pops, regions, sites},
        users::Model as User,
    },
    services::tenant_context::TenantContext,
};
use axum::http::{HeaderMap, StatusCode};
use loco_rs::{Error, controller::ErrorDetail, prelude::*};
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct CreateRegionRequest {
    name: String,
    slug: String,
    timezone: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateSiteRequest {
    name: String,
    slug: String,
    site_type: Option<String>,
    timezone: Option<String>,
    region_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreatePopRequest {
    site_id: String,
    name: String,
    slug: String,
    pop_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct RegionResponse {
    id: String,
    name: String,
    slug: String,
    timezone: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct SiteResponse {
    id: String,
    region_id: Option<String>,
    name: String,
    slug: String,
    site_type: String,
    timezone: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct PopResponse {
    id: String,
    site_id: String,
    name: String,
    slug: String,
    pop_type: String,
    status: String,
}

fn forbidden(message: &str) -> Result<Response> {
    Err(Error::CustomError(
        StatusCode::FORBIDDEN,
        ErrorDetail::new("forbidden", message),
    ))
}

fn request_tenant_id(headers: &HeaderMap) -> std::result::Result<Uuid, ()> {
    let raw = headers
        .get("x-tenant-id")
        .ok_or(())?
        .to_str()
        .map_err(|_| ())?;
    Uuid::parse_str(raw).map_err(|_| ())
}

async fn resolve_scope(
    ctx: &AppContext,
    auth: &auth::JWT,
    headers: &HeaderMap,
) -> std::result::Result<TenantContext, ()> {
    let public_id = request_tenant_id(headers)?;
    let user = User::find_by_pid(&ctx.db, &auth.claims.pid)
        .await
        .map_err(|_| ())?;
    TenantContext::resolve(&ctx.db, &user, public_id)
        .await
        .map_err(|_| ())
}

fn validate_name(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && value.len() <= 120
}

fn validate_slug(value: &str) -> bool {
    let is_slug_character = |character: char| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
    };
    let is_alphanumeric =
        |character: char| character.is_ascii_lowercase() || character.is_ascii_digit();

    !value.is_empty()
        && value.len() <= 63
        && value.chars().all(is_slug_character)
        && value.chars().next().is_some_and(is_alphanumeric)
        && value.chars().last().is_some_and(is_alphanumeric)
}

fn validate_timezone(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && value.len() <= 64 && value.is_ascii()
}

async fn list_regions(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) => scope,
        Err(()) => return forbidden("tenant access denied"),
    };

    let rows = regions::Entity::find()
        .filter(regions::Column::TenantId.eq(scope.tenant_id()))
        .order_by_asc(regions::Column::Name)
        .limit(500)
        .all(&ctx.db)
        .await?;

    let response = rows
        .into_iter()
        .map(|region| RegionResponse {
            id: region.public_id.to_string(),
            name: region.name,
            slug: region.slug,
            timezone: region.timezone,
            status: region.status,
        })
        .collect::<Vec<_>>();

    format::json(response)
}

async fn create_region(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Json(params): Json<CreateRegionRequest>,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) if scope.can_manage_network() => scope,
        Ok(_) => return forbidden("network management permission required"),
        Err(()) => return forbidden("tenant access denied"),
    };

    if !validate_name(&params.name) || !validate_slug(&params.slug) {
        return bad_request("invalid region name or slug");
    }

    let timezone = params.timezone.unwrap_or_else(|| "UTC".to_owned());
    if !validate_timezone(&timezone) {
        return bad_request("invalid timezone");
    }

    let inserted = regions::ActiveModel {
        public_id: ActiveValue::Set(Uuid::new_v4()),
        tenant_id: ActiveValue::Set(scope.tenant_id()),
        name: ActiveValue::Set(params.name.trim().to_owned()),
        slug: ActiveValue::Set(params.slug.clone()),
        timezone: ActiveValue::Set(timezone),
        status: ActiveValue::Set("active".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await;

    let row = match inserted {
        Ok(row) => row,
        Err(_) => return bad_request("region slug already exists or request is invalid"),
    };

    format::json(RegionResponse {
        id: row.public_id.to_string(),
        name: row.name,
        slug: row.slug,
        timezone: row.timezone,
        status: row.status,
    })
}

async fn list_sites(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) => scope,
        Err(()) => return forbidden("tenant access denied"),
    };

    let rows = sites::Entity::find()
        .filter(sites::Column::TenantId.eq(scope.tenant_id()))
        .order_by_asc(sites::Column::Name)
        .limit(500)
        .all(&ctx.db)
        .await?;

    let mut response = Vec::with_capacity(rows.len());
    for site in rows {
        let region_id = match site.region_id {
            Some(id) => regions::Entity::find_by_id(id)
                .one(&ctx.db)
                .await?
                .filter(|region| region.tenant_id == scope.tenant_id())
                .map(|region| region.public_id.to_string()),
            None => None,
        };

        response.push(SiteResponse {
            id: site.public_id.to_string(),
            region_id,
            name: site.name,
            slug: site.slug,
            site_type: site.site_type,
            timezone: site.timezone,
            status: site.status,
        });
    }

    format::json(response)
}

async fn create_site(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Json(params): Json<CreateSiteRequest>,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) if scope.can_manage_network() => scope,
        Ok(_) => return forbidden("network management permission required"),
        Err(()) => return forbidden("tenant access denied"),
    };

    if !validate_name(&params.name) || !validate_slug(&params.slug) {
        return bad_request("invalid site name or slug");
    }

    let (region_db_id, region_public_id) = if let Some(public_id) = params.region_id.as_deref() {
        let parsed = match Uuid::parse_str(public_id) {
            Ok(value) => value,
            Err(_) => return bad_request("region_id must be a valid region UUID"),
        };
        let region = regions::Entity::find()
            .filter(regions::Column::PublicId.eq(parsed))
            .filter(regions::Column::TenantId.eq(scope.tenant_id()))
            .one(&ctx.db)
            .await?;
        match region {
            Some(region) => (Some(region.id), Some(region.public_id.to_string())),
            None => return not_found("region not found in active tenant"),
        }
    } else {
        (None, None)
    };

    let timezone = params.timezone.unwrap_or_else(|| "UTC".to_owned());
    if !validate_timezone(&timezone) {
        return bad_request("invalid timezone");
    }
    let site_type = params.site_type.unwrap_or_else(|| "facility".to_owned());
    if !validate_name(&site_type) {
        return bad_request("invalid site type");
    }

    let inserted = sites::ActiveModel {
        public_id: ActiveValue::Set(Uuid::new_v4()),
        tenant_id: ActiveValue::Set(scope.tenant_id()),
        region_id: ActiveValue::Set(region_db_id),
        name: ActiveValue::Set(params.name.trim().to_owned()),
        slug: ActiveValue::Set(params.slug.clone()),
        site_type: ActiveValue::Set(site_type),
        timezone: ActiveValue::Set(timezone),
        status: ActiveValue::Set("active".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await;

    let row = match inserted {
        Ok(row) => row,
        Err(_) => return bad_request("site slug already exists or request is invalid"),
    };

    format::json(SiteResponse {
        id: row.public_id.to_string(),
        region_id: region_public_id,
        name: row.name,
        slug: row.slug,
        site_type: row.site_type,
        timezone: row.timezone,
        status: row.status,
    })
}

async fn list_pops(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) => scope,
        Err(()) => return forbidden("tenant access denied"),
    };

    let rows = pops::Entity::find()
        .filter(pops::Column::TenantId.eq(scope.tenant_id()))
        .order_by_asc(pops::Column::Name)
        .limit(500)
        .all(&ctx.db)
        .await?;

    let mut response = Vec::with_capacity(rows.len());
    for pop in rows {
        let site = sites::Entity::find_by_id(pop.site_id).one(&ctx.db).await?;
        let site_id = site
            .filter(|site| site.tenant_id == scope.tenant_id())
            .map(|site| site.public_id.to_string())
            .unwrap_or_default();

        response.push(PopResponse {
            id: pop.public_id.to_string(),
            site_id,
            name: pop.name,
            slug: pop.slug,
            pop_type: pop.pop_type,
            status: pop.status,
        });
    }

    format::json(response)
}

async fn create_pop(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Json(params): Json<CreatePopRequest>,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) if scope.can_manage_network() => scope,
        Ok(_) => return forbidden("network management permission required"),
        Err(()) => return forbidden("tenant access denied"),
    };

    if !validate_name(&params.name) || !validate_slug(&params.slug) {
        return bad_request("invalid POP name or slug");
    }
    let site_public_id = match Uuid::parse_str(&params.site_id) {
        Ok(value) => value,
        Err(_) => return bad_request("site_id must be a valid site UUID"),
    };
    let site = sites::Entity::find()
        .filter(sites::Column::PublicId.eq(site_public_id))
        .filter(sites::Column::TenantId.eq(scope.tenant_id()))
        .one(&ctx.db)
        .await?;
    let Some(site) = site else {
        return not_found();
    };

    let pop_type = params.pop_type.unwrap_or_else(|| "network".to_owned());
    if !validate_name(&pop_type) {
        return bad_request("invalid POP type");
    }

    let inserted = pops::ActiveModel {
        public_id: ActiveValue::Set(Uuid::new_v4()),
        tenant_id: ActiveValue::Set(scope.tenant_id()),
        site_id: ActiveValue::Set(site.id),
        name: ActiveValue::Set(params.name.trim().to_owned()),
        slug: ActiveValue::Set(params.slug.clone()),
        pop_type: ActiveValue::Set(pop_type),
        status: ActiveValue::Set("active".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await;

    let row = match inserted {
        Ok(row) => row,
        Err(_) => return bad_request("POP slug already exists or request is invalid"),
    };

    format::json(PopResponse {
        id: row.public_id.to_string(),
        site_id: site.public_id.to_string(),
        name: row.name,
        slug: row.slug,
        pop_type: row.pop_type,
        status: row.status,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/network")
        .add("/regions", get(list_regions))
        .add("/regions", post(create_region))
        .add("/sites", get(list_sites))
        .add("/sites", post(create_site))
        .add("/pops", get(list_pops))
        .add("/pops", post(create_pop))
}
