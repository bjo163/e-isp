use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pops")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub public_id: Uuid,
    pub tenant_id: i64,
    pub site_id: i64,
    pub name: String,
    pub slug: String,
    pub pop_type: String,
    pub status: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
