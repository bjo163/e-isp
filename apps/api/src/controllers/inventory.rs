use crate::{
    models::_entities::{device_interfaces, device_models, devices, pops, sites, vendors},
    models::users::Model as User,
    services::tenant_context::TenantContext,
};
use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use loco_rs::{Error, controller::ErrorDetail, prelude::*};
use sea_orm::{ActiveValue, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Select};
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

#[derive(Debug, Deserialize)]
struct DeviceListQuery {
    limit: Option<usize>,
    after: Option<String>,
    q: Option<String>,
    role: Option<String>,
    lifecycle_status: Option<String>,
    site_id: Option<String>,
    pop_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct DevicePage {
    limit: usize,
    next_cursor: Option<String>,
    has_more: bool,
}

#[derive(Debug, Serialize)]
struct DeviceListResponse {
    items: Vec<DeviceResponse>,
    page: DevicePage,
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

fn device_list_limit(value: Option<usize>) -> std::result::Result<usize, &'static str> {
    let limit = value.unwrap_or(50);
    if (1..=100).contains(&limit) {
        Ok(limit)
    } else {
        Err("limit must be between 1 and 100")
    }
}

fn device_list_cursor(value: Option<&str>) -> std::result::Result<Option<&str>, &'static str> {
    match value {
        Some(cursor) if valid_inventory_key(cursor) => Ok(Some(cursor.trim())),
        Some(_) => Err("after must be a valid inventory_key cursor"),
        None => Ok(None),
    }
}

fn device_search_term(value: Option<&str>) -> std::result::Result<Option<&str>, &'static str> {
    match value {
        Some(query) if query.trim().len() <= 120 => {
            let trimmed = query.trim();
            Ok((!trimmed.is_empty()).then_some(trimmed))
        }
        Some(_) => Err("q must be at most 120 characters"),
        None => Ok(None),
    }
}

struct DeviceQueryFilters<'a> {
    limit: usize,
    cursor: Option<&'a str>,
    search: Option<&'a str>,
    role: Option<&'a str>,
    lifecycle_status: Option<&'a str>,
    site_id: Option<i64>,
    pop_id: Option<i64>,
}

fn build_device_list_query(
    tenant_id: i64,
    filters: DeviceQueryFilters<'_>,
) -> Select<devices::Entity> {
    let mut query = devices::Entity::find().filter(devices::Column::TenantId.eq(tenant_id));

    if let Some(cursor) = filters.cursor {
        query = query.filter(devices::Column::InventoryKey.gt(cursor.to_owned()));
    }
    if let Some(role) = filters.role {
        query = query.filter(devices::Column::DeviceRole.eq(role.to_owned()));
    }
    if let Some(status) = filters.lifecycle_status {
        query = query.filter(devices::Column::LifecycleStatus.eq(status.to_owned()));
    }
    if let Some(search) = filters.search {
        let search = search.to_owned();
        query = query.filter(
            Condition::any()
                .add(devices::Column::InventoryKey.contains(search.clone()))
                .add(devices::Column::Name.contains(search.clone()))
                .add(devices::Column::Hostname.contains(search.clone()))
                .add(devices::Column::ManagementAddress.contains(search)),
        );
    }
    if let Some(site_id) = filters.site_id {
        query = query.filter(devices::Column::SiteId.eq(site_id));
    }
    if let Some(pop_id) = filters.pop_id {
        query = query.filter(devices::Column::PopId.eq(pop_id));
    }

    query
        .order_by_asc(devices::Column::InventoryKey)
        .limit((filters.limit + 1) as u64)
}

