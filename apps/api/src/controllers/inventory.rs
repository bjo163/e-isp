use crate::{
    models::_entities::{device_interfaces, device_models, devices, pops, sites, vendors},
    models::users::Model as User,
    services::tenant_context::TenantContext,
};
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use loco_rs::{Error, controller::ErrorDetail, prelude::*};
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct CreateDeviceRequest {
    inventory_key: String,
    name: String,
    hostname: Option<String>,
    management_address: Option<String>,
    serial_number: Option<String>,
    device_role: Option<String>,
    lifecycle_status: Option<String>,
    site_id: Option<String>,
    pop_id: Option<String>,
    vendor_id: Option<String>,
    device_model_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateInterfaceRequest {
    stable_key: String,
    if_index: Option<i32>,
    name: String,
    description: Option<String>,
    interface_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeviceResponse {
    id: String,
    inventory_key: String,
    name: String,
    hostname: Option<String>,
    management_address: Option<String>,
    serial_number: Option<String>,
    device_role: String,
    lifecycle_status: String,
    site_id: Option<String>,
    pop_id: Option<String>,
    vendor_id: Option<String>,
    device_model_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct InterfaceResponse {
    id: String,
    device_id: String,
    stable_key: String,
    if_index: Option<i32>,
    name: String,
    description: Option<String>,
    interface_type: String,
    admin_state: String,
    operational_state: String,
}

fn request_tenant_id(headers: &HeaderMap) -> std::result::Result<Uuid, ()> {
    let raw = headers
        .get("x-tenant-id")
        .ok_or(())?
        .to_str()
        .map_err(|_| ())?;
    Uuid::parse_str(raw).map_err(|_| ())
}

fn forbidden(message: &str) -> Result<Response> {
    Err(Error::CustomError(
        StatusCode::FORBIDDEN,
        ErrorDetail::new("forbidden", message),
    ))
}

async fn resolve_scope(
    ctx: &AppContext,
    auth: &auth::JWT,
    headers: &HeaderMap,
) -> std::result::Result<TenantContext, ()> {
    let tenant_public_id = request_tenant_id(headers)?;
    let user = User::find_by_pid(&ctx.db, &auth.claims.pid)
        .await
        .map_err(|_| ())?;
    TenantContext::resolve(&ctx.db, &user, tenant_public_id)
        .await
        .map_err(|_| ())
}

fn valid_name(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && value.len() <= 120
}

fn valid_inventory_key(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 80
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_.:".contains(character))
}

fn valid_role(value: &str) -> bool {
    matches!(
        value,
        "router" | "switch" | "firewall" | "olt" | "server" | "wireless" | "other"
    )
}

fn valid_lifecycle(value: &str) -> bool {
    matches!(value, "planned" | "active" | "maintenance" | "retired")
}

async fn load_device(
    ctx: &AppContext,
    scope: &TenantContext,
    public_id: Uuid,
) -> Result<Option<devices::Model>> {
    Ok(devices::Entity::find()
        .filter(devices::Column::PublicId.eq(public_id))
        .filter(devices::Column::TenantId.eq(scope.tenant_id()))
        .one(&ctx.db)
        .await?)
}

async fn device_response(
    ctx: &AppContext,
    row: devices::Model,
    tenant_id: i64,
) -> Result<DeviceResponse> {
    let site_id = match row.site_id {
        Some(id) => sites::Entity::find_by_id(id)
            .filter(sites::Column::TenantId.eq(tenant_id))
            .one(&ctx.db)
            .await?
            .map(|site| site.public_id.to_string()),
        None => None,
    };
    let pop_id = match row.pop_id {
        Some(id) => pops::Entity::find_by_id(id)
            .filter(pops::Column::TenantId.eq(tenant_id))
            .one(&ctx.db)
            .await?
            .map(|pop| pop.public_id.to_string()),
        None => None,
    };
    let vendor_id = match row.vendor_id {
        Some(id) => vendors::Entity::find_by_id(id)
            .one(&ctx.db)
            .await?
            .map(|vendor| vendor.public_id.to_string()),
        None => None,
    };
    let device_model_id = match row.device_model_id {
        Some(id) => device_models::Entity::find_by_id(id)
            .one(&ctx.db)
            .await?
            .map(|model| model.public_id.to_string()),
        None => None,
    };

    Ok(DeviceResponse {
        id: row.public_id.to_string(),
        inventory_key: row.inventory_key,
        name: row.name,
        hostname: row.hostname,
        management_address: row.management_address,
        serial_number: row.serial_number,
        device_role: row.device_role,
        lifecycle_status: row.lifecycle_status,
        site_id,
        pop_id,
        vendor_id,
        device_model_id,
    })
}

async fn list_devices(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) => scope,
        Err(()) => return forbidden("tenant access denied"),
    };
    let rows = devices::Entity::find()
        .filter(devices::Column::TenantId.eq(scope.tenant_id()))
        .order_by_asc(devices::Column::Name)
        .limit(500)
        .all(&ctx.db)
        .await?;

    let mut response = Vec::with_capacity(rows.len());
    for row in rows {
        response.push(device_response(&ctx, row, scope.tenant_id()).await?);
    }
    format::json(response)
}

async fn get_device(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Path(raw_id): Path<String>,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) => scope,
        Err(()) => return forbidden("tenant access denied"),
    };
    let public_id = match Uuid::parse_str(&raw_id) {
        Ok(value) => value,
        Err(_) => return not_found(),
    };
    let Some(row) = load_device(&ctx, &scope, public_id).await? else {
        return not_found();
    };
    format::json(device_response(&ctx, row, scope.tenant_id()).await?)
}

