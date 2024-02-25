use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter, CreateMessage};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use serde_json::json;
use strum_macros::{Display, EnumString, EnumVariantNames};
use ts_rs::TS;

use crate::{
    impls::{self, target_types::TargetType, utils::get_user_perms},
    Error,
};
use kittycat::perms;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, TS, EnumString, EnumVariantNames, Display, Clone)]
#[ts(export, export_to = ".generated/RPCMethod.ts")]
pub enum RPCMethod {
    PremiumAdd {
        target_id: String,
        reason: String,
        premium_tier: String,
        time_period_hours: i32,
    },
    PremiumRemove {
        target_id: String,
        reason: String,
    },
    VoteBanAdd {
        target_id: String,
        reason: String,
    },
    VoteBanRemove {
        target_id: String,
        reason: String,
    },
    RemoveBot {
        target_id: String,
        reason: String,
    },
}

impl Default for RPCMethod {
    fn default() -> Self {
        RPCMethod::PremiumAdd {
            target_id: "bot_id".to_string(),
            reason: "reason".to_string(),
            premium_tier: "basic".to_string(),
            time_period_hours: 24,
        }
    }
}

pub enum RPCSuccess {
    NoContent,
    Content(String),
}

impl RPCSuccess {
    pub fn content(&self) -> Option<&str> {
        match self {
            RPCSuccess::Content(c) => Some(c),
            _ => None,
        }
    }
}

/// Represents a single RPC field
#[derive(Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = ".generated/RPCField.ts")]
pub struct RPCField {
    pub id: String,
    pub label: String,
    pub field_type: FieldType,
    pub icon: String,
    pub placeholder: String,
}

impl RPCField {
    fn target_id() -> Self {
        RPCField {
            id: "target_id".to_string(),
            label: "Target ID".to_string(),
            field_type: FieldType::Text,
            icon: "ic:twotone-access-time-filled".to_string(),
            placeholder: "The Target ID to perform the action on".to_string(),
        }
    }

    fn reason() -> Self {
        RPCField {
            id: "reason".to_string(),
            label: "Reason".to_string(),
            field_type: FieldType::Textarea,
            icon: "material-symbols:question-mark".to_string(),
            placeholder: "Reason for performing this action".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = ".generated/RPCFieldType.ts")]
// Allow dead code
#[allow(dead_code)]
/// Represents a field type
pub enum FieldType {
    Text,
    Textarea,
    Number,
    Hour, // Time expressed as a number of hours
    Boolean,
}

pub struct RPCHandle {
    pub pool: PgPool,
    pub cache_http: impls::cache::CacheHttpImpl,
    pub user_id: String,
    pub target_type: TargetType,
}

impl RPCMethod {
    pub fn supported_target_types(&self) -> Vec<TargetType> {
        match self {
            RPCMethod::PremiumAdd { .. } => vec![TargetType::Guild],
            RPCMethod::PremiumRemove { .. } => vec![TargetType::Guild],
            RPCMethod::VoteBanAdd { .. } => vec![TargetType::User],
            RPCMethod::VoteBanRemove { .. } => vec![TargetType::User],
            RPCMethod::RemoveBot { .. } => vec![TargetType::Guild],
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::PremiumAdd { .. } => "Adds premium to an entity for a given time period",
            Self::PremiumRemove { .. } => "Removes premium from an entity",
            Self::VoteBanAdd { .. } => "Vote-bans the entity in question",
            Self::VoteBanRemove { .. } => "Removes the vote-ban from the entity in question",
            Self::RemoveBot { .. } => {
                "Removes the bot from the entity"
            }
        }
        .to_string()
    }

    pub fn label(&self) -> String {
        match self {
            Self::PremiumAdd { .. } => "Add Premium",
            Self::PremiumRemove { .. } => "Remove Premium",
            Self::VoteBanAdd { .. } => "Vote Ban",
            Self::VoteBanRemove { .. } => "Unvote Ban",
            Self::RemoveBot { .. } => "Remove Bot",
        }
        .to_string()
    }

