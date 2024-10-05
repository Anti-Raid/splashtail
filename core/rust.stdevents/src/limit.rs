use serde::{Deserialize, Serialize};
use silverpelt::ar_event::{typetag, AntiraidCustomEvent};
use strum_macros::{Display, EnumString, VariantNames};

pub const LIMIT_TARGET_ID: u64 = 0x1;

/// Standard limit types.
#[derive(
    EnumString,
    Display,
    PartialEq,
    VariantNames,
    Clone,
    Copy,
    Debug,
    Serialize,
    Hash,
    Eq,
    Deserialize,
)]
#[strum(serialize_all = "snake_case")]
pub enum LimitTypes {
    MemberAdd,
    RoleAdd,               // set
    RoleUpdate,            // set
    RoleRemove,            // set
    RoleGivenToMember,     // set
    RoleRemovedFromMember, // set
    MemberRolesUpdated,    // set
    ChannelAdd,            // set
    ChannelUpdate,         // set
    ChannelRemove,         // set
    Kick,                  // set
    Ban,                   // set
    Unban,                 // set
    MessageCreate,         // set
    PruneMembers,          // set
    Custom(u8),
}

/// A handle limit action event is dispatched on operations performed by users that can be limited
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HandleLimitActionEvent {
    pub user_id: serenity::all::UserId,
    pub limit: LimitTypes,
    pub target: Option<String>,
    pub action_data: serde_json::Value,

    /// The channel to send the response to
    ///
    /// Optional, if unset, the response MUST NOT attempt to be sent
    #[serde(skip)]
    pub send_chan: Option<tokio::sync::mpsc::Sender<HandleLimitActionEventResponse>>,
}

/// A handle limit action event response is sent back to the requesting module on handling of an event
pub struct HandleLimitActionEventResponse {
    pub is_limited: bool,
}

#[typetag::serde]
impl AntiraidCustomEvent for HandleLimitActionEvent {
    fn target(&self) -> u64 {
        LIMIT_TARGET_ID
    }

    fn event_name(&self) -> &'static str {
        "Limits:HandleLimitActionEvent"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
