#![allow(elided_lifetimes_in_paths)]

pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Domain migrations are added alongside their corresponding bounded issues.
        Vec::new()
    }
}
