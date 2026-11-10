use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "sites")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub public_id: Uuid,
    pub tenant_id: i64,
    pub region_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub site_type: String,
    pub timezone: String,
    pub status: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
