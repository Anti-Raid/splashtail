use crate::lang_lua::state;
use futures_util::StreamExt;
use mlua::prelude::*;
use serenity::all::Mentionable;
use std::sync::Arc;

/// An action executor is used to execute actions such as kick/ban/timeout from Lua
/// templates
pub struct DiscordActionExecutor {
    template_data: Arc<state::TemplateData>,
    guild_id: serenity::all::GuildId,
    serenity_context: serenity::all::Context,
    shard_messenger: serenity::all::ShardMessenger,
    cache_http: botox::cache::CacheHttpImpl,
    reqwest_client: reqwest::Client,
    ratelimits: Arc<state::LuaRatelimits>,
}

// @userdata DiscordActionExecutor
//
// Executes actions on discord
impl DiscordActionExecutor {
    pub fn check_action(&self, action: String) -> Result<(), crate::Error> {
        if !self
            .template_data
            .pragma
            .allowed_caps
            .contains(&format!("discord:{}", action))
        {
            return Err("Discord action not allowed in this template context".into());
        }

        self.ratelimits.check(&action)?;

        Ok(())
    }

    pub async fn user_in_guild(&self, user_id: serenity::all::UserId) -> Result<(), crate::Error> {
        let Some(member) = sandwich_driver::member_in_guild(
            &self.cache_http,
            &self.reqwest_client,
            self.guild_id,
            user_id,
        )
        .await?
        else {
            return Err("User not found in guild".into());
        };

        if member.user.id != user_id {
            return Err("User not found in guild".into());
        }

        Ok(())
    }

    pub async fn check_permissions(
        &self,
        user_id: serenity::all::UserId,
        needed_permissions: serenity::all::Permissions,
    ) -> Result<(), crate::Error> {
        let guild =
            sandwich_driver::guild(&self.cache_http, &self.reqwest_client, self.guild_id).await?; // Get the guild

        let Some(member) = sandwich_driver::member_in_guild(
            &self.cache_http,
            &self.reqwest_client,
            self.guild_id,
            user_id,
        )
        .await?
        else {
            return Err("Bot user not found in guild".into());
        }; // Get the bot user

        if !splashcore_rs::serenity_backport::member_permissions(&guild, &member)
            .contains(needed_permissions)
        {
            return Err(format!(
                "Bot does not have the required permissions: {:?}",
                needed_permissions
            )
            .into());
        }

        Ok(())
    }

    pub async fn check_permissions_and_hierarchy(
        &self,
        user_id: serenity::all::UserId,
        target_id: serenity::all::UserId,
        needed_permissions: serenity::all::Permissions,
    ) -> Result<(), crate::Error> {
        let guild =
            sandwich_driver::guild(&self.cache_http, &self.reqwest_client, self.guild_id).await?; // Get the guild

        let Some(member) = sandwich_driver::member_in_guild(
            &self.cache_http,
            &self.reqwest_client,
            self.guild_id,
            user_id,
        )
        .await?
        else {
            return Err(format!("User not found in guild: {}", user_id.mention()).into());
        }; // Get the bot user

        if !splashcore_rs::serenity_backport::member_permissions(&guild, &member)
            .contains(needed_permissions)
        {
            return Err(format!(
                "User does not have the required permissions: {:?}: {}",
                needed_permissions, user_id
            )
            .into());
        }

        let Some(target_member) = sandwich_driver::member_in_guild(
            &self.cache_http,
            &self.reqwest_client,
            self.guild_id,
            target_id,
        )
        .await?
        else {
            return Err("Target user not found in guild".into());
        }; // Get the target user

        let higher_id = guild
            .greater_member_hierarchy(&member, &target_member)
            .ok_or_else(|| {
                format!(
                    "User does not have a higher role than the target user: {}",
                    user_id.mention()
                )
            })?;

        if higher_id != member.user.id {
            return Err(format!(
                "User does not have a higher role than the target user: {}",
                user_id.mention()
            )
            .into());
        }

        Ok(())
    }
}

