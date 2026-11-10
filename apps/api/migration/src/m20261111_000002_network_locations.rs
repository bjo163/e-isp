use loco_rs::schema::{self, ColType};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        schema::create_table(
            manager,
            "regions",
            &[
                ("id", ColType::PkAuto),
                ("public_id", ColType::UuidUniq),
                ("tenant_id", ColType::BigInteger),
                ("name", ColType::String),
                ("slug", ColType::String),
                ("timezone", ColType::StringWithDefault("UTC".to_owned())),
                ("status", ColType::StringWithDefault("active".to_owned())),
            ],
            &[("tenants", "tenant_id")],
        )
        .await?;

        schema::create_table(
            manager,
            "sites",
            &[
                ("id", ColType::PkAuto),
                ("public_id", ColType::UuidUniq),
                ("tenant_id", ColType::BigInteger),
                ("region_id", ColType::BigIntegerNull),
                ("name", ColType::String),
                ("slug", ColType::String),
                ("site_type", ColType::StringWithDefault("facility".to_owned())),
                ("timezone", ColType::StringWithDefault("UTC".to_owned())),
                ("status", ColType::StringWithDefault("active".to_owned())),
            ],
            &[("tenants", "tenant_id"), ("regions?", "region_id")],
        )
        .await?;

        schema::create_table(
            manager,
            "pops",
            &[
                ("id", ColType::PkAuto),
                ("public_id", ColType::UuidUniq),
                ("tenant_id", ColType::BigInteger),
                ("site_id", ColType::BigInteger),
                ("name", ColType::String),
                ("slug", ColType::String),
                ("pop_type", ColType::StringWithDefault("network".to_owned())),
                ("status", ColType::StringWithDefault("active".to_owned())),
            ],
            &[("tenants", "tenant_id"), ("sites", "site_id")],
        )
        .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_regions_tenant_slug")
                    .table("regions")
                    .col(Alias::new("tenant_id"))
                    .col(Alias::new("slug"))
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_sites_tenant_slug")
                    .table("sites")
                    .col(Alias::new("tenant_id"))
                    .col(Alias::new("slug"))
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_pops_tenant_slug")
                    .table("pops")
                    .col(Alias::new("tenant_id"))
                    .col(Alias::new("slug"))
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_pops_site")
                    .table("pops")
                    .col(Alias::new("site_id"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (index_name, table_name) in [
            ("idx_pops_site", "pops"),
            ("uq_pops_tenant_slug", "pops"),
            ("uq_sites_tenant_slug", "sites"),
            ("uq_regions_tenant_slug", "regions"),
        ] {
            manager
                .drop_index(
                    Index::drop()
                        .name(index_name)
                        .table(table_name)
                        .to_owned(),
                )
                .await?;
        }

        schema::drop_table(manager, "pops").await?;
        schema::drop_table(manager, "sites").await?;
        schema::drop_table(manager, "regions").await?;
        Ok(())
    }
}