async fn list_devices(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Query(params): Query<DeviceListQuery>,
) -> Result<Response> {
    let scope = match resolve_scope(&ctx, &auth, &headers).await {
        Ok(scope) => scope,
        Err(()) => return forbidden("tenant access denied"),
    };

    let limit = match device_list_limit(params.limit) {
        Ok(value) => value,
        Err(message) => return bad_request(message),
    };
    let cursor = match device_list_cursor(params.after.as_deref()) {
        Ok(value) => value,
        Err(message) => return bad_request(message),
    };
    let search = match device_search_term(params.q.as_deref()) {
        Ok(value) => value,
        Err(message) => return bad_request(message),
    };

    if params
        .role
        .as_deref()
        .is_some_and(|value| !valid_role(value))
    {
        return bad_request("unsupported device role filter");
    }
    if params
        .lifecycle_status
        .as_deref()
        .is_some_and(|value| !valid_lifecycle(value))
    {
        return bad_request("unsupported device lifecycle filter");
    }

    let site_id = if let Some(raw_site_id) = params.site_id.as_deref() {
        let public_id = match Uuid::parse_str(raw_site_id) {
            Ok(value) => value,
            Err(_) => return bad_request("site_id must be a valid UUID"),
        };
        let Some(site) = sites::Entity::find()
            .filter(sites::Column::PublicId.eq(public_id))
            .filter(sites::Column::TenantId.eq(scope.tenant_id()))
            .one(&ctx.db)
            .await?
        else {
            return not_found();
        };
        Some(site.id)
    } else {
        None
    };

    let pop_id = if let Some(raw_pop_id) = params.pop_id.as_deref() {
        let public_id = match Uuid::parse_str(raw_pop_id) {
            Ok(value) => value,
            Err(_) => return bad_request("pop_id must be a valid UUID"),
        };
        let Some(pop) = pops::Entity::find()
            .filter(pops::Column::PublicId.eq(public_id))
            .filter(pops::Column::TenantId.eq(scope.tenant_id()))
            .one(&ctx.db)
            .await?
        else {
            return not_found();
        };
        Some(pop.id)
    } else {
        None
    };

    let query = build_device_list_query(
        scope.tenant_id(),
        DeviceQueryFilters {
            limit,
            cursor,
            search,
            role: params.role.as_deref(),
            lifecycle_status: params.lifecycle_status.as_deref(),
            site_id,
            pop_id,
        },
    );

    let mut rows = query
        .order_by_asc(devices::Column::InventoryKey)
        .limit((limit + 1) as u64)
        .all(&ctx.db)
        .await?;
    let has_more = rows.len() > limit;
    if has_more {
        rows.truncate(limit);
    }
    let next_cursor = if has_more {
        rows.last().map(|row| row.inventory_key.clone())
    } else {
        None
    };

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(device_response(&ctx, row, scope.tenant_id()).await?);
    }

    format::json(DeviceListResponse {
        items,
        page: DevicePage {
            limit,
            next_cursor,
            has_more,
        },
    })
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
    if params
        .hostname
        .as_deref()
        .is_some_and(|hostname| hostname.trim().is_empty() || hostname.len() > 253)
    {
        return bad_request("invalid hostname");
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
    format::json(device_response(&ctx, row, scope.tenant_id()).await?)
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
    if params
        .description
        .as_deref()
        .is_some_and(|description| description.len() > 512)
    {
        return bad_request("interface description is too long");
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

#[cfg(test)]
mod device_list_query_tests {
    use super::*;
    use sea_orm::{DbBackend, QueryTrait};

    #[test]
    fn device_list_limit_defaults_and_enforces_bounds() {
        assert_eq!(device_list_limit(None), Ok(50));
        assert_eq!(device_list_limit(Some(1)), Ok(1));
        assert_eq!(device_list_limit(Some(100)), Ok(100));
        assert!(device_list_limit(Some(0)).is_err());
        assert!(device_list_limit(Some(101)).is_err());
    }

    #[test]
    fn device_list_cursor_accepts_only_valid_inventory_keys() {
        assert_eq!(device_list_cursor(None), Ok(None));
        assert_eq!(
            device_list_cursor(Some("core-rtr-01")),
            Ok(Some("core-rtr-01"))
        );
        assert_eq!(
            device_list_cursor(Some("  core-rtr-01  ")),
            Ok(Some("core-rtr-01"))
        );
        assert!(device_list_cursor(Some("")).is_err());
        assert!(device_list_cursor(Some("bad cursor")).is_err());
        assert!(device_list_cursor(Some("key/with/slash")).is_err());
    }

    #[test]
    fn device_search_term_trims_and_bounds_input() {
        assert_eq!(device_search_term(None), Ok(None));
        assert_eq!(device_search_term(Some("   ")), Ok(None));
        assert_eq!(device_search_term(Some(" core ")), Ok(Some("core")));
        assert!(device_search_term(Some(&"x".repeat(121))).is_err());
    }

    #[test]
    fn list_filter_values_match_supported_inventory_enums() {
        assert!(valid_role("router"));
        assert!(valid_role("olt"));
        assert!(!valid_role("modem"));
        assert!(valid_lifecycle("active"));
        assert!(valid_lifecycle("maintenance"));
        assert!(!valid_lifecycle("online"));
    }

    #[test]
    fn device_query_uses_tenant_keyset_filters_and_bounded_ordered_limit() {
        let statement = build_device_list_query(
            42,
            DeviceQueryFilters {
                limit: 50,
                cursor: Some("core-rtr-01"),
                search: Some("edge"),
                role: Some("router"),
                lifecycle_status: Some("active"),
                site_id: Some(7),
                pop_id: Some(9),
            },
        )
        .build(DbBackend::Postgres);
        let sql = statement.sql.to_lowercase();

        assert!(sql.contains("\"devices\".\"tenant_id\""));
        assert!(sql.contains("\"devices\".\"inventory_key\" >"));
        assert!(sql.contains("\"devices\".\"inventory_key\" like"));
        assert!(sql.contains("\"devices\".\"name\" like"));
        assert!(sql.contains("\"devices\".\"hostname\" like"));
        assert!(sql.contains("\"devices\".\"management_address\" like"));
        assert!(sql.contains("\"devices\".\"device_role\""));
        assert!(sql.contains("\"devices\".\"lifecycle_status\""));
        assert!(sql.contains("\"devices\".\"site_id\""));
        assert!(sql.contains("\"devices\".\"pop_id\""));
        assert!(sql.contains("order by \"devices\".\"inventory_key\" asc"));
        assert!(sql.contains("limit 51"));
        assert!(!sql.contains(" offset "));
        assert!(!sql.contains("core-rtr-01"));
        assert!(!sql.contains("edge"));
    }

    #[test]
    fn device_query_without_optional_filters_still_scopes_tenant() {
        let statement = build_device_list_query(
            17,
            DeviceQueryFilters {
                limit: 100,
                cursor: None,
                search: None,
                role: None,
                lifecycle_status: None,
                site_id: None,
                pop_id: None,
            },
        )
        .build(DbBackend::Postgres);
        let sql = statement.sql.to_lowercase();

        assert!(sql.contains("\"devices\".\"tenant_id\""));
        assert!(sql.contains("order by \"devices\".\"inventory_key\" asc"));
        assert!(sql.contains("limit 101"));
        assert!(!sql.contains(" offset "));
    }
}