    pub async fn handle(&self, state: RPCHandle) -> Result<RPCSuccess, Error> {
        // First ensure that target type on handle is in supported target types
        if !self.supported_target_types().contains(&state.target_type) {
            return Err("This method does not support this target type yet".into());
        }

        // Next, ensure we have the permissions needed
        let user_perms = get_user_perms(&state.pool, &state.user_id)
        .await?
        .resolve();

        let required_perm = perms::build("rpc", &self.to_string());
        if !perms::has_perm(&user_perms, &required_perm) {
            return Err(format!(
                "You need {} permission to use {}",
                required_perm,
                &self.to_string()
            )
            .into());
        }

        // Insert into rpc_logs
        let id = sqlx::query!(
            "INSERT INTO rpc_logs (method, user_id, data) VALUES ($1, $2, $3) RETURNING id",
            self.to_string(),
            &state.user_id,
            json!(self)
        )
        .fetch_one(&state.pool)
        .await?;

        // Get number of requests in the last 7 minutes
        let res = sqlx::query!(
            "SELECT COUNT(*) FROM rpc_logs WHERE user_id = $1 AND NOW() - created_at < INTERVAL '7 minutes'",
            &state.user_id
        )
        .fetch_one(&state.pool)
        .await
        .map_err(|_| "Failed to get ratelimit count")?;

        let count = res.count.unwrap_or_default();
        
        if count > 5 {
            sqlx::query!(
                "DELETE FROM staffpanel__authchain WHERE user_id = $1",
                &state.user_id,
            )
            .execute(&state.pool)
            .await
            .map_err(|_| "Failed to reset user token")?;

            return Err("Rate limit exceeded. Wait 5-10 minutes and try again?".into());
        }

        // Now we can handle the method
        let resp = self.handle_method(&state).await;

        if resp.is_ok() {
            sqlx::query!(
                "UPDATE rpc_logs SET state = $1 WHERE id = $2",
                "success",
                id.id
            )
            .execute(&state.pool)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE rpc_logs SET state = $1 WHERE id = $2",
                resp.as_ref()
                    .err()
                    .ok_or("Err variant doesnt have an error!")?
                    .to_string(),
                id.id
            )
            .execute(&state.pool)
            .await?;
        }

        resp
    }

