import { Colors, EmbedBuilder, PermissionsBitField } from "discord.js";
import { Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";
import { createTaskEmbed } from "../core/common/taskEmbed";
import { TaskCreateResponse } from "../generatedTypes/types";

/*
type BackupCreateOpts struct {
	I PerChannel                int            `json:"per_channel" description:"The number of messages per channel"`
	I MaxMessages               int            `json:"max_messages" description:"The maximum number of messages to backup"`
	I BackupMessages            bool           `json:"backup_messages" description:"Whether to backup messages or not"`
	I BackupAttachments         bool           `json:"backup_attachments" description:"Whether to backup attachments or not"`
	I IgnoreMessageBackupErrors bool           `json:"ignore_message_backup_errors" description:"Whether to ignore errors while backing up messages or not and skip these channels"`
	I RolloverLeftovers         bool           `json:"rollover_leftovers" description:"Whether to attempt rollover of leftover message quota to another channels or not"`
	SpecialAllocations          map[string]int `json:"special_allocations" description:"Specific channel allocation overrides"`
	Encrypt                     bool           `json:"encrypt" description:"Whether to encrypt the backup or not"`
}

type BackupRestoreOpts struct {
	I ProtectedChannels []string `json:"protected_channels" description:"Channels to protect from being deleted"`
	I BackupSource      string   `json:"backup_source" description:"The source of the backup"`
	I Decrypt           string   `json:"decrypt" description:"The key to decrypt backups with, if any"`
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
        .addStringOption((opt) => {
            opt.setName("protected_channels")
            .setDescription("Channels to protect seperated by commas")
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

        return sub
    }),
    execute: async (ctx) => {
        let sc = ctx.interaction.options.getSubcommand()
        switch (sc) {
            case "create":
                let messages = ctx.interaction.options.getBoolean("messages")
                let attachments = ctx.interaction.options.getBoolean("attachments")
                let maxMessages = ctx.interaction.options.getInteger("max_messages")
                let perChannel = ctx.interaction.options.getInteger("per_channel")
                let rolloverLeftovers = ctx.interaction.options.getBoolean("rollover_leftovers")
                let ignoreMessageBackupErrors = ctx.interaction.options.getBoolean("ignore_message_backup_errors")
                let password = ctx.interaction.options.getString("password") || ""

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
                    data: {
                        "name": "create_backup"
                    },
                    args: {
                        "server_id": ctx.guild.id,
                        "options": {
                            "max_messages": maxMessages || 500,
                            "backup_messages": messages || false,
                            "backup_attachments": attachments || false,
                            "per_channel": perChannel || 100,
                            "rollover_leftovers": rolloverLeftovers || true,
                            "ignore_message_backup_errors": ignoreMessageBackupErrors || false,
                            "encrypt": password
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

                let task = await ctx.client.redis.pollForTask(tcr, {
                    timeout: 60000, // 1 minute timeout
                    targetId: ctx.guild.id,
                    targetType: "Server",
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

                if(protectedChannels.length > 0) {
                    protectedChannels = protectedChannels?.map((v) => v.trim())?.filter((v) => v.length > 0)
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
                    data: {
                        "name": "restore_backup"
                    },
                    args: {
                        "server_id": ctx.guild.id,
                        "options": {
                            "backup_source": backupFile.url,
                            "decrypt": password2,
                            "protected_channels": protectedChannels
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

                let task2 = await ctx.client.redis.pollForTask(tcr2, {
                    timeout: 60000, // 1 minute timeout
                    targetId: ctx.guild.id,
                    targetType: "Server",
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