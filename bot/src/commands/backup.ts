import { Colors, EmbedBuilder, PermissionsBitField } from "discord.js";
import { Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";
import { createTaskEmbed, pollTask } from "../core/common/taskEmbed";
import { Task, TaskCreateResponse } from "../generatedTypes/types";

const defaultAssets = ["icon", "banner", "splash"]

/*
type BackupCreateOpts struct {
	I PerChannel                int            `json:"per_channel" description:"The number of messages per channel"`
	I MaxMessages               int            `json:"max_messages" description:"The maximum number of messages to backup"`
	I BackupMessages            bool           `json:"backup_messages" description:"Whether to backup messages or not"`
	I BackupAttachments         bool           `json:"backup_attachments" description:"Whether to backup attachments or not"`
	I BackupGuildAssets         []string       `json:"backup_guild_assets" description:"What assets to back up"`
    I IgnoreMessageBackupErrors bool           `json:"ignore_message_backup_errors" description:"Whether to ignore errors while backing up messages or not and skip these channels"`
	I RolloverLeftovers         bool           `json:"rollover_leftovers" description:"Whether to attempt rollover of leftover message quota to another channels or not"`
	SpecialAllocations          map[string]int `json:"special_allocations" description:"Specific channel allocation overrides"`
	I Encrypt                   string           `json:"encrypt" description:"Whether to encrypt the backup or not"`
}

type BackupRestoreOpts struct {
    IgnoreRestoreErrors bool     `json:"ignore_restore_errors" description:"Whether to ignore errors while restoring or not"`
	I ProtectedChannels []string `json:"protected_channels" description:"Channels to protect from being deleted"`
	I BackupSource      string   `json:"backup_source" description:"The source of the backup"`
	I Decrypt           string   `json:"decrypt" description:"The key to decrypt backups with, if any"`
	I ChannelRestoreMode ChannelRestoreMode `json:"channel_restore_mode" description:"Channel backup restore method. Use 'full' if unsure"`
    RoleRestoreMode    RoleRestoreMode    `json:"role_restore_mode" description:"Role backup restore method. Use 'full' if unsure"`
}
*/

let command: Command = {
    userPerms: [PermissionsBitField.Flags.ManageGuild],
    botPerms: [PermissionsBitField.Flags.ManageGuild],
    interactionData: new SlashCommandBuilder()
    .setName("backup")
	.setDescription("Create, load and get info on backups of your server!")
    .addSubcommand((sub) => {
        sub.setName("create")
        .setDescription("Create a backup of your server")
        .addBooleanOption((opt) => {
            opt.setName("messages")
            .setDescription("Whether to include messages in the backup (up to 500)")

            return opt
        })
        .addBooleanOption((opt) => {
            opt.setName("attachments")
            .setDescription("Whether to include attachments in the backup. Requires 'messages' to be enabled")

            return opt
        })
        .addStringOption((opt) => {
            opt.setName("backup_guild_assets")
            .setDescription("\"What assets to back up in comma-seperated form (icon,splash,banner)\"")

            return opt
        })
        .addBooleanOption((opt) => {
            opt.setName("rollover_leftovers")
            .setDescription("Roll over leftover message quotas to other channels. May make backups slower. Defaults to true")

            return opt
        })
        .addIntegerOption((opt) => {
            opt.setName("ignore_message_backup_errors")
            .setDescription("Whether to ignore errors while backing up messages or not and skip these channels")
            .setRequired(false)

            return opt
        })
        .addIntegerOption((opt) => {
            opt.setName("max_messages")
            .setDescription("The maximum number of messages to backup. Defaults to 500")
            .setRequired(false)

            return opt
        })
        .addIntegerOption((opt) => {
            opt.setName("per_channel")
            .setDescription("The number of messages to backup per channel. Defaults to 100")
            .setRequired(false)

            return opt
        })
        .addStringOption((opt) => {
            opt.setName("password")
            .setDescription("Password to encrypt backup with. Should not be reused")
            .setRequired(false)

            return opt
        })        

        return sub
    })
    .addSubcommand((sub) => {
        sub.setName("restore")
        .setDescription("Restore a backup of your server")
        .addAttachmentOption((opt) => {
            opt.setName("backup_file")
            .setDescription("The backup file to restore")

            return opt
        })
        .addStringOption((opt) => {
            opt.setName("password")
            .setDescription("Password to decrypt backup with. Should not be reused")
            .setRequired(false)

            return opt
        })
        .addStringOption((opt) => {
            opt.setName("channel_restore_mode")
            .setDescription("Channel restore mode. Defaults to full. Use 'full' if unsure")
            .addChoices(
                {"name": "Full", "value": "full"},
                {"name": "Difference-Based", "value": "diff"},
                {"name": "Ignore Existing", "value": "ignore_existing"}
            )
            .setRequired(false)

            return opt
        })
        .addStringOption((opt) => {
            opt.setName("role_restore_mode")
            .setDescription("Role restore mode. Defaults to full. Use 'full' if unsure")
            .addChoices(
                {"name": "Full", "value": "full"},
            )
            .setRequired(false)

            return opt
        })
        .addStringOption((opt) => {
            opt.setName("protected_channels")
            .setDescription("Channels to protect seperated by commas")
            .setRequired(false)

            return opt
        })
        .addStringOption((opt) => {
            opt.setName("protected_roles")
            .setDescription("Roles to protect seperated by commas")
            .setRequired(false)

            return opt
        })
        .addBooleanOption((opt) => {
            opt.setName("ignore_restore_errors")
            .setDescription("Whether to ignore errors while restoring or not")
            .setRequired(false)

            return opt
        })

        return sub
    }),
    execute: async (ctx) => {
        let sc = ctx.interaction.options.getSubcommand()
        switch (sc) {
            case "create":
                let messages = ctx.interaction.options.getBoolean("messages")
                let attachments = ctx.interaction.options.getBoolean("attachments")
                let backupGuildAssets = ctx.interaction.options.getString("backup_guild_assets")?.split(",") || defaultAssets
                let maxMessages = ctx.interaction.options.getInteger("max_messages")
                let perChannel = ctx.interaction.options.getInteger("per_channel")
                let rolloverLeftovers = ctx.interaction.options.getBoolean("rollover_leftovers")
                let ignoreMessageBackupErrors = ctx.interaction.options.getBoolean("ignore_message_backup_errors")
                let password = ctx.interaction.options.getString("password") || ""

                if(backupGuildAssets.length > 0) {
                    backupGuildAssets = backupGuildAssets?.map((v) => v.trim())?.filter((v) => v.length > 0)
                }

                if(!messages && attachments) {
                    return FinalResponse.reply({
                        embeds: [
                            new EmbedBuilder()
                            .setTitle("Creating backup")
                            .setDescription(":x: You cannot backup attachments without also enabling backup of messages")
                            .setColor(Colors.Red)
                        ]
                    })
                }

                await ctx.reply({
                    embeds: [
                        new EmbedBuilder()
                        .setTitle("Creating backup")
                        .setDescription(":yellow_circle: Please wait, starting backup task...")
                        .setColor(Colors.Blurple)
                    ],
                    ephemeral: true
                })

                let handle = await ctx.client.redis.sendIpcRequest({
                    scope: "splashtail",
                    action: "create_task",
                    args: {
                        "name": "guild_create_backup"
                    },
                    output: {
                        "ServerID": ctx.guild.id,
                        "Options": {
                            "MaxMessages": maxMessages || 500,
                            "BackupMessages": messages || false,
                            "BackupAttachments": attachments || false,
                            "BackupGuildAssets": backupGuildAssets || defaultAssets,
                            "PerChannel": perChannel || 100,
                            "RolloverLeftovers": rolloverLeftovers || true,
                            "IgnoreMessageBackupErrors": ignoreMessageBackupErrors || false,
                            "Encrypt": password
                        }
                    }
                }, null, {})

                if(!handle) throw new Error("Invalid IPC handle")

                let rmap = await handle.fetch() 
                
                if(rmap.size == 0 || !rmap.has(-1)) {
                    return FinalResponse.edit({
                        embeds: [
                            new EmbedBuilder()
                            .setTitle("Creating backup")
                            .setDescription(":x: Failed to create task. No response from co-ordinator server.")
                            .setColor(Colors.Red)
                        ]
                    })
                }

                let res = rmap.get(-1)
                
                if(res?.output?.error) {
                    return FinalResponse.edit({
                        embeds: [
                            new EmbedBuilder()
                            .setTitle("Creating backup")
                            .setDescription(`:x: ${res?.output?.error || "Failed to create task"}`)
                            .setColor(Colors.Red)
                        ]
                    })
                }

                let tcr: TaskCreateResponse = res?.output

                if(!tcr?.task_id) {
                    return FinalResponse.edit({
                        embeds: [
                            new EmbedBuilder()
                            .setTitle("Creating backup")
                            .setDescription(`:x: Failed to create task. No task ID returned.`)
                            .setColor(Colors.Red)
                        ]
                    })
                }

                await ctx.edit({
                    embeds: [
                        new EmbedBuilder()
                        .setTitle("Creating backup")
                        .setDescription(`:white_check_mark: Task created with ID \`${tcr?.task_id}\`. Waiting for task to complete...`)
                        .setColor(Colors.Green)
                    ]
                })

                let task = await pollTask(tcr?.task_id, {
                    callback: async (task) => {
                        await ctx.edit(createTaskEmbed(ctx, task))
                    }
                })

                if(task?.state == "completed") {
                    return FinalResponse.dummy()
                } else {
                    return FinalResponse.dummy()
                }
                break;
            case "restore":
                let backupFile = ctx.interaction.options.getAttachment("backup_file")
                let password2 = ctx.interaction.options.getString("password") || ""
                let protectedChannels = ctx.interaction.options.getString("protected_channels")?.split(",") || []
                let protectedRoles = ctx.interaction.options.getString("protected_roles")?.split(",") || []
                let channelRestoreMode = ctx.interaction.options.getString("channel_restore_mode") || "full"
                let roleRestoreMode = ctx.interaction.options.getString("role_restore_mode") || "full"
                let ignoreRestoreErrors = ctx.interaction.options.getBoolean("ignore_restore_errors") || false

                if(protectedChannels.length > 0) {
                    protectedChannels = protectedChannels?.map((v) => v.trim())?.filter((v) => v.length > 0)
                }

                if(protectedRoles.length > 0) {
                    protectedRoles = protectedRoles?.map((v) => v.trim())?.filter((v) => v.length > 0)
                }

                if(!protectedChannels?.includes(ctx?.interaction?.channelId)) {
                    protectedChannels.push(ctx?.interaction?.channelId)
                }

                if(!backupFile) {
                    return FinalResponse.reply({
                        embeds: [
                            new EmbedBuilder()
                            .setTitle("Restoring backup")
                            .setDescription(":x: No backup file provided")
                            .setColor(Colors.Red)
                        ]
                    })
                }

                let url = backupFile.url || backupFile?.proxyURL

                if(!url) {
                    return FinalResponse.reply({
                        embeds: [
                            new EmbedBuilder()
                            .setTitle("Restoring backup")
                            .setDescription(":x: No backup file provided [url missing]")
                            .setColor(Colors.Red)
                        ]
                    })
                }

                await ctx.reply({
                    embeds: [
                        new EmbedBuilder()
                        .setTitle("Restoring backup")
                        .setDescription(":yellow_circle: Please wait, starting restore task...")
                        .setColor(Colors.Blurple)
                    ],
                    ephemeral: true
                })

                let handle2 = await ctx.client.redis.sendIpcRequest({
                    scope: "splashtail",
                    action: "create_task",
                    args: {
                        "name": "guild_restore_backup"
                    },
                    output: {
                        "ServerID": ctx.guild.id,
                        "Options": {
                            "BackupSource": backupFile.url,
                            "Decrypt": password2,
                            "ProtectedChannels": protectedChannels,
                            "ProtectedRoles": protectedRoles,
                            "ChannelRestoreMode": channelRestoreMode,
                            "RoleRestoreMode": roleRestoreMode,
                            "IgnoreRestoreErrors": ignoreRestoreErrors
                        }
                    }
                }, null, {})

                if(!handle2) throw new Error("Invalid IPC handle")

                let rmap2 = await handle2.fetch()

                if(rmap2.size == 0 || !rmap2.has(-1)) {
                    return FinalResponse.edit({
                        embeds: [
                            new EmbedBuilder()
                            .setTitle("Restoring backup")
                            .setDescription(":x: Failed to create task. No response from co-ordinator server.")
                            .setColor(Colors.Red)
                        ]
                    })
                }

                let res2 = rmap2.get(-1)

                if(res2?.output?.error) {
                    return FinalResponse.edit({
                        embeds: [
                            new EmbedBuilder()
                            .setTitle("Restoring backup")
                            .setDescription(`:x: ${res2?.output?.error || "Failed to create task"}`)
                            .setColor(Colors.Red)
                        ]
                    })
                }

                let tcr2: TaskCreateResponse = res2?.output

                if(!tcr2?.task_id) {
                    return FinalResponse.edit({
                        embeds: [
                            new EmbedBuilder()
                            .setTitle("Restoring backup")
                            .setDescription(`:x: Failed to create task. No task ID returned.`)
                            .setColor(Colors.Red)
                        ]
                    })
                }

                await ctx.edit({
                    embeds: [
                        new EmbedBuilder()
                        .setTitle("Restoring backup")
                        .setDescription(`:white_check_mark: Task created with ID \`${tcr2?.task_id}\`. Waiting for task to complete...`)
                        .setColor(Colors.Green)
                    ]
                })

                let task2 = await pollTask(tcr2?.task_id, {
                    callback: async (task) => {
                        await ctx.edit(createTaskEmbed(ctx, task))
                    }
                })

                if(task2?.state == "completed") {
                    return FinalResponse.dummy()
                } else {
                    return FinalResponse.dummy()
                }
            default:
                return FinalResponse.reply({
                    embeds: [
                        new EmbedBuilder()
                        .setTitle("Backup")
                        .setDescription("Create, load and get info on backups of your server!")
                        .addFields([
                            {
                                name: "Create",
                                value: "Create a backup of your server"
                            },
                            {
                                name: "Load",
                                value: "Load a backup of your server"
                            },
                            {
                                name: "Download",
                                value: "Download a backup of your server"
                            },
                            {
                                name: "Info",
                                value: "Get info on a backup of your server"
                            },
                            {
                                name: "You selected",
                                value: sc
                            }
                        ])
                        .setColor(Colors.Blurple)
                    ]
                })
        }
    }
}

export default command;