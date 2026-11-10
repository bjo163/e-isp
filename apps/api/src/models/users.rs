use loco_rs::{
    auth::jwt,
    hash,
    prelude::*,
};
use sea_orm::{ActiveModelBehavior, ActiveValue, ConnectionTrait, EntityTrait, QueryFilter};
use serde_json::Map;
use uuid::Uuid;

pub use super::_entities::users::{self, ActiveModel, Entity, Model};

#[async_trait]
impl ActiveModelBehavior for users::ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            let mut active = self;
            active.pid = ActiveValue::Set(Uuid::new_v4());
            Ok(active)
        } else {
            Ok(self)
        }
    }
}

impl Model {
    pub async fn find_by_email<C>(db: &C, email: &str) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        let user = Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await?;
        user.ok_or(ModelError::EntityNotFound)
    }

    pub async fn find_by_pid<C>(db: &C, pid: &str) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        let pid = Uuid::parse_str(pid).map_err(|error| ModelError::Any(error.into()))?;
        let user = Entity::find()
            .filter(users::Column::Pid.eq(pid))
            .one(db)
            .await?;
        user.ok_or(ModelError::EntityNotFound)
    }

    #[must_use]
    pub fn verify_password(&self, password: &str) -> bool {
        hash::verify_password(password, &self.password)
    }

    pub fn generate_jwt(&self, secret: &str, expiration: u64) -> ModelResult<String> {
        jwt::JWT::new(secret)
            .generate_token(expiration, self.pid.to_string(), Map::new())
            .map_err(ModelError::from)
    }

    pub async fn create_with_password<C>(
        db: &C,
        email: &str,
        name: &str,
        password: &str,
    ) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        let normalized_email = email.trim().to_ascii_lowercase();
        if normalized_email.len() > 254 || !normalized_email.contains('@') {
            return Err(ModelError::msg("invalid email address"));
        }
        if name.trim().len() < 2 || name.len() > 120 {
            return Err(ModelError::msg("name must contain 2 to 120 characters"));
        }
        if password.chars().count() < 12 || password.chars().count() > 256 {
            return Err(ModelError::msg("password must contain 12 to 256 characters"));
        }

        let existing = Entity::find()
            .filter(users::Column::Email.eq(&normalized_email))
            .one(db)
            .await?;
        if existing.is_some() {
            return Err(ModelError::EntityAlreadyExists {});
        }

        let password_hash =
            hash::hash_password(password).map_err(|error| ModelError::Any(error.into()))?;
        let user = users::ActiveModel {
            email: ActiveValue::Set(normalized_email),
            password: ActiveValue::Set(password_hash),
            name: ActiveValue::Set(name.trim().to_owned()),
            is_active: ActiveValue::Set(true),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(user)
    }
}