pub fn plugin_docs() -> templating_docgen::Plugin {
    templating_docgen::Plugin::default()
        .name("@antiraid/discord")
        .description("This plugin allows for templates to interact with the Discord API")
        // Serenity types
        .type_mut("Serenity.User", "A user object in Discord, as represented by AntiRaid. Internal fields are subject to change", |t| {
            t
            .example(std::sync::Arc::new(serenity::model::user::User::default()))
            .refers_to_serenity("serenity::model::user::User")
        })

        // audit log
        .type_mut("Serenity.AuditLogs", "A audit log in Discord, as represented by AntiRaid. Internal fields are subject to change", |t| {
            t
            .refers_to_serenity("serenity::model::guild::audit_log::AuditLogs")
        })
        .type_mut("Serenity.AuditLogs.Action", "An audit log action in Discord, as represented by AntiRaid. Internal fields are subject to change", |t| {
            t
            .example(std::sync::Arc::new(serenity::model::guild::audit_log::Action::GuildUpdate))
            .refers_to_serenity("serenity::model::guild::audit_log::Action")
        })

        // channel
        .type_mut("Serenity.GuildChannel", "A guild channel in Discord, as represented by AntiRaid. Internal fields are subject to change", |t| {
            t
            .example(std::sync::Arc::new(serenity::model::channel::GuildChannel::default()))
            .refers_to_serenity("serenity::model::channel::GuildChannel")
        })

        // permissions
        .type_mut("Serenity.PermissionOverwrite", "A permission overwrite in Discord, as represented by AntiRaid. Internal fields are subject to change", |t| {
            t
            .example(std::sync::Arc::new(serenity::model::channel::PermissionOverwrite {
                allow: serenity::model::permissions::Permissions::all(),
                deny: serenity::model::permissions::Permissions::all(),
                kind: serenity::model::channel::PermissionOverwriteType::Role(serenity::model::id::RoleId::default()),
            }))
            .refers_to_serenity("serenity::model::channel::PermissionOverwrite")
        })

        // forum emoji
        .type_mut("Serenity.ForumEmoji", "A forum emoji in Discord, as represented by AntiRaid. Internal fields are subject to change", |t| {
            t
            .example(std::sync::Arc::new(serenity::model::channel::ForumEmoji::Id(serenity::model::id::EmojiId::default())))
            .refers_to_serenity("serenity::model::channel::ForumEmoji")
        })

        // Options
        .type_mut("GetAuditLogOptions", "Options for getting audit logs in Discord", |t| {
            t
            .example(std::sync::Arc::new(types::GetAuditLogOptions::default()))
            .field("action_type", |f| {
                f
                .typ("Serenity.AuditLogs.Action?")
                .description("The action type to filter by")
            })
            .field("user_id", |f| {
                f
                .typ("string?")
                .description("The user ID to filter by")
            })
            .field("before", |f| {
                f
                .typ("string?")
                .description("The entry ID to filter by")
            })
            .field("limit", |f| {
                f
                .typ("number?")
                .description("The limit of entries to return")
            })
        })
        .type_mut("GetChannelOptions", "Options for getting a channel in Discord", |t| {
            t
            .example(std::sync::Arc::new(types::GetChannelOptions::default()))
            .field("channel_id", |f| {
                f
                .typ("string")
                .description("The channel ID to get")
            })
        })
        .type_mut("EditChannelOptions", "Options for editing a channel in Discord", |t| {
            t
            .example(std::sync::Arc::new(types::EditChannelOptions::default()))
            .field("channel_id", |f| {
                f
                .typ("string")
                .description("The channel ID to edit")
            })
            .field("reason", |f| {
                f
                .typ("string")
                .description("The reason for editing the channel")
            })
            .field("name", |f| {
                f
                .typ("string?")
                .description("The name of the channel")
            })
            .field("type", |f| {
                f
                .typ("string?")
                .description("The type of the channel")
            })
            .field("position", |f| {
                f
                .typ("number?")
                .description("The position of the channel")
            })
            .field("topic", |f| {
                f
                .typ("string?")
                .description("The topic of the channel")
            })
            .field("nsfw", |f| {
                f
                .typ("bool?")
                .description("Whether the channel is NSFW")
            })
            .field("rate_limit_per_user", |f| {
                f
                .typ("number?")
                .description("The rate limit per user/Slow mode of the channel")
            })
            .field("bitrate", |f| {
                f
                .typ("number?")
                .description("The bitrate of the channel")
            })
            .field("permission_overwrites", |f| {
                f
                .typ("{Serenity.PermissionOverwrite}?")
                .description("The permission overwrites of the channel")
            })
            .field("parent_id", |f| {
                f
                .typ("string??")
                .description("The parent ID of the channel")
            })
            .field("rtc_region", |f| {
                f
                .typ("string??")
                .description("The RTC region of the channel")
            })
            .field("video_quality_mode", |f| {
                f
                .typ("string?")
                .description("The video quality mode of the channel")
            })
            .field("default_auto_archive_duration", |f| {
                f
                .typ("string?")
                .description("The default auto archive duration of the channel")
            })
            .field("flags", |f| {
                f
                .typ("string?")
                .description("The flags of the channel")
            })
            .field("available_tags", |f| {
                f
                .typ("{Serenity.ForumTag}?")
                .description("The available tags of the channel")
            })
            .field("default_reaction_emoji", |f| {
                f
                .typ("Serenity.ForumEmoji??")
                .description("The default reaction emoji of the channel")
            })
            .field("default_thread_rate_limit_per_user", |f| {
                f
                .typ("number?")
                .description("The default thread rate limit per user")
            })
            .field("default_sort_order", |f| {
                f
                .typ("string?")
                .description("The default sort order of the channel")
            })
            .field("default_forum_layout", |f| {
                f
                .typ("string?")
                .description("The default forum layout of the channel")
            })
        })
        .type_mut("EditThreadOptions", "Options for editing a thread in Discord", |t| {
            t
            .example(std::sync::Arc::new(types::EditThreadOptions::default()))
            .field("channel_id", |f| {
                f
                .typ("string")
                .description("The channel ID to edit")
            })
            .field("reason", |f| {
                f
                .typ("string")
                .description("The reason for editing the channel")
            })
            .field("name", |f| {
                f
                .typ("string?")
                .description("The name of the thread")
            })
            .field("archived", |f| {
                f
                .typ("bool?")
                .description("Whether the thread is archived")
            })
            .field("auto_archive_duration", |f| {
                f
                .typ("string?")
                .description("The auto archive duration of the thread")
            })
            .field("locked", |f| {
                f
                .typ("bool?")
                .description("Whether the thread is locked")
            })
            .field("invitable", |f| {
                f
                .typ("bool?")
                .description("Whether the thread is invitable")
            })
            .field("rate_limit_per_user", |f| {
                f
                .typ("number?")
                .description("The rate limit per user/Slow mode of the thread")
            })
            .field("flags", |f| {
                f
                .typ("string?")
                .description("The flags of the thread")
            })
            .field("applied_tags", |f| {
                f
                .typ("{Serenity.ForumTag}?")
                .description("The applied tags of the thread")
            })
        })
        .type_mut("DeleteChannelOption", "Options for deleting a channel in Discord", |t| {
            t
            .example(std::sync::Arc::new(types::DeleteChannelOption::default()))
            .field("channel_id", |f| {
                f
                .typ("string")
                .description("The channel ID to delete")
            })
            .field("reason", |f| {
                f
                .typ("string")
                .description("The reason for deleting the channel")
            })
        })
        .type_mut("CreateMessageEmbedField", "A field in a message embed", |t| {
            t
            .example(std::sync::Arc::new(types::messages::CreateMessageEmbedField::default()))
            .field("name", |f| {
                f
                .typ("string")
                .description("The name of the field")
            })
            .field("value", |f| {
                f
                .typ("string")
                .description("The value of the field")
            })
            .field("inline", |f| {
                f
                .typ("bool")
                .description("Whether the field is inline")
            })
        })
        .type_mut("CreateMessageEmbedAuthor", "An author in a message embed", |t| {
            t
            .example(std::sync::Arc::new(types::messages::CreateMessageEmbedAuthor::default()))
            .field("name", |f| {
                f
                .typ("string")
                .description("The name of the author")
            })
            .field("url", |f| {
                f
                .typ("string?")
                .description("The URL of the author")
            })
            .field("icon_url", |f| {
                f
                .typ("string?")
                .description("The icon URL of the author")
            })
        })
        .type_mut("CreateMessageEmbedFooter", "A footer in a message embed", |t| {
            t
            .example(std::sync::Arc::new(types::messages::CreateMessageEmbedFooter::default()))
            .field("text", |f| {
                f
                .typ("string")
                .description("The text of the footer")
            })
            .field("icon_url", |f| {
                f
                .typ("string?")
                .description("The icon URL of the footer")
            })
        })
        .type_mut("CreateMessageEmbed", "An embed in a message", |t| {
            t
            .example(std::sync::Arc::new(types::messages::CreateMessageEmbed::default()))
            .field("title", |f| {
                f
                .typ("string?")
                .description("The title of the embed")
            })
            .field("description", |f| {
                f
                .typ("string?")
                .description("The description of the embed")
            })
            .field("url", |f| {
                f
                .typ("string?")
                .description("The URL of the embed")
            })
            .field("timestamp", |f| {
                f
                .typ("string?")
                .description("The timestamp of the embed")
            })
            .field("color", |f| {
                f
                .typ("string?")
                .description("The color of the embed")
            })
            .field("footer", |f| {
                f
                .typ("{Serenity.CreateMessageEmbedFooter}?")
                .description("The footer of the embed")
            })
            .field("image", |f| {
                f
                .typ("string?")
                .description("The image URL of the embed")
            })
            .field("thumbnail", |f| {
                f
                .typ("string?")
                .description("The thumbnail URL of the embed")
            })
            .field("author", |f| {
                f
                .typ("{Serenity.CreateMessageEmbedAuthor}?")
                .description("The author of the embed")
            })
            .field("fields", |f| {
                f
                .typ("{Serenity.CreateMessageEmbedField}?")
                .description("The fields of the embed")
            })
        })
        .type_mut("CreateMessageAttachment", "An attachment in a message", |t| {
            t
            .example(std::sync::Arc::new(types::messages::CreateMessageAttachment::default()))
            .field("filename", |f| {
                f
                .typ("string")
                .description("The filename of the attachment")
            })
            .field("description", |f| {
                f
                .typ("string?")
                .description("The description (if any) of the attachment")
            })
            .field("content", |f| {
                f
                .typ("{byte}")
                .description("The content of the attachment")
            })
        })
        .type_mut("CreateMessage", "Options for creating a message in Discord", |t| {
            t
            .example(std::sync::Arc::new(types::messages::CreateMessage::default()))
            .field("embeds", |f| {
                f
                .typ("{Serenity.CreateMessageEmbed}?")
                .description("The embeds of the message")
            })
            .field("content", |f| {
                f
                .typ("string?")
                .description("The content of the message")
            })
            .field("attachments", |f| {
                f
                .typ("{Serenity.CreateMessageAttachment}?")
                .description("The attachments of the message")
            })
        })
        .type_mut(
            "MessageHandle",
            "A handle to a message in Discord, as represented by AntiRaid. Internal fields are subject to change",
            |mut t| {
                t
                .method_mut("data", |m| {
                    m
                    .description("Gets the data of the message")
                    .return_("data", |r| {
                        r
                        .typ("any")
                        .description("The inner data of the message")
                    })
                })
            }
        )
        .type_mut(
            "DiscordExecutor",
            "DiscordExecutor allows templates to access/use the Discord API in a sandboxed form.",
            |mut t| {
                t
                .method_mut("get_audit_logs", |typ| {
                    typ
                    .description("Gets the audit logs")
                    .parameter("data", |p| {
                        p.typ("GetAuditLogOptions").description("Options for getting audit logs.")
                    })
                    .return_("SerenityAuditLogs", |p| {
                        p.description("The audit log entry")
                    })
                })
                .method_mut("get_channel", |typ| {
                    typ
                    .description("Gets a channel")
                    .parameter("data", |p| {
                        p.typ("GetChannelOptions").description("Options for getting a channel.")
                    })
                    .return_("Serenity.GuildChannel", |p| {
                        p.description("The guild channel")
                    })
                })
                .method_mut("edit_channel", |typ| {
                    typ
                    .description("Edits a channel")
                    .parameter("data", |p| {
                        p.typ("EditChannelOptions").description("Options for editing a channel.")
                    })
                    .return_("Serenity.GuildChannel", |p| {
                        p.description("The guild channel")
                    })
                })
                .method_mut("edit_thread", |typ| {
                    typ
                    .description("Edits a thread")
                    .parameter("data", |p| {
                        p.typ("EditThreadOptions").description("Options for editing a thread.")
                    })
                    .return_("Serenity.GuildChannel", |p| {
                        p.description("The guild channel")
                    })
                })
                .method_mut("delete_channel", |typ| {
                    typ
                    .description("Deletes a channel")
                    .parameter("data", |p| {
                        p.typ("DeleteChannelOption").description("Options for deleting a channel.")
                    })
                    .return_("Serenity.GuildChannel", |p| {
                        p.description("The guild channel")
                    })
                })
                .method_mut("create_message", |typ| {
                    typ
                    .description("Creates a message")
                    .parameter("data", |p| {
                        p.typ("CreateMessage").description("Options for creating a message.")
                    })
                    .return_("MessageHandle", |p| {
                        p.description("The message")
                    })
                })
            }
        )
        .method_mut("new", |mut m| {
            m.parameter("token", |p| p.typ("TemplateContext").description("The token of the template to use."))
            .return_("executor", |r| r.typ("DiscordExecutor").description("A discord executor."))
        })
}

