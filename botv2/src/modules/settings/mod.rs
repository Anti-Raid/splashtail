use indexmap::indexmap;
use futures::future::FutureExt;
use crate::silverpelt::config_opt::{ConfigOption, Column, ColumnType};

mod modules;
mod perms;
mod commands;
mod am_toggles;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "settings",
        name: "Settings",
        description: "Configure the bot to your liking",
        toggleable: false,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                modules::modules(),
                indexmap! {
                    "list" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "list"),
                    "enable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "enable"),
                    "disable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "disable"),
                },
            ),
            (
                commands::commands(),
                indexmap! {
                    "check" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("commands", "check"),
                    "enable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("commands", "enable"),
                    "disable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("commands", "disable"),
                },
            ),
            (
                perms::perms(),
                indexmap! {
                    "list" => crate::silverpelt::CommandExtendedData::kittycat_simple("perms", "list"),
                    "modrole" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["perms.editrole".to_string(), "perms.manage".to_string()],
                                    native_perms: vec![],
                                    inner_and: true,
                                    outer_and: false,
                                },
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::MANAGE_ROLES],
                                    inner_and: true,
                                    outer_and: false,
                                }
                            ],
                            checks_needed: 1,
                        },
                        ..Default::default()
                    },
                    "delrole" => crate::silverpelt::CommandExtendedData::kittycat_simple("perms", "delrole"),
                },
            ),
        ],
        on_startup: vec![
            Box::new(move |data| {
                am_toggles::setup(data).boxed()
            }),
        ],
        config_options: vec![
            ConfigOption {
                id: "guild_channels",
                name: "Guild Channels",
                description: "Channel configuration for this guild",
                table: "guild_channels",
                guild_id: "guild_id",
                row_must_exist: false,
                hint: Some("guild_channels".to_string()),
                columns: vec![
                    Column {
                        id: "channel_type",
                        name: "Channel Type",
                        column_type: ColumnType::String,
                        nullable: false,
                        unique: true,
                        array: false,
                        hint: Some("channel_type".to_string()),
                    },
                    Column {
                        id: "channel_id",
                        name: "Channel ID",
                        column_type: ColumnType::Channel,
                        nullable: false,
                        unique: true,
                        array: false,
                        hint: Some("channel_id".to_string()),
                    },
                ]
            }
        ],
        ..Default::default()
    }
}
