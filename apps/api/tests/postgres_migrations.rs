use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection,
    sea_query::{Alias, Expr, ExprTrait, Query},
};
use uuid::Uuid;

async fn insert_and_get_id(
    db: &DatabaseConnection,
    table: &str,
    public_id: Uuid,
    insert_sql: &str,
) -> Result<i64, Box<dyn std::error::Error>> {
    db.execute_unprepared(insert_sql).await?;
    let query = Query::select()
        .column(Alias::new("id"))
        .from(Alias::new(table))
        .and_where(Expr::col(Alias::new("public_id")).eq(public_id))
        .to_owned();
    let row = db
        .query_one(&query)
        .await?
        .ok_or("inserted row could not be found")?;
    Ok(row.try_get("", "id")?)
}

#[tokio::test]
async fn fresh_postgres_database_accepts_identity_network_and_device_inventory_schema()
-> Result<(), Box<dyn std::error::Error>> {
    let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
        // Local unit-test runs without PostgreSQL keep working. CI always sets this.
        return Ok(());
    };

    let db = Database::connect(&database_url).await?;
    Migrator::fresh(&db).await?;
    Migrator::up(&db, None).await?;

    for table in [
        "users",
        "organizations",
        "tenants",
        "memberships",
        "regions",
        "sites",
        "pops",
        "vendors",
        "device_models",
        "devices",
        "device_interfaces",
    ] {
        db.execute_unprepared(&format!("SELECT 1 FROM {table} LIMIT 0"))
            .await?;
    }

    let suffix = Uuid::new_v4().simple().to_string();
    let organization_public_id = Uuid::new_v4();
    let organization_slug = format!("org-{suffix}");
    let organization_sql = format!(
        "INSERT INTO organizations (public_id, name, slug, status) VALUES ('{organization_public_id}', 'Smoke Test ISP', '{organization_slug}', 'active')"
    );
    let organization_id = insert_and_get_id(
        &db,
        "organizations",
        organization_public_id,
        &organization_sql,
    )
    .await?;

    let tenant_public_id = Uuid::new_v4();
    let tenant_slug = format!("tenant-{suffix}");
    let tenant_sql = format!(
        "INSERT INTO tenants (public_id, organization_id, name, slug, is_active) VALUES ('{tenant_public_id}', {organization_id}, 'Production', '{tenant_slug}', TRUE)"
    );
    let tenant_id = insert_and_get_id(&db, "tenants", tenant_public_id, &tenant_sql).await?;

    let region_public_id = Uuid::new_v4();
    let region_slug = format!("region-{suffix}");
    let region_sql = format!(
        "INSERT INTO regions (public_id, tenant_id, name, slug, timezone, status) VALUES ('{region_public_id}', {tenant_id}, 'West Region', '{region_slug}', 'Asia/Jakarta', 'active')"
    );
    let region_id = insert_and_get_id(&db, "regions", region_public_id, &region_sql).await?;

    let site_public_id = Uuid::new_v4();
    let site_slug = format!("site-{suffix}");
    let site_sql = format!(
        "INSERT INTO sites (public_id, tenant_id, region_id, name, slug, site_type, timezone, status) VALUES ('{site_public_id}', {tenant_id}, {region_id}, 'Core Facility', '{site_slug}', 'facility', 'Asia/Jakarta', 'active')"
    );
    let site_id = insert_and_get_id(&db, "sites", site_public_id, &site_sql).await?;

    let pop_public_id = Uuid::new_v4();
    let pop_slug = format!("pop-{suffix}");
    let pop_sql = format!(
        "INSERT INTO pops (public_id, tenant_id, site_id, name, slug, pop_type, status) VALUES ('{pop_public_id}', {tenant_id}, {site_id}, 'Core POP', '{pop_slug}', 'core', 'active')"
    );
    db.execute_unprepared(&pop_sql).await?;

    let vendor_public_id = Uuid::new_v4();
    let vendor_slug = format!("vendor-{suffix}");
    let vendor_sql = format!(
        "INSERT INTO vendors (public_id, slug, display_name) VALUES ('{vendor_public_id}', '{vendor_slug}', 'Smoke Vendor')"
    );
    let vendor_id = insert_and_get_id(&db, "vendors", vendor_public_id, &vendor_sql).await?;

    let model_public_id = Uuid::new_v4();
    let model_sql = format!(
        "INSERT INTO device_models (public_id, vendor_id, model_code, display_name) VALUES ('{model_public_id}', {vendor_id}, 'MODEL-TEST', 'Test Router')"
    );
    let model_id = insert_and_get_id(&db, "device_models", model_public_id, &model_sql).await?;

    let device_public_id = Uuid::new_v4();
    let inventory_key = format!("device-{suffix}");
    let device_sql = format!(
        "INSERT INTO devices (public_id, tenant_id, site_id, pop_id, vendor_id, device_model_id, inventory_key, name, management_address, device_role, lifecycle_status) VALUES ('{device_public_id}', {tenant_id}, {site_id}, (SELECT id FROM pops WHERE public_id = '{pop_public_id}'), {vendor_id}, {model_id}, '{inventory_key}', 'Test Router', '192.0.2.10', 'router', 'active')"
    );
    let device_id = insert_and_get_id(&db, "devices", device_public_id, &device_sql).await?;

    let interface_public_id = Uuid::new_v4();
    let interface_sql = format!(
        "INSERT INTO device_interfaces (public_id, tenant_id, device_id, stable_key, if_index, name, interface_type, admin_state, operational_state) VALUES ('{interface_public_id}', {tenant_id}, {device_id}, 'ether1', 1, 'ether1', 'ethernet', 'unknown', 'unknown')"
    );
    insert_and_get_id(&db, "device_interfaces", interface_public_id, &interface_sql).await?;

    let duplicate_vendor_model = format!(
        "INSERT INTO device_models (public_id, vendor_id, model_code, display_name) VALUES ('{}', {vendor_id}, 'MODEL-TEST', 'Duplicate Model')",
        Uuid::new_v4()
    );
    assert!(db.execute_unprepared(&duplicate_vendor_model).await.is_err());

    let duplicate_inventory_key = format!(
        "INSERT INTO devices (public_id, tenant_id, inventory_key, name) VALUES ('{}', {tenant_id}, '{inventory_key}', 'Duplicate Device')",
        Uuid::new_v4()
    );
    assert!(db.execute_unprepared(&duplicate_inventory_key).await.is_err());

    let duplicate_interface_key = format!(
        "INSERT INTO device_interfaces (public_id, tenant_id, device_id, stable_key, name) VALUES ('{}', {tenant_id}, {device_id}, 'ether1', 'Duplicate Interface')",
        Uuid::new_v4()
    );
    assert!(db.execute_unprepared(&duplicate_interface_key).await.is_err());

    let duplicate_region_slug = format!(
        "INSERT INTO regions (public_id, tenant_id, name, slug, timezone, status) VALUES ('{}', {tenant_id}, 'Duplicate Region', '{region_slug}', 'UTC', 'active')",
        Uuid::new_v4()
    );
    assert!(db.execute_unprepared(&duplicate_region_slug).await.is_err());

    let duplicate_pop_slug = format!(
        "INSERT INTO pops (public_id, tenant_id, site_id, name, slug, pop_type, status) VALUES ('{}', {tenant_id}, {site_id}, 'Duplicate POP', '{pop_slug}', 'core', 'active')",
        Uuid::new_v4()
    );
    assert!(db.execute_unprepared(&duplicate_pop_slug).await.is_err());

    Ok(())
}