mod types {
    use crate::lang_lua::plugins::typesext::MultiOption;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct GetAuditLogOptions {
        pub action_type: Option<serenity::all::audit_log::Action>,
        pub user_id: Option<serenity::all::UserId>,
        pub before: Option<serenity::all::AuditLogEntryId>,
        pub limit: Option<serenity::nonmax::NonMaxU8>,
    }

    impl Default for GetAuditLogOptions {
        fn default() -> Self {
            Self {
                action_type: Some(serenity::all::audit_log::Action::GuildUpdate),
                user_id: Some(serenity::all::UserId::default()),
                before: Some(serenity::all::AuditLogEntryId::default()),
                limit: Some(serenity::nonmax::NonMaxU8::default()),
            }
        }
    }

    #[derive(Default, serde::Serialize, serde::Deserialize)]
    pub struct GetChannelOptions {
        pub channel_id: serenity::all::ChannelId,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct EditChannelOptions {
        pub channel_id: serenity::all::ChannelId,
        pub reason: String,

        // Fields that can be edited
        pub name: Option<String>,                                     // done
        pub r#type: Option<serenity::all::ChannelType>,               // done
        pub position: Option<u16>,                                    // done
        pub topic: Option<String>,                                    // done
        pub nsfw: Option<bool>,                                       // done
        pub rate_limit_per_user: Option<serenity::nonmax::NonMaxU16>, // done
        pub bitrate: Option<u32>,                                     // done
        pub permission_overwrites: Option<Vec<serenity::all::PermissionOverwrite>>, // done
        pub parent_id: MultiOption<serenity::all::ChannelId>,         // done
        pub rtc_region: MultiOption<String>,                          // done
        pub video_quality_mode: Option<serenity::all::VideoQualityMode>, // done
        pub default_auto_archive_duration: Option<serenity::all::AutoArchiveDuration>, // done
        pub flags: Option<serenity::all::ChannelFlags>,               // done
        pub available_tags: Option<Vec<serenity::all::ForumTag>>,     // done
        pub default_reaction_emoji: MultiOption<serenity::all::ForumEmoji>, // done
        pub default_thread_rate_limit_per_user: Option<serenity::nonmax::NonMaxU16>, // done
        pub default_sort_order: Option<serenity::all::SortOrder>,     // done
        pub default_forum_layout: Option<serenity::all::ForumLayoutType>, // done
    }

