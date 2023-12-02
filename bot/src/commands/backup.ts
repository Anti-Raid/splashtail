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
                        .setDescription(":yellow: Please wait, starting backup task...")
                        .setColor(Colors.Blurple)
                    ]
                })

                return FinalResponse.dummy()
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