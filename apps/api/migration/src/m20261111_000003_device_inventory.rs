use loco_rs::schema::{self, ColType};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        schema::create_table(
            manager,
            "vendors",
            &[
                ("id", ColType::PkAuto),
                ("public_id", ColType::UuidUniq),
                ("slug", ColType::StringUniq),
                ("display_name", ColType::String),
            ],
            &[],
        )
        .await?;

        schema::create_table(
            manager,
            "device_models",
            &[
                ("id", ColType::PkAuto),
                ("public_id", ColType::UuidUniq),
                ("vendor_id", ColType::BigInteger),
                ("model_code", ColType::String),
                ("display_name", ColType::String),
            ],
            &[("vendors", "vendor_id")],
        )
        .await?;

        schema::create_table(
            manager,
            "devices",
            &[
                ("id", ColType::PkAuto),
                ("public_id", ColType::UuidUniq),
                ("tenant_id", ColType::BigInteger),
                ("site_id", ColType::BigIntegerNull),
                ("pop_id", ColType::BigIntegerNull),
                ("vendor_id", ColType::BigIntegerNull),
                ("device_model_id", ColType::BigIntegerNull),
                ("inventory_key", ColType::String),
                ("name", ColType::String),
                ("hostname", ColType::StringNull),
                ("management_address", ColType::StringNull),
                ("serial_number", ColType::StringNull),
                ("device_role", ColType::StringWithDefault("other".to_owned())),
                ("lifecycle_status", ColType::StringWithDefault("planned".to_owned())),
            ],
            &[
                ("tenants", "tenant_id"),
                ("sites?", "site_id"),
                ("pops?", "pop_id"),
                ("vendors?", "vendor_id"),
                ("device_models?", "device_model_id"),
            ],
        )
        .await?;

        schema::create_table(
            manager,
            "device_interfaces",
            &[
                ("id", ColType::PkAuto),
                ("public_id", ColType::UuidUniq),
                ("tenant_id", ColType::BigInteger),
                ("device_id", ColType::BigInteger),
                ("stable_key", ColType::String),
                ("if_index", ColType::IntegerNull),
                ("name", ColType::String),
                ("description", ColType::StringNull),
                ("interface_type", ColType::StringWithDefault("unknown".to_owned())),
                ("admin_state", ColType::StringWithDefault("unknown".to_owned())),
                ("operational_state", ColType::StringWithDefault("unknown".to_owned())),
            ],
            &[("tenants", "tenant_id"), ("devices", "device_id")],
        )
        .await?;

        for (name, table, columns) in [
            ("uq_device_models_vendor_code", "device_models", vec!["vendor_id", "model_code"]),
            ("uq_devices_tenant_inventory_key", "devices", vec!["tenant_id", "inventory_key"]),
            ("uq_device_interfaces_device_stable_key", "device_interfaces", vec!["device_id", "stable_key"]),
        ] {
            let mut index = Index::create().name(name).table(table);
            for column in columns {
                index = index.col(Alias::new(column));
            }
            manager.create_index(index.unique().to_owned()).await?;
        }

        manager
            .create_index(
                Index::create()
                    .name("idx_devices_tenant_site")
                    .table("devices")
                    .col(Alias::new("tenant_id"))
                    .col(Alias::new("site_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_devices_tenant_pop")
                    .table("devices")
                    .col(Alias::new("tenant_id"))
                    .col(Alias::new("pop_id"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (name, table) in [
            ("idx_devices_tenant_pop", "devices"),
            ("idx_devices_tenant_site", "devices"),
            ("uq_device_interfaces_device_stable_key", "device_interfaces"),
            ("uq_devices_tenant_inventory_key", "devices"),
            ("uq_device_models_vendor_code", "device_models"),
        ] {
            manager
                .drop_index(Index::drop().name(name).table(table).to_owned())
                .await?;
        }

        schema::drop_table(manager, "device_interfaces").await?;
        schema::drop_table(manager, "devices").await?;
        schema::drop_table(manager, "device_models").await?;
        schema::drop_table(manager, "vendors").await?;
        Ok(())
    }
}