    impl Default for EditChannelOptions {
        fn default() -> Self {
            Self {
                channel_id: serenity::all::ChannelId::default(),
                reason: String::default(),
                name: Some("my-channel".to_string()),
                r#type: Some(serenity::all::ChannelType::Text),
                position: Some(7),
                topic: Some("My channel topic".to_string()),
                nsfw: Some(true),
                rate_limit_per_user: Some(serenity::nonmax::NonMaxU16::new(5).unwrap()),
                bitrate: None,
                permission_overwrites: None,
                parent_id: MultiOption::new(Some(serenity::all::ChannelId::default())),
                rtc_region: MultiOption::new(Some("us-west".to_string())),
                video_quality_mode: Some(serenity::all::VideoQualityMode::Auto),
                default_auto_archive_duration: Some(serenity::all::AutoArchiveDuration::OneDay),
                flags: Some(serenity::all::ChannelFlags::all()),
                available_tags: None,
                default_reaction_emoji: MultiOption::new(Some(serenity::all::ForumEmoji::Id(
                    serenity::all::EmojiId::default(),
                ))),
                default_thread_rate_limit_per_user: None,
                default_sort_order: None,
                default_forum_layout: None,
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct EditThreadOptions {
        pub channel_id: serenity::all::ChannelId,
        pub reason: String,

        // Fields that can be edited
        pub name: Option<String>,
        pub archived: Option<bool>,
        pub auto_archive_duration: Option<serenity::all::AutoArchiveDuration>,
        pub locked: Option<bool>,
        pub invitable: Option<bool>,
        pub rate_limit_per_user: Option<serenity::nonmax::NonMaxU16>,
        pub flags: Option<serenity::all::ChannelFlags>,
        pub applied_tags: Option<Vec<serenity::all::ForumTag>>,
    }

    impl Default for EditThreadOptions {
        fn default() -> Self {
            Self {
                channel_id: serenity::all::ChannelId::default(),
                reason: String::default(),
                name: Some("my-thread".to_string()),
                archived: Some(false),
                auto_archive_duration: Some(serenity::all::AutoArchiveDuration::OneDay),
                locked: Some(false),
                invitable: Some(true),
                rate_limit_per_user: Some(serenity::nonmax::NonMaxU16::new(5).unwrap()),
                flags: Some(serenity::all::ChannelFlags::all()),
                applied_tags: None,
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct DeleteChannelOption {
        pub channel_id: serenity::all::ChannelId,
        pub reason: String,
    }

    impl Default for DeleteChannelOption {
        fn default() -> Self {
            Self {
                channel_id: serenity::all::ChannelId::default(),
                reason: "My reason here".to_string(),
            }
        }
    }

    pub mod messages {
        use limits::{embed_limits, message_limits};
        use serde::{Deserialize, Serialize};

        pub fn get_char_limit(total_chars: usize, limit: usize, max_chars: usize) -> usize {
            if max_chars <= total_chars {
                return 0;
            }

            // If limit is 6000 and max_chars - total_chars is 1000, return 1000 etc.
            std::cmp::min(limit, max_chars - total_chars)
        }

        pub fn slice_chars(
            s: &str,
            total_chars: &mut usize,
            limit: usize,
            max_chars: usize,
        ) -> String {
            let char_limit = get_char_limit(*total_chars, limit, max_chars);

            if char_limit == 0 {
                return String::new();
            }

            if s.len() > char_limit {
                *total_chars += char_limit;
                s.chars().take(char_limit).collect()
            } else {
                *total_chars += s.len();
                s.to_string()
            }
        }

        /// Represents an embed field
        #[derive(Serialize, Deserialize, Debug, Default, Clone)]
        pub struct CreateMessageEmbedField {
            /// The name of the field
            pub name: String,
            /// The value of the field
            pub value: String,
            /// Whether the field is inline
            pub inline: bool,
        }

        /// Represents an embed author
        #[derive(Serialize, Deserialize, Debug, Default, Clone)]
        pub struct CreateMessageEmbedAuthor {
            /// The name of the author
            pub name: String,
            /// The URL of the author, must be a valid URL
            pub url: Option<String>,
            /// The icon URL of the author, must be a valid URL
            pub icon_url: Option<String>,
        }

        /// Represents an embed footer
        #[derive(Serialize, Deserialize, Debug, Default, Clone)]
        pub struct CreateMessageEmbedFooter {
            /// The text of the footer
            pub text: String,
            /// The icon URL of the footer, must be a valid URL
            pub icon_url: Option<String>,
        }

        /// Represents a message embed
        #[derive(Serialize, Deserialize, Debug, Default, Clone)]
        pub struct CreateMessageEmbed {
            /// The title set by the template
            pub title: Option<String>,
            /// The description set by the template
            pub description: Option<String>,
            /// The URL the embed should link to
            pub url: Option<String>,
            /// The timestamp to display on the embed
            pub timestamp: Option<String>,
            /// The color of the embed
            pub color: Option<serenity::all::Color>,
            /// The footer of the embed
            pub footer: Option<CreateMessageEmbedFooter>,
            /// The image URL for the embed
            pub image: Option<String>,
            /// The thumbnail URL for the embed
            pub thumbnail: Option<String>,
            /// The author of the embed
            pub author: Option<CreateMessageEmbedAuthor>,
            /// The fields that were set by the template
            pub fields: Option<Vec<CreateMessageEmbedField>>,
        }

        /// Message attachment
        #[derive(Serialize, Deserialize, Debug, Default, Clone)]
        pub struct CreateMessageAttachment {
            pub filename: String,
            pub description: Option<String>,
            pub content: Vec<u8>,
        }

        /// Represents a message that can be created by templates
        #[derive(Serialize, Deserialize, Debug, Default, Clone)]
        pub struct CreateMessage {
            /// Embeds [current_index, embeds]
            pub embeds: Option<Vec<CreateMessageEmbed>>,
            /// What content to set on the message
            pub content: Option<String>,
            /// The attachments
            pub attachments: Option<Vec<CreateMessageAttachment>>,
        }

        /// Converts a templated message to a discord reply
        ///
        /// This method also handles all of the various discord message+embed limits as well, returning an error if unable to comply
        pub fn to_discord_reply<'a>(
            message: CreateMessage,
        ) -> Result<DiscordReply<'a>, crate::Error> {
            let mut total_chars = 0;
            let mut total_content_chars = 0;
            let mut embeds = Vec::new();

            if let Some(t_embeds) = message.embeds {
                for template_embed in t_embeds {
                    if embeds.len() >= embed_limits::EMBED_MAX_COUNT {
                        break;
                    }

                    let mut set = false; // Is something set on the embed?
                    let mut embed = serenity::all::CreateEmbed::default();

                    if let Some(title) = &template_embed.title {
                        // Slice title to EMBED_TITLE_LIMIT
                        embed = embed.title(slice_chars(
                            title,
                            &mut total_chars,
                            embed_limits::EMBED_TITLE_LIMIT,
                            embed_limits::EMBED_TOTAL_LIMIT,
                        ));
                        set = true;
                    }

                    if let Some(description) = &template_embed.description {
                        // Slice description to EMBED_DESCRIPTION_LIMIT
                        embed = embed.description(
                            slice_chars(
                                description,
                                &mut total_chars,
                                embed_limits::EMBED_DESCRIPTION_LIMIT,
                                embed_limits::EMBED_TOTAL_LIMIT,
                            )
                            .to_string(),
                        );
                        set = true;
                    }

                    if let Some(url) = &template_embed.url {
                        if url.is_empty() {
                            return Err("URL cannot be empty".into());
                        }

                        if !url.starts_with("http://") && !url.starts_with("https://") {
                            return Err("URL must start with http:// or https://".into());
                        }

                        embed = embed.url(url.clone());
                        set = true;
                    }

                    if let Some(timestamp) = &template_embed.timestamp {
                        let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp)
                            .map_err(|e| format!("Invalid timestamp provided to embed: {}", e))?;
                        embed = embed.timestamp(timestamp);
                        set = true;
                    }

                    if let Some(color) = template_embed.color {
                        embed = embed.color(color);
                        set = true;
                    }

                    if let Some(footer) = &template_embed.footer {
                        let text = slice_chars(
                            &footer.text,
                            &mut total_chars,
                            embed_limits::EMBED_FOOTER_TEXT_LIMIT,
                            embed_limits::EMBED_TOTAL_LIMIT,
                        );

                        let mut cef = serenity::all::CreateEmbedFooter::new(text);

                        if let Some(footer_icon_url) = &footer.icon_url {
                            if footer_icon_url.is_empty() {
                                return Err("Footer icon URL cannot be empty".into());
                            }

                            if !footer_icon_url.starts_with("http://")
                                && !footer_icon_url.starts_with("https://")
                            {
                                return Err(
                                    "Footer icon URL must start with http:// or https://".into()
                                );
                            }

                            cef = cef.icon_url(footer_icon_url.clone());
                        }

                        embed = embed.footer(cef);

                        set = true;
                    }

                    if let Some(image) = &template_embed.image {
                        if image.is_empty() {
                            return Err("Image URL cannot be empty".into());
                        }

                        if !image.starts_with("http://") && !image.starts_with("https://") {
                            return Err("Image URL must start with http:// or https://".into());
                        }

                        embed = embed.image(image.clone());
                        set = true;
                    }

                    if let Some(thumbnail) = &template_embed.thumbnail {
                        if thumbnail.is_empty() {
                            return Err("Thumbnail URL cannot be empty".into());
                        }

                        if !thumbnail.starts_with("http://") && !thumbnail.starts_with("https://") {
                            return Err("Thumbnail URL must start with http:// or https://".into());
                        }

                        embed = embed.thumbnail(thumbnail.clone());
                        set = true;
                    }

                    if let Some(author) = &template_embed.author {
                        let name = slice_chars(
                            &author.name,
                            &mut total_chars,
                            embed_limits::EMBED_AUTHOR_NAME_LIMIT,
                            embed_limits::EMBED_TOTAL_LIMIT,
                        );

                        let mut cea = serenity::all::CreateEmbedAuthor::new(name);

                        if let Some(url) = &author.url {
                            if url.is_empty() {
                                return Err("Author URL cannot be empty".into());
                            }

                            if !url.starts_with("http://") && !url.starts_with("https://") {
                                return Err("Author URL must start with http:// or https://".into());
                            }

                            cea = cea.url(url.clone());
                        }

                        if let Some(icon_url) = &author.icon_url {
                            if icon_url.is_empty() {
                                return Err("Author icon URL cannot be empty".into());
                            }

                            if !icon_url.starts_with("http://") && !icon_url.starts_with("https://")
                            {
                                return Err(
                                    "Author icon URL must start with http:// or https://".into()
                                );
                            }

                            cea = cea.icon_url(icon_url.clone());
                        }

                        embed = embed.author(cea);

                        set = true;
                    }

                    if let Some(fields) = template_embed.fields {
                        if !fields.is_empty() {
                            set = true;
                        }

                        for (count, field) in fields.into_iter().enumerate() {
                            if count >= embed_limits::EMBED_FIELDS_MAX_COUNT {
                                break;
                            }

                            let name = field.name.trim();
                            let value = field.value.trim();

                            if name.is_empty() || value.is_empty() {
                                continue;
                            }

                            // Slice field name to EMBED_FIELD_NAME_LIMIT
                            let name = slice_chars(
                                name,
                                &mut total_chars,
                                embed_limits::EMBED_FIELD_NAME_LIMIT,
                                embed_limits::EMBED_TOTAL_LIMIT,
                            );

                            // Slice field value to EMBED_FIELD_VALUE_LIMIT
                            let value = slice_chars(
                                value,
                                &mut total_chars,
                                embed_limits::EMBED_FIELD_VALUE_LIMIT,
                                embed_limits::EMBED_TOTAL_LIMIT,
                            );

                            embed = embed.field(name, value, field.inline);
                        }
                    }

                    if set {
                        embeds.push(embed);
                    }
                }
            }

            // Now handle content
            let content = message.content.map(|c| {
                slice_chars(
                    &c,
                    &mut total_content_chars,
                    message_limits::MESSAGE_CONTENT_LIMIT,
                    message_limits::MESSAGE_CONTENT_LIMIT,
                )
            });

            // Lastly handle attachments
            let mut attachments = Vec::new();

            if let Some(attach) = message.attachments {
                if attach.len() > message_limits::MESSAGE_MAX_ATTACHMENT_COUNT {
                    return Err(format!(
                        "Too many attachments, limit is {}",
                        message_limits::MESSAGE_MAX_ATTACHMENT_COUNT
                    )
                    .into());
                }

                for attachment in attach {
                    let desc = attachment.description.unwrap_or_default();
                    if desc.len() > message_limits::MESSAGE_ATTACHMENT_DESCRIPTION_LIMIT {
                        return Err(format!(
                            "Attachment description exceeds limit of {}",
                            message_limits::MESSAGE_ATTACHMENT_DESCRIPTION_LIMIT
                        )
                        .into());
                    }

                    let content = attachment.content;

                    if content.is_empty() {
                        return Err("Attachment content cannot be empty".into());
                    }

                    if content.len() > message_limits::MESSAGE_ATTACHMENT_CONTENT_BYTES_LIMIT {
                        return Err(format!(
                            "Attachment content exceeds limit of {} bytes",
                            message_limits::MESSAGE_ATTACHMENT_CONTENT_BYTES_LIMIT
                        )
                        .into());
                    }

                    let mut ca =
                        serenity::all::CreateAttachment::bytes(content, attachment.filename);

                    if !desc.is_empty() {
                        ca = ca.description(desc);
                    }

                    attachments.push(ca);
                }
            }

            if content.is_none() && embeds.is_empty() && attachments.is_empty() {
                return Err("No content/embeds/attachments set".into());
            }

            Ok(DiscordReply {
                embeds,
                content,
                attachments,
            })
        }

        #[derive(Default)]
        pub struct DiscordReply<'a> {
            pub content: Option<String>,
            pub embeds: Vec<serenity::all::CreateEmbed<'a>>,
            pub attachments: Vec<serenity::all::CreateAttachment<'a>>,
        }

        impl<'a> DiscordReply<'a> {
            pub fn to_create_message(self) -> serenity::all::CreateMessage<'a> {
                let mut message = serenity::all::CreateMessage::default();

                if let Some(content) = self.content {
                    message = message.content(content);
                }

                message = message.embeds(self.embeds);

                for attachment in self.attachments {
                    message = message.add_file(attachment);
                }

                message
            }

            #[allow(dead_code)]
            pub fn to_edit_message(self) -> serenity::all::EditMessage<'a> {
                let mut message = serenity::all::EditMessage::default();

                if let Some(content) = self.content {
                    message = message.content(content);
                }

                message = message.embeds(self.embeds);

                // NOTE: This resets old attachments
                for attachment in self.attachments {
                    message = message.new_attachment(attachment);
                }

                message
            }
        }
    }