    /// The low-level method handler
    async fn handle_method(&self, state: &RPCHandle) -> Result<RPCSuccess, Error> {
        match self {
            RPCMethod::PremiumAdd {
                target_id,
                reason,
                premium_tier,
                time_period_hours,
            } => {
                match state.target_type {
                    TargetType::Guild => {
                        // Ensure the server actually exists
                        let server = sqlx::query!("SELECT COUNT(*) FROM guilds WHERE id = $1", target_id)
                            .fetch_one(&state.pool)
                            .await?;

                        if server.count.unwrap_or_default() == 0 {
                            return Err("Server does not exist".into());
                        }

                        // Set premium_period_length which is a postgres interval
                        sqlx::query!(
                            "UPDATE guilds SET start_premium_period = NOW(), premium_period_length = make_interval(hours => $1), premium_tier = $2 WHERE id = $3",
                            time_period_hours,
                            premium_tier,
                            target_id
                        )
                        .execute(&state.pool)
                        .await?;
                    },
                    TargetType::User => todo!("User premium support is currently planned but not implemented"),
                }

                let msg = CreateMessage::new().embed(
                    CreateEmbed::default()
                        .title("Premium Added!")
                        .description(format!(
                            "<@{}> has added premium tier {} to <@{}> for {} hours",
                            &state.user_id, premium_tier, target_id, time_period_hours
                        ))
                        .field("Reason", reason, true)
                        .footer(CreateEmbedFooter::new(
                            "Well done, young traveller! Use it wisely...",
                        ))
                        .color(0x00ff00),
                );

                crate::config::CONFIG
                    .channels
                    .mod_logs
                    .send_message(&state.cache_http, msg)
                    .await?;

                Ok(RPCSuccess::NoContent)
            }
            RPCMethod::PremiumRemove { target_id, reason } => {
                match state.target_type {
                    TargetType::Guild => {
                        // Ensure the server actually exists
                        let server = sqlx::query!("SELECT COUNT(*) FROM guilds WHERE id = $1", target_id)
                            .fetch_one(&state.pool)
                            .await?;

                        if server.count.unwrap_or_default() == 0 {
                            return Err("Server does not exist".into());
                        }

                        sqlx::query!(
                            "UPDATE guilds SET premium_tier = NULL WHERE id = $1",
                            target_id
                        )
                        .execute(&state.pool)
                        .await?;
                    },
                    TargetType::User => todo!("User premium support is currently planned but not implemented"),
                }

                let msg = CreateMessage::new().embed(
                    CreateEmbed::default()
                        .title("Premium Removed!")
                        .description(format!(
                            "<@{}> has removed premium from <@{}>",
                            state.user_id, target_id
                        ))
                        .field("Reason", reason, true)
                        .footer(CreateEmbedFooter::new(
                            "Well done, young traveller. Sad to see you go...",
                        ))
                        .color(0xFF0000),
                );

                crate::config::CONFIG
                    .channels
                    .mod_logs
                    .send_message(&state.cache_http, msg)
                    .await?;

                Ok(RPCSuccess::NoContent)
            }
            RPCMethod::VoteBanAdd { target_id, reason } => {
                match state.target_type {
                    TargetType::Guild => {
                        // Ensure the server actually exists
                        let server = sqlx::query!("SELECT COUNT(*) FROM guilds WHERE id = $1", target_id)
                            .fetch_one(&state.pool)
                            .await?;

                        if server.count.unwrap_or_default() == 0 {
                            return Err("Server does not exist".into());
                        }

                        sqlx::query!(
                            "UPDATE guilds SET vote_banned = true WHERE id = $1",
                            target_id
                        )
                        .execute(&state.pool)
                        .await?;
                    },
                    TargetType::User => {
                        // Ensure the user actually exists
                        let user = sqlx::query!("SELECT COUNT(*) FROM users WHERE user_id = $1", target_id)
                            .fetch_one(&state.pool)
                            .await?;

                        if user.count.unwrap_or_default() == 0 {
                            return Err("User does not exist".into());
                        }

                        sqlx::query!(
                            "UPDATE users SET vote_banned = true WHERE user_id = $1",
                            target_id
                        )
                        .execute(&state.pool)
                        .await?;
                    },
                }

                let msg = CreateMessage::new().embed(
                    CreateEmbed::default()
                        .title("Vote Ban Edit!")
                        .description(format!(
                            "<@{}> has set the vote ban on <@{}>",
                            state.user_id, target_id,
                        ))
                        .field("Reason", reason, true)
                        .footer(CreateEmbedFooter::new(
                            "Remember: don't abuse our services!",
                        ))
                        .color(0xFF0000),
                );

                crate::config::CONFIG
                    .channels
                    .mod_logs
                    .send_message(&state.cache_http, msg)
                    .await?;

                Ok(RPCSuccess::NoContent)
            }
            RPCMethod::VoteBanRemove { target_id, reason } => {
                match state.target_type {
                    TargetType::Guild => {
                        // Ensure the server actually exists
                        let server = sqlx::query!("SELECT COUNT(*) FROM guilds WHERE id = $1", target_id)
                            .fetch_one(&state.pool)
                            .await?;

                        if server.count.unwrap_or_default() == 0 {
                            return Err("Server does not exist".into());
                        }

                        sqlx::query!(
                            "UPDATE guilds SET vote_banned = false WHERE id = $1",
                            target_id
                        )
                        .execute(&state.pool)
                        .await?;
                    },
                    TargetType::User => {
                        // Ensure the user actually exists
                        let user = sqlx::query!("SELECT COUNT(*) FROM users WHERE user_id = $1", target_id)
                            .fetch_one(&state.pool)
                            .await?;

                        if user.count.unwrap_or_default() == 0 {
                            return Err("User does not exist".into());
                        }

                        sqlx::query!(
                            "UPDATE users SET vote_banned = false WHERE user_id = $1",
                            target_id
                        )
                        .execute(&state.pool)
                        .await?;
                    },
                }

                let msg = CreateMessage::new().embed(
                    CreateEmbed::default()
                        .title("Vote Ban Removed!")
                        .description(format!(
                            "<@{}> has removed the vote ban on <@{}>",
                            state.user_id, target_id,
                        ))
                        .field("Reason", reason, true)
                        .footer(CreateEmbedFooter::new(
                            "Remember: don't abuse our services!",
                        ))
                        .color(0xFF0000),
                );

                crate::config::CONFIG
                    .channels
                    .mod_logs
                    .send_message(&state.cache_http, msg)
                    .await?;

                Ok(RPCSuccess::NoContent)
            },
            RPCMethod::RemoveBot { .. } => {
                Err("Not implemented yet".into())
            }
        }
    }

    // Returns a set of RPCField's for a given enum variant
    pub fn method_fields(&self) -> Vec<RPCField> {
        match self {
            RPCMethod::PremiumAdd { .. } => vec![
                RPCField::target_id(),
                RPCField {
                    id: "premium_tier".to_string(),
                    label: "Premium Tier".to_string(),
                    field_type: FieldType::Text,
                    icon: "material-symbols:timer".to_string(),
                    placeholder: "Premium tier (e.g: trial/basic etc.)".to_string(),
                },
                RPCField {
                    id: "time_period_hours".to_string(),
                    label: "Time [X unit(s)]".to_string(),
                    field_type: FieldType::Hour,
                    icon: "material-symbols:timer".to_string(),
                    placeholder: "Time period. Format: X years/days/hours".to_string(),
                },
                RPCField::reason(),
            ],
            RPCMethod::PremiumRemove { .. } => vec![RPCField::target_id(), RPCField::reason()],
            RPCMethod::VoteBanAdd { .. } => vec![RPCField::target_id(), RPCField::reason()],
            RPCMethod::VoteBanRemove { .. } => vec![RPCField::target_id(), RPCField::reason()],
            RPCMethod::RemoveBot { .. } => vec![RPCField::target_id(), RPCField::reason()],
        }
    }
}
