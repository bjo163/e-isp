use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};
use uuid::Uuid;

async fn insert_and_get_id(
    db: &sea_orm::DatabaseConnection,
    sql: String,
) -> Result<i64, Box<dyn std::error::Error>> {
    let row = db
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await?
        .ok_or("insert did not return a row")?;
    Ok(row.try_get("", "id")?)
}

#[tokio::test]
async fn fresh_postgres_database_accepts_identity_and_network_location_schema()
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
    ] {
        db.execute_unprepared(&format!("SELECT 1 FROM {table} LIMIT 0"))
            .await?;
    }

    let suffix = Uuid::new_v4().simple().to_string();
    let organization_public_id = Uuid::new_v4();
    let organization_slug = format!("org-{suffix}");
    let organization_id = insert_and_get_id(
        &db,
        format!(
            "INSERT INTO organizations (public_id, name, slug, status) VALUES ('{organization_public_id}', 'Smoke Test ISP', '{organization_slug}', 'active') RETURNING id"
        ),
    )
    .await?;

    let tenant_public_id = Uuid::new_v4();
    let tenant_slug = format!("tenant-{suffix}");
    let tenant_id = insert_and_get_id(
        &db,
        format!(
            "INSERT INTO tenants (public_id, organization_id, name, slug, is_active) VALUES ('{tenant_public_id}', {organization_id}, 'Production', '{tenant_slug}', TRUE) RETURNING id"
        ),
    )
    .await?;

    let region_public_id = Uuid::new_v4();
    let region_slug = format!("region-{suffix}");
    let region_id = insert_and_get_id(
        &db,
        format!(
            "INSERT INTO regions (public_id, tenant_id, name, slug, timezone, status) VALUES ('{region_public_id}', {tenant_id}, 'West Region', '{region_slug}', 'Asia/Jakarta', 'active') RETURNING id"
        ),
    )
    .await?;

    let site_public_id = Uuid::new_v4();
    let site_slug = format!("site-{suffix}");
    let site_id = insert_and_get_id(
        &db,
        format!(
            "INSERT INTO sites (public_id, tenant_id, region_id, name, slug, site_type, timezone, status) VALUES ('{site_public_id}', {tenant_id}, {region_id}, 'Core Facility', '{site_slug}', 'facility', 'Asia/Jakarta', 'active') RETURNING id"
        ),
    )
    .await?;

    let pop_public_id = Uuid::new_v4();
    let pop_slug = format!("pop-{suffix}");
    db.execute_unprepared(&format!(
        "INSERT INTO pops (public_id, tenant_id, site_id, name, slug, pop_type, status) VALUES ('{pop_public_id}', {tenant_id}, {site_id}, 'Core POP', '{pop_slug}', 'core', 'active')"
    ))
    .await?;

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