    /// Represents a message that can be sent to a channel
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct SendMessageChannelAction {
        pub channel_id: serenity::all::ChannelId, // Channel *must* be in the same guild
        pub message: messages::CreateMessage,
    }
}

impl LuaUserData for DiscordActionExecutor {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Audit Log

        // Should be documented
        methods.add_async_method("get_audit_logs", |lua, this, data: LuaValue| async move {
            let data = lua.from_value::<types::GetAuditLogOptions>(data)?;

            this.check_action("get_audit_logs".to_string())
                .map_err(LuaError::external)?;

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions(bot_userid, serenity::all::Permissions::VIEW_AUDIT_LOG)
                .await
                .map_err(LuaError::external)?;

            let logs = this
                .serenity_context
                .http
                .get_audit_logs(
                    this.guild_id,
                    data.action_type,
                    data.user_id,
                    data.before,
                    data.limit,
                )
                .await
                .map_err(LuaError::external)?;

            let v = lua.to_value(&logs)?;

            Ok(v)
        });

        // Auto Moderation, not yet finished and hence not documented yet
        methods.add_async_method(
            "list_auto_moderation_rules",
            |lua, this, _: ()| async move {
                this.check_action("list_auto_moderation_rules".to_string())
                    .map_err(LuaError::external)?;

                let bot_userid = this.serenity_context.cache.current_user().id;

                this.check_permissions(bot_userid, serenity::all::Permissions::MANAGE_GUILD)
                    .await
                    .map_err(LuaError::external)?;

                let rules = this
                    .serenity_context
                    .http
                    .get_automod_rules(this.guild_id)
                    .await
                    .map_err(LuaError::external)?;

                let v = lua.to_value(&rules)?;

                Ok(v)
            },
        );

