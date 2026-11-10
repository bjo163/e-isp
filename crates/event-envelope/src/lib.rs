//! Typed, versioned transport envelopes for normalized collector events.

use chrono::{DateTime, Utc};
use e_isp_domain_core::TenantId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CURRENT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub event_id: Uuid,
    pub schema_version: u16,
    pub tenant_id: TenantId,
    pub source_id: String,
    pub target_id: String,
    pub occurred_at: DateTime<Utc>,
    pub ingested_at: DateTime<Utc>,
    pub payload: T,
}

impl<T> EventEnvelope<T> {
    #[must_use]
    pub fn new(
        tenant_id: TenantId,
        source_id: String,
        target_id: String,
        occurred_at: DateTime<Utc>,
        payload: T,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            schema_version: CURRENT_SCHEMA_VERSION,
            tenant_id,
            source_id,
            target_id,
            occurred_at,
            ingested_at: Utc::now(),
            payload,
        }
    }

    #[must_use]
    pub const fn is_supported_version(&self) -> bool {
        self.schema_version == CURRENT_SCHEMA_VERSION
    }

    pub fn validate(&self) -> Result<(), EnvelopeValidationError> {
        if self.source_id.trim().is_empty() {
            return Err(EnvelopeValidationError::EmptySource);
        }
        if self.target_id.trim().is_empty() {
            return Err(EnvelopeValidationError::EmptyTarget);
        }
        if self.source_id.len() > 128 || self.target_id.len() > 128 {
            return Err(EnvelopeValidationError::IdentifierTooLong);
        }
        if !self.is_supported_version() {
            return Err(EnvelopeValidationError::UnsupportedSchema(
                self.schema_version,
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeValidationError {
    EmptySource,
    EmptyTarget,
    IdentifierTooLong,
    UnsupportedSchema(u16),
}

impl std::fmt::Display for EnvelopeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySource => f.write_str("source_id must not be empty"),
            Self::EmptyTarget => f.write_str("target_id must not be empty"),
            Self::IdentifierTooLong => {
                f.write_str("source and target identifiers must be at most 128 bytes")
            }
            Self::UnsupportedSchema(version) => {
                write!(f, "unsupported event schema version: {version}")
            }
        }
    }
}

impl std::error::Error for EnvelopeValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestPayload {
        interface_state: String,
    }

    #[test]
    fn envelope_uses_current_schema_and_round_trips() {
        let tenant_id = TenantId::new();
        let event = EventEnvelope::new(
            tenant_id,
            "collector-a".to_owned(),
            "device-1".to_owned(),
            Utc::now(),
            TestPayload {
                interface_state: "up".to_owned(),
            },
        );
        assert!(event.validate().is_ok());
        let encoded = serde_json::to_string(&event);
        assert!(encoded.is_ok());
        if let Ok(encoded) = encoded {
            let decoded = serde_json::from_str::<EventEnvelope<TestPayload>>(&encoded);
            assert_eq!(decoded.ok(), Some(event));
        }
    }

    #[test]
    fn envelope_rejects_empty_source() {
        let event = EventEnvelope::new(
            TenantId::new(),
            String::new(),
            "device-1".to_owned(),
            Utc::now(),
            TestPayload {
                interface_state: "up".to_owned(),
            },
        );
        assert_eq!(event.validate(), Err(EnvelopeValidationError::EmptySource));
    }

    #[test]
    fn envelope_rejects_unsupported_schema() {
        let mut event = EventEnvelope::new(
            TenantId::new(),
            "collector-a".to_owned(),
            "device-1".to_owned(),
            Utc::now(),
            TestPayload {
                interface_state: "up".to_owned(),
            },
        );
        event.schema_version = CURRENT_SCHEMA_VERSION + 1;
        assert_eq!(
            event.validate(),
            Err(EnvelopeValidationError::UnsupportedSchema(
                CURRENT_SCHEMA_VERSION + 1
            ))
        );
    }
}
