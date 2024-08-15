use futures_util::FutureExt;
use module_settings::{
    data_stores::PostgresDataStore,
    types::{
        settings_wrap_columns, settings_wrap_datastore, settings_wrap_postactions,
        settings_wrap_precheck, Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption,
        InnerColumnType, OperationSpecific, OperationType, SettingsError,
    },
};
use splashcore_rs::value::Value;
use std::sync::LazyLock;

use super::types::{DehoistOptions, FakeBotDetectionOptions, GuildProtectionOptions};

pub static INSPECTOR_OPTIONS: LazyLock<ConfigOption> = LazyLock::new(|| ConfigOption {
    id: "options",
    name: "Inspector Options",
    description: "Setup inspector here",
    table: "inspector__options",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "guild_id",
    max_entries: Some(1),
    data_store: settings_wrap_datastore(PostgresDataStore {}),
    columns: settings_wrap_columns(vec![
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "Guild ID of the server in question",
        ),
        Column {
            id: "minimum_account_age",
            name: "Minimum Account Age",
            description: "Minimum account age required to join the server",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "maximum_account_age",
            name: "Maximum Account Age",
            description: "Maximum account age to join the server",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "anti_invite",
            name: "Anti Invite",
            description: "Number of stings to give when an invite is sent",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "anti_everyone",
            name: "Anti Everyone",
            description: "Number of stings to give when an everyone ping is sent",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "sting_retention",
            name: "Sting Retention",
            description: "Number of seconds to keep stings for",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "hoist_detection",
            name: "Hoist Detection",
            description: "Hoist detection options",
            column_type: ColumnType::new_scalar(InnerColumnType::BitFlag {
                values: DehoistOptions::all()
                    .into_iter()
                    .map(|x| (x.to_string(), x.bits() as i64))
                    .collect(),
            }),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "guild_protection",
            name: "Guild Protection",
            description: "Guild protection options",
            column_type: ColumnType::new_scalar(InnerColumnType::BitFlag {
                values: GuildProtectionOptions::all()
                    .into_iter()
                    .map(|x| (x.to_string(), x.bits() as i64))
                    .collect(),
            }),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![ColumnAction::NativeAction {
                action: Box::new(|ctx, state| {
                    async move {
                        let Some(Value::Integer(gp)) = state.state.get("guild_protection") else {
                            return Err(SettingsError::MissingOrInvalidField {
                                field: "guild_protection".to_string(),
                                src: "index->NativeAction [default_pre_checks]".to_string(),
                            });
                        };

                        let gp_flags = GuildProtectionOptions::from_bits_truncate(
                            (*gp).try_into().map_err(|e| SettingsError::Generic {
                                message: format!(
                                    "Error while converting guild protection flags: {}",
                                    e
                                ),
                                typ: "value_error".to_string(),
                                src: "inspector__options.guild_protection".to_string(),
                            })?,
                        );

                        if gp_flags.contains(GuildProtectionOptions::DISABLED) {
                            // Delete from inspector__guilds
                            sqlx::query!(
                                "DELETE FROM inspector__guilds WHERE guild_id = $1",
                                ctx.guild_id.to_string(),
                            )
                            .execute(&ctx.data.pool)
                            .await
                            .map_err(|e| SettingsError::Generic {
                                message: format!("Error while deleting guild: {}", e),
                                typ: "database_error".to_string(),
                                src: "inspector__options.guild_protection".to_string(),
                            })?;
                        } else {
                            // Fetch guild
                            let guild = match proxy_support::guild(
                                &ctx.data.cache_http,
                                &ctx.data.reqwest,
                                ctx.guild_id,
                            )
                            .await
                            {
                                Ok(guild) => guild,
                                Err(e) => {
                                    return Err(SettingsError::Generic {
                                        message: format!("Error while fetching guild: {}", e),
                                        typ: "api_error".to_string(),
                                        src: "inspector__options.guild_protection".to_string(),
                                    });
                                }
                            };

                            // Save guild
                            match (super::guildprotect::Snapshot {
                                guild_id: ctx.guild_id,
                                name: guild.name.to_string(),
                                icon: guild.icon.map(|x| x.to_string()),
                            })
                            .save(&ctx.data.pool, &ctx.data.reqwest, &ctx.data.object_store)
                            .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(SettingsError::Generic {
                                        message: format!("Error while saving guild: {}", e),
                                        typ: "database_error".to_string(),
                                        src: "inspector__options.guild_protection".to_string(),
                                    });
                                }
                            }
                        }

                        Ok(())
                    }
                    .boxed()
                }),
                on_condition: Some(|ctx, _state| Ok(ctx.operation_type != OperationType::View)),
            }]),
        },
        Column {
            id: "fake_bot_detection",
            name: "Fake Bot Detection",
            description: "Fake bot detection options",
            column_type: ColumnType::new_scalar(InnerColumnType::BitFlag {
                values: FakeBotDetectionOptions::all()
                    .into_iter()
                    .map(|x| (x.to_string(), x.bits() as i64))
                    .collect(),
            }),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
    ]),
    title_template: "Servers Inspector Setup",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            corresponding_command: "inspector list",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Create => OperationSpecific {
            corresponding_command: "inspector setup",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Update => OperationSpecific {
            corresponding_command: "inspector update",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Delete => OperationSpecific {
            corresponding_command: "inspector disable",
            columns_to_set: indexmap::indexmap! {},
        },
    },
    post_actions: settings_wrap_postactions(vec![ColumnAction::NativeAction {
        action: Box::new(|_ctx, state| {
            async move {
                let Some(Value::String(guild_id)) = state.state.get("guild_id") else {
                    return Err(SettingsError::MissingOrInvalidField {
                        field: "guild_id".to_string(),
                        src: "index->NativeAction [post_actions]".to_string(),
                    });
                };

                let guild_id = guild_id.parse::<serenity::all::GuildId>().map_err(|e| {
                    SettingsError::Generic {
                        message: format!("Error while parsing guild_id: {}", e),
                        typ: "value_error".to_string(),
                        src: "inspector__options.guild_id".to_string(),
                    }
                })?;

                super::cache::BASIC_ANTISPAM_CONFIG_CACHE
                    .invalidate(&guild_id)
                    .await;

                Ok(())
            }
            .boxed()
        }),
        on_condition: Some(|ctx, _state| Ok(ctx.operation_type != OperationType::View)),
    }]),
});