        // Not yet documented, not yet stable
        methods.add_async_method(
            "get_auto_moderation_rule",
            |lua, this, data: LuaValue| async move {
                let rule_id: serenity::all::RuleId = lua.from_value(data)?;

                this.check_action("get_auto_moderation_rule".to_string())
                    .map_err(LuaError::external)?;

                let bot_userid = this.serenity_context.cache.current_user().id;

                this.check_permissions(bot_userid, serenity::all::Permissions::MANAGE_GUILD)
                    .await
                    .map_err(LuaError::external)?;

                let rule = this
                    .serenity_context
                    .http
                    .get_automod_rule(this.guild_id, rule_id)
                    .await
                    .map_err(LuaError::external)?;

                let v = lua.to_value(&rule)?;

                Ok(v)
            },
        );

        // Not yet documented, not yet stable
        methods.add_async_method(
            "create_auto_moderation_rule",
            |lua, this, data: LuaValue| async move {
                #[derive(serde::Serialize, serde::Deserialize)]
                pub struct CreateAutoModerationRuleOptions {
                    name: String,
                    reason: String,
                    event_type: serenity::all::AutomodEventType,
                    trigger: serenity::all::Trigger,
                    actions: Vec<serenity::all::automod::Action>,
                    enabled: Option<bool>,
                    exempt_roles: Option<Vec<serenity::all::RoleId>>,
                    exempt_channels: Option<Vec<serenity::all::ChannelId>>,
                }

                let data: CreateAutoModerationRuleOptions = lua.from_value(data)?;

                this.check_action("create_auto_moderation_rule".to_string())
                    .map_err(LuaError::external)?;

                let bot_userid = this.serenity_context.cache.current_user().id;

                this.check_permissions(bot_userid, serenity::all::Permissions::MANAGE_GUILD)
                    .await
                    .map_err(LuaError::external)?;

                let mut rule = serenity::all::EditAutoModRule::new();
                rule = rule
                    .name(data.name)
                    .event_type(data.event_type)
                    .trigger(data.trigger)
                    .actions(data.actions);

                if let Some(enabled) = data.enabled {
                    rule = rule.enabled(enabled);
                }

                if let Some(exempt_roles) = data.exempt_roles {
                    if exempt_roles.len() > 20 {
                        return Err(LuaError::external(
                            "A maximum of 20 exempt_roles can be provided",
                        ));
                    }

                    rule = rule.exempt_roles(exempt_roles);
                }

                if let Some(exempt_channels) = data.exempt_channels {
                    if exempt_channels.len() > 50 {
                        return Err(LuaError::external(
                            "A maximum of 50 exempt_channels can be provided",
                        ));
                    }

                    rule = rule.exempt_channels(exempt_channels);
                }

                let rule = this
                    .serenity_context
                    .http
                    .create_automod_rule(this.guild_id, &rule, Some(data.reason.as_str()))
                    .await
                    .map_err(LuaError::external)?;

                let v = lua.to_value(&rule)?;

                Ok(v)
            },
        );

        /*methods.add_async_method(
            "edit_auto_moderation_rule",
            |lua, this, data: LuaValue| async move {
                #[derive(serde::Serialize, serde::Deserialize)]
                pub struct EditAutoModerationRuleOptions {
                    rule_id: serenity::all::RuleId,
                    reason: String,
                    name: Option<String>,
                    event_type: Option<serenity::all::AutomodEventType>,
                    trigger_metadata: Option<serenity::all::TriggerMetadata>,
                    actions: Vec<serenity::all::automod::Action>,
                    enabled: Option<bool>,
                    exempt_roles: Option<Vec<serenity::all::RoleId>>,
                    exempt_channels: Option<Vec<serenity::all::ChannelId>>,
                }

                let data: EditAutoModerationRuleOptions = lua.from_value(data)?;

                this.check_action("edit_auto_moderation_rule".to_string())
                    .map_err(LuaError::external)?;

                let bot_userid = this.serenity_context.cache.current_user().id;

                this.check_permissions(bot_userid, serenity::all::Permissions::MANAGE_GUILD)
                    .await
                    .map_err(LuaError::external)?;

                let mut rule = serenity::all::EditAutoModRule::new();

                if let Some(name) = data.name {
                    rule = rule.name(name);
                }

                if let Some(event_type) = data.event_type {
                    rule = rule.event_type(event_type);
                }

                if let Some(trigger_metadata) = data.trigger_metadata {
                    rule = rule.trigger(trigger)
                }

                rule = rule
                    .name(data.name)
                    .event_type(data.event_type)
                    .trigger(data.trigger)
                    .actions(data.actions);

                if let Some(enabled) = data.enabled {
                    rule = rule.enabled(enabled);
                }

                if let Some(exempt_roles) = data.exempt_roles {
                    if exempt_roles.len() > 20 {
                        return Err(LuaError::external(
                            "A maximum of 20 exempt_roles can be provided",
                        ));
                    }

                    rule = rule.exempt_roles(exempt_roles);
                }

                if let Some(exempt_channels) = data.exempt_channels {
                    if exempt_channels.len() > 50 {
                        return Err(LuaError::external(
                            "A maximum of 50 exempt_channels can be provided",
                        ));
                    }

                    rule = rule.exempt_channels(exempt_channels);
                }

                let rule = this
                    .serenity_context
                    .http
                    .create_automod_rule(this.guild_id, &rule, Some(data.reason.as_str()))
                    .await
                    .map_err(LuaError::external)?;

                let v = lua.to_value(&rule)?;

                Ok(v)
            },
        );*/

