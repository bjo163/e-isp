use loco_rs::schema::{self, ColType};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        schema::create_table(
            manager,
            "users",
            &[
                ("id", ColType::PkAuto),
                ("pid", ColType::UuidUniq),
                ("email", ColType::StringUniq),
                ("password", ColType::String),
                ("name", ColType::String),
                ("is_active", ColType::BooleanWithDefault(true)),
            ],
            &[],
        )
        .await?;

        schema::create_table(
            manager,
            "organizations",
            &[
                ("id", ColType::PkAuto),
                ("public_id", ColType::UuidUniq),
                ("name", ColType::String),
                ("slug", ColType::StringUniq),
                ("status", ColType::StringWithDefault("active".to_owned())),
            ],
            &[],
        )
        .await?;

        schema::create_table(
            manager,
            "tenants",
            &[
                ("id", ColType::PkAuto),
                ("public_id", ColType::UuidUniq),
                ("organization_id", ColType::BigInteger),
                ("name", ColType::String),
                ("slug", ColType::StringUniq),
                ("is_active", ColType::BooleanWithDefault(true)),
            ],
            &[("organizations", "organization_id")],
        )
        .await?;

        schema::create_table(
            manager,
            "memberships",
            &[
                ("id", ColType::PkAuto),
                ("public_id", ColType::UuidUniq),
                ("user_id", ColType::BigInteger),
                ("tenant_id", ColType::BigInteger),
                ("role", ColType::StringWithDefault("viewer".to_owned())),
                ("is_active", ColType::BooleanWithDefault(true)),
            ],
            &[("users", "user_id"), ("tenants", "tenant_id")],
        )
        .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_memberships_user_tenant")
                    .table("memberships")
                    .col(Alias::new("user_id"))
                    .col(Alias::new("tenant_id"))
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uq_memberships_user_tenant")
                    .table("memberships")
                    .to_owned(),
            )
            .await?;
        schema::drop_table(manager, "memberships").await?;
        schema::drop_table(manager, "tenants").await?;
        schema::drop_table(manager, "organizations").await?;
        schema::drop_table(manager, "users").await?;
        Ok(())
    }
}