async fn create_device(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Json(params): Json<CreateDeviceRequest>,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) if scope.can_manage_network() => scope,
        Ok(_) => return forbidden("network management permission required"),
        Err(()) => return forbidden("tenant access denied"),
    };

    if !valid_inventory_key(&params.inventory_key) || !valid_name(&params.name) {
        return bad_request("invalid inventory_key or device name");
    }
    let role = params.device_role.unwrap_or_else(|| "other".to_owned());
    if !valid_role(&role) {
        return bad_request("unsupported device role");
    }
    let lifecycle = params
        .lifecycle_status
        .unwrap_or_else(|| "planned".to_owned());
    if !valid_lifecycle(&lifecycle) {
        return bad_request("unsupported device lifecycle status");
    }
    if let Some(hostname) = params.hostname.as_deref() {
        if hostname.trim().is_empty() || hostname.len() > 253 {
            return bad_request("invalid hostname");
        }
    }
    if params
        .serial_number
        .as_deref()
        .is_some_and(|serial| serial.trim().is_empty() || serial.len() > 128)
    {
        return bad_request("invalid serial number");
    }
    let management_address = match params.management_address.as_deref() {
        Some(value) => match value.parse::<IpAddr>() {
            Ok(address) => Some(address.to_string()),
            Err(_) => return bad_request("management_address must be an IPv4 or IPv6 address"),
        },
        None => None,
    };

    let mut site_db_id = if let Some(raw) = params.site_id.as_deref() {
        let site_id = match Uuid::parse_str(raw) {
            Ok(value) => value,
            Err(_) => return bad_request("site_id must be a valid UUID"),
        };
        match sites::Entity::find()
            .filter(sites::Column::PublicId.eq(site_id))
            .filter(sites::Column::TenantId.eq(scope.tenant_id()))
            .one(&ctx.db)
            .await?
        {
            Some(site) => Some(site.id),
            None => return not_found(),
        }
    } else {
        None
    };

    let pop_db_id = if let Some(raw) = params.pop_id.as_deref() {
        let pop_id = match Uuid::parse_str(raw) {
            Ok(value) => value,
            Err(_) => return bad_request("pop_id must be a valid UUID"),
        };
        match pops::Entity::find()
            .filter(pops::Column::PublicId.eq(pop_id))
            .filter(pops::Column::TenantId.eq(scope.tenant_id()))
            .one(&ctx.db)
            .await?
        {
            Some(pop) => {
                if site_db_id.is_some_and(|site_id| site_id != pop.site_id) {
                    return bad_request("POP does not belong to the selected site");
                }
                if site_db_id.is_none() {
                    site_db_id = Some(pop.site_id);
                }
                Some(pop.id)
            }
            None => return not_found(),
        }
    } else {
        None
    };

    let (vendor_db_id, model_db_id) = match (
        params.vendor_id.as_deref(),
        params.device_model_id.as_deref(),
    ) {
        (Some(vendor_raw), Some(model_raw)) => {
            let vendor_uuid = match Uuid::parse_str(vendor_raw) {
                Ok(value) => value,
                Err(_) => return bad_request("vendor_id must be a valid UUID"),
            };
            let model_uuid = match Uuid::parse_str(model_raw) {
                Ok(value) => value,
                Err(_) => return bad_request("device_model_id must be a valid UUID"),
            };
            let vendor = vendors::Entity::find()
                .filter(vendors::Column::PublicId.eq(vendor_uuid))
                .one(&ctx.db)
                .await?;
            let model = device_models::Entity::find()
                .filter(device_models::Column::PublicId.eq(model_uuid))
                .one(&ctx.db)
                .await?;
            match (vendor, model) {
                (Some(vendor), Some(model)) if vendor.id == model.vendor_id => {
                    (Some(vendor.id), Some(model.id))
                }
                _ => return bad_request("device model does not belong to the selected vendor"),
            }
        }
        (Some(vendor_raw), None) => {
            let vendor_uuid = match Uuid::parse_str(vendor_raw) {
                Ok(value) => value,
                Err(_) => return bad_request("vendor_id must be a valid UUID"),
            };
            let vendor = vendors::Entity::find()
                .filter(vendors::Column::PublicId.eq(vendor_uuid))
                .one(&ctx.db)
                .await?;
            match vendor {
                Some(vendor) => (Some(vendor.id), None),
                None => return not_found(),
            }
        }
        (None, Some(model_raw)) => {
            let model_uuid = match Uuid::parse_str(model_raw) {
                Ok(value) => value,
                Err(_) => return bad_request("device_model_id must be a valid UUID"),
            };
            let model = device_models::Entity::find()
                .filter(device_models::Column::PublicId.eq(model_uuid))
                .one(&ctx.db)
                .await?;
            match model {
                Some(model) => (Some(model.vendor_id), Some(model.id)),
                None => return not_found(),
            }
        }
        (None, None) => (None, None),
    };

    let row = devices::ActiveModel {
        public_id: ActiveValue::Set(Uuid::new_v4()),
        tenant_id: ActiveValue::Set(scope.tenant_id()),
        site_id: ActiveValue::Set(site_db_id),
        pop_id: ActiveValue::Set(pop_db_id),
        vendor_id: ActiveValue::Set(vendor_db_id),
        device_model_id: ActiveValue::Set(model_db_id),
        inventory_key: ActiveValue::Set(params.inventory_key.trim().to_owned()),
        name: ActiveValue::Set(params.name.trim().to_owned()),
        hostname: ActiveValue::Set(params.hostname.map(|value| value.trim().to_owned())),
        management_address: ActiveValue::Set(management_address),
        serial_number: ActiveValue::Set(params.serial_number.map(|value| value.trim().to_owned())),
        device_role: ActiveValue::Set(role),
        lifecycle_status: ActiveValue::Set(lifecycle),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await;

    let row = match row {
        Ok(row) => row,
        Err(_) => return bad_request("inventory_key already exists or device data is invalid"),
    };
    format::json(device_response(&ctx, row).await?)
}