        // Channel

        // Should be documented
        methods.add_async_method("get_channel", |lua, this, data: LuaValue| async move {
            let data = lua.from_value::<types::GetChannelOptions>(data)?;

            this.check_action("get_channel".to_string())
                .map_err(LuaError::external)?;

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.user_in_guild(bot_userid)
                .await
                .map_err(LuaError::external)?;

            let channel = this
                .serenity_context
                .http
                .get_channel(data.channel_id)
                .await
                .map_err(LuaError::external)?;

            let v = lua.to_value(&channel)?;

            Ok(v)
        });

        // Should be documented
        methods.add_async_method("edit_channel", |lua, this, data: LuaValue| async move {
            let data = lua.from_value::<types::EditChannelOptions>(data)?;

            this.check_action("edit_channel".to_string())
                .map_err(LuaError::external)?;

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions(bot_userid, serenity::all::Permissions::MANAGE_CHANNELS)
                .await
                .map_err(LuaError::external)?;

            let mut ec = serenity::all::EditChannel::default(); // Create a new EditChannel struct

            if let Some(name) = data.name {
                ec = ec.name(name);
            }

            if let Some(r#type) = data.r#type {
                ec = ec.kind(r#type);
            }

            if let Some(position) = data.position {
                ec = ec.position(position);
            }

            if let Some(topic) = data.topic {
                if topic.len() > 1024 {
                    return Err(LuaError::external(
                        "Topic must be less than 1024 characters",
                    ));
                }
                ec = ec.topic(topic);
            }

            if let Some(nsfw) = data.nsfw {
                ec = ec.nsfw(nsfw);
            }

            if let Some(rate_limit_per_user) = data.rate_limit_per_user {
                if rate_limit_per_user.get() > 21600 {
                    return Err(LuaError::external(
                        "Rate limit per user must be less than 21600 seconds",
                    ));
                }

                ec = ec.rate_limit_per_user(rate_limit_per_user);
            }

            if let Some(bitrate) = data.bitrate {
                ec = ec.bitrate(bitrate);
            }

            // TODO: Handle permission overwrites permissions
            if let Some(permission_overwrites) = data.permission_overwrites {
                ec = ec.permissions(permission_overwrites);
            }

            if let Some(parent_id) = data.parent_id.inner {
                ec = ec.category(parent_id);
            }

            if let Some(rtc_region) = data.rtc_region.inner {
                ec = ec.voice_region(rtc_region.map(|x| x.into()));
            }

            if let Some(video_quality_mode) = data.video_quality_mode {
                ec = ec.video_quality_mode(video_quality_mode);
            }

            if let Some(default_auto_archive_duration) = data.default_auto_archive_duration {
                ec = ec.default_auto_archive_duration(default_auto_archive_duration);
            }

            if let Some(flags) = data.flags {
                ec = ec.flags(flags);
            }

            if let Some(available_tags) = data.available_tags {
                let mut cft = Vec::new();

                for tag in available_tags {
                    if tag.name.len() > 20 {
                        return Err(LuaError::external(
                            "Tag name must be less than 20 characters",
                        ));
                    }

                    let cftt =
                        serenity::all::CreateForumTag::new(tag.name).moderated(tag.moderated);

                    // TODO: Emoji support

                    cft.push(cftt);
                }

                ec = ec.available_tags(cft);
            }

            if let Some(default_reaction_emoji) = data.default_reaction_emoji.inner {
                ec = ec.default_reaction_emoji(default_reaction_emoji);
            }

            if let Some(default_thread_rate_limit_per_user) =
                data.default_thread_rate_limit_per_user
            {
                ec = ec.default_thread_rate_limit_per_user(default_thread_rate_limit_per_user);
            }

            if let Some(default_sort_order) = data.default_sort_order {
                ec = ec.default_sort_order(default_sort_order);
            }

            if let Some(default_forum_layout) = data.default_forum_layout {
                ec = ec.default_forum_layout(default_forum_layout);
            }

            let channel = this
                .serenity_context
                .http
                .edit_channel(data.channel_id, &ec, Some(data.reason.as_str()))
                .await
                .map_err(LuaError::external)?;

            let v = lua.to_value(&channel)?;

            Ok(v)
        });

        // Should be documented
        methods.add_async_method("edit_thread", |lua, this, data: LuaValue| async move {
            let data = lua.from_value::<types::EditThreadOptions>(data)?;

            this.check_action("edit_channel".to_string())
                .map_err(LuaError::external)?;

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions(
                bot_userid,
                serenity::all::Permissions::MANAGE_CHANNELS
                    | serenity::all::Permissions::MANAGE_THREADS,
            )
            .await
            .map_err(LuaError::external)?;

            let mut ec = serenity::all::EditThread::default(); // Create a new EditThread struct

            if let Some(name) = data.name {
                ec = ec.name(name);
            }

            if let Some(archived) = data.archived {
                ec = ec.archived(archived);
            }

            if let Some(auto_archive_duration) = data.auto_archive_duration {
                ec = ec.auto_archive_duration(auto_archive_duration);
            }

            if let Some(locked) = data.locked {
                ec = ec.locked(locked);
            }

            if let Some(invitable) = data.invitable {
                ec = ec.invitable(invitable);
            }

            if let Some(rate_limit_per_user) = data.rate_limit_per_user {
                ec = ec.rate_limit_per_user(rate_limit_per_user);
            }

            if let Some(flags) = data.flags {
                ec = ec.flags(flags);
            }

            if let Some(applied_tags) = data.applied_tags {
                ec = ec.applied_tags(applied_tags.iter().map(|x| x.id).collect::<Vec<_>>());
            }

            let channel = this
                .serenity_context
                .http
                .edit_thread(data.channel_id, &ec, Some(data.reason.as_str()))
                .await
                .map_err(LuaError::external)?;

            let v = lua.to_value(&channel)?;
            Ok(v)
        });

        // Should be documented
        methods.add_async_method(
            "delete_channel",
            |lua, this, channel_id: LuaValue| async move {
                let data = lua.from_value::<types::DeleteChannelOption>(channel_id)?;

                this.check_action("delete_channel".to_string())
                    .map_err(LuaError::external)?;

                let bot_userid = this.serenity_context.cache.current_user().id;

                this.check_permissions(bot_userid, serenity::all::Permissions::MANAGE_CHANNELS)
                    .await
                    .map_err(LuaError::external)?;

                let channel = this
                    .serenity_context
                    .http
                    .delete_channel(data.channel_id, Some(data.reason.as_str()))
                    .await
                    .map_err(LuaError::external)?;

                let v = lua.to_value(&channel)?;
                Ok(v)
            },
        );

        // Ban/Kick/Timeout, not yet documented as it is not yet stable
        methods.add_async_method("create_guild_ban", |lua, this, data: LuaValue| async move {
            /// A ban action
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct BanAction {
                user_id: serenity::all::UserId,
                reason: String,
                delete_message_seconds: Option<u32>,
            }

            let data = lua.from_value::<BanAction>(data)?;

            this.check_action("ban".to_string())
                .map_err(LuaError::external)?;

            let delete_message_seconds = {
                if let Some(seconds) = data.delete_message_seconds {
                    if seconds > 604800 {
                        return Err(LuaError::external(
                            "Delete message seconds must be between 0 and 604800",
                        ));
                    }

                    seconds
                } else {
                    0
                }
            };

            if data.reason.len() > 128 || data.reason.is_empty() {
                return Err(LuaError::external(
                    "Reason must be less than 128 characters and not empty",
                ));
            }

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions_and_hierarchy(
                bot_userid,
                data.user_id,
                serenity::all::Permissions::BAN_MEMBERS,
            )
            .await
            .map_err(LuaError::external)?;

            this.serenity_context
                .http
                .ban_user(
                    this.guild_id,
                    data.user_id,
                    (delete_message_seconds / 86400)
                        .try_into()
                        .map_err(LuaError::external)?, // TODO: Fix in serenity
                    Some(data.reason.as_str()),
                )
                .await
                .map_err(LuaError::external)?;

            Ok(())
        });

        // Ban/Kick/Timeout, not yet documented as it is not yet stable
        methods.add_async_method("kick", |lua, this, data: LuaValue| async move {
            /// A kick action
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct KickAction {
                user_id: serenity::all::UserId,
                reason: String,
            }

            let data = lua.from_value::<KickAction>(data)?;

            this.check_action("kick".to_string())
                .map_err(LuaError::external)?;

            if data.reason.len() > 128 || data.reason.is_empty() {
                return Err(LuaError::external(
                    "Reason must be less than 128 characters and not empty",
                ));
            }

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions_and_hierarchy(
                bot_userid,
                data.user_id,
                serenity::all::Permissions::KICK_MEMBERS,
            )
            .await
            .map_err(LuaError::external)?;

            this.serenity_context
                .http
                .kick_member(this.guild_id, data.user_id, Some(data.reason.as_str()))
                .await
                .map_err(LuaError::external)?;

            Ok(())
        });

        // Ban/Kick/Timeout, not yet documented as it is not yet stable
        methods.add_async_method("timeout", |lua, this, data: LuaValue| async move {
            /// A timeout action
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct TimeoutAction {
                user_id: serenity::all::UserId,
                reason: String,
                duration_seconds: u64,
            }

            let data = lua.from_value::<TimeoutAction>(data)?;

            this.check_action("timeout".to_string())
                .map_err(LuaError::external)?;

            if data.reason.len() > 128 || data.reason.is_empty() {
                return Err(LuaError::external(
                    "Reason must be less than 128 characters and not empty",
                ));
            }

            if data.duration_seconds > 60 * 60 * 24 * 28 {
                return Err(LuaError::external(
                    "Timeout duration must be less than 28 days",
                ));
            }

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions_and_hierarchy(
                bot_userid,
                data.user_id,
                serenity::all::Permissions::MODERATE_MEMBERS,
            )
            .await
            .map_err(LuaError::external)?;

            let communication_disabled_until =
                chrono::Utc::now() + std::time::Duration::from_secs(data.duration_seconds);

            this.guild_id
                .edit_member(
                    &this.serenity_context.http,
                    data.user_id,
                    serenity::all::EditMember::new()
                        .audit_log_reason(data.reason.as_str())
                        .disable_communication_until(communication_disabled_until.into()),
                )
                .await
                .map_err(LuaError::external)?;

            Ok(())
        });

        // Should be documented
        methods.add_async_method("create_message", |lua, this, data: LuaValue| async move {
            let data = lua.from_value::<types::SendMessageChannelAction>(data)?;

            this.check_action("create_message".to_string())
                .map_err(LuaError::external)?;

            let msg =
                types::messages::to_discord_reply(data.message).map_err(LuaError::external)?;

            // Perform required checks
            let channel = sandwich_driver::channel(
                &this.cache_http,
                &this.reqwest_client,
                Some(this.guild_id),
                data.channel_id,
            )
            .await
            .map_err(LuaError::external)?;

            let Some(channel) = channel else {
                return Err(LuaError::external("Channel not found"));
            };

            let Some(guild_channel) = channel.guild() else {
                return Err(LuaError::external("Channel not in guild"));
            };

            if guild_channel.guild_id != this.guild_id {
                return Err(LuaError::external("Channel not in guild"));
            }

            let bot_user_id = this.serenity_context.cache.current_user().id;

            let bot_user = sandwich_driver::member_in_guild(
                &this.cache_http,
                &this.reqwest_client,
                this.guild_id,
                bot_user_id,
            )
            .await
            .map_err(LuaError::external)?;

            let Some(bot_user) = bot_user else {
                return Err(LuaError::external("Bot user not found"));
            };

            let guild =
                sandwich_driver::guild(&this.cache_http, &this.reqwest_client, this.guild_id)
                    .await
                    .map_err(LuaError::external)?;

            // Check if the bot has permissions to send messages in the given channel
            if !guild
                .user_permissions_in(&guild_channel, &bot_user)
                .send_messages()
            {
                return Err(LuaError::external(
                    "Bot does not have permission to send messages in the given channel",
                ));
            }

            let cm = msg.to_create_message();

            let msg = guild_channel
                .send_message(&this.serenity_context.http, cm)
                .await
                .map_err(LuaError::external)?;

            Ok(MessageHandle {
                message: msg,
                shard_messenger: this.shard_messenger.clone(),
            })
        });
    }
}

pub struct MessageHandle {
    message: serenity::all::Message,
    shard_messenger: serenity::all::ShardMessenger,
}

impl LuaUserData for MessageHandle {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("data", |lua, this, _: ()| {
            let v = lua.to_value(&this.message)?;
            Ok(v)
        });

