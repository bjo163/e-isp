#![allow(elided_lifetimes_in_paths)]

pub use sea_orm_migration::prelude::*;

mod m20261111_000001_identity_tenancy;
mod m20261111_000002_network_locations;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261111_000001_identity_tenancy::Migration),
            Box::new(m20261111_000002_network_locations::Migration),
        ]
    }
}
