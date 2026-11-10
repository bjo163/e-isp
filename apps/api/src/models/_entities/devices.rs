use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "devices")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub public_id: Uuid,
    pub tenant_id: i64,
    pub site_id: Option<i64>,
    pub pop_id: Option<i64>,
    pub vendor_id: Option<i64>,
    pub device_model_id: Option<i64>,
    pub inventory_key: String,
    pub name: String,
    pub hostname: Option<String>,
    pub management_address: Option<String>,
    pub serial_number: Option<String>,
    pub device_role: String,
    pub lifecycle_status: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