        // Not yet documented
        methods.add_method("await_component_interaction", |_, this, _: ()| {
            let stream = super::typesext::LuaStream::new(
                this.message
                    .id
                    .await_component_interactions(this.shard_messenger.clone())
                    .timeout(std::time::Duration::from_secs(60))
                    .stream()
                    .map(|interaction| MessageComponentHandle { interaction }),
            );

            Ok(stream)
        });
    }
}

// Not yet documented
pub struct MessageComponentHandle {
    pub interaction: serenity::all::ComponentInteraction,
}

impl LuaUserData for MessageComponentHandle {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("custom_id", |_, this, _: ()| {
            Ok(this.interaction.data.custom_id.to_string())
        });

        methods.add_method("data", |lua, this, _: ()| {
            let v = lua.to_value(&this.interaction)?;
            Ok(v)
        });
    }
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "new",
        lua.create_function(|lua, (token,): (crate::TemplateContextRef,)| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            let executor = DiscordActionExecutor {
                template_data: token.template_data.clone(),
                guild_id: data.guild_id,
                cache_http: botox::cache::CacheHttpImpl::from_ctx(&data.serenity_context),
                serenity_context: data.serenity_context.clone(),
                shard_messenger: data.shard_messenger.clone(),
                reqwest_client: data.reqwest_client.clone(),
                ratelimits: data.actions_ratelimits.clone(),
            };

            Ok(executor)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
