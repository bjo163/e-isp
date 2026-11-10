use loco_rs::prelude::*;
use sea_orm::ActiveValue;
use uuid::Uuid;

use crate::models::{
    _entities::{memberships, organizations, tenants},
    users::Model as User,
};

pub struct AdminCreate;

#[async_trait]
impl Task for AdminCreate {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "admin:create".to_owned(),
            detail: "Create the first e-ISP administrator and its initial organization/tenant. Usage: cargo loco task admin:create email:admin@example.com name:Administrator password:<12+ chars> organization:Example tenant:Production org_slug:example tenant_slug:production".to_owned(),
        }
    }

    async fn run(&self, app_context: &AppContext, vars: &Vars) -> Result<()> {
        let email = vars.cli_arg("email")?;
        let name = vars.cli_arg("name")?;
        let password = vars.cli_arg("password")?;
        let organization_name = vars.cli_arg("organization")?;
        let tenant_name = vars.cli_arg("tenant")?;
        let organization_slug = vars.cli_arg("org_slug")?;
        let tenant_slug = vars.cli_arg("tenant_slug")?;

        validate_slug(organization_slug)?;
        validate_slug(tenant_slug)?;

        let transaction = app_context.db.begin().await?;
        let user = User::create_with_password(&transaction, email, name, password)
            .await
            .map_err(|error| Error::string(&format!("could not create administrator: {error}")))?;

        let organization = organizations::ActiveModel {
            public_id: ActiveValue::Set(Uuid::new_v4()),
            name: ActiveValue::Set(organization_name.trim().to_owned()),
            slug: ActiveValue::Set(organization_slug.to_ascii_lowercase()),
            status: ActiveValue::Set("active".to_owned()),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;

        let tenant = tenants::ActiveModel {
            public_id: ActiveValue::Set(Uuid::new_v4()),
            organization_id: ActiveValue::Set(organization.id),
            name: ActiveValue::Set(tenant_name.trim().to_owned()),
            slug: ActiveValue::Set(tenant_slug.to_ascii_lowercase()),
            is_active: ActiveValue::Set(true),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;

        memberships::ActiveModel {
            public_id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(user.id),
            tenant_id: ActiveValue::Set(tenant.id),
            role: ActiveValue::Set("owner".to_owned()),
            is_active: ActiveValue::Set(true),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;

        transaction.commit().await?;
        tracing::info!(
            user_id = %user.pid,
            organization_id = %organization.public_id,
            tenant_id = %tenant.public_id,
            "initial e-ISP administrator and tenant created"
        );
        Ok(())
    }
}

fn is_slug_character(character: char) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
}

fn is_alphanumeric(character: char) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit()
}

fn validate_slug(value: &str) -> Result<()> {
    let valid = !value.is_empty()
        && value.len() <= 63
        && value.chars().all(is_slug_character)
        && value.chars().next().is_some_and(is_alphanumeric)
        && value.chars().last().is_some_and(is_alphanumeric);

    if valid {
        Ok(())
    } else {
        Err(Error::string(
            "slug must be 1-63 lowercase letters, digits, or internal hyphens",
        ))
    }
}
