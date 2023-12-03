import { Colors, EmbedBuilder, PermissionsBitField } from "discord.js";
import { Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";

let command: Command = {
    userPerms: [PermissionsBitField.Flags.ManageGuild],
    botPerms: [PermissionsBitField.Flags.ManageGuild],
    interactionData: new SlashCommandBuilder()
    .setName("backup")
	.setDescription("Create, load and get info on backups of your server!")
    .addSubcommand((sub) => {
        sub.setName("create")
        .setDescription("Create a backup of your server")

        return sub
    }),
    execute: async (ctx) => {
        switch (ctx.interaction.options.getSubcommand()) {
            case "create":
                await ctx.reply({
                    embeds: [
                        new EmbedBuilder()
                        .setTitle("Creating backup")
                        .setDescription(":yellow_circle: Please wait, starting backup task...")
                        .setColor(Colors.Blurple)
                    ]
                })

                let handle = await ctx.client.redis.sendIpcRequest({
                    scope: "splashtail",
                    action: "create_task",
                    data: {
                        "name": "create_backup"
                    },
                    args: {
                        "server_id": ctx.guild.id
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

                let task: TaskCreateResponse = res?.output

                if(!task?.task_id) {
                    return FinalResponse.edit({
                        embeds: [
                            new EmbedBuilder()
                            .setTitle("Creating backup")
                            .setDescription(`:x: Failed to create task. No task ID returned.`)
                            .setColor(Colors.Red)
                        ]
                    })
                }

                return FinalResponse.edit({
                    embeds: [
                        new EmbedBuilder()
                        .setTitle("Creating backup")
                        .setDescription(`:white_check_mark: Task created with ID \`${task?.task_id}\`. Waiting for task to complete...`)
                        .setColor(Colors.Green)
                    ]
                })
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
                        ])
                        .setColor(Colors.Blurple)
                    ]
                })
        }
    }
}

export default command;