async fn list_interfaces(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Path(raw_id): Path<String>,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) => scope,
        Err(()) => return forbidden("tenant access denied"),
    };
    let device_uuid = match Uuid::parse_str(&raw_id) {
        Ok(value) => value,
        Err(_) => return not_found(),
    };
    let Some(device) = load_device(&ctx, &scope, device_uuid).await? else {
        return not_found();
    };
    let rows = device_interfaces::Entity::find()
        .filter(device_interfaces::Column::TenantId.eq(scope.tenant_id()))
        .filter(device_interfaces::Column::DeviceId.eq(device.id))
        .order_by_asc(device_interfaces::Column::Name)
        .limit(500)
        .all(&ctx.db)
        .await?;
    let response = rows
        .into_iter()
        .map(|row| InterfaceResponse {
            id: row.public_id.to_string(),
            device_id: device.public_id.to_string(),
            stable_key: row.stable_key,
            if_index: row.if_index,
            name: row.name,
            description: row.description,
            interface_type: row.interface_type,
            admin_state: row.admin_state,
            operational_state: row.operational_state,
        })
        .collect::<Vec<_>>();
    format::json(response)
}

async fn create_interface(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Path(raw_id): Path<String>,
    Json(params): Json<CreateInterfaceRequest>,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) if scope.can_manage_network() => scope,
        Ok(_) => return forbidden("network management permission required"),
        Err(()) => return forbidden("tenant access denied"),
    };
    let device_uuid = match Uuid::parse_str(&raw_id) {
        Ok(value) => value,
        Err(_) => return not_found(),
    };
    let Some(device) = load_device(&ctx, &scope, device_uuid).await? else {
        return not_found();
    };
    if params.stable_key.trim().is_empty()
        || params.stable_key.len() > 128
        || !valid_name(&params.name)
        || params.if_index.is_some_and(|value| value <= 0)
    {
        return bad_request("invalid interface identity or name");
    }
    let interface_type = params
        .interface_type
        .unwrap_or_else(|| "unknown".to_owned());
    if !valid_name(&interface_type) {
        return bad_request("invalid interface type");
    }
    if let Some(description) = params.description.as_deref() {
        if description.len() > 512 {
            return bad_request("interface description is too long");
        }
    }

    let result = device_interfaces::ActiveModel {
        public_id: ActiveValue::Set(Uuid::new_v4()),
        tenant_id: ActiveValue::Set(scope.tenant_id()),
        device_id: ActiveValue::Set(device.id),
        stable_key: ActiveValue::Set(params.stable_key.trim().to_owned()),
        if_index: ActiveValue::Set(params.if_index),
        name: ActiveValue::Set(params.name.trim().to_owned()),
        description: ActiveValue::Set(params.description.map(|value| value.trim().to_owned())),
        interface_type: ActiveValue::Set(interface_type),
        admin_state: ActiveValue::Set("unknown".to_owned()),
        operational_state: ActiveValue::Set("unknown".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await;

    let row = match result {
        Ok(row) => row,
        Err(_) => return bad_request("interface stable_key already exists for this device"),
    };
    format::json(InterfaceResponse {
        id: row.public_id.to_string(),
        device_id: device.public_id.to_string(),
        stable_key: row.stable_key,
        if_index: row.if_index,
        name: row.name,
        description: row.description,
        interface_type: row.interface_type,
        admin_state: row.admin_state,
        operational_state: row.operational_state,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/network")
        .add("/devices", get(list_devices))
        .add("/devices", post(create_device))
        .add("/devices/{device_id}", get(get_device))
        .add("/devices/{device_id}/interfaces", get(list_interfaces))
        .add("/devices/{device_id}/interfaces", post(create_interface))
}
