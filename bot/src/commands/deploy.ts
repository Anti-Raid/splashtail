import { EmbedBuilder, Routes } from "discord.js";
import { BotStaffPerms, Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";

let command: Command = {
    userPerms: [],
    botPerms: [],
    botStaffPerms: [BotStaffPerms.Owner],
    interactionData: new SlashCommandBuilder()
    .setName("deploy")
    .addSubcommand((subcommand) => {
        subcommand
        .setName("all")
        .setDescription("Deploys all commands to the mothership!")

        return subcommand
    })
    .addSubcommand((subcommand) => {
        subcommand
        .setName("guild")
        .setDescription("Deploys guild commands to the mothership!")

        return subcommand
    })
    .addSubcommand((subcommand) => {
        subcommand
        .setName("global")
        .setDescription("Deploys global commands to the mothership!")

        return subcommand
    })
    .addSubcommand((subcommand) => {
        subcommand
        .setName("command")
        .setDescription("Deploys a specific command to the mothership!")
        .addStringOption((option) => option
            .setName("command")
            .setDescription("Which command to deploy?")
            .setRequired(true)
        )

        return subcommand
    })
	.setDescription("Deploys commands to the mothership!"),
    execute: async (ctx) => {
        const subcommand = ctx.interaction.options.getSubcommand()


        // Get the global commands
        let globalCommands = []
        let guildOnlyCommands = []

        switch (subcommand) {
            case "command":
                // Special case
                const commandName = ctx.interaction.options.getString("command")

                if(!commandName) {
                    return FinalResponse.reply({
                        content: "You must provide a command name!",
                    })
                }

                if(!ctx.client.commands.has(commandName)) {
                    return FinalResponse.reply({
                        content: "This command does not exist!",
                    })
                }

                if(ctx.client.commands.get(commandName)?.botStaffPerms?.includes(BotStaffPerms.Owner)) {
                    guildOnlyCommands.push(ctx.client.commands.get(commandName).interactionData.setDMPermission(false).toJSON())
                } else {
                    globalCommands.push(ctx.client.commands.get(commandName).interactionData.setDMPermission(false).toJSON())
                }
                break;
            case "global":
                for (const [id, command] of ctx.client.commands) {
                    if(command?.botStaffPerms?.includes(BotStaffPerms.Owner)) {
                        continue
                    }

                    globalCommands.push(command.interactionData.setDMPermission(false).toJSON())
                }
                break;
            case "guild":
                for (const [id, command] of ctx.client.commands) {
                    if(command?.botStaffPerms?.includes(BotStaffPerms.Owner)) {
                        guildOnlyCommands.push(command.interactionData.setDMPermission(false).toJSON())
                    }
                }
                break;
            case "all":
                for (const [id, command] of ctx.client.commands) {
                    if(command?.botStaffPerms?.includes(BotStaffPerms.Owner)) {
                        guildOnlyCommands.push(command.interactionData.setDMPermission(false).toJSON())
                    } else {
                        globalCommands.push(command.interactionData.setDMPermission(false).toJSON())
                    }
                }
                break;
            default:
            for (const [id, command] of ctx.client.commands) {
                if(command?.botStaffPerms?.includes(BotStaffPerms.Owner)) {
                    guildOnlyCommands.push(command.interactionData.setDMPermission(false).toJSON())
                } else {
                    globalCommands.push(command.interactionData.setDMPermission(false).toJSON())
                }
            }
            break;
        }

        await ctx.reply({
            embeds: [
                new EmbedBuilder()
                    .setColor("Orange")
                    .setDescription(
                        `Commands Deploying...`
                    )
                    .addFields(
                        {
                            name: "Global Commands",
                            value: `\`\`\`json\n${globalCommands.map(c => c.name)}\`\`\``,
                            inline: true
                        },
                        {
                            name: "Guild Only Commands",
                            value: `\`\`\`json\n${guildOnlyCommands.map(c => c.name)}\`\`\``,
                            inline: true
                        }
                    ),
            ],
        })

        await ctx.client.rest.put(Routes.applicationCommands(ctx.client.application.id), { body: globalCommands })
        await ctx.client.rest.put(Routes.applicationGuildCommands(ctx.client.application.id, ctx.client.config.servers.main), { body: guildOnlyCommands })

        await ctx.edit({
            embeds: [
                new EmbedBuilder()
                    .setColor("Orange")
                    .setDescription(
                        `Commands Deployed...`
                    )
                    .addFields(
                        {
                            name: "Global Commands",
                            value: `\`\`\`json\n${globalCommands.map(c => c.name)}\`\`\``,
                            inline: true
                        },
                        {
                            name: "Guild Only Commands",
                            value: `\`\`\`json\n${guildOnlyCommands.map(c => c.name)}\`\`\``,
                            inline: true
                        }
                    ),
            ],
        })

        return FinalResponse.dummy()
    }
}

export default command;