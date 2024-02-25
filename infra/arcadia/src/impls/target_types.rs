use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, EnumVariantNames};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(
    Serialize, Deserialize, PartialEq, EnumString, ToSchema, TS, EnumVariantNames, Clone, Default,
)]
#[ts(export, export_to = ".generated/TargetType.ts")]
pub enum TargetType {
    #[default]
    Guild,
    User,
}

impl Display for TargetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetType::Guild => write!(f, "guilds"),
            TargetType::User => write!(f, "user"),
        }
    }
}

impl TargetType {
    #[allow(dead_code)] // TODO: Use this basic support
    pub fn id(&self) -> String {
        match self {
            TargetType::Guild => "id".to_string(),
            TargetType::User => "user_id".to_string(),
        }
    }
}