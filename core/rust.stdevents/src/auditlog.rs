use gwevent::field::Field;
use silverpelt::ar_event::{typetag, AntiraidCustomEvent};

pub const AUDITLOG_TARGET_ID: u64 = 0x0;

/// This can be used to trigger a custom audit log dispatch event
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuditLogDispatchEvent {
    pub event_name: String,
    pub event_titlename: String,
    pub event_data: indexmap::IndexMap<String, Field>,
}

#[typetag::serde]
impl AntiraidCustomEvent for AuditLogDispatchEvent {
    fn target(&self) -> u64 {
        AUDITLOG_TARGET_ID
    }

    fn event_name(&self) -> &'static str {
        "AuditLog:DispatchEvent"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